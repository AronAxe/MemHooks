"""Extract exactly one changelog release; does not publish or change a release."""
from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

NUMBER = r"(?:0|[1-9][0-9]*)"
PRERELEASE = rf"(?:{NUMBER}|[0-9]*[A-Za-z-][0-9A-Za-z-]*)"
SEMVER = rf"{NUMBER}\.{NUMBER}\.{NUMBER}(?:-{PRERELEASE}(?:\.{PRERELEASE})*)?(?:\+[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?"
RELEASE = re.compile(rf"(?:\[(v?{SEMVER})\]|(v?{SEMVER}))(?=[\s(#]|$)")
HEADING = re.compile(r"^ {0,3}(#{1,2})[ \t]+(.*)$")
FENCE = re.compile(r"^ {0,3}(`{3,}|~{3,})(.*)$")


def extract_release_notes(text: str, version: str) -> str:
    """Return the requested H2 section, heading included; fail on absent/duplicate versions."""
    version = version.strip().removeprefix("v")
    if not re.fullmatch(SEMVER, version):
        raise ValueError(f"invalid release version: {version!r}")
    lines = text.splitlines(keepends=True)
    boundaries: list[tuple[int, str | None]] = []
    fence = ""
    for index, line in enumerate(lines):
        candidate = FENCE.match(line.rstrip("\r\n"))
        if fence:
            if (candidate and candidate[1][0] == fence[0]
                    and len(candidate[1]) >= len(fence) and not candidate[2].strip()):
                fence = ""
            continue
        if candidate:
            fence = candidate[1]
            continue
        heading = HEADING.match(line.rstrip("\r\n"))
        if heading:
            release = RELEASE.match(heading[2]) if heading[1] == "##" else None
            label = (release[1] or release[2]).removeprefix("v") if release else None
            boundaries.append((index, label))
    matches = [i for i, (_, label) in enumerate(boundaries) if label == version]
    if len(matches) != 1:
        raise ValueError(f"expected exactly one section for {version}; found {len(matches)}")
    position = matches[0]
    start = boundaries[position][0]
    end = boundaries[position + 1][0] if position + 1 < len(boundaries) else len(lines)
    return "".join(lines[start:end]).strip() + "\n"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("version", help="exact semver, with optional v prefix")
    parser.add_argument("--changelog", type=Path, default=Path("CHANGELOG.md"))
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    try:
        notes = extract_release_notes(args.changelog.read_text(encoding="utf-8"), args.version)
        if args.output:
            args.output.write_text(notes, encoding="utf-8")
        else:
            sys.stdout.write(notes)
    except (OSError, ValueError) as error:
        parser.exit(1, f"release notes: {error}\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
