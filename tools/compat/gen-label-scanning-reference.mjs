// Independent labels and the combined icon/measurement callback order from pinned Pele.
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { pathToFileURL } from 'node:url';
import { gzipSync } from 'node:zlib';
import { build } from '../../.research/pele/node_modules/esbuild/lib/main.js';
const upstreamCommit = JSON.parse(readFileSync('tests/upstream/inventory.json')).commit;
if (execFileSync('git', ['-C', '.research/pele', 'rev-parse', 'HEAD'], { encoding: 'utf8' }).trim() !== upstreamCommit)
  throw Error('Unexpected reference revision');
execFileSync('git', ['-C', '.research/pele', 'diff', '--exit-code', 'HEAD', '--', 'src']);
mkdirSync('target/compat', { recursive: true });
const outfile = 'target/compat/label-scanning-reference.mjs';
await build({ stdin: { contents: `
export {layoutLabel} from './.research/pele/src/text/label.ts';
export {metricsMeasurer} from './.research/pele/src/text/measurer.ts';
export {withIcons} from './.research/pele/src/text/icons.ts';
`, resolveDir: process.cwd() }, bundle: true, format: 'esm', platform: 'node', outfile });
const o = await import(pathToFileURL(`${process.cwd()}/${outfile}`).href);
const view = new DataView(new ArrayBuffer(8));
const bits = n => { view.setFloat64(0, n); return view.getBigUint64(0).toString(16).padStart(16, '0'); };
const freeze = v => typeof v === 'number' ? { $bits: bits(v) } : Array.isArray(v) ? v.map(freeze)
  : v && typeof v === 'object' ? Object.fromEntries(Object.entries(v).map(([k, x]) => [k, freeze(x)])) : v;
const sources = ['', '<b></b>', 'a<foo>b</foo>c', '<B>AV</B><I>中文😀</I>', 'fa<x>:fa-car', '<b>fa<x>:fa-car</b>',
  'fa:fa-car<x>fa:fa-car', 'fa:fa-missing<x> fa:fa-car', '&am<x>p;', 'ﬂ<x>°lt¶ß', '<foo><bar/>AV</foo>',
  'AV <x data="v">AV</x> AV', '**AV** <x>_中_</x>', 'a<br/>b\\Nc<BR \u3000/>d\n', '<br/><br/>',
  'one\n\n', 'a<br b>b', '<x <b>AV</b>>tail', '<b>AV<i>中</i> tail</b>', 'a\u0000<x>b',
  'AV_<x>tail', '😀<b>é\u0301</b> &amp;', '<STRONG>fa:fa-car</STRONG>', 'fa:fa-missing <b></b> tail'];
const atoms = ['AV', '中文', '😀', '&amp;', ' ', '\t', '\u3000', '<x>', '</x>', '<b>', '</b>', '<i>', '</i>',
  '*', '_', '**', '__', '<br/>', '\n', '\\n', 'fa:fa-car', 'fa:fa-missing', 'fa<x>:fa-car', 'ﬂ°lt¶ß'];
let state = 0x1abe17;
const next = () => state = (Math.imul(state, 1664525) + 1013904223) >>> 0;
for (let i = 0; i < 64; i++) {
  let s = '';
  for (let j = 0, count = 1 + next() % 12; j < count; j++) s += atoms[next() % atoms.length];
  sources.push(s);
}
const cases = [];
for (const source of sources) for (const markdown of [false, true]) for (const icons of [false, true])
  for (const size of [16, -0.1]) for (const max of [0, 40, Infinity]) for (const base of [0, 3]) for (const mode of ['metrics', 'custom']) {
    const trace = [];
    const resolver = name => { trace.push(['icon', name]); return name === 'fa:fa-car' ? '<path/>' : null; };
    const measurer = { width(text, fontSize, style) {
      trace.push(['width', text, bits(fontSize), style]);
      const w = o.metricsMeasurer.width(text, fontSize, style);
      return mode === 'custom' ? w - (text.includes('AV') ? fontSize * 0.75 : 0) : w;
    } };
    const expected = o.withIcons(icons ? resolver : undefined, () => freeze(o.layoutLabel(source, markdown, measurer, size, max, base)));
    cases.push({ source, markdown, icons, size: bits(size), max: bits(max), base, mode, expected, trace });
  }
const hash = p => createHash('sha256').update(readFileSync(p)).digest('hex');
writeFileSync('tests/fixtures/unit/label-scanning-reference.json.gz', gzipSync(JSON.stringify({ upstreamCommit,
  seed: '0x1abe17', generatedSources: 64, sourceCases: sources.length,
  scope: 'Supplementary valid-Rust-string label values and exact combined callback order; not an original-test port.',
  sourceHashes: { label: hash('.research/pele/src/text/label.ts'), icons: hash('.research/pele/src/text/icons.ts') }, cases }), { level: 9 }));
console.log(JSON.stringify({ sources: sources.length, cases: cases.length }));
