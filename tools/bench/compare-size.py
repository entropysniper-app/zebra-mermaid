#!/usr/bin/env python3
"""Measure a frozen release baseline or compare two; never strips a binary."""

import argparse
import hashlib
import json
import re
import shutil
import subprocess
from pathlib import Path


def size_command():
    executable = shutil.which("llvm-size")
    if executable:
        return [executable]
    if shutil.which("xcrun"):
        probe = subprocess.run(
            ["xcrun", "--find", "llvm-size"], capture_output=True, text=True
        )
        if probe.returncode == 0:
            return [probe.stdout.strip()]
    raise SystemExit("llvm-size is required to measure object code sections")


def measure(path, command):
    result = subprocess.run(
        command + ["--format=sysv", str(path)], capture_output=True, text=True
    )
    # An unreadable archive member must not silently become a size reduction.
    if result.returncode or result.stderr.strip():
        raise SystemExit(f"llvm-size could not read all of {path.name}: {result.stderr}")
    sections = {}
    for line in result.stdout.splitlines():
        row = re.fullmatch(r"\s*(\S+)\s+(\d+)\s+(\d+)\s*", line)
        if row and row[1] != "Total":
            sections[row[1]] = sections.get(row[1], 0) + int(row[2])
    text = sum(
        value
        for name, value in sections.items()
        if name in ("__text", ".text") or name.startswith(".text.")
    )
    if not text:
        raise SystemExit(f"No supported nonempty text section found in {path.name}")
    return {
        "fileBytes": path.stat().st_size,
        "textBytes": text,
        "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
        "sectionsBytes": dict(sorted(sections.items())),
    }


def compare(before, after, command):
    rows = []
    left = json.loads((before / "manifest.json").read_text())
    right = json.loads((after / "manifest.json").read_text())
    for field in ("buildCommand", "releaseProfile", "rustc", "buildEnvironment"):
        if left[field] != right[field]:
            raise SystemExit(f"Build configuration differs: {field}")
    if left["artifacts"] != right["artifacts"]:
        raise SystemExit("Artifact sets differ")
    for name, relative in left["artifacts"].items():
        path = Path(relative)
        if path.is_absolute() or ".." in path.parts:
            raise SystemExit("Artifact paths must stay inside their frozen directory")
        b = measure(before / path, command)
        a = measure(after / path, command)
        rows.append({
            "artifact": name,
            "path": relative,
            "before": b,
            "after": a,
            "fileDeltaBytes": a["fileBytes"] - b["fileBytes"],
            "textDeltaBytes": a["textBytes"] - b["textBytes"],
            "fileAfterOverBefore": a["fileBytes"] / b["fileBytes"],
            "textAfterOverBefore": a["textBytes"] / b["textBytes"],
        })
    return {"beforeBuild": left, "afterBuild": right, "rows": rows}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--before", type=Path, required=True)
    parser.add_argument("--after", type=Path)
    parser.add_argument("--report", type=Path, required=True)
    args = parser.parse_args()
    command = size_command()
    version = subprocess.check_output(command + ["--version"], text=True)
    version = next((s.strip() for s in version.splitlines() if "version" in s), "unknown")
    if args.after:
        result = compare(args.before, args.after, command)
    else:
        manifest = json.loads((args.before / "manifest.json").read_text())
        measurements = {}
        for name, relative in manifest["artifacts"].items():
            path = Path(relative)
            if path.is_absolute() or ".." in path.parts:
                raise SystemExit("Artifact paths must stay inside their frozen directory")
            measurements[name] = {"path": relative, **measure(args.before / path, command)}
        result = {"baselineBuild": manifest, "baselineArtifacts": measurements}
    result.update({
        "tool": {"name": "llvm-size", "version": version},
        "method": {
            "measurementOnly": True,
            "latencyTimingInThisScript": False,
            "stripped": False,
            "textSections": ["__text", ".text", ".text.*"],
            "archiveCode": "Sum of text sections of all members, before host dead stripping; not the final host link increment.",
            "fileBytes": "Entire unchanged artifact, including data, metadata, symbols and archive overhead.",
            "textBytes": "Instruction text only; stubs, unwind tables, constants and symbols are listed separately in sectionsBytes.",
        },
        "scriptSha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
    })
    args.report.parent.mkdir(parents=True, exist_ok=True)
    args.report.write_text(json.dumps(result, indent=2) + "\n")
    if args.after:
        for row in result["rows"]:
            print(f"{row['artifact']}: file {row['before']['fileBytes']} -> {row['after']['fileBytes']}; text {row['before']['textBytes']} -> {row['after']['textBytes']}")
    else:
        for name, row in result["baselineArtifacts"].items():
            print(f"{name}: file {row['fileBytes']}; text {row['textBytes']}")


if __name__ == "__main__":
    main()
