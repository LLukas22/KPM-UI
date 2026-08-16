#!/usr/bin/env python3

import json
import pathlib
import sys
import tarfile


def main() -> None:
    if len(sys.argv) != 3:
        raise SystemExit(f"usage: {sys.argv[0]} PACKAGE_DIR OUTPUT_DIR")

    package_dir = pathlib.Path(sys.argv[1])
    output_dir = pathlib.Path(sys.argv[2])
    manifest_path = package_dir / "manifest.json"

    if not package_dir.is_dir() or not manifest_path.is_file():
        raise SystemExit(f"invalid package directory: {package_dir}")

    manifest = json.loads(manifest_path.read_text())
    package_id = manifest["id"]
    version = manifest["version"]
    platforms = manifest.get("supported_platforms") or ["kindleany"]

    if len(version) != 3 or not all(isinstance(part, int) for part in version):
        raise SystemExit("manifest version must contain three integers")
    if any(character.isspace() or character.isupper() for character in package_id):
        raise SystemExit("package id must not contain whitespace or uppercase letters")

    entries = sorted(package_dir.iterdir(), key=lambda path: path.name)
    reserved = {"rootfs", "startup.sh"}
    conflicts = reserved.intersection(entry.name for entry in entries)
    if conflicts:
        raise SystemExit(f"reserved package path: {sorted(conflicts)[0]}")

    version_text = ".".join(map(str, version))
    platform_text = "-".join(platforms)
    output_dir.mkdir(parents=True, exist_ok=True)
    artifact = output_dir / f"{package_id}_{version_text}_{platform_text}.kpkg"

    with tarfile.open(artifact, "w:gz", compresslevel=5) as archive:
        for entry in entries:
            archive.add(entry, arcname=entry.name)

    print(f"Packed {artifact}")


if __name__ == "__main__":
    main()
