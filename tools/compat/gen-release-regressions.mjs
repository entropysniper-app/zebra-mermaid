// Supplemental release comparisons. Expectations come only from the pinned Node source.
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { build } from '../../.research/pele/node_modules/esbuild/lib/main.js';
import { pathToFileURL } from 'node:url';
import { loadOracle, answer } from './oracle.mjs';

const { commit } = JSON.parse(readFileSync('tests/upstream/inventory.json'));
if (execFileSync('git', ['-C', '.research/pele', 'rev-parse', 'HEAD'], { encoding: 'utf8' }).trim() !== commit) throw Error('Unexpected reference revision');
execFileSync('git', ['-C', '.research/pele', 'diff', '--exit-code', 'HEAD', '--', 'src']);
const version = JSON.parse(readFileSync('.research/pele/package.json')).version;
const out = `tests/fixtures/pele-${version}`;
mkdirSync(out, { recursive: true });
const oracle = await loadOracle();
const bodies = [
  'flowchart TD\nA[a#35;b] --> -->',
  'flowchart TD\n  A{a}  \n\n  C --> --> D',
  'flowchart TD\na@{shape: "#35;x"}',
  'agentflow-beta TB\n  a@{ shape: rounded }  \n  b -.-> c',
  'agentflow-beta\na["#35;"] --> b@{shape: bad}',
  'agentflow-beta\na["#35;"] --> b@{shape: [bad}',
  'usecase-beta\nactor A actor B',
  'usecase-beta\nA "b',
  'usecase-beta\nactor A\nA --> B\nstyle Z fill:red',
  'usecase-beta\njson P@{"a":"#35;", "b":}',
  'usecase-beta\nactor "😀#35;" as A\nstyle Z fill:red',
  'treeView-beta\n└─ "bad',
  'treeView-beta\n├── \n',
  'treeView-beta\n└─ "ok"\t"bad',
  'treeView-beta\n└── "ok"\t\t"bad',
  'treeView-beta\n"bad\nhello ─ world\nx\ny',
  'sankey\n\na,b,1\n\nb,c,abc',
  'sankey\n\n\n  a,b\n',
  'sankey\na,b,1\n\nb,c,',
  'sankey\n"a#35;😀",b,1\n\nb,c,-',
  'pie\n "#35;x &amp;y" : -1',
  'radar-beta\n axis a["#35;x &amp;y"], b\n curve c{ b: 1 }',
  'venn-beta\nset "B#x"\nset A\nunion A,"B#35;x"',
  'eventmodeling\ntf 1 evt A\ntf 1 evt B',
  'classDiagram\nclass A {\n a#35;b\n}\n???',
];
const prefixes = ['', '\n\n  ', '---\ntitle: T\n---\n', '%% a comment\n\n', '%%{init:\n{}\n}%%\n', '---\ntitle: T\n---\n%% a comment\n\n'];
const options = { idPrefix: 'release-regression', now: 1792368000000 };
const rows = [];
for (const [bodyIndex, body] of bodies.entries()) for (const [prefixIndex, prefix] of prefixes.entries()) for (const crlf of [false, true]) {
  const source = crlf ? (prefix + body).replaceAll('\n', '\r\n') : prefix + body;
  const id = `position-${bodyIndex}-${prefixIndex}-${crlf ? 'crlf' : 'lf'}`;
  const expect = answer(oracle, { id, op: 'render', source, options });
  delete expect.id;
  rows.push({ id, source, options, expect });
}
{
  const source = '---\ntitle: x\n---\npie\n  "中文" 1\n';
  const id = 'cli-json-error';
  const expect = answer(oracle, { id, op: 'render', source, options });
  delete expect.id;
  rows.push({ id, source, options, expect });
}
for (const type of ['flowchart TD', 'agentflow-beta TB', 'erDiagram']) for (const label of ['java\ufffescript:alert(1)', '\\\\nevil.example', '\t/\\nevil.example', '/\t/evil.example', '👩‍💻']) {
  const source = `${type}\nA["${label}"]:::internal-link`;
  const id = `internal-${rows.length}`;
  const opts = { ...options, linkSchemes: ['about'] };
  const expect = answer(oracle, { id, op: 'render', source, options: opts });
  delete expect.id;
  rows.push({ id, source, options: opts, expect });
}
writeFileSync(`${out}/release-regressions.jsonl`, rows.map(r => JSON.stringify(r)).join('\n') + '\n');

const tokenInputs = {
  gantt: ['a% x\nb'], journey: ['journey\ntitle\nfoo\n'],
  quadrant: ['quadrantChart\né%% x\n', 'quadrantChart\n😀%% x\n'],
  sequence: ['é%% c\n'], timeline: ['timeline\ntitle\nNext\n'], xychart: ['xychart\né%% x\n'],
};
const modules = Object.keys(tokenInputs);
const contents = modules.map((m, i) => `export {tokenize as t${i}} from './.research/pele/src/diagrams/${m}/lexer.ts';`).join('\n');
const bundle = 'target/compat/release-token-regressions.mjs';
await build({ stdin: { contents, resolveDir: process.cwd() }, bundle: true, format: 'esm', platform: 'node', outfile: bundle });
const tokens = await import(pathToFileURL(`${process.cwd()}/${bundle}`));
const lexerReferences = modules.flatMap((m, i) => tokenInputs[m].map(source => ({ diagram: m, source, tokens: tokens[`t${i}`](source) })));
writeFileSync(`${out}/release-regressions.metadata.json`, JSON.stringify({ version, commit, runtime: { node: process.version }, cases: rows.length, method: 'Supplemental full-response and lexer comparisons against pinned upstream only. Original 1:1 tests are mapped separately.', lexerReferences }, null, 2) + '\n');
console.log(JSON.stringify({ version, cases: rows.length, lexerReferences }));
