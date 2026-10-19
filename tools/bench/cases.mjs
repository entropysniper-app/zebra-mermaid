// Writes target/bench/cases.json: Pele's benchmark inputs (scripts/bench.mjs: the flowchart
// documentation examples and generated trees and meshes) and every documentation corpus file,
// grouped by diagram type. Both engines are timed on exactly these inputs.
import { readFileSync, readdirSync, writeFileSync, mkdirSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

const here = dirname(fileURLToPath(import.meta.url));
const root = join(here, '../..');
const corpus = join(root, '.research/pele/tests/corpus');

function chain(n, fan) {
  let s = 'flowchart TD\n';
  for (let i = 1; i < n; i++) s += `  n${Math.floor((i - 1) / fan)}[Node ${Math.floor((i - 1) / fan)}] --> n${i}[Node ${i}]\n`;
  return s;
}

function mesh(n, extra) {
  let s = 'flowchart TD\n';
  let seed = 7;
  const rnd = () => (seed = (Math.imul(seed, 1664525) + 1013904223) >>> 0) / 4294967296;
  for (let i = 1; i < n; i++) s += `  n${Math.floor(rnd() * i)} --> n${i}\n`;
  for (let i = 0; i < extra; i++) s += `  n${Math.floor(rnd() * n)} -->|label ${i}| n${Math.floor(rnd() * n)}\n`;
  return s;
}

const read = (file) => JSON.parse(readFileSync(join(corpus, file), 'utf8')).map((x) => (typeof x === 'string' ? x : x.src));
const cases = [
  { name: 'docs: all flowchart examples', inputs: read('flowchart-docs.json') },
  { name: 'tree: 50 nodes', inputs: [chain(50, 3)] },
  { name: 'mesh: 50 nodes, 20 extra labelled edges', inputs: [mesh(50, 20)] },
  { name: 'tree: 500 nodes', inputs: [chain(500, 3)] },
  { name: 'mesh: 500 nodes, 200 extra labelled edges', inputs: [mesh(500, 200)] },
];
for (const file of readdirSync(corpus).filter((f) => f.endsWith('.json')).sort()) {
  cases.push({ name: `corpus: ${file.replace(/\.json$/, '')}`, inputs: read(file) });
}
mkdirSync(join(root, 'target/bench'), { recursive: true });
writeFileSync(join(root, 'target/bench/cases.json'), JSON.stringify(cases));
console.log(`wrote ${cases.length} cases`);
