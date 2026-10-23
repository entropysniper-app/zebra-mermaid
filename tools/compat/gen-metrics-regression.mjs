import {mkdirSync as ensureReleaseDir} from 'node:fs';
ensureReleaseDir('tests/fixtures/pele-1.9.0/unit',{recursive:true});
// Freeze the pinned default text measurer's exact binary64 results.
import {readFileSync, writeFileSync} from 'node:fs';
import {execFileSync} from 'node:child_process';
import {gzipSync} from 'node:zlib';
import {loadOracle} from './oracle.mjs';

const upstreamCommit = JSON.parse(readFileSync('tests/upstream/inventory.json')).commit;
if (execFileSync('git', ['-C', '.research/pele', 'rev-parse', 'HEAD'], {encoding: 'utf8'}).trim() !== upstreamCommit) {
  throw Error('Unexpected reference revision');
}
execFileSync('git', ['-C', '.research/pele', 'diff', '--exit-code', 'HEAD', '--', 'src']);
const oracle = await loadOracle();
const sources = ['', 'AV '.repeat(1024), 'WWWiii000.'.repeat(256), '中文😀e\u0301\u200b'.repeat(128)];
for (let code = 0; code < 128; code++) sources.push(String.fromCodePoint(code));
const boundaries = [0x300, 0x36f, 0x1100, 0x115f, 0x200b, 0x200f, 0x2e80, 0xa4cf,
  0xac00, 0xd7a3, 0xf900, 0xfaff, 0xfe00, 0xfe0f, 0xfe30, 0xfe4f, 0xfeff,
  0xff00, 0xff60, 0xffe0, 0xffe6, 0x10000, 0x1f600, 0x10ffff];
for (const boundary of boundaries) for (const delta of [-1, 0, 1]) {
  const code = boundary + delta;
  if (code >= 0 && code <= 0x10ffff && (code < 0xd800 || code > 0xdfff)) sources.push(String.fromCodePoint(code));
}
let seed = 0x51a710;
const random = () => (seed = (Math.imul(seed, 1664525) + 1013904223) >>> 0);
const scalar = () => {
  const value = random() % (0x110000 - 0x800);
  return value < 0xd800 ? value : value + 0x800;
};
for (let i = 0; i < 4096; i++) sources.push(String.fromCodePoint(scalar()));
for (let i = 0; i < 128; i++) {
  let text = '';
  for (let j = 0; j < 64; j++) text += String.fromCodePoint(j % 3 === 0 ? random() % 128 : scalar());
  sources.push(text);
}
const sizes = [0, 0.1, 1, 16, 23.5, 1000, -1.25];
const styles = [0, 1, 2, 3, 4, 5, 6, 7, 128, 255];
const view = new DataView(new ArrayBuffer(8));
const bits = value => {
  view.setFloat64(0, value);
  return view.getBigUint64(0).toString(16).padStart(16, '0');
};
const widths = sources.map(source => styles.flatMap(style => sizes.map(size => bits(oracle.metricsMeasurer.width(source, size, style)))));
writeFileSync('tests/fixtures/pele-1.9.0/unit/metrics-regression.json.gz', gzipSync(JSON.stringify({
  upstreamCommit, seed: '0x51a710', randomScalars: 4096, randomStrings: 128,
  sizes: sizes.map(bits), styles, sources, widths,
}), {level: 9}));
console.log(JSON.stringify({sources: sources.length, comparisons: sources.length * styles.length * sizes.length}));
