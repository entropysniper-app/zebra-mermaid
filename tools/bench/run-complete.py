#!/usr/bin/env python3
import datetime
import json
import os
import pathlib
import subprocess

root = pathlib.Path(__file__).resolve().parents[2]
output = root / 'target/bench-complete'
output.mkdir(parents=True, exist_ok=True)
environment = {**os.environ, 'TZ': 'UTC', 'CARGO_TARGET_DIR': 'target'}
environment.setdefault('BENCH_REPORT_SUFFIX', '-pele-1-9-0')
environment.setdefault('BENCHMARK_TAG', 'unreleased-pele1.9.0')
reference = root / '.research/pele'
revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=reference, text=True).strip()
assert revision == 'e7f092a4de147cc32210913c20e36eaa5fcd0ef1', 'Unexpected Pele revision'
subprocess.run(['git', 'diff', '--quiet', 'HEAD', '--', 'src', 'scripts'], cwd=reference, check=True)
for variable in ('BENCH_MS', 'BENCH_SAMPLES'):
    environment.pop(variable, None)
steps = [
    ('pele-build', ['node', 'scripts/build.mjs'], root / '.research/pele'),
    ('rust-build', ['cargo', 'build', '--release', '--locked', '--workspace', '--all-features', '--examples', '--bins'], root),
    ('cross-build', ['cargo', 'build', '--release', '--locked', '--manifest-path', 'tools/cross-bench/Cargo.toml', '--target-dir', 'target/cross-bench-build'], root),
    ('cases', ['node', 'tools/bench/cases.mjs'], root),
    ('upstream', ['node', 'tools/bench/run-upstream.mjs'], root),
    ('publish-upstream', ['node', 'tools/bench/report-upstream.mjs'], root),
    ('extended', ['node', 'tools/bench/run.mjs'], root),
    ('publish-extended', ['node', 'tools/bench/compare.mjs'], root),
    ('resources', ['python3', 'tools/bench/profile-resources.py'], root),
    ('cross', ['python3', 'tools/bench/run-cross.py'], root),
    ('publish-cross', ['python3', 'tools/bench/publish-cross.py'], root),
    ('publish-complete', ['python3', 'tools/bench/publish-complete.py'], root),
]
records = []
for name, command, directory in steps:
    started = datetime.datetime.now(datetime.timezone.utc).isoformat()
    print(f'{started}: {name}', flush=True)
    with (output / f'{name}.log').open('w') as log:
        result = subprocess.run(command, cwd=directory, env=environment, stdout=log, stderr=subprocess.STDOUT)
    records.append({'step': name, 'command': command, 'cwd': directory.relative_to(root).as_posix(), 'startedAt': started,
                    'finishedAt': datetime.datetime.now(datetime.timezone.utc).isoformat(), 'exitCode': result.returncode})
    (output / 'execution.json').write_text(json.dumps(records, indent=2) + '\n')
    if result.returncode != 0:
        raise SystemExit(f'{name} failed; see target/bench-complete/{name}.log')
print('Complete benchmark report written to docs/BENCHMARK-COMPLETE.zh-CN.md', flush=True)
