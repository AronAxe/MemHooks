"""Sync explicitly owned wiki files; preserve unrelated pages and refuse edit conflicts."""
from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
from pathlib import Path, PurePosixPath

MANIFEST = ".memhooks-managed-wiki.json"
LEGACY_SYNC = re.compile(r"docs: sync MemHooks wiki from ([0-9a-f]{40})")


def digest(content: bytes) -> str:
    return hashlib.sha256(content).hexdigest()


def valid_name(name: str) -> str:
    if not isinstance(name, str):
        raise TypeError("managed path must be a string")
    parts = name.split("/")
    if (not name or "\\" in name or ":" in name or any(ord(c) < 32 for c in name)
            or any(part in ("", ".", "..") or part.lower() == ".git" for part in parts)
            or name == MANIFEST or PurePosixPath(name).is_absolute()):
        raise ValueError(f"unsafe managed path: {name!r}")
    return name


def checked_file(root: Path, name: str) -> Path:
    """Reject symlink/special-file traversal before inspecting or mutating any content."""
    path = root
    parts = name.split("/")
    for index, part in enumerate(parts):
        path = path / part
        if path.is_symlink():
            raise ValueError(f"symlink in wiki path: {path}")
        if path.exists() and not (path.is_file() if index == len(parts) - 1 else path.is_dir()):
            raise ValueError(f"unexpected wiki path type: {path}")
    return path


def validate_manifest(files: dict[str, str]) -> dict[str, str]:
    if not isinstance(files, dict):
        raise TypeError("manifest files must be an object")
    for name, checksum in files.items():
        valid_name(name)
        if not isinstance(checksum, str) or not re.fullmatch(r"[0-9a-f]{64}", checksum):
            raise ValueError(f"invalid managed-file checksum: {name!r}")
    return files


def git(repo: Path, *args: str) -> bytes:
    return subprocess.run(["git", "-C", str(repo), *args], check=True,
                          stdout=subprocess.PIPE, stderr=subprocess.PIPE).stdout


def legacy_manifest(repo: Path, wiki: Path) -> dict[str, str]:
    """Bootstrap ONLY from recorded successful legacy sync source commits.

    Newest content wins for each ever-copied path. This also finds pages deleted
    before the last legacy sync. Unknown history is never inferred from filenames.
    """
    files: dict[str, str] = {}
    seen = set()
    for subject in git(wiki, "log", "--format=%s").decode("utf-8").splitlines():
        match = LEGACY_SYNC.fullmatch(subject)
        if not match or match[1] in seen:
            continue
        ref = match[1]
        seen.add(ref)
        for entry in git(repo, "ls-tree", "-rz", ref, "--", "wiki/").split(b"\0"):
            if not entry:
                continue
            metadata, raw_path = entry.split(b"\t", 1)
            mode, kind, blob = metadata.split()
            if mode not in (b"100644", b"100755") or kind != b"blob":
                raise ValueError("legacy wiki snapshot contains a non-regular file")
            name = valid_name(raw_path.decode("utf-8").removeprefix("wiki/"))
            if name not in files:
                files[name] = digest(git(repo, "cat-file", "blob", blob.decode("ascii")))
    return files


def sync_pages(source: Path, destination: Path, *, previous: dict[str, str] | None = None,
               dry_run: bool = False) -> dict[str, list[str]]:
    for root in (source, destination):
        if root.is_symlink() or not root.is_dir():
            raise ValueError(f"expected a regular wiki directory: {root}")
    source, destination = source.resolve(), destination.resolve()
    if source.is_relative_to(destination) or destination.is_relative_to(source):
        raise ValueError("source and destination wiki directories must not overlap")
    manifest = checked_file(destination, MANIFEST)
    if manifest.exists():
        data = json.loads(manifest.read_text(encoding="utf-8"))
        if not isinstance(data, dict) or type(data.get("version")) is not int or data["version"] != 1:
            raise ValueError("unsupported managed-wiki manifest")
        previous = validate_manifest(data.get("files"))
    else:
        previous = validate_manifest({} if previous is None else previous)
    current = {}
    for path in sorted(source.rglob("*")):
        name = valid_name(path.relative_to(source).as_posix())
        if path.is_symlink():
            raise ValueError(f"source wiki symlink: {name}")
        if path.is_dir():
            continue
        current[name] = checked_file(source, name).read_bytes()
    # Complete preflight BEFORE deleting or overwriting anything. A manual edit
    # to a managed page is a conflict, not permission to erase someone else's work.
    writes, deletes = [], []
    for name in sorted(current.keys() | previous.keys()):
        path = checked_file(destination, name)
        existing = path.read_bytes() if path.exists() else None
        incoming = current.get(name)
        if existing == incoming:
            continue
        if existing is not None and digest(existing) != previous.get(name):
            raise ValueError(f"wiki edit/ownership conflict: {name}; reconcile before syncing")
        if incoming is None:
            deletes.append(name)
        else:
            writes.append(name)
    if not dry_run:
        for name in deletes:
            checked_file(destination, name).unlink()
        for name in writes:
            target = checked_file(destination, name)
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(current[name])
        manifest.write_text(json.dumps({"version": 1, "files": {
            name: digest(content) for name, content in sorted(current.items())
        }}, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    return {"written": writes, "removed": deletes}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("wiki_checkout", type=Path)
    parser.add_argument("--repo", type=Path, default=Path("."))
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()
    try:
        manifest = checked_file(args.wiki_checkout, MANIFEST)
        previous = None if manifest.exists() else legacy_manifest(args.repo, args.wiki_checkout)
        result = sync_pages(args.repo / "wiki", args.wiki_checkout,
                            previous=previous, dry_run=args.dry_run)
        print(json.dumps(result, ensure_ascii=False))
    except (OSError, ValueError, TypeError, subprocess.CalledProcessError) as error:
        parser.exit(1, f"wiki sync: {error}\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
