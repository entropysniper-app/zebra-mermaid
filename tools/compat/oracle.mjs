// Development-only oracle: the pinned Pele snapshot, bundled with its own esbuild, answering
// JSONL requests. The Rust library never runs this; tests and fixture generators do.
//
//   node tools/compat/oracle.mjs < requests.jsonl > answers.jsonl
//
// A request is {"op": ..., ...}; the answer keeps its "id". Operations: render, parse,
// preprocess, yaml, url, label, num, math, entities, style.
// Fixtures are computed in UTC, so that dates do not depend on the machine's time zone. The
// Rust side renders with `timezone_offset_minutes` unset, which is UTC.
process.env.TZ = 'UTC';
import { build } from '../../.research/pele/node_modules/esbuild/lib/main.js';
import { createInterface } from 'node:readline';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { dirname, join } from 'node:path';
import { mkdirSync } from 'node:fs';

const here = dirname(fileURLToPath(import.meta.url));
const root = join(here, '../..');
const outDir = join(root, 'target/compat');
const outFile = join(outDir, 'oracle.bundle.mjs');

export async function loadOracle() {
  mkdirSync(outDir, { recursive: true });
  const entry = join(here, 'oracle-entry.ts');
  // The entry's mtime does not change when the pinned upstream checkout changes.
  // Always build from its current source so reference values cannot use an old bundle.
  await build({
    entryPoints: [entry],
    bundle: true,
    format: 'esm',
    platform: 'node',
    outfile: outFile,
    logLevel: 'error',
    loader: { '.json': 'json' },
  });
  const mod = await import(pathToFileURL(outFile).href);
  mod.register(...mod.all);
  return mod;
}

function special(n) {
  if (typeof n === 'number' && !Number.isFinite(n)) return { $number: String(n) };
  return n;
}

// JSON with NaN/Infinity written as {"$number": "..."} and Maps as entry lists.
export function plain(value) {
  if (value instanceof Map) return { $map: [...value].map(([k, v]) => [plain(k), plain(v)]) };
  if (Array.isArray(value)) return value.map(plain);
  if (value && typeof value === 'object') {
    const out = {};
    // defineProperty keeps a key such as `__proto__` as an own property.
    for (const k of Object.keys(value)) Object.defineProperty(out, k, { value: plain(value[k]), enumerable: true, writable: true, configurable: true });
    return out;
  }
  return special(value);
}

function errorOf(e) {
  if (e && e.name === 'PeleError') {
    return { code: e.code, message: e.message, line: e.line, column: e.column, snippet: e.snippet, type: e.type, mermaidType: e.mermaidType };
  }
  return { code: 'exception', name: e?.name ?? 'Error', message: String(e?.message ?? e) };
}

function iconsFrom(table) {
  if (!table) return undefined;
  return (name) => (Object.prototype.hasOwnProperty.call(table, name) ? table[name] : null);
}

export function answer(o, req) {
  const id = req.id;
  try {
    switch (req.op) {
      case 'render': {
        const options = { ...(req.options ?? {}) };
        for (const [k, v] of Object.entries(options)) if (v && typeof v === 'object' && typeof v.$number === 'string') options[k] = Number(v.$number);
        if (options.icons) options.icons = iconsFrom(options.icons);
        const r = o.render(req.source, options);
        return { id, ok: true, type: r.type, mermaidType: r.mermaidType, width: r.width, height: r.height, svg: r.svg, links: r.links };
      }
      case 'parse': {
        const m = o.parse(req.source, req.options ?? {});
        return { id, ok: true, model: plain(m) };
      }
      case 'preprocess': {
        const p = o.preprocess(req.source);
        return { id, ok: true, text: p.text, title: p.title ?? null, config: plain(p.config), withComments: p.withComments, lineOffset: p.lineOffset, encoded: o.encodeEntities(p.text) };
      }
      case 'yaml':
        return { id, ok: true, value: plain(o.parseYaml(req.source)) };
      case 'url':
        return { id, ok: true, value: o.sanitizeUrl(req.source), safe: o.safeUrl(req.source) };
      case 'entities':
        return { id, ok: true, value: o.decodeEntities(req.source) };
      case 'label': {
        const l = o.layoutLabel(req.source, !!req.markdown, o.metricsMeasurer, req.size ?? 16, req.max ?? 4000, req.base ?? 0);
        return { id, ok: true, label: l };
      }
      case 'num':
        return { id, ok: true, values: req.values.map((v) => o.num(v)) };
      case 'math': {
        const f = Math[req.fn];
        return { id, ok: true, values: req.args.map((a) => special(f(...a))), bits: req.args.map((a) => bits(f(...a))) };
      }
      case 'style':
        return { id, ok: true, value: o.resolveStyle(req.declarations) };
      default:
        return { id, ok: false, error: { code: 'bad-op', message: String(req.op) } };
    }
  } catch (e) {
    return { id, ok: false, error: errorOf(e) };
  }
}

function bits(x) {
  const b = new DataView(new ArrayBuffer(8));
  b.setFloat64(0, x);
  return b.getBigUint64(0).toString(16).padStart(16, '0');
}

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) {
  const o = await loadOracle();
  const rl = createInterface({ input: process.stdin, crlfDelay: Infinity });
  for await (const line of rl) {
    if (!line.trim()) continue;
    process.stdout.write(JSON.stringify(answer(o, JSON.parse(line))) + '\n');
  }
}
