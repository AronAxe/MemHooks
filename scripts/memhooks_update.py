#!/usr/bin/env python3
"""Compatibility launcher for the normative Rust MemHooks maintainer.

The Rust `memhooks` binary owns parsing, root resolution, locking, atomic writes,
and frontmatter maintenance. Keeping this tiny launcher preserves existing
Hermes hook configurations without maintaining a second data model in Python.
"""

from __future__ import annotations

import os
import shutil
import sys
from pathlib import Path


def find_memhooks() -> str | None:
    configured = os.getenv("MEMHOOKS_BIN")
    if configured:
        path = Path(configured).expanduser()
        if path.is_file():
            return str(path.resolve())

    installed = shutil.which("memhooks")
    if installed:
        return installed

    repo = Path(__file__).resolve().parents[1]
    suffix = ".exe" if os.name == "nt" else ""
    for profile in ("release", "debug"):
        candidate = repo / "target" / profile / f"memhooks{suffix}"
        if candidate.is_file():
            return str(candidate)
    return None


def main() -> int:
    binary = find_memhooks()
    if not binary:
        print(
            "MemHooks binary not found. Install it with `cargo install memhooks` "
            "or set MEMHOOKS_BIN.",
            file=sys.stderr,
        )
        return 127

    args = sys.argv[1:] or ["event"]
    os.execv(binary, [binary, *args])
    return 127  # unreachable unless execv itself fails


if __name__ == "__main__":
    raise SystemExit(main())
