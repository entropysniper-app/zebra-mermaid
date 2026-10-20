// Independent expectations from the pinned upstream font-style function.
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { pathToFileURL } from 'node:url';
import { gzipSync } from 'node:zlib';
import { build } from '../../.research/pele/node_modules/esbuild/lib/main.js';

const upstreamCommit = JSON.parse(readFileSync('tests/upstream/inventory.json')).commit;
if (execFileSync('git', ['-C', '.research/pele', 'rev-parse', 'HEAD'], { encoding: 'utf8' }).trim() !== upstreamCommit)
  throw Error('Reference revision differs');
execFileSync('git', ['-C', '.research/pele', 'diff', '--exit-code', 'HEAD', '--', 'src']);
mkdirSync('target/compat', { recursive: true });
const outfile = 'target/compat/font-list-reference.mjs';
await build({ entryPoints: ['.research/pele/src/svg/theme.ts'], bundle: true, format: 'esm', platform: 'node', outfile });
const { fontStyle } = await import(pathToFileURL(`${process.cwd()}/${outfile}`).href);
const inputs = ['', 'Arial', 'sans-serif', '""', "''", '.', '-', '_', '1', 'a,', ',a', '"x', "'x\"", 'a.. ',
  '思源黑体, Roboto', '𝔘nicode, Ⅷ, 𝟠', 'é', 'é', 'url(x)', 'Arial; color:red', 'A\\B', 'a\u0000b', '"a\u0000b"',
  '"a\rb"', '"a\nb"', "'a\"b'", '"a\'b"', '"a,b"', 'A\tB', 'A B', ' A , B ', 'a,,b'];
for (let c = 0; c < 128; c++) {
  const ch = String.fromCodePoint(c);
  inputs.push(ch, `a${ch}b`, `${ch}Arial`, `Arial${ch}`, `"a${ch}b"`, `'a${ch}b'`);
}
const spaces = ['\t', '\v', '\f', ' ', '\u00a0', '\u1680', ...Array.from({ length: 11 }, (_, i) => String.fromCodePoint(0x2000 + i)),
  '\u202f', '\u205f', '\u3000', '\ufeff', '\n', '\r', '\u2028', '\u2029'];
for (const space of spaces) inputs.push(`${space}Arial${space}`, `Arial${space},${space}serif`, `"a${space}b"`, `a${space}b`);
let state = 0x16f017;
const next = () => state = (Math.imul(state, 1664525) + 1013904223) >>> 0;
const names = ['Arial', 'sans-serif', '思源黑体', 'École', 'Ⅷ', '𝔘nicode', '𝟠', '_a', '-A', 'A.B', 'Noto Sans SC'];
const atoms = [...names, ...spaces, '"', "'", ',', '\\', ';', ':', '@', '(', ')', '<', '>', '{', '}', '.', '\u0301', '😀', '\u0000'];
for (let i = 0; i < 4096; i++) {
  if (i % 2 === 0) {
    const name = names[next() % names.length];
    const other = names[next() % names.length];
    const space = spaces[next() % spaces.length];
    inputs.push(i % 4 === 0 ? `${space}${name},${space}"${other}"${space}` : `'${name}', ${other}`);
  } else {
    let value = '';
    for (let j = 0, n = 1 + next() % 12; j < n; j++) value += atoms[next() % atoms.length];
    inputs.push(value);
  }
}
inputs.push('Arial, '.repeat(8192) + '思源黑体', ' '.repeat(65536), '"' + 'é'.repeat(32768) + '"', 'A'.repeat(65536) + ';');
const fallback = fontStyle(16, undefined, undefined);
const prefix = fallback.slice(0, -'sans-serif)'.length);
const cases = inputs.map(input => ({ input, accepted: fontStyle(16, input, undefined) === prefix + input + ')', style: fontStyle(16, input, input) }));
const value = { upstreamCommit, seed: '0x16f017', generatedCases: 4096,
  scope: 'Supplementary font-list acceptance and exact public font-style output on valid Rust strings; not an original-test port.',
  upstreamThemeSha256: createHash('sha256').update(readFileSync('.research/pele/src/svg/theme.ts')).digest('hex'), cases };
writeFileSync('tests/fixtures/unit/font-list-reference.json.gz', gzipSync(JSON.stringify(value), { level: 9 }));
console.log(JSON.stringify({ cases: cases.length, accepted: cases.filter(c => c.accepted).length, rejected: cases.filter(c => !c.accepted).length }));
