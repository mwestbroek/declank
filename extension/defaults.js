// Shared by the isolated-world content script and the popup. Both load this as
// a plain script, so everything here is a top-level global.

const DECLANK_STORAGE_KEY = 'declank_book';

// Message tags. Both worlds listen on the same window, and so does the page, so
// every message carries a source tag and anything unrecognised is ignored.
const DECLANK_MAIN = 'declank-main';
const DECLANK_ISOLATED = 'declank-isolated';

// Written to storage on first run. After that, storage is the only source of
// truth and this is never consulted again.
const DECLANK_DEFAULT_BOOK = {
  version: 1,
  enabled: true,
  rules: [
    { id: 'utilise', enabled: true, kind: 'lemma-verb', pattern: 'utilise', replacement: 'use' },
    { id: 'delve', enabled: true, kind: 'lemma-verb', pattern: 'delve', replacement: 'look' },
    { id: 'leverage', enabled: true, kind: 'lemma-verb', pattern: 'leverage', replacement: 'use' },
    { id: 'showcase', enabled: true, kind: 'lemma-verb', pattern: 'showcase', replacement: 'show' },
    { id: 'intricacy', enabled: true, kind: 'lemma-noun', pattern: 'intricacy', replacement: 'detail' },
    { id: 'fast-paced', enabled: true, kind: 'literal', pattern: "in today's fast-paced world", replacement: 'currently' },
    { id: 'landscape', enabled: true, kind: 'literal', pattern: 'the ever-evolving landscape of', replacement: 'the' },
    { id: 'crucial', enabled: true, kind: 'literal', pattern: 'it is crucial to note that', replacement: '' },
    { id: 'tapestry', enabled: true, kind: 'literal', pattern: 'rich tapestry of', replacement: 'range of' },
    { id: 'not-just', enabled: true, kind: 'template', pattern: "it's not just {A}, it's {B}", replacement: "it's {B}" },
    { id: 'in-question', enabled: true, kind: 'template', pattern: 'the {A} in question', replacement: 'the {A}' }
  ]
};
