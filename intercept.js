(() => {
  console.log('[sanitiser] installed v2');
  const originalFetch = window.fetch;

  let hits = 0;
  let partialLines = 0;

  window.fetch = async (...args) => {
    const response = await originalFetch(...args);
    const url = typeof args[0] === 'string' ? args[0] : args[0]?.url ?? '';

    if (!response.body || !url.includes('completion')) return response;
    console.log('[sanitiser] intercepting', url);
    hits = 0;
    partialLines = 0;

    const decoder = new TextDecoder();
    const encoder = new TextEncoder();

    const rewriter = new TransformStream({
      transform(chunk, controller) {
        const text = decoder.decode(chunk, { stream: true });

        const out = text.split('\n').map(line => {
          if (!line.startsWith('data: ')) return line;

          let obj;
          try {
            obj = JSON.parse(line.slice(6));
          } catch {
            // Chunk ended mid-line, so this is truncated JSON.
            // Pass through untouched and count it.
            partialLines++;
            return line;
          }

          if (typeof obj?.delta?.text === 'string') {
            const before = obj.delta.text;
            const after = before.replace(/\band\b/gi, '&');
            if (after !== before) {
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
        console.log(
          `[sanitiser] done. replacements: ${hits}, truncated lines: ${partialLines}`
        );
      }
    });

    return new Response(response.body.pipeThrough(rewriter), {
      status: response.status,
      statusText: response.statusText,
      headers: response.headers
    });
  };
})();
