console.log('[declank] installed');

const originalFetch = window.fetch;
const decoder = new TextDecoder();
const encoder = new TextEncoder();

const wasmBytes = Uint8Array.from(atob(self.DECLANK_WASM_B64), c => c.charCodeAt(0));

const ready = wasm_bindgen({ module_or_path: wasmBytes }).then(() => {
  console.log('[declank] engine ready');
});

const declank = (s) => wasm_bindgen.declank(s);

// Hold back at most this much text while waiting for a sentence boundary.
// Bounds both latency and how much could be lost if a stream ends abnormally.
const MAX_HOLD = 400;

// ---------- sentence splitting ----------
//
// Naive: a full stop, question mark or exclamation mark followed by whitespace,
// or a newline. Known to be wrong on "e.g.", "Dr.", decimals and ellipses.
// Newlines count because headings, list items and table rows often contain no
// terminator at all and would otherwise never flush.

function isBoundaryAt(text, i) {
  const c = text[i];
  if (c === '\n') return true;
  if (c === '.' || c === '!' || c === '?') {
    const next = text[i + 1];
    return next !== undefined && /\s/.test(next);
  }
  return false;
}

/** Index just past the last boundary, or 0 if there is none. */
function lastBoundary(text) {
  for (let i = text.length - 1; i >= 0; i--) {
    if (isBoundaryAt(text, i)) return i + 1;
  }
  return 0;
}

/** Split into sentence-ish chunks. Concatenating the result reproduces the input. */
function splitSentences(text) {
  const out = [];
  let start = 0;
  for (let i = 0; i < text.length; i++) {
    if (isBoundaryAt(text, i)) {
      out.push(text.slice(start, i + 1));
      start = i + 1;
    }
  }
  if (start < text.length) out.push(text.slice(start));
  return out;
}

// ---------- code protection ----------
//
// The engine has no idea what markdown is, so anything that must not be
// rewritten is filtered out here and never reaches it.
//
// Fenced blocks need state that survives across chunks, since a fence opens in
// one delta and closes many deltas later. That state lives in the caller and is
// threaded through, which is why these take a `state` argument rather than
// keeping their own.
//
// Deliberately shallow. It does not understand nested fences, fences inside
// blockquotes, or indented four-space code blocks. Those can be added when
// they actually cause trouble.

function isFenceLine(unit) {
  const trimmed = unit.trim();
  return trimmed.startsWith('```') || trimmed.startsWith('~~~');
}

