#!/usr/bin/env node
// Freeze the COMPLETE collected suite, including website, browser, skip, todo and each() rows.
import { readFileSync, writeFileSync, readdirSync, mkdirSync, cpSync } from 'node:fs';
import { resolve, join, relative } from 'node:path';
import { fileURLToPath } from 'node:url';
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';

const root = resolve(fileURLToPath(new URL('../..', import.meta.url)));
const upstream = join(root, '.research/pele');
const commit = 'e7f092a4de147cc32210913c20e36eaa5fcd0ef1';
const hash = value => createHash('sha256').update(value).digest('hex');
const git = (...args) => execFileSync('git', ['-C', upstream, ...args], { encoding: 'utf8' }).trim();
if (git('rev-parse', 'HEAD') !== commit) throw new Error('Pele revision differs from the pinned reference');
execFileSync('git', ['-C', upstream, 'diff', '--exit-code', commit, '--', 'tests', 'src', 'website/src', 'vitest.config.ts']);
const reportPath = join(root, 'target/compat/pele-test-run.json');
if (!process.argv.includes('--from-report')) {
  mkdirSync(join(root, 'target/compat'), { recursive: true });
  execFileSync(join(upstream, 'node_modules/.bin/vitest'), ['run', '--includeTaskLocation', `--reporter=${join(root, 'tools/test-parity/reporter.mjs')}`], {
    cwd: upstream, env: { ...process.env, PELE_PARITY_REPORT: reportPath }, stdio: 'inherit',
  });
}
const report = JSON.parse(readFileSync(reportPath, 'utf8'));
if (report.reason !== 'passed' || report.errors.length || report.files.some(f => f.errors.length)) throw new Error('Reference run did not pass or failed collection');
const walk = dir => readdirSync(dir, { withFileTypes: true }).flatMap(e => e.isDirectory() ? walk(join(dir, e.name)) : [join(dir, e.name)]);
const actual = walk(join(upstream, 'tests')).filter(p => /\.(?:test|spec)\.(?:ts|js)$/.test(p)).map(p => relative(upstream, p)).sort();
if (JSON.stringify(actual) !== JSON.stringify(report.files.map(f => f.path).sort())) throw new Error('Collected files do not cover every upstream test file');
const entries = [], files = [], baseline = { passed: 0, skipped: 0, todo: 0 };
for (const file of report.files) {
  const source = readFileSync(join(upstream, file.path));
  if (hash(source) !== file.sourceSha256) throw new Error(`Reference source changed: ${file.path}`);
  files.push({ path: file.path, sha256: file.sourceSha256, tests: file.tests.length });
  for (const test of file.tests) {
    const mode = test.options.mode;
    if (!['run','skip','todo'].includes(mode)) throw new Error(`Unexpected mode: ${mode}`);
    const state = mode === 'todo' ? 'todo' : test.result.state;
    if (!(state in baseline)) throw new Error(`Unexpected reference result: ${state}`);
    baseline[state]++;
    const identity = JSON.stringify([commit, file.path, test.index, test.fullName]);
    entries.push({
      id: hash(identity).slice(0, 24), file: file.path, index: test.index, name: test.name,
      suites: test.ancestry, fullName: test.fullName, location: test.location,
      mode, upstreamResult: state,
      scope: file.path.startsWith('tests/website/') ? 'website' : /^tests\/(?:mount|zoom|browsers|entries)\.test\.ts$/.test(file.path) || file.path === 'tests/util/measurer.test.ts' ? 'host-or-packaging' : 'rust-core',
      timeoutMs: test.options.timeout,
    });
  }
}
const out = join(root, 'tests/upstream');
mkdirSync(out, { recursive: true });
cpSync(join(upstream, 'tests'), join(out, 'source/tests'), { recursive: true });
cpSync(join(upstream, 'LICENSE'), join(out, 'source/LICENSE'));
const sourceFiles = walk(join(upstream, 'tests')).sort().map(path => ({ path: relative(upstream, path), sha256: hash(readFileSync(path)) }));
writeFileSync(join(out, 'inventory.json'), JSON.stringify({ schemaVersion: 1, commit, baseline, total: entries.length, files, sourceFiles, tests: entries }, null, 2) + '\n');
console.log(JSON.stringify({ files: files.length, tests: entries.length, baseline }));
