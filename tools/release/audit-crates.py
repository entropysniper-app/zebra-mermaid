#!/usr/bin/env python3
"""Check Cargo's distributable archives against the repository sources and notices."""
import argparse
import hashlib
import json
import pathlib
import subprocess
import tarfile


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=pathlib.Path)
    parser.add_argument("--require-clean", action="store_true")
    args = parser.parse_args()
    root = pathlib.Path(__file__).resolve().parents[2]
    metadata = json.loads(subprocess.check_output(
        ["cargo", "metadata", "--format-version", "1", "--no-deps"], cwd=root, text=True))
    results = []
    for package in metadata["packages"]:
        if package["publish"] == []:
            continue
        name, version = package["name"], package["version"]
        filename = f"{name}-{version}.crate"
        candidates = [root / "target/package/tmp-registry" / filename,
                      root / "target/package" / filename]
        archive_path = next((p for p in candidates if p.is_file()), None)
        if archive_path is None:
            raise RuntimeError(f"No packaged archive for {name}; run cargo publish --dry-run first")
        package_root = pathlib.Path(package["manifest_path"]).parent
        with tarfile.open(archive_path) as archive:
            prefix = f"{name}-{version}/"
            members = archive.getmembers()
            paths = [m.name.removeprefix(prefix) for m in members]
            for member, path in zip(members, paths):
                if not member.name.startswith(prefix) or ".." in pathlib.PurePosixPath(path).parts:
                    raise RuntimeError(f"Unsafe package path: {member.name}")
                if not member.isfile():
                    raise RuntimeError(f"Unexpected non-file entry: {member.name}")
                if path.startswith(("tests/", "examples/", "target/", ".research/")):
                    raise RuntimeError(f"Development asset in {name}: {path}")
                if path.startswith(("src/", "include/")):
                    if archive.extractfile(member).read() != (package_root / path).read_bytes():
                        raise RuntimeError(f"Published source differs: {name}/{path}")
            for notice in ["LICENSE", "THIRD_PARTY_NOTICES.md"]:
                if archive.extractfile(prefix + notice).read() != (root / notice).read_bytes():
                    raise RuntimeError(f"Missing or outdated notice: {name}/{notice}")
            for required in ["Cargo.toml", "README.md", "src/lib.rs" if name != "zebra-mermaid-cli" else "src/main.rs"]:
                if required not in paths:
                    raise RuntimeError(f"Missing package entry: {name}/{required}")
            if name.endswith("-ffi") and "include/zebra_mermaid.h" not in paths:
                raise RuntimeError("The FFI archive must include the C header")
            manifest = archive.extractfile(prefix + "Cargo.toml").read().decode()
            if "path = \"../zebra-mermaid\"" in manifest:
                raise RuntimeError(f"Unresolved local runtime dependency: {name}")
            vcs = json.loads(archive.extractfile(prefix + ".cargo_vcs_info.json").read())
            if args.require_clean and vcs["git"].get("dirty", False):
                raise RuntimeError(f"Archive was built from a dirty checkout: {name}")
        results.append({"crate": name, "version": version, "files": len(paths),
                        "archive": str(archive_path.relative_to(root)),
                        "archiveBytes": archive_path.stat().st_size,
                        "sha256": hashlib.sha256(archive_path.read_bytes()).hexdigest(),
                        "runtimeSourcesMatchWorkspace": True, "licensesIncluded": True,
                        "developmentAssetsExcluded": True, "vcs": vcs})
    report = {"packages": results, "allAuditsPassed": True}
    if args.output:
        destination = root / args.output
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    main()
