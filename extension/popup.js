// The popup runs in its own extension page, with its own copy of the engine.
// That copy exists only to validate rules: it is the same parser the content
// script uses, so a rule accepted here cannot be rejected there. Duplicating
// the validation in JavaScript would drift.

const wasmBytes = Uint8Array.from(atob(self.DECLANK_WASM_B64), c => c.charCodeAt(0));
const engineReady = wasm_bindgen({ module_or_path: wasmBytes });

const els = {
  master: document.getElementById('master'),
  count: document.getElementById('count'),
  status: document.getElementById('status'),
  rules: document.getElementById('rules'),
  empty: document.getElementById('empty'),
  form: document.getElementById('add'),
  kind: document.getElementById('kind'),
  pattern: document.getElementById('pattern'),
  replacement: document.getElementById('replacement'),
  hint: document.getElementById('hint')
};

const HINTS = {
  'literal': 'Matched exactly, plus a capitalised version for the start of a sentence.',
  'lemma-verb': 'Give the base form. Other tenses are generated: utilise, utilises, utilised, utilising.',
  'lemma-noun': 'Give the singular. The plural is generated.',
  'template': 'Use {A}, {B}, {C} for the parts that vary. Must start with fixed text.'
};

let book = null;

// ---------- storage ----------

async function readBook() {
  const got = await chrome.storage.local.get(DECLANK_STORAGE_KEY);
  return got[DECLANK_STORAGE_KEY] || DECLANK_DEFAULT_BOOK;
}

async function writeBook() {
  await chrome.storage.local.set({ [DECLANK_STORAGE_KEY]: book });
}

// ---------- validation ----------

/**
 * Runs a candidate book through the real engine and returns the errors it
 * reports, keyed by rule id.
 */
async function validate(candidate) {
  await engineReady;
  const report = JSON.parse(wasm_bindgen.set_rules(JSON.stringify(candidate)));
  const byId = new Map();
  for (const e of report.errors || []) byId.set(e.id, e.error);
  return byId;
}

function showStatus(message) {
  els.status.textContent = message;
  els.status.classList.add('shown');
}

function clearStatus() {
  els.status.textContent = '';
  els.status.classList.remove('shown');
}

// ---------- rendering ----------

const KIND_LABELS = {
  'literal': 'literal',
  'lemma-verb': 'verb',
  'lemma-noun': 'noun',
  'template': 'template'
};

function ruleRow(rule, problem) {
  const row = document.createElement('div');
  row.className = 'rule' + (rule.enabled ? '' : ' off');

  const toggle = document.createElement('input');
  toggle.type = 'checkbox';
  toggle.checked = rule.enabled;
  toggle.title = rule.enabled ? 'Turn off' : 'Turn on';
  toggle.addEventListener('change', async () => {
    rule.enabled = toggle.checked;
    await writeBook();
    render();
  });

  const body = document.createElement('div');
  body.className = 'body';

  const swap = document.createElement('div');
  swap.className = 'swap';

  const from = document.createElement('span');
  from.className = 'from';
  from.textContent = rule.pattern;

  const arrow = document.createElement('span');
  arrow.className = 'arrow';
  arrow.textContent = '\u2192';

  const to = document.createElement('span');
  to.className = 'to' + (rule.replacement ? '' : ' empty');
  to.textContent = rule.replacement || 'deleted';

  swap.append(from, arrow, to);

  const meta = document.createElement('div');
  meta.className = 'meta';
  meta.textContent = problem ? `${KIND_LABELS[rule.kind]} — ${problem}` : KIND_LABELS[rule.kind];
  if (problem) meta.style.color = 'var(--strike)';

  body.append(swap, meta);

  const remove = document.createElement('button');
  remove.className = 'remove';
  remove.type = 'button';
  remove.textContent = '\u00d7';
  remove.title = 'Delete this rule';
  remove.addEventListener('click', async () => {
    book.rules = book.rules.filter(r => r.id !== rule.id);
    await writeBook();
    render();
  });

  row.append(toggle, body, remove);
  return row;
}

async function render() {
  els.master.checked = book.enabled !== false;

  const problems = await validate(book);

  const live = book.rules.filter(r => r.enabled && !problems.has(r.id)).length;
  els.count.textContent =
    book.enabled === false
      ? `${book.rules.length} rules, paused`
      : `${live} of ${book.rules.length} rules active`;

  els.rules.replaceChildren(
    ...book.rules.map(r => ruleRow(r, problems.get(r.id)))
  );
  els.empty.hidden = book.rules.length > 0;
}

// ---------- events ----------

els.master.addEventListener('change', async () => {
  book.enabled = els.master.checked;
  await writeBook();
  render();
});

els.kind.addEventListener('change', () => {
  els.hint.textContent = HINTS[els.kind.value];
});

els.form.addEventListener('submit', async (event) => {
  event.preventDefault();
  clearStatus();

  const candidate = {
    id: crypto.randomUUID(),
    enabled: true,
    kind: els.kind.value,
    pattern: els.pattern.value,
    replacement: els.replacement.value
  };

  if (!candidate.pattern.trim()) {
    showStatus('Enter the text to find.');
    return;
  }

  // Validate against a copy, so a bad rule never reaches storage.
  const trial = { ...book, rules: [...book.rules, candidate] };
  const problems = await validate(trial);
  if (problems.has(candidate.id)) {
    showStatus(problems.get(candidate.id));
    return;
  }

  book = trial;
  await writeBook();
  els.pattern.value = '';
  els.replacement.value = '';
  render();
});

// Another tab or window may have changed the book.
chrome.storage.onChanged.addListener((changes, area) => {
  if (area !== 'local') return;
  const change = changes[DECLANK_STORAGE_KEY];
  if (!change || !change.newValue) return;
  book = change.newValue;
  render();
});

(async () => {
  els.hint.textContent = HINTS[els.kind.value];
  book = await readBook();
  render();
})();
