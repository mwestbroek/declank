console.log('[declank] installed v6');

const originalFetch = window.fetch;
const decoder = new TextDecoder();
const encoder = new TextEncoder();

const wasmBytes = Uint8Array.from(atob(self.DECLANK_WASM_B64), c => c.charCodeAt(0));

const ready = wasm_bindgen({ module_or_path: wasmBytes }).then(() => {
  console.log('[declank] engine ready');
});

const rewrite = (s) => wasm_bindgen.rewrite(s);
//
// ---------- streaming path ----------

function rewriteStream(response) {
  let hits = 0;
  let partialLines = 0;

  const rewriter = new TransformStream({
    transform(chunk, controller) {
      const text = decoder.decode(chunk, { stream: true });

      const out = text.split('\n').map(line => {
        if (!line.startsWith('data: ')) return line;

        let obj;
        try {
          obj = JSON.parse(line.slice(6));
        } catch {
          partialLines++;
          return line;
        }

        if (typeof obj?.delta?.text === 'string') {
          const after = rewrite(obj.delta.text);
          if (after !== obj.delta.text) {
            hits++;
            obj.delta.text = after;
            return 'data: ' + JSON.stringify(obj);
          }
        }
        return line;
      }).join('\n');

      controller.enqueue(encoder.encode(out));
    },

    flush() {
      console.log(`[declank] stream done. replacements: ${hits}, truncated lines: ${partialLines}`);
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
    const after = rewrite(m.text);
    if (after !== m.text) { historyHits++; m.text = after; }
  }

  if (Array.isArray(m.content)) {
    for (const block of m.content) {
      if (block?.type === 'text' && typeof block.text === 'string') {
        const after = rewrite(block.text);
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

  // Engine may still be loading. Awaiting a settled promise costs one microtask.
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
