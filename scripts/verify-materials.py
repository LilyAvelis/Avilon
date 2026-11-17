"""Verify warehouse contents without Katran, downloads, models, or native effects."""
import hashlib
import json
from pathlib import Path
import stat

root = Path(__file__).resolve().parents[1] / "materials"
manifest = json.loads((root / "INVENTORY.json").read_text(encoding="utf-8"))
expected = {entry["path"] for entry in manifest["files"]}
if len(expected) != len(manifest["files"]):
    raise RuntimeError("Duplicate material paths")
actual = set()
for path in root.rglob("*"):
    info = path.lstat()
    if path.is_symlink() or getattr(info, "st_file_attributes", 0) & getattr(stat, "FILE_ATTRIBUTE_REPARSE_POINT", 0x400):
        raise RuntimeError("Linked material path")
    if path.is_file() and path != root / "INVENTORY.json":
        actual.add(path.relative_to(root).as_posix())
if actual != expected:
    raise RuntimeError("Material inventory differs: " + str(actual ^ expected))
for entry in manifest["files"]:
    path = root / entry["path"]
    if not path.resolve().is_relative_to(root.resolve()):
        raise RuntimeError("Material escaped warehouse")
    content = path.read_bytes()
    if len(content) != entry["bytes"] or hashlib.sha256(content).hexdigest() != entry["sha256"]:
        raise RuntimeError("Material hash mismatch: " + entry["path"])
for entry in manifest["originals"]:
    if hashlib.sha256((root / entry["path"]).read_bytes()).hexdigest() != entry["sha256"]:
        raise RuntimeError("Original snapshot changed")
print(f"Verified {len(expected)} material files and {len(manifest['originals'])} original source snapshots")
