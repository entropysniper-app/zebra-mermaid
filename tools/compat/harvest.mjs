// Harvests the inputs that Pele's own test suite gives to render() and parse(), so the Rust
// port can be compared on them. The pinned snapshot in .research/pele is not modified: it is
// copied to target/compat/harvest/, where src/core.ts gets a logging wrapper, and vitest runs
// there. Expected outputs are then recomputed with the clean oracle bundle.
//
//   node tools/compat/harvest.mjs            # writes target/compat/harvest/inputs.jsonl
//   node tools/compat/harvest.mjs --fixtures # also writes tests/fixtures/harvest/<type>.jsonl
import { cpSync, rmSync, mkdirSync, readFileSync, writeFileSync, readdirSync, symlinkSync, existsSync, renameSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { gzipSync } from 'node:zlib';
import { loadOracle, answer } from './oracle.mjs';

const here = dirname(fileURLToPath(import.meta.url));
const root = join(here, '../..');
const pele = join(root, '.research/pele');
// Fixtures end up in target/compat/harvest/fixtures; everything else happens in a work
// directory, and the fixtures directory is swapped in at the end, so tests running meanwhile
// never see half-written files.
const out = join(root, 'target/compat/harvest');
const work = join(root, 'target/compat/harvest-work');
const copy = join(work, 'pele');
const logs = join(work, 'logs');

// The full set is written to target/compat/harvest/fixtures/ (not committed). The committed
// sample in tests/fixtures/harvest/ keeps at most SAMPLE cases per type, chosen by hash, and
// leaves out inputs and outputs above these sizes.
const MAX_SOURCE = 20000;
const MAX_SVG = 300000;
const SAMPLE = 400;
const FIXED_NOW = Date.parse('2026-10-19T00:00:00Z');

const SERIALIZABLE = [
  'fontFamily', 'fontFamilyMono', 'fontSize', 'idPrefix', 'responsive', 'maxWidth', 'autoDirection', 'directionBreakpoint',
  'padding', 'limit', 'outputLimit', 'maxEdges', 'linkSchemes', 'imageSchemes', 'links', 'images', 'linkRel', 'config', 'now',
];

const WRAPPER = `
// ---- harvest wrapper (added by tools/compat/harvest.mjs) ----
import { appendFileSync as __append } from 'node:fs';
import { threadId as __thread } from 'node:worker_threads';
import { metricsMeasurer as __metrics } from './text/measurer.js';
const __sources = new WeakMap();
function __log(entry) {
  const dir = process.env.HARVEST_DIR;
  if (!dir) return;
  try {
    __append(dir + '/' + process.pid + '-' + __thread + '.jsonl', JSON.stringify(entry) + '\\n');
  } catch {}
}
function __opts(options) {
  const out = {};
  const skip = [];
  for (const [k, v] of Object.entries(options ?? {})) {
    // JSON has no Infinity (Pele's stress tests pass limit: Infinity); write it tagged.
    if (${JSON.stringify(SERIALIZABLE)}.includes(k)) out[k] = v instanceof Date ? v.getTime() : typeof v === 'number' && !Number.isFinite(v) ? { $number: String(v) } : v;
    // Pele's own tests mostly pass the default Node measurer explicitly; that one is kept.
    else if (k === 'measurer' && v === __metrics) continue;
    else if (k === 'icons' || k === 'measurer') skip.push(k);
    else skip.push(k);
  }
  return { out, skip };
}
export function parse(text, options = {}) {
  const model = __parse(text, options);
  if (typeof model === 'object' && model !== null) __sources.set(model, { text, options });
  return model;
}
export function render(source, options = {}) {
  const icons = options.icons;
  const table = {};
  const wrapped = icons ? { ...options, icons: (name) => { const v = icons(name); table[name] = v ?? null; return v; } } : options;
  let text = source;
  let parseOptions = {};
  if (typeof source !== 'string') {
    const found = __sources.get(source);
    if (!found) return __render(source, wrapped);
    text = found.text;
    parseOptions = found.options ?? {};
  }
  const result = __render(source, wrapped);
  const p = __opts(parseOptions);
  const r = __opts(options);
  const skip = [...p.skip, ...r.skip].filter((k) => k !== 'icons');
  __log({ text, options: { ...r.out, limit: p.out.limit ?? r.out.limit, maxEdges: p.out.maxEdges ?? r.out.maxEdges, config: p.out.config ?? r.out.config }, icons: icons ? table : undefined, skip });
  return result;
}
`;

function patchCore() {
  const file = join(copy, 'src/core.ts');
  let text = readFileSync(file, 'utf8');
  text = text.replace('export function parse(text: string', 'function __parse(text: string');
  text = text.replace('export function render(source: string | DiagramModel', 'function __render(source: string | DiagramModel');
  // mount() calls parse/render through mountWith; keep it working with the originals.
  text = text.replace('return mountWith({ parse, render }, element, text, options);', 'return mountWith({ parse: __parse, render: __render }, element, text, options);');
  if (!text.includes('function __parse(') || !text.includes('function __render(')) throw new Error('core.ts changed shape; update the harvest patch');
  writeFileSync(file, text + WRAPPER);
}

function harvest() {
  rmSync(work, { recursive: true, force: true });
  mkdirSync(logs, { recursive: true });
  for (const part of ['src', 'tests', 'scripts', 'package.json', 'tsconfig.json', 'vitest.config.ts']) {
    if (existsSync(join(pele, part))) cpSync(join(pele, part), join(copy, part), { recursive: true });
  }
  symlinkSync(join(pele, 'node_modules'), join(copy, 'node_modules'));
  patchCore();
  const run = spawnSync(join(copy, 'node_modules/.bin/vitest'), ['run', '--reporter=dot', '--exclude', 'tests/website/**'], {
    cwd: copy,
    env: { ...process.env, HARVEST_DIR: logs, NO_COLOR: '1', FORCE_COLOR: '' },
    encoding: 'utf8',
    maxBuffer: 1 << 28,
  });
  writeFileSync(join(work, 'vitest.log'), run.stdout + run.stderr);
  const tail = (run.stdout + run.stderr).split('\n').filter((l) => /Test Files|Tests /.test(l)).join('\n');
  console.error(tail);

  const seen = new Map();
  for (const f of readdirSync(logs)) {
    for (const line of readFileSync(join(logs, f), 'utf8').split('\n')) {
      if (!line) continue;
      const e = JSON.parse(line);
      if (e.skip.length > 0 || typeof e.text !== 'string') continue;
      const options = Object.fromEntries(Object.entries(e.options).filter(([, v]) => v !== undefined));
      const key = createHash('sha256').update(JSON.stringify([e.text, options, e.icons ?? null])).digest('hex');
      if (!seen.has(key)) seen.set(key, { key, source: e.text, options, icons: e.icons });
    }
  }
  const rows = [...seen.values()].sort((a, b) => (a.key < b.key ? -1 : 1));
  writeFileSync(join(work, 'inputs.jsonl'), rows.map((r) => JSON.stringify(r)).join('\n') + '\n');
  return rows;
}

const rows = process.argv.includes('--reuse-inputs')
  ? readFileSync(join(work, 'inputs.jsonl'), 'utf8').trim().split('\n').map(JSON.parse)
  : harvest();
console.error(`harvested ${rows.length} distinct render inputs`);

if (process.argv.includes('--fixtures')) {
  const o = await loadOracle();
  const byType = new Map();
  let kept = 0;
  let dropped = 0;
  for (const r of rows) {
    let type;
    try {
      type = o.detectType(r.source) ?? 'none';
    } catch {
      type = 'error';
    }
    // Missing or non-finite `now` makes Pele consult Date.now(). Normalize that
    // host dependency before evaluating both implementations; retain the original
    // argument so this is never reported as coverage of an injected system clock.
    const originalNow = r.options.now;
    const numericNow = originalNow && typeof originalNow === 'object'
      ? Number(originalNow.$number) : Number(originalNow);
    const clockNormalized = originalNow === undefined || !Number.isFinite(numericNow);
    if (clockNormalized) r.options.now = FIXED_NOW;
    const options = { ...r.options, ...(r.icons ? { icons: r.icons } : {}) };
    const a = answer(o, { op: 'render', source: r.source, options });
    delete a.id;
    if (r.source.length > MAX_SOURCE || (a.svg && a.svg.length > MAX_SVG)) {
      dropped++;
      continue;
    }
    kept++;
    if (!byType.has(type)) byType.set(type, []);
    byType.get(type).push({ id: r.key.slice(0, 16), source: r.source, options: r.options, icons: r.icons ?? null, expect: a,
      ...(originalNow !== undefined && clockNormalized ? { clockInput: originalNow, clockNormalizedTo: FIXED_NOW } : {}) });
  }
  const full = join(work, 'fixtures');
  rmSync(full, { recursive: true, force: true });
  const dir = join(root, 'tests/fixtures/harvest');
  rmSync(dir, { recursive: true, force: true });
  mkdirSync(dir, { recursive: true });
  mkdirSync(full, { recursive: true });
  let sampled = 0;
  for (const [type, list] of byType) {
    writeFileSync(join(full, `${type}.jsonl`), list.map((x) => JSON.stringify(x)).join('\n') + '\n');
    const sample = list.slice(0, SAMPLE);
    sampled += sample.length;
    writeFileSync(join(dir, `${type}.jsonl.gz`), gzipSync(sample.map((x) => JSON.stringify(x)).join('\n') + '\n', { level: 9 }));
  }
  console.error(`sample: ${sampled} committed cases`);
  mkdirSync(out, { recursive: true });
  const live = join(out, 'fixtures');
  const old = join(out, 'fixtures.old');
  rmSync(old, { recursive: true, force: true });
  if (existsSync(live)) renameSync(live, old);
  renameSync(full, live);
  rmSync(old, { recursive: true, force: true });
  writeFileSync(join(out, 'inputs.jsonl'), readFileSync(join(work, 'inputs.jsonl')));
  console.error(`fixtures: ${kept} kept, ${dropped} left out for size`);
}
