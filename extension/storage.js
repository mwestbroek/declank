// Isolated world. This half exists only because chrome.storage is unavailable
// in the MAIN world, where the engine has to live in order to patch fetch.
// Its whole job is to read the stored rule book and hand it across.

function sendBook(book) {
  window.postMessage(
    { source: DECLANK_ISOLATED, type: 'rules', book: JSON.stringify(book) },
    window.location.origin
  );
}

async function loadBook() {
  const got = await chrome.storage.local.get(DECLANK_STORAGE_KEY);
  const stored = got[DECLANK_STORAGE_KEY];
  if (stored) return stored;

  // First run on this profile.
  await chrome.storage.local.set({ [DECLANK_STORAGE_KEY]: DECLANK_DEFAULT_BOOK });
  return DECLANK_DEFAULT_BOOK;
}

async function pushBook() {
  let book;
  try {
    book = await loadBook();
  } catch (e) {
    // Send something regardless. The MAIN world blocks fetches until rules
    // arrive, so staying silent would stall the page.
    console.warn('[declank] could not read stored rules, using defaults', e);
    book = DECLANK_DEFAULT_BOOK;
  }
  sendBook(book);
}

// Two paths, because the load order of the two content scripts is not
// guaranteed: push on load, and answer a request from the MAIN world.
window.addEventListener('message', (event) => {
  if (event.source !== window) return;
  const data = event.data;
  if (!data || data.source !== DECLANK_MAIN) return;
  if (data.type === 'request-rules') pushBook();
});

// The popup writes to storage; this is how the change reaches an open tab.
chrome.storage.onChanged.addListener((changes, area) => {
  if (area !== 'local') return;
  const change = changes[DECLANK_STORAGE_KEY];
  if (!change || !change.newValue) return;
  sendBook(change.newValue);
});

pushBook();
