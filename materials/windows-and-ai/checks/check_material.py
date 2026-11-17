"""Offline reference-shelf checks; no Cargo, native windows, Clipboard or model."""
from __future__ import annotations

import argparse
import ast
import hashlib
import json
from pathlib import Path
import subprocess
import sys

sys.dont_write_bytecode = True


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--synthetic", action="store_true", help="run the two stdlib fixture files")
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    manifest = json.loads((root / "provenance.json").read_text(encoding="utf-8"))
    expected = set()
    for entry in manifest["files"]:
        path = root / entry["path"]
        if path.is_symlink() or not path.resolve(strict=True).is_relative_to(root):
            raise RuntimeError("Snapshot path escapes the material")
        relative = path.relative_to(root).as_posix()
        if relative in expected:
            raise RuntimeError("Duplicate provenance path")
        expected.add(relative)
        data = path.read_bytes()
        if len(data) != entry["bytes"] or hashlib.sha256(data).hexdigest() != entry["sha256"]:
            raise RuntimeError("Snapshot hash mismatch: " + relative)

    allowed = expected | set(manifest["local_files"]) | {"provenance.json"}
    actual = {p.relative_to(root).as_posix() for p in root.rglob("*") if p.is_file()}
    if actual - allowed:
        raise RuntimeError("Unexpected files in reference shelf: " + ", ".join(sorted(actual - allowed)))
    if expected - actual:
        raise RuntimeError("Missing snapshot files")

    parsed = 0
    for path in sorted((root / "reference").rglob("*.py")):
        ast.parse(path.read_text(encoding="utf-8-sig"), filename=str(path))
        parsed += 1

    fixtures = []
    if args.synthetic:
        for relative in (
            "reference/tests/folder_semantic_cache_test.py",
            "reference/scripts/ai-components/test_verify.py",
        ):
            result = subprocess.run(
                [sys.executable, "-B", str(root / relative)],
                cwd=root / "reference", capture_output=True, text=True, timeout=30,
            )
            if result.returncode:
                raise RuntimeError(relative + " failed:\n" + result.stdout + result.stderr)
            fixtures.append({"path": relative, "result": "passed", "output": result.stderr.strip()})

    print(json.dumps({
        "snapshots_verified": len(expected), "python_files_parsed": parsed,
        "synthetic_fixtures": fixtures,
        "native_effects": "not run", "model_inference": "not run",
        "network_or_installation": "not run",
    }, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
