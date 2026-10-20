// Independent ECMAScript references for decimal output; no Rust-generated expectations.
import {writeFileSync} from 'node:fs';
import {gzipSync} from 'node:zlib';

const view = new DataView(new ArrayBuffer(8));
const inputs = [];
const add = value => {
  view.setFloat64(0, value);
  inputs.push(view.getBigUint64(0));
};
for (const value of [0, -0, NaN, Infinity, -Infinity, Number.MIN_VALUE, Number.MAX_VALUE,
  0.5, 1.25, 2.5, 9.5, 9.95, 99.5, 100.5, 250, 350, 1250, 2250, 2.5e20, 2.5e21,
  1.005, 1e-7, 1e-6, 1e20, 1e21,
  Number.MAX_SAFE_INTEGER, Number.MAX_SAFE_INTEGER + 1]) {
  add(value);
  add(-value);
}
for (const exponent of [-1074, -1022, 1023, ...Array.from({length: 121}, (_, i) => i - 60)]) {
  add(2 ** exponent);
  add(-(2 ** exponent));
}
for (let exponent = -324; exponent <= 308; exponent++) {
  const value = 10 ** exponent;
  add(value);
  add(-value);
  view.setFloat64(0, value);
  const bits = view.getBigUint64(0);
  if (bits > 0n && Number.isFinite(value)) inputs.push(bits - 1n, bits + 1n);
}
let seed = 0x6d0d3en;
for (let i = 0; i < 4096; i++) {
  seed = (6364136223846793005n * seed + 1442695040888963407n) & ((1n << 64n) - 1n);
  inputs.push(seed);
}
const precisions = [1, 2, 6, 12, 17, 20, 100];
const fractions = [0, 2, 6, 20, 100];
const cases = [...new Set(inputs)].map(bits => {
  view.setBigUint64(0, bits);
  const value = view.getFloat64(0);
  return {
    bits: bits.toString(16).padStart(16, '0'),
    string: String(value),
    precision: precisions.map(precision => ({precision, expected: value.toPrecision(precision)})),
    fixed: fractions.map(fraction => ({fraction, expected: value.toFixed(fraction)})),
  };
});
const reference = {runtime: process.version, seed: '0x6d0d3e', randomCases: 4096, precisions, fractions, cases};
writeFileSync('tests/fixtures/unit/js-decimal-regression.json.gz', gzipSync(JSON.stringify(reference), {level: 9}));
console.log(JSON.stringify({cases: cases.length, precisionCases: cases.length * precisions.length,
  fixedCases: cases.length * fractions.length, runtime: process.version}));
