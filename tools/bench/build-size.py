#!/usr/bin/env python3
"""Build and freeze full-feature release CLI/FFI artifacts outside timing runs."""

import argparse
import hashlib
import json
import os
import shutil
import subprocess
from pathlib import Path


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--revision", required=True)
    args = parser.parse_args()
    source, output = args.source.resolve(), args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    if (output / "manifest.json").exists():
        raise SystemExit("Refusing to overwrite a frozen manifest")
    # Do not silently compare builds made with different local profile overrides.
    overrides = [
        name for name in os.environ
        if name in ("RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "CARGO_BUILD_RUSTFLAGS")
        or name.startswith("CARGO_PROFILE_RELEASE_")
    ]
    if overrides:
        raise SystemExit("Remove local build overrides first: " + ", ".join(overrides))
    command = [
        "cargo", "build", "--release", "--locked", "-p", "zebra-mermaid-cli",
        "-p", "zebra-mermaid-ffi", "--all-features",
    ]
    # Identical panic/source paths in both builds; checkout paths do not affect size.
    env = {**os.environ, "RUSTFLAGS": f"--remap-path-prefix={source}=."}
    paths = [source / "Cargo.toml", source / "Cargo.lock"]
    for folder in ("crates", "tools/profiler"):
        paths += [
            path for path in (source / folder).rglob("*")
            if path.is_file() and (path.suffix == ".rs" or path.name == "Cargo.toml")
        ]
    hashes = {
        str(path.relative_to(source)): hashlib.sha256(path.read_bytes()).hexdigest()
        for path in sorted(paths)
    }
    with (output / "build.log").open("w") as log:
        subprocess.run(
            command + ["--target-dir", str(output / "build")], cwd=source,
            env=env, stdout=log, stderr=subprocess.STDOUT, check=True,
        )
    assert all(hashlib.sha256((source / p).read_bytes()).hexdigest() == h for p, h in hashes.items())
    release = output / "build/release"
    artifacts = {"cli": "zebra-mermaid", "ffi-static": "libzebra_mermaid_ffi.a"}
    dynamic = [name for name in ("libzebra_mermaid_ffi.dylib", "libzebra_mermaid_ffi.so") if (release / name).is_file()]
    if len(dynamic) != 1 or not all((release / name).is_file() for name in artifacts.values()):
        raise SystemExit("Expected release CLI, static archive and one dynamic library")
    artifacts["ffi-dynamic"] = dynamic[0]
    for name in artifacts.values():
        shutil.copy2(release / name, output / name)
    profile = (source / "Cargo.toml").read_text().split("[profile.release]\n", 1)[1].split("\n[", 1)[0].strip()
    manifest = {
        "revision": args.revision,
        "buildCommand": command,
        "releaseProfile": profile,
        "rustc": subprocess.check_output(["rustc", "--version"], text=True).strip(),
        "buildEnvironment": {"RUSTFLAGS": "--remap-path-prefix=<source-root>=."},
        "sourceHashes": hashes,
        "artifacts": artifacts,
        "buildScriptSha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
    }
    (output / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    print("Frozen release CLI, FFI static archive and dynamic library")


if __name__ == "__main__":
    main()