/** Rewrite prose while leaving `inline code spans` untouched. */
function rewriteInline(text) {
  // The capture group keeps the delimiters in the output of split(), so the
  // pieces can be reassembled exactly.
  return text
    .split(/(`[^`\n]*`)/)
    .map(part => (part.startsWith('`') ? part : declank(part)))
    .join('');
}

/** Process one sentence-ish unit, honouring and updating fence state. */
function processUnit(unit, state) {
  if (isFenceLine(unit)) {
    state.inFence = !state.inFence;
    return unit;
  }
  if (state.inFence) return unit;
  return rewriteInline(unit);
}

/** Split a run of text into units and process each one. */
function processText(text, state) {
  return splitSentences(text)
    .map(unit => processUnit(unit, state))
    .join('');
}

/** Whole-message entry point: fence state starts fresh each time. */
function rewriteWhole(text) {
  return processText(text, { inFence: false });
}

// ---------- streaming path ----------

function rewriteStream(response) {
  let lineBuffer = '';
  let sentenceBuffer = '';
  let pendingEvent = null;
  let lastIndex = 0;
  let flushedAtEnd = false;

  // One response is one message, so a single fence state spans the stream.
  const fenceState = { inFence: false };

  // Emit whatever is held back as a synthetic delta event.
  function flushSentenceBuffer() {
    if (sentenceBuffer.length === 0) return [];
    const text = processText(sentenceBuffer, fenceState);
    sentenceBuffer = '';
    const event = {
      type: 'content_block_delta',
      index: lastIndex,
      delta: { type: 'text_delta', text }
    };
    return ['event: content_block_delta', 'data: ' + JSON.stringify(event)];
  }

  function processLine(line) {
    // Hold the event line so it can be emitted together with its data line.
    if (line.startsWith('event: ')) {
      pendingEvent = line;
      return [];
    }

    if (!line.startsWith('data: ')) return [line];

    const eventLine = pendingEvent;
    pendingEvent = null;
    const emit = (dataLine, before = []) =>
      eventLine ? [...before, eventLine, dataLine] : [...before, dataLine];

    let obj;
    try {
      obj = JSON.parse(line.slice(6));
    } catch (e) {
      // Should not happen now that lines are reassembled before parsing.
      console.warn('[declank] unparseable data line, passing through');
      return emit(line);
    }

    if (obj?.type === 'content_block_delta' && typeof obj?.delta?.text === 'string') {
      if (typeof obj.index === 'number') lastIndex = obj.index;

      sentenceBuffer += obj.delta.text;

      let ready = '';
      const cut = lastBoundary(sentenceBuffer);
      if (cut > 0) {
        ready = processText(sentenceBuffer.slice(0, cut), fenceState);
        sentenceBuffer = sentenceBuffer.slice(cut);
      } else if (sentenceBuffer.length > MAX_HOLD) {
        // No boundary in sight. Release rather than hold indefinitely.
        // A partial line cannot be recognised as a fence marker, so a fence
        // opened by an unusually long line would be missed. Rare enough to
        // accept.
        ready = processText(sentenceBuffer, fenceState);
        sentenceBuffer = '';
      }

      obj.delta.text = ready;
      return emit('data: ' + JSON.stringify(obj));
    }

    // Flush the tail before the block or message closes.
    if (
      obj?.type === 'content_block_stop' ||
      obj?.type === 'message_delta' ||
      obj?.type === 'message_stop'
    ) {
      const injected = flushSentenceBuffer();
      if (injected.length > 0) flushedAtEnd = true;
      return emit(line, injected);
    }

    return emit(line);
  }

  const rewriter = new TransformStream({
    transform(chunk, controller) {
      lineBuffer += decoder.decode(chunk, { stream: true });

      const lines = lineBuffer.split('\n');
      // The final element may be an incomplete line; keep it for the next chunk.
      lineBuffer = lines.pop();

      const out = [];
      for (const line of lines) out.push(...processLine(line));

      if (out.length > 0) {
        controller.enqueue(encoder.encode(out.join('\n') + '\n'));
      }
    },

    flush(controller) {
      const out = [];
      if (lineBuffer.length > 0) out.push(...processLine(lineBuffer));
      if (pendingEvent) out.push(pendingEvent);
      out.push(...flushSentenceBuffer());

      if (out.length > 0) {
        controller.enqueue(encoder.encode(out.join('\n')));
      }
      if (!flushedAtEnd && sentenceBuffer.length > 0) {
        console.warn('[declank] stream ended with text still buffered');
      }
      if (fenceState.inFence) {
        console.warn('[declank] stream ended inside an unclosed code fence');
      }
      console.log('[declank] stream done');
    }
  });

  return new Response(response.body.pipeThrough(rewriter), {
    status: response.status,
    statusText: response.statusText,
    headers: response.headers
  });
}

// ---------- history path ----------

let historyHits = 0;

function rewriteMessage(m) {
  if (m?.sender !== 'assistant') return;

  if (typeof m.text === 'string') {
    const after = rewriteWhole(m.text);
    if (after !== m.text) { historyHits++; m.text = after; }
  }

  if (Array.isArray(m.content)) {
    for (const block of m.content) {
      if (block?.type === 'text' && typeof block.text === 'string') {
        const after = rewriteWhole(block.text);
        if (after !== block.text) { historyHits++; block.text = after; }
      }
    }
  }
}

function rewriteConversation(c) {
  if (Array.isArray(c?.chat_messages)) c.chat_messages.forEach(rewriteMessage);
}

async function rewriteHistory(response) {
  const raw = await response.text();

  const passthrough = () => new Response(raw, {
    status: response.status,
    statusText: response.statusText,
    headers: response.headers
  });

  let doc;
  try {
    doc = JSON.parse(raw);
  } catch {
    return passthrough();
  }

  if (doc === null || typeof doc !== 'object') return passthrough();

  historyHits = 0;

  try {
    if (Array.isArray(doc)) {
      doc.forEach(rewriteConversation);
    } else if (Array.isArray(doc.conversations)) {
      doc.conversations.forEach(rewriteConversation);
    } else {
      rewriteConversation(doc);
    }
  } catch (e) {
    console.warn('[declank] history rewrite failed, passing through', e);
    return passthrough();
  }

  console.log(`[declank] history rewritten. replacements: ${historyHits}`);

  const headers = new Headers(response.headers);
  headers.delete('content-length');

  return new Response(JSON.stringify(doc), {
    status: response.status,
    statusText: response.statusText,
    headers
  });
}

// ---------- dispatch ----------

window.fetch = async (...args) => {
  const response = await originalFetch(...args);
  const url = typeof args[0] === 'string' ? args[0] : args[0]?.url ?? '';

  if (!response.ok || !response.body) return response;

  const interesting = url.includes('/completion') || url.includes('/chat_conversations');
  if (!interesting) return response;

  try {
    await ready;
  } catch (e) {
    console.warn('[declank] engine failed to load, passing through', e);
    return response;
  }

  if (url.includes('/completion')) {
    console.log('[declank] intercepting stream');
    return rewriteStream(response);
  }

  console.log('[declank] intercepting history');
  return rewriteHistory(response);
};