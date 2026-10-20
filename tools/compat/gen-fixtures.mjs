// Writes the differential fixtures under tests/fixtures/ from the pinned Pele snapshot.
//
//   node tools/compat/gen-fixtures.mjs
//
// Each fixture line holds an input and what Pele 1.6.0 (commit 44da152b) answers for it, run
// in Node with Pele's built-in Arial estimate as the text measurer. The Rust tests read these
// files; they never run Node themselves.
import { readFileSync, readdirSync, writeFileSync, mkdirSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';
import { execFileSync } from 'node:child_process';
import { loadOracle, answer } from './oracle.mjs';

const here = dirname(fileURLToPath(import.meta.url));
const root = join(here, '../..');
const pele = join(root, '.research/pele');
const out = join(root, 'tests/fixtures');
const commit = execFileSync('git', ['-C', pele, 'rev-parse', 'HEAD'], { encoding: 'utf8' }).trim();
if (commit !== '44da152bbb79b91f03e68016f42499bb25bf6e82') {
  throw Error('Historical fixtures require the original 44da152b reference. For the current release use tools/compat/gen-release-fixtures.mjs.');
}

// Fixed options for every render fixture, so that output does not depend on the clock.
export const RENDER_OPTIONS = { idPrefix: 'baseline', now: Date.parse('2026-10-19T00:00:00Z') };

const o = await loadOracle();

function writeJsonl(path, rows) {
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, rows.map((r) => JSON.stringify(r)).join('\n') + '\n');
}

// A small deterministic generator (mulberry32), so fixtures are reproducible.
function rng(seed) {
  let t = seed >>> 0;
  return () => {
    t = (t + 0x6d2b79f5) >>> 0;
    let r = Math.imul(t ^ (t >>> 15), t | 1);
    r ^= r + Math.imul(r ^ (r >>> 7), r | 61);
    return ((r ^ (r >>> 14)) >>> 0) / 4294967296;
  };
}

function pick(r, list) {
  return list[Math.floor(r() * list.length)];
}

function randomText(r, alphabet, maxLen) {
  const n = Math.floor(r() * maxLen);
  let s = '';
  for (let i = 0; i < n; i++) s += pick(r, alphabet);
  return s;
}

// ----- Docs corpus: every entry of Pele's tests/corpus, rendered with fixed options. -----
const corpusDir = join(pele, 'tests/corpus');
const byType = new Map();
let total = 0;
for (const file of readdirSync(corpusDir).filter((n) => n.endsWith('.json')).sort()) {
  const inputs = JSON.parse(readFileSync(join(corpusDir, file), 'utf8'));
  inputs.forEach((input, index) => {
    const source = typeof input === 'string' ? input : input.src;
    const type = (() => {
      try {
        return o.detectType(source) ?? 'none';
      } catch {
        return 'error';
      }
    })();
    const a = answer(o, { id: `${file}#${index}`, op: 'render', source, options: RENDER_OPTIONS });
    const row = { id: a.id, source, options: RENDER_OPTIONS, expect: a };
    delete row.expect.id;
    if (!byType.has(type)) byType.set(type, []);
    byType.get(type).push(row);
    total++;
  });
}
for (const [type, rows] of byType) writeJsonl(join(out, 'corpus', `${type}.jsonl`), rows);

// ----- Unit fixtures -----
const units = {};

// YAML: hand-picked cases plus random documents built from YAML pieces.
{
  const cases = [
    'a: 1', 'a: -0.5e3', 'a: 0x1F', 'a: -0o17', 'a: 0b101', 'a: .inf', 'a: -.Inf', 'a: .nan', 'a: ~', 'a: null', 'a: True',
    'a: 01', 'a: 1.', 'a: .5', 'a: +7', 'a: "x\\ty\\u00e9\\U0001F600"', "a: 'it''s'", 'a: "\\xZZ"', 'a: "\\q"', 'a: "\\uD83D\\uDE00"',
    'a: [1, 2, [3, {b: c}]]', 'a: [1 2]', '{a: 1 b: 2}', 'a: {x: , y}', 'a: [1, , 2]', 'a: [1,\n  2,\n  3]', 'a: {b: [1,\n 2]}',
    'a: |\n  one\n  two\n', 'a: >\n  one\n  two\n\n  three\n', 'a: |-\n  x\n', 'a: |+\n  x\n\n', 'a: |2\n   x\n', 'a: >-\n  a\n   b\n  c\n',
    'a: plain\n  continued\n\n  after blank', 'a:\n  - x\n  - y: 1\n    z: 2\n  -\n    - deep', '- a\n- b', '-\n  - a', '- a: 1\n  b: 2',
    'a: 1\n  b: 2', 'a:\n    b: 1\n  c: 2', '\ta: 1', 'a: 1\n\tb: 2', 'a: 1 # comment', "a: 'x' # c", 'a: "x" junk', 'a: [1, 2] junk',
    'a: [1, 2', 'a: "open', "a: 'open", '"quoted key": 1', "'k''s': 2", 'key with spaces: 3', '? complex', 'a: b: c',
    '---\na: 1\n...', '---\na: 1\n---\nb: 2', 'a: &x 1', 'a: *x', 'a: !tag x', '1: one\n0: zero\nb: two', '__proto__: 1',
    'a:\n  b:\n    c:\n      d: 1', 'a: ' + '['.repeat(70) + ']'.repeat(70), 'a: ' + '['.repeat(60) + ']'.repeat(60), '标题: 中文 # 注释',
    'a: "line1\n  line2\n\n  line3"', "a: 'x\n\n  y'", 'a: x\n#comment\nb: y', 'a:\n\nb: 1', '', '   ', '# only comment',
    'a: :x', 'a: x:y', 'a: x: y', 'a: -', 'a: - b', '-1: x', '- - a', 'a: ""', "a: ''", 'a: [a, "b", \'c\', {d: e}]',
    'config:\n  pie:\n    textPosition: 0.5\n  themeVariables:\n    pieOuterStrokeWidth: "5px"', 'a: {b: 1, b: 2}', 'a: 1\na: 2',
  ];
  const r = rng(7);
  const pieces = ['a', 'b', 'key', ':', ': ', ' ', '  ', '\n', '\n  ', '- ', '[', ']', '{', '}', ',', '"', "'", '#', '|', '>', '1', '-2.5', 'x y', 'é', '中', '\\', 'null', 'true', '~', '0x10', '\t'];
  for (let i = 0; i < 400; i++) cases.push(randomText(r, pieces, 14));
  units.yaml = cases.map((source, i) => {
    const a = answer(o, { id: i, op: 'yaml', source });
    return { source, expect: a.ok ? { ok: true, value: a.value } : { ok: false, message: a.error.message } };
  });
}

// URLs: the bypasses sanitize-url guards against, and ordinary addresses.
{
  const cases = [
    '', ' ', 'javascript:alert(1)', 'JaVaScRiPt:alert(1)', '  javascript:alert(1)', 'java\u0000script:alert(1)', '&#106;avascript:alert(1)',
    '&#x6A;avascript:alert(1)', '%6A%61vascript:alert(1)', '%256A%2561vascript:alert(1)', 'data:text/html,<script>', 'vbscript:msgbox',
    'java\\tscript:alert(1)', 'java%5Ctscript:alert(1)', '&#0000106&#0000097vascript:alert(1)', 'jav&#x09;ascript:alert(1)',
    'javascript&colon;alert(1)', '&newline;javascript:alert(1)', 'https://Example.com/a b', 'HTTPS://EXAMPLE.COM/', 'http://%zz',
    'https://example.com:8080/x', 'http://a.com/x:y', 'mailto:a@b.c', 'tel:+1', 'ftp://x', 'file:///etc/passwd', '//share/x',
    '\\\\share\\x', '/relative', './x', '../x', '#anchor', '?q=1', 'plain', 'www.example.com', 'https://例子.测试/路径',
    'https://example.com/%E4%B8%AD', 'https://user:pw@example.com', 'http://[::1]:80/', 'http://1.2.3.4', 'https://x?a=1&b=2',
    'line1\nhttps://x', 'x\njavascript:alert(1)', '\u2028javascript:alert(1)', '%', '%E4%B8', 'http://x/%', '&#55357;&#56832;',
    'https://x\\y', ' https://x ', 'ja\u200bvascript:alert(1)', '\ufeffjavascript:alert(1)', 'obsidian://open?vault=a',
  ];
  const r = rng(11);
  const alphabet = ['a', 'j', 's', ':', '/', '%', '2', '5', '6', 'A', '&', '#', ';', 'x', '1', '0', '\\', 't', 'n', ' ', '.', 'h', 'p', 'é', '\t', '\u0000'];
  for (let i = 0; i < 300; i++) cases.push(randomText(r, alphabet, 18));
  units.url = cases.map((source, i) => {
    const a = answer(o, { id: i, op: 'url', source });
    return { source, expect: a.ok ? { ok: true, value: a.value, safe: a.safe } : { ok: false, message: a.error.message } };
  });
}

