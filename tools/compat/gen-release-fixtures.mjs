// Add a reference for the current release without rewriting historical golden outputs.
import { readFileSync, readdirSync, writeFileSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { fileURLToPath } from 'node:url';
import { resolve } from 'node:path';
import { gzipSync } from 'node:zlib';
import { loadOracle, answer } from './oracle.mjs';

const root = fileURLToPath(new URL('../..', import.meta.url));
const upstream = resolve(root, '.research/pele');
const inventory = JSON.parse(readFileSync(resolve(root, 'tests/upstream/inventory.json')));
const git = (...args) => execFileSync('git', ['-C', upstream, ...args], { encoding: 'utf8' }).trim();
if (git('rev-parse', 'HEAD') !== inventory.commit) throw Error('Unexpected Pele revision');
git('diff', '--exit-code', 'HEAD', '--', 'src', 'tests', 'package.json');
const { version } = JSON.parse(readFileSync(resolve(upstream, 'package.json')));
const oracle = await loadOracle();
const options = { idPrefix: 'baseline', now: Date.parse('2026-10-19T00:00:00Z') };
const rows = [];
const sourceFiles = [];
for (const name of readdirSync(resolve(upstream, 'tests/corpus')).filter(f => f.endsWith('.json')).sort()) {
  const bytes = readFileSync(resolve(upstream, 'tests/corpus', name));
  sourceFiles.push({ path: `tests/corpus/${name}`, sha256: createHash('sha256').update(bytes).digest('hex') });
  JSON.parse(bytes).forEach((input, index) => {
    const source = typeof input === 'string' ? input : input.src;
    const id = `${name}#${index}`;
    const expected = answer(oracle, { id, op: 'render', source, options });
    delete expected.id;
    rows.push({ id, source, options, expect: expected });
  });
}
const metadata = { version, commit: inventory.commit, runtime: { node: process.version }, sourceFiles, cases: rows.length,
  method: 'Every original tests/corpus JSON entry, original order, fixed Arial metrics and UTC clock. Expected SVG, dimensions, links and complete PeleError fields come only from the pinned upstream source. Original 1:1 tests are tracked separately.' };
writeFileSync(resolve(root, `tests/fixtures/pele-${version}.jsonl.gz`), gzipSync(rows.map(r => JSON.stringify(r)).join('\n') + '\n', { level: 9 }));
writeFileSync(resolve(root, `tests/fixtures/pele-${version}.metadata.json`), JSON.stringify(metadata, null, 2) + '\n');
console.log(JSON.stringify({ version, commit: inventory.commit, cases: rows.length }));
