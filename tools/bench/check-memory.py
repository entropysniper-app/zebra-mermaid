#!/usr/bin/env python3
"""Check allocator ownership each round, sample idle RSS, and independently run system leaks."""
import hashlib, json, os, pathlib, re, shutil, subprocess, sys

root = pathlib.Path(__file__).resolve().parents[2]
target = root / os.environ.get('CARGO_TARGET_DIR', 'target')
out = root / os.environ.get('ZM_MEMORY_OUTPUT', 'target/profiling/memory')
out.mkdir(parents=True, exist_ok=False)
binary = out / 'memory-check'
shutil.copy2(target / 'release/zebra-mermaid-memory-check', binary)
cases = root / 'target/bench/cases.json'
stress = root / 'target/profiling/stress/cases.json'
heavy = [c for c in json.loads(stress.read_text()) if c['name'].startswith(('stress: class/50 ', 'stress: class/51 ', 'stress: agentflow/44 '))]
assert len(heavy) == 3
heavy_path = out / 'heavy-cases.json'
heavy_path.write_text(json.dumps(heavy))

def rss(pid):
    if sys.platform == 'darwin':
        return int(subprocess.check_output(['ps', '-o', 'rss=', '-p', str(pid)], text=True).strip()) * 1024
    text = pathlib.Path('/proc/%d/status' % pid).read_text()
    return int(re.search(r'^VmRSS:\s+(\d+)', text, re.M)[1]) * 1024

runs = []
soak = '--stress-soak' in sys.argv[1:]
suites = [('stress', heavy_path, [('parsed', 300), ('cache', 300)])] if soak else [
    ('corpus', cases, [('render', 1000), ('parsed', 200), ('cache', 200), ('ffi', 200)]),
    ('stress', heavy_path, [('render', 100), ('parsed', 30), ('cache', 30)])]
for suite, path, modes in suites:
    for mode, rounds in modes:
        name = suite + '-' + mode
        env = {**os.environ, 'TZ': 'UTC', 'ZM_PROFILE_CASES': str(path), 'ZM_MEMORY_ACK': '1'}
        events = []
        with (out / (name + '.stderr')).open('w') as errors:
            p = subprocess.Popen([str(binary), mode, str(rounds)], cwd=root, env=env, stdin=subprocess.PIPE,
                                 stdout=subprocess.PIPE, stderr=errors, text=True, bufsize=1)
            for line in p.stdout:
                event = json.loads(line)
                if event['event'] in ('loaded', 'checkpoint'):
                    event['rssBytes'] = rss(p.pid)
                    p.stdin.write('\n'); p.stdin.flush()
                events.append(event)
            assert p.wait() == 0, (name, 'See ignored stderr log')
        assert events[-1]['event'] == 'complete'
        checkpoints = [e for e in events if e['event'] == 'checkpoint']
        assert len(checkpoints) >= 20 and all(e['retainedBytes'] == e['retainedBlocks'] == 0 for e in checkpoints)
        tail = checkpoints[len(checkpoints)//2:]
        run = {'name': name, 'result': events[-1], 'checkpoints': checkpoints,
               'rssFirstCheckpointBytes': checkpoints[0]['rssBytes'], 'rssLastCheckpointBytes': checkpoints[-1]['rssBytes'],
               'rssSecondHalfRangeBytes': max(e['rssBytes'] for e in tail) - min(e['rssBytes'] for e in tail),
               'inputSha256': hashlib.sha256(path.read_bytes()).hexdigest()}
        runs.append(run)
        (out / (name + '.json')).write_text(json.dumps(run, indent=2) + '\n')
        print(name, 'rounds', rounds, 'retained 0 bytes / 0 blocks', flush=True)

system = []
if not soak and sys.platform == 'darwin' and shutil.which('leaks'):
    for mode in ['negative-control', 'render', 'parsed', 'cache', 'ffi']:
        env = {**os.environ, 'TZ': 'UTC', 'ZM_PROFILE_CASES': str(cases)}
        env.pop('ZM_MEMORY_ACK', None)
        r = subprocess.run(['leaks', '--quiet', '--atExit', '--', str(binary), mode, '10'], cwd=root, env=env,
                           capture_output=True, text=True)
        raw = r.stdout + '\n' + r.stderr
        (out / ('system-' + mode + '.txt')).write_text(raw)
        counts = re.findall(r'(\d+) leaks? for (\d+) total leaked bytes', raw)
        assert counts, ('leaks failed to produce a finding', mode, r.returncode)
        count, size = map(int, counts[-1])
        if mode == 'negative-control':
            assert count >= 1 and size >= 4096 and r.returncode != 0
        else:
            assert r.returncode == 0 and count == size == 0, ('system leak detected', mode)
        system.append({'mode': mode, 'rounds': 0 if mode == 'negative-control' else 10,
                       'leaks': count, 'leakedBytes': size, 'exitCode': r.returncode,
                       'rawSha256': hashlib.sha256(raw.encode()).hexdigest()})
        print('system leaks:', mode, count, size, flush=True)

report = {'method': 'System allocator counts bytes and blocks, checked after EVERY round after output/model drops and cache.clear. Five warm-ups; inputs excluded. RSS sampled with the child paused after drops at 20 checkpoints. Cache capacity may remain until cache drop. Weak owners test eviction, clear and final external Arc drop. Separate forgotten-allocation control detects 4096 bytes / 1 block. Instrumented timings are NOT CPU benchmarks.',
          'binarySha256': hashlib.sha256(binary.read_bytes()).hexdigest(), 'runs': runs, 'systemLeaks': system}
(out / 'summary.json').write_text(json.dumps(report, indent=2) + '\n')
print('Memory validation complete.', flush=True)