// Entity decoding.
{
  const cases = ['a &amp; b', '&#35;&#x41;', '&unknown;', '&#0;', '&#55296;', '&#1114112;', '&#128512;', '&&amp;', 'ﬂ°quot¶ß', 'ﬂ°°35¶ßx', 'ﬂ°nope¶ß', 'ﬂ°°0¶ß', 'ﬂ°a¶x ﬂ°amp¶ß', 'ﬂ°°¶ß', 'ﬂ°°1.5¶ß'];
  units.entities = cases.map((source) => ({ source, expect: answer(o, { op: 'entities', source }).value }));
}

// Coordinates written by num().
{
  const r = rng(3);
  const values = [0, -0, 0.004, -0.004, 0.005, -0.005, 1.005, -1.005, 2.345, -2.345, 1e10, 1e11, -1e11, 99999999999.995, 0.125, -0.125, 1 / 3, NaN, Infinity, -Infinity];
  for (let i = 0; i < 2000; i++) values.push((r() - 0.5) * Math.pow(10, Math.floor(r() * 12) - 4));
  for (let i = 0; i < 500; i++) values.push(Math.round((r() - 0.5) * 20000) / 200);
  units.num = values.map((v) => ({ value: Number.isFinite(v) ? v : { $number: String(v) }, expect: o.num(v) }));
}

// Math functions, compared bit for bit.
{
  const r = rng(5);
  const sample = () => (r() - 0.5) * Math.pow(10, Math.floor(r() * 10) - 3);
  const fns = { sin: 1, cos: 1, tan: 1, atan: 1, asin: 1, acos: 1, exp: 1, log: 1, log10: 1, log2: 1, atan2: 2, pow: 2, hypot: 2 };
  const rows = [];
  for (const [fn, arity] of Object.entries(fns)) {
    const args = [];
    for (let i = 0; i < 400; i++) {
      let a = sample();
      if (fn === 'asin' || fn === 'acos') a = r() * 2 - 1;
      if (fn === 'log' || fn === 'log10' || fn === 'log2') a = Math.abs(a);
      if (fn === 'pow') args.push([Math.abs(a), (r() - 0.5) * 8]);
      else args.push(arity === 1 ? [a] : [a, sample()]);
    }
    for (const a of [[0], [Math.PI], [Math.PI / 2], [1e300]]) if (arity === 1) args.push(a);
    if (fn === 'pow') args.push([2, 0.5], [0.5, 3], [0.99, 7], [10, -3], [-2, 3], [1, Infinity], [0, 0]);
    if (fn === 'hypot') args.push([3, 4], [0, 0], [1e-300, 1e-300], [1e300, 1e300]);
    for (const a of args) rows.push({ fn, args: a.map((x) => (Number.isFinite(x) ? x : { $number: String(x) })), expect: bits(Math[fn](...a)) });
  }
  units.math = rows;
}

// Label layout, compared span by span.
{
  const r = rng(13);
  const cases = [
    'Hello', '  spaced  ', 'a<br/>b<BR >c\\nd\\Ne\nf', '<b>bold</b> and <i>it</i>', '**bold** _it_ snake_case', '__x__ *y* ', 'a * b * c',
    '这是一个很长的中文标签用于测试换行', '（中文）。测试、句子！', 'mixed 中文 and English words here', 'fa:fa-car drive', '&amp; &lt;tag&gt;',
    'ﬂ°quot¶ßquoted', 'long words without any spaces at all in this label', '日本語のテキストを折り返す', 'emoji 😀 test 😀😀',
    '<span class="x">a</span>', '<b>unclosed', '</b>closing', '<em>em</em><strong>strong</strong>', '  \n  indented\n    more\n',
    'a\u3000b\u3000c', 'x'.repeat(80), 'a b c d e f g h i j k l m n o p q r s t u v w x y z', '<br>', '\\n', '`code`',
  ];
  const alphabet = ['a', 'b', ' ', '  ', '中', '文', '。', '（', '）', '<b>', '</b>', '<i>', '*', '**', '_', '<br>', '\\n', '\n', 'word', 'é', '&amp;', 'fa:fa-x', '😀', '\u3000', ','];
  for (let i = 0; i < 300; i++) cases.push(randomText(r, alphabet, 16));
  const rows = [];
  cases.forEach((source, i) => {
    for (const markdown of [false, true]) {
      const max = [4000, 120, 60][i % 3];
      const base = i % 4 === 0 ? 1 : 0;
      const a = answer(o, { op: 'label', source, markdown, size: 16, max, base });
      rows.push({ source, markdown, size: 16, max, base, expect: a.label });
    }
  });
  units.label = rows;
}

// Preprocessing: front matter, directives, comments, entity codes.
{
  const cases = [
    'pie\n  "a": 1', '---\ntitle: Hello\nconfig:\n  pie:\n    textPosition: 0.5\n---\npie\n  "a": 1\n', '---\ntitle: x\n---', '  ---\n  title: Indented\n  ---\npie',
    '%%{init: {\'pie\': {\'textPosition\': 0.3}}}%%\npie\n', '%%{wrap}%%\ngraph TD', '%%{init: {bad json}}%%\npie', 'pie\r\n%% a comment\r\n  "a": 1\r\n%%\r\n',
    '\n\n  %% c\npie', 'graph TD\nA["<span class="x">a</span>"]', 'graph TD\nA[#quot;x#59;]', 'style A fill:#f9f;', 'classDef c fill:#f9f,stroke:#333;',
    '---\ntitle: [a, b]\ndisplayMode: compact\n---\ngantt', '---\nconfig: {a: 1}\n---\n%%{init: {"a": 2, "b": {"c": 1}}}%%\npie', '---\nbad: [\n---\npie',
    '%%{initialize: {"x": 1}}%%pie', '%%{init: {"a": 1}}%% %%{init: {"a": 2}}%%\npie', 'pie %% trailing', 'pie\n%%{ notdirective\n', '\u2028%% x\npie',
    '---\n---\npie', '---\na: 1\n---\n---\nb: 2\n---\npie', 'lifestyle: #ccc;', '#1; #name; #9x; ##;',
  ];
  units.preprocess = cases.map((source) => {
    const a = answer(o, { op: 'preprocess', source });
    return { source, expect: a.ok ? { ok: true, text: a.text, title: a.title, config: a.config, withComments: a.withComments, lineOffset: a.lineOffset, encoded: a.encoded } : { ok: false, message: a.error.message, code: a.error.code } };
  });
}

// Author styles.
{
  const cases = [
    ['fill:#f9f', 'stroke:#333', 'stroke-width:4px', 'color:red', 'font-weight:bold'],
    ['fill:url(#x)', 'stroke: expression (1)', 'color:red}', 'fill:javascript:alert(1)'],
    ['font-size:20px', 'font-weight: 700 !IMPORTANT'], ['font-size: 2em', 'font-style:italic', 'opacity:0.5'], [':x', 'nocolon', 'fill:'],
    ['FILL:Red', 'Stroke-Dasharray: 5 5', 'rx:3', 'ry:2'], ['font-weight:600', 'font-weight:lighter'], ['fill:#abc', 'fill:#def'],
  ];
  units.style = cases.map((declarations) => ({ declarations, expect: o.resolveStyle(declarations) }));
}

for (const [name, rows] of Object.entries(units)) writeJsonl(join(out, 'unit', `${name}.jsonl`), rows);

function bits(x) {
  const b = new DataView(new ArrayBuffer(8));
  b.setFloat64(0, x);
  return b.getBigUint64(0).toString(16).padStart(16, '0');
}

console.log(JSON.stringify({ corpus: total, types: [...byType.keys()].length, units: Object.fromEntries(Object.entries(units).map(([k, v]) => [k, v.length])) }));
