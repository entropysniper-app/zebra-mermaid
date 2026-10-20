// Independent literal-label, break-label and SVG references from the pinned clean source.
import {readFileSync, writeFileSync, mkdirSync} from 'node:fs';
import {execFileSync} from 'node:child_process';
import {pathToFileURL} from 'node:url';
import {gzipSync} from 'node:zlib';
import {build} from '../../.research/pele/node_modules/esbuild/lib/main.js';

const upstreamCommit = JSON.parse(readFileSync('tests/upstream/inventory.json')).commit;
if (execFileSync('git', ['-C', '.research/pele', 'rev-parse', 'HEAD'], {encoding: 'utf8'}).trim() !== upstreamCommit) throw Error('Unexpected reference revision');
execFileSync('git', ['-C', '.research/pele', 'diff', '--exit-code', 'HEAD', '--', 'src']);
mkdirSync('target/compat', {recursive: true});
const outfile = 'target/compat/literal-label-reference.mjs';
await build({stdin: {contents: `
export {literalLabel, breakLabel, leftLabelSvg} from './.research/pele/src/diagrams/common/literal-label.ts';
export {metricsMeasurer} from './.research/pele/src/text/measurer.ts';
`, resolveDir: process.cwd()}, bundle: true, format: 'esm', platform: 'node', outfile});
const o = await import(pathToFileURL(`${process.cwd()}/${outfile}`).href);
const view = new DataView(new ArrayBuffer(8));
const bits = value => { view.setFloat64(0, value); return view.getBigUint64(0).toString(16).padStart(16, '0'); };
const freeze = value => {
  if (typeof value === 'number') return {$bits: bits(value)};
  if (Array.isArray(value)) return value.map(freeze);
  if (value && typeof value === 'object') return Object.fromEntries(Object.entries(value).map(([k, v]) => [k, freeze(v)]));
  return value;
};
const tracked = mode => {
  const trace = [];
  return {trace, width(text, size, style) {
    trace.push([text, bits(size), style]);
    const width = o.metricsMeasurer.width(text, size, style);
    return mode === 'custom' ? width - (text.includes('AV') ? size * 0.75 : 0) : width;
  }};
};
const sources = ['', '\n', '   ', 'AV AV AV', 'alpha beta  gamma\tdelta', '\u3000中文（测试）。好\u3000', '😀e\u0301AV😀',
  'one\ntwo\n', '\u0000\u0001<&"\u007f', 'a\r\nb\r', '(中文) foo', 'AV '.repeat(64), 'filename_'.repeat(32), '中文😀e\u0301 '.repeat(32)];
const literal = [];
for (const text of sources) for (const size of [0.1, 16, 23.5]) for (const room of [0, 40, 120, Infinity]) for (const style of [0, 1, 4]) for (const mode of ['metrics', 'custom']) {
  const m = tracked(mode);
  literal.push({text, size: bits(size), room: bits(room), style, mode, expected: freeze(o.literalLabel(text, m, size, room, style)), trace: m.trace});
}
let seed = 0x17e2b6;
const random = () => (seed = (Math.imul(seed, 1664525) + 1013904223) >>> 0);
const labels = [];
for (let i = 0; i < 128; i++) {
  const size = [0, 0.1, 16, 23.5][i % 4], lineHeight = Math.round(size * 1.5);
  const lines = [], widths = [];
  for (let line = 0; line < i % 4; line++) {
    const spans = [];
    for (let j = 0; j < (i + line) % 5; j++) {
      const text = sources[random() % 11], style = random() % 8;
      const icon = (i + j) % 11 === 0 ? 'fa:fa-test' : undefined;
      spans.push({text, style, width: icon ? size * 1.1 : o.metricsMeasurer.width(text, size, style), ...(icon ? {icon} : {})});
    }
    lines.push(spans); widths.push(spans.reduce((sum, s) => sum + s.width, 0));
  }
  labels.push({lines, widths, width: Math.max(0, ...widths), height: lines.length * lineHeight, size, lineHeight});
}
// Isolated invisible text, empty styled spans and icons exercise SVG emission boundaries.
for (const text of ['', '\u0000\u0001\u007f', '\t\r\n', '<&"', 'AV😀']) for (const style of [0, 1, 2, 3]) {
  const width = o.metricsMeasurer.width(text, 16, style);
  labels.push({lines: [[{text, style, width}]], widths: [width], width, height: 24, size: 16, lineHeight: 24});
}
const broken = [], svg = [];
for (const label of labels) {
  for (const room of [0, 40, 120, Infinity]) for (const mode of ['metrics', 'custom']) {
    const m = tracked(mode);
    broken.push({label: freeze(label), room: bits(room), mode, expected: freeze(o.breakLabel(label, m, room)), trace: m.trace});
  }
  for (const attrs of ['', ' class="label"', ' font-weight="bold" font-style="italic"']) {
    svg.push({label: freeze(label), x: bits(-12.345), top: bits(0.1), attrs, expected: o.leftLabelSvg(label, -12.345, 0.1, attrs)});
  }
}
writeFileSync('tests/fixtures/unit/literal-label-regression.json.gz', gzipSync(JSON.stringify({upstreamCommit, seed: '0x17e2b6', literal, broken, svg}), {level: 9}));
console.log(JSON.stringify({literal: literal.length, broken: broken.length, svg: svg.length}));
