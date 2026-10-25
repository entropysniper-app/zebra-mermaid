// Correctness-only replay of all 39 extended workloads and the unmodified original bench inputs.
// The benchmark drivers' --verify-only mode does no calibration, sampling or timing.
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { createHash } from 'node:crypto';
import { isDeepStrictEqual } from 'node:util';

process.env.TZ = 'UTC';
const root = fileURLToPath(new URL('../..', import.meta.url));
process.chdir(root);
const upstream = resolve(root, '.research/pele');
const { commit } = JSON.parse(readFileSync('tests/upstream/inventory.json'));
const { version } = JSON.parse(readFileSync(resolve(upstream, 'package.json')));
const execute = (cmd, args, cwd = root) => execFileSync(cmd, args, { cwd, encoding: 'utf8', maxBuffer: 128 * 1024 * 1024 });
if (execute('git', ['rev-parse', 'HEAD'], upstream).trim() !== commit) throw Error('Unexpected Pele revision');
execute('git', ['diff', '--exit-code', 'HEAD', '--', 'src', 'scripts'], upstream);
execute(process.execPath, ['scripts/build.mjs'], upstream);
execute(process.execPath, ['tools/bench/cases.mjs']);
const { render } = await import(pathToFileURL(resolve(upstream, 'dist/pele.js')));
const out = resolve(root, `target/alignment-${version}`);
mkdirSync(out, { recursive: true });
const sha = value => createHash('sha256').update(value).digest('hex');
const errors = [];
const summary = { version, commit, runtime: { node: process.version, rustc: execute('rustc', ['--version']).trim() },
  method: 'Exact SVG strings and dimensions for successes; exact code, message, line and column for rejections. Unchanged inputs/options/order. Correctness-only execution; no timings measured.', groups: [] };

const casesBytes = readFileSync('target/bench/cases.json');
const cases = JSON.parse(casesBytes);
const driver = resolve(root, 'target/release/examples/bench');
execute(driver, [resolve(out, 'rust-workloads.json'), '--verify-only']);
const actual = JSON.parse(readFileSync(resolve(out, 'rust-workloads.json')));
if (actual.enabledTypes.length !== 33) throw Error('All 33 types must be enabled');
summary.extended = { groups: cases.length, inputs: 0, successes: 0, rejections: 0, matched: 0, inputSha256: sha(casesBytes), driverSha256: sha(readFileSync(driver)) };
for (const [i, group] of cases.entries()) {
  const rows = [];
  if (actual.cases[i].name !== group.name || actual.cases[i].verification.length !== group.inputs.length) throw Error('Workloads differ');
  for (const [j, source] of group.inputs.entries()) {
    let expected;
    try {
      const result = render(source, { now: 1792368000000 });
      expected = { ok: true, svg: result.svg, width: result.width, height: result.height };
      summary.extended.successes++;
    } catch (e) {
      expected = { ok: false, code: e.code, message: e.message, line: e.line ?? 0, column: e.column ?? 0 };
      summary.extended.rejections++;
    }
    const observed = actual.cases[i].verification[j];
    const matched = isDeepStrictEqual(observed, expected);
    if (!matched) errors.push(`${group.name}#${j}`);
    summary.extended.inputs++;
    summary.extended.matched += Number(matched);
    rows.push({ index: j, sourceSha256: sha(source), matched, ok: expected.ok,
      ...(expected.ok ? { width: expected.width, height: expected.height, svgSha256: sha(expected.svg) } : { error: expected }) });
  }
  summary.groups.push({ name: group.name, rows });
}

const script = readFileSync(resolve(upstream, 'scripts/bench.mjs'), 'utf8');
const prefix = script.slice(0, script.indexOf('function time(')).replace("'../dist/pele.js'", JSON.stringify(pathToFileURL(resolve(upstream, 'dist/pele.js')).href));
let original;
try {
  process.chdir(upstream);
  original = (await import('data:text/javascript;base64,' + Buffer.from(prefix + '\nexport { cases };').toString('base64'))).cases;
} finally { process.chdir(root); }
const input = resolve(out, 'original-workloads.json');
writeFileSync(input, JSON.stringify(original.map(([name, inputs]) => ({ name, inputs }))));
const originalDriver = resolve(root, 'target/release/examples/bench_upstream');
execute(originalDriver, [input, resolve(out, 'rust-original-workloads.json'), '--verify-only']);
const originalActual = JSON.parse(readFileSync(resolve(out, 'rust-original-workloads.json')));
summary.original = { groups: original.length, inputs: 0, matched: 0, scriptSha256: sha(script), inputSha256: sha(readFileSync(input)), driverSha256: sha(readFileSync(originalDriver)) };
for (const [i, [name, inputs]] of original.entries()) for (const [j, source] of inputs.entries()) {
  const result = render(source);
  const expected = { svg: result.svg, width: result.width, height: result.height };
  const matched = isDeepStrictEqual(originalActual.verification[i][j], expected);
  summary.original.inputs++;
  summary.original.matched += Number(matched);
  if (!matched) errors.push(`original ${name}#${j}`);
}
summary.errors = errors;
summary.success = errors.length === 0;
writeFileSync(resolve(out, 'workloads-validation.json'), JSON.stringify(summary, null, 2) + '\n');
console.log(JSON.stringify({ extended: summary.extended, original: summary.original, success: summary.success, errors }));
if (!summary.success) process.exitCode = 1;
