#!/usr/bin/env python3
"""Read-only preflight for common build/test drivers, outside benchmark timing."""

import argparse
import collections
import datetime
import json
import subprocess
from pathlib import Path


DRIVERS = {
    "cargo", "rustc", "xcodebuild", "swift", "swiftc", "swift-frontend",
    "clang", "clang++", "cc", "c++", "gcc", "g++", "ninja", "make",
}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--report", type=Path)
    args = parser.parse_args()
    snapshot = subprocess.check_output(["ps", "-A", "-o", "comm="], text=True)
    found = collections.Counter(
        name for line in snapshot.splitlines()
        for name in [Path(line.strip()).name] if name in DRIVERS
    )
    report = {
        "observedAtUtc": datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "ok": not found,
        "activeBuildOrTestDrivers": dict(sorted(found.items())),
        "scope": "Read-only process-name snapshot. It detects listed drivers at observation time only, not all workloads or sustained idleness. No process is stopped or changed; no paths, host identity or hardware details are recorded.",
    }
    if args.report:
        args.report.parent.mkdir(parents=True, exist_ok=True)
        args.report.write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(report))
    raise SystemExit(0 if report["ok"] else 1)


if __name__ == "__main__":
    main()
