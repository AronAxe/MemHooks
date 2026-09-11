#!/usr/bin/env python3
"""Zero-LLM MemHooks maintainer.

Three jobs:

1. ``event`` (default): consume a runtime hook event on stdin, extract touched
   project paths, and maintain a tiny auto-recall block in the nearest
   MEMHOOKS.md files. No model call is made.
2. ``note``: add one explicit retrieval cue discovered by the *current* agent
   while it is already reasoning. Optional priority, role applicability, memory
   categories, connection emphasis, and typed/salient entities can be preserved
   with the cue.
3. ``init``: enable MemHooks at a repository root.

The script never writes memory content. It writes retrieval cues only.
"""

from __future__ import annotations

import argparse
import json
import math
import os
import re
import shlex
import subprocess
import sys
from pathlib import Path
from typing import Any, Iterable

HOOK_FILENAME = os.getenv("MEMHOOKS_FILENAME", "MEMHOOKS.md")
AUTO_START = "<!-- memhooks:auto:start -->"
AUTO_END = "<!-- memhooks:auto:end -->"
NOTES_START = "<!-- memhooks:notes:start -->"
NOTES_END = "<!-- memhooks:notes:end -->"
MAX_AUTO_PATHS = int(os.getenv("MEMHOOKS_AUTO_PATHS", "12"))
MAX_NOTES = int(os.getenv("MEMHOOKS_MAX_NOTES", "16"))
VALID_MEMORY_TYPES = ("world", "experience", "observation")
VALID_CONNECTION_TYPES = ("semantic", "temporal", "entity", "causal")
SKIP_PARTS = {
    ".git", ".hg", ".svn", ".venv", "venv", "node_modules", "dist", "build",
    "__pycache__", ".idea", ".vscode", ".pytest_cache", ".mypy_cache",
}
PATH_KEYS = {
    "path", "file", "filename", "file_path", "filepath", "target", "source",
    "destination", "dest", "output", "output_path", "workdir", "cwd",
}
FILE_TOKEN = re.compile(r"(?<![\w:/.-])([A-Za-z0-9_.~/-]+\.[A-Za-z0-9_.-]{1,12})(?![\w.-])")


def _git_root(cwd: Path) -> Path | None:
    try:
        out = subprocess.run(
            ["git", "-C", str(cwd), "rev-parse", "--show-toplevel"],
            capture_output=True,
            text=True,
            timeout=2,
            check=False,
        )
        if out.returncode == 0 and out.stdout.strip():
            return Path(out.stdout.strip()).resolve()
    except Exception:
        pass
    return None


def _nearest_enabled_root(cwd: Path) -> Path | None:
    """Return git root if it contains MEMHOOKS.md, else highest hooked ancestor."""
    git = _git_root(cwd)
    if git and (git / HOOK_FILENAME).is_file():
        return git
    hits = [d for d in (cwd, *cwd.parents) if (d / HOOK_FILENAME).is_file()]
    return hits[-1] if hits else None


def _safe_relative(candidate: str, cwd: Path, root: Path) -> Path | None:
    candidate = candidate.strip().strip("'\"`[](){}<>,;:")
    if not candidate or candidate.startswith(("http://", "https://", "git@")):
        return None
    candidate = os.path.expandvars(os.path.expanduser(candidate))
    p = Path(candidate)
    if not p.is_absolute():
        p = cwd / p
    try:
        p = p.resolve(strict=False)
        rel = p.relative_to(root)
    except Exception:
        return None
    if not rel.parts or any(part in SKIP_PARTS for part in rel.parts):
        return None
    if p.name == HOOK_FILENAME:
        return None
    if p.exists() and p.is_dir():
        return None
    if not p.suffix and not p.exists():
        return None
    return rel


def _strings(value: Any) -> Iterable[tuple[str | None, str]]:
    if isinstance(value, dict):
        for k, v in value.items():
            if isinstance(v, str):
                yield str(k).lower(), v
            else:
                yield from _strings(v)
    elif isinstance(value, list):
        for item in value:
            yield from _strings(item)
    elif isinstance(value, str):
        yield None, value


def _extract_paths(tool_input: Any, cwd: Path, root: Path) -> set[Path]:
    found: set[Path] = set()
    for key, text in _strings(tool_input):
        candidates: list[str] = []
        if key in PATH_KEYS:
            candidates.append(text)
        candidates.extend(m.group(1) for m in FILE_TOKEN.finditer(text))
        if any(ch in text for ch in " /\\"):
            try:
                candidates.extend(tok for tok in shlex.split(text) if "/" in tok or "\\" in tok)
            except Exception:
                pass
        for raw in candidates:
            rel = _safe_relative(raw, cwd, root)
            if rel is not None:
                found.add(rel)
    return found


def _read(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except FileNotFoundError:
        return ""
    except Exception:
        return ""


def _replace_block(text: str, start: str, end: str, block: str) -> str:
    if start in text and end in text:
        before = text.split(start, 1)[0].rstrip()
        after = text.split(end, 1)[1].lstrip("\n")
        pieces = [before, block.rstrip()]
        if after:
            pieces.append(after.rstrip())
        return "\n\n".join(p for p in pieces if p) + "\n"
    base = text.rstrip()
    return base + ("\n\n" if base else "") + block.rstrip() + "\n"


def _ensure_base(path: Path) -> str:
    text = _read(path)
    if text.strip():
        return text
    return (
        "---\n"
        "schema: memhooks/v1\n"
        "inherits: true\n"
        "memory_types: []\n"
        "connection_types: []\n"
        "entities: []\n"
        "---\n\n"
        "# MemHooks\n"
    )


def _parse_auto_paths(text: str) -> list[str]:
    if AUTO_START not in text or AUTO_END not in text:
        return []
    body = text.split(AUTO_START, 1)[1].split(AUTO_END, 1)[0]
    return re.findall(r"^- `([^`]+)`\s*$", body, flags=re.M)


def _auto_block(paths: list[str]) -> str:
    listing = "\n".join(f"- `{p}`" for p in paths)
    return (
        f"{AUTO_START}\n"
        "## Auto-maintained recall anchors\n\n"
        "Before substantive work here, recall prior **decisions, constraints, "
        "failures, fixes, rejected approaches, and unresolved issues** involving:\n"
        f"{listing}\n\n"
        "These are retrieval cues, not memory contents. This deterministic path "
        "maintainer does not guess priority, role applicability, memory categories, "
        "connection emphasis, entity types, or entity salience. If this turn "
        "establishes a durable semantic cue, record it with the MemHooks note "
        "helper before finishing.\n"
        f"{AUTO_END}"
    )


def _update_directory(root: Path, rel_files: Iterable[Path]) -> int:
    grouped: dict[Path, list[str]] = {}
    for rel in rel_files:
        grouped.setdefault(rel.parent, []).append(rel.as_posix())

    writes = 0
    for rel_dir, additions in grouped.items():
        directory = root / rel_dir
        if not directory.is_dir():
            directory = root
        hook = directory / HOOK_FILENAME
        text = _ensure_base(hook)
        existing = _parse_auto_paths(text)
        ordered: list[str] = []
        for item in existing + additions:
            if item not in ordered:
                ordered.append(item)
        ordered = ordered[-MAX_AUTO_PATHS:]
        new = _replace_block(text, AUTO_START, AUTO_END, _auto_block(ordered))
        if new != text:
            hook.parent.mkdir(parents=True, exist_ok=True)
            hook.write_text(new, encoding="utf-8")
            writes += 1
    return writes


def _valid_unit_interval(value: Any) -> float | None:
    try:
        number = float(value)
    except (TypeError, ValueError):
        return None
    if not math.isfinite(number) or not 0.0 <= number <= 1.0:
        return None
    return number


def _normalize_note(value: Any) -> dict[str, Any] | None:
    if isinstance(value, str):
        query = " ".join(value.strip().split())
        return {"query": query} if query else None
    if not isinstance(value, dict):
        return None

    query = " ".join(str(value.get("query") or "").strip().split())
    if not query:
        return None

    note: dict[str, Any] = {"query": query}

    if "priority" in value:
        priority = _valid_unit_interval(value.get("priority"))
        if priority is not None:
            note["priority"] = priority

    when = value.get("when")
    if isinstance(when, dict):
        roles = [
            " ".join(str(v).strip().split())
            for v in (when.get("roles") or [])
            if " ".join(str(v).strip().split())
        ]
        if roles:
            note["when"] = {"roles": list(dict.fromkeys(roles))}

    memory_types = [
        str(v).strip().lower()
        for v in (value.get("memory_types") or [])
        if str(v).strip().lower() in VALID_MEMORY_TYPES
    ]
    if memory_types:
        note["memory_types"] = list(dict.fromkeys(memory_types))

    connection_types = [
        str(v).strip().lower()
        for v in (value.get("connection_types") or [])
        if str(v).strip().lower() in VALID_CONNECTION_TYPES
    ]
    if connection_types:
        note["connection_types"] = list(dict.fromkeys(connection_types))

    entities: list[dict[str, Any] | str] = []
    for raw in value.get("entities") or []:
        if isinstance(raw, str):
            name = " ".join(raw.strip().split())
            if name:
                entities.append(name)
        elif isinstance(raw, dict):
            name = " ".join(str(raw.get("name") or "").strip().split())
            if not name:
                continue
            entity: dict[str, Any] = {"name": name}
            entity_type = str(raw.get("type") or "").strip()
            if entity_type:
                entity["type"] = entity_type
            if "salience" in raw:
                salience = _valid_unit_interval(raw.get("salience"))
                if salience is not None:
                    entity["salience"] = salience
            entities.append(entity)
    if entities:
        note["entities"] = entities

    return note


def _notes_from(text: str) -> list[dict[str, Any]]:
    if NOTES_START not in text or NOTES_END not in text:
        return []

    body = text.split(NOTES_START, 1)[1].split(NOTES_END, 1)[0]

    # Structured format: a fenced JSON array.
    match = re.search(r"```json\s*(\[.*?\])\s*```", body, flags=re.S | re.I)
    if match:
        try:
            raw = json.loads(match.group(1))
        except Exception:
            raw = []
        notes = []
        if isinstance(raw, list):
            for item in raw:
                note = _normalize_note(item)
                if note is not None:
                    notes.append(note)
        return notes

    # Backward compatibility with old markdown bullet notes.
    notes = []
    for item in re.findall(r"^- (.+)$", body, flags=re.M):
        note = _normalize_note(item)
        if note is not None:
            notes.append(note)
    return notes


def _notes_block(notes: list[dict[str, Any]]) -> str:
    payload = json.dumps(notes, ensure_ascii=False, indent=2)
    return (
        f"{NOTES_START}\n"
        "## In-session retrieval cues\n\n"
        "```json\n"
        f"{payload}\n"
        "```\n\n"
        "These are routing cues only. `priority` ranks recall requests under "
        "context pressure; `when.roles` scopes applicability; `memory_types` "
        "classify memories; `connection_types` describe semantic/temporal/entity/"
        "causal emphasis; entity `type` and `salience` apply only to the entity. "
        "Durable facts belong in the memory backend.\n"
        f"{NOTES_END}"
    )


def _parse_entity_arg(raw: str) -> dict[str, Any] | str:
    raw = raw.strip()
    if not raw:
        raise ValueError("empty entity")

    if raw.startswith("{"):
        obj = json.loads(raw)
        if not isinstance(obj, dict):
            raise ValueError("entity JSON must be an object")
        name = " ".join(str(obj.get("name") or obj.get("text") or "").strip().split())
        if not name:
            raise ValueError("entity JSON requires name/text")
        entity: dict[str, Any] = {"name": name}
        entity_type = str(obj.get("type") or "").strip()
        if entity_type:
            entity["type"] = entity_type
        if "salience" in obj:
            salience = _valid_unit_interval(obj.get("salience"))
            if salience is None:
                raise ValueError("entity salience must be between 0.0 and 1.0")
            entity["salience"] = salience
        return entity

    return " ".join(raw.split())


def add_note(
    cwd: Path,
    query: str,
    priority: float | None = None,
    roles: list[str] | None = None,
    memory_types: list[str] | None = None,
    connection_types: list[str] | None = None,
    entities: list[dict[str, Any] | str] | None = None,
) -> int:
    root = _nearest_enabled_root(cwd)
    if root is None:
        print("MemHooks is not enabled here (no ancestor MEMHOOKS.md).", file=sys.stderr)
        return 2

    query = " ".join(query.strip().split())
    if not query:
        return 2

    if priority is not None and _valid_unit_interval(priority) is None:
        print("MemHooks note priority must be between 0.0 and 1.0.", file=sys.stderr)
        return 2

    normalized_roles = [
        " ".join(role.strip().split()) for role in (roles or []) if role.strip()
    ]
    normalized_roles = list(dict.fromkeys(normalized_roles))

    target_dir = next((d for d in (cwd, *cwd.parents) if (d / HOOK_FILENAME).is_file()), cwd)
    try:
        target_dir.relative_to(root)
    except ValueError:
        target_dir = root

    hook = target_dir / HOOK_FILENAME
    text = _ensure_base(hook)
    notes = _notes_from(text)

    incoming_payload: dict[str, Any] = {
        "query": query,
        "memory_types": memory_types or [],
        "connection_types": connection_types or [],
        "entities": entities or [],
    }
    if priority is not None:
        incoming_payload["priority"] = priority
    if normalized_roles:
        incoming_payload["when"] = {"roles": normalized_roles}

    incoming = _normalize_note(incoming_payload)
    if incoming is None:
        return 2

    replaced = False
    for i, existing in enumerate(notes):
        if existing.get("query") != query:
            continue
        merged = dict(existing)

        if "priority" in incoming:
            merged["priority"] = incoming["priority"]

        if "when" in incoming:
            prior_roles = list((merged.get("when") or {}).get("roles") or [])
            for role in incoming["when"].get("roles") or []:
                if role not in prior_roles:
                    prior_roles.append(role)
            if prior_roles:
                merged["when"] = {"roles": prior_roles}

        for key in ("memory_types", "connection_types", "entities"):
            if key not in incoming:
                continue
            prior = list(merged.get(key) or [])
            for item in incoming[key]:
                if item not in prior:
                    prior.append(item)
            if prior:
                merged[key] = prior

        notes[i] = _normalize_note(merged) or incoming
        replaced = True
        break

    if not replaced:
        notes.append(incoming)

    notes = notes[-MAX_NOTES:]
    new = _replace_block(text, NOTES_START, NOTES_END, _notes_block(notes))
    if new != text:
        hook.write_text(new, encoding="utf-8")
    return 0


def init(cwd: Path) -> int:
    root = _git_root(cwd) or cwd
    hook = root / HOOK_FILENAME
    if hook.exists():
        return 0
    name = root.name
    hook.write_text(
        "---\n"
        "schema: memhooks/v1\n"
        "inherits: true\n"
        "memory_types: []\n"
        "connection_types: []\n"
        "entities: []\n"
        "---\n\n"
        "# MemHooks\n\n"
        f"Recall prior decisions, constraints, failures, fixes, rejected approaches, "
        f"and unresolved issues concerning the `{name}` project before making "
        "substantive changes.\n\n"
        "If a turn establishes a durable non-obvious retrieval cue, record one "
        "concise future-retrieval question with the MemHooks note helper. Preserve "
        "priority, role applicability, memory category, connection emphasis, and "
        "typed/salient entities only when they are actually known; do not guess.\n",
        encoding="utf-8",
    )
    return 0


def handle_event(payload: dict[str, Any]) -> int:
    raw_cwd = payload.get("cwd") or os.getcwd()
    try:
        cwd = Path(raw_cwd).expanduser().resolve()
    except Exception:
        return 0
    root = _nearest_enabled_root(cwd)
    if root is None:
        return 0
    if str(payload.get("hook_event_name") or "") != "post_tool_call":
        return 0
    paths = _extract_paths(payload.get("tool_input"), cwd, root)
    if paths:
        _update_directory(root, paths)
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description="Maintain MEMHOOKS.md with zero extra LLM calls.")
    sub = parser.add_subparsers(dest="cmd")
    sub.add_parser("event", help="read a hook event JSON object from stdin")

    p_init = sub.add_parser("init", help="enable MemHooks in this repository")
    p_init.add_argument("path", nargs="?", default=".")

    p_note = sub.add_parser("note", help="add one semantic retrieval cue")
    p_note.add_argument("--cwd", default=".")
    p_note.add_argument("--query", required=True)
    p_note.add_argument(
        "--priority",
        type=float,
        default=None,
        help="optional retrieval priority from 0.0 to 1.0",
    )
    p_note.add_argument(
        "--role",
        action="append",
        default=[],
        help="optional applicable role; repeat as needed",
    )
    p_note.add_argument(
        "--memory-type",
        action="append",
        choices=VALID_MEMORY_TYPES,
        default=[],
        help="optional memory category; repeat as needed",
    )
    p_note.add_argument(
        "--connection-type",
        action="append",
        choices=VALID_CONNECTION_TYPES,
        default=[],
        help="optional connection emphasis; repeat as needed",
    )
    p_note.add_argument(
        "--entity",
        action="append",
        default=[],
        help=(
            "optional entity name or JSON object, e.g. "
            "'{\"name\":\"OpenAI\",\"type\":\"ORG\",\"salience\":0.9}'"
        ),
    )

    args = parser.parse_args()
    cmd = args.cmd or "event"

    if cmd == "init":
        return init(Path(args.path).expanduser().resolve())

    if cmd == "note":
        if args.priority is not None and _valid_unit_interval(args.priority) is None:
            parser.error("--priority must be between 0.0 and 1.0")
        try:
            entities = [_parse_entity_arg(raw) for raw in args.entity]
        except (ValueError, json.JSONDecodeError) as exc:
            print(f"Invalid --entity: {exc}", file=sys.stderr)
            return 2
        return add_note(
            Path(args.cwd).expanduser().resolve(),
            args.query,
            priority=args.priority,
            roles=args.role,
            memory_types=args.memory_type,
            connection_types=args.connection_type,
            entities=entities,
        )

    try:
        payload = json.load(sys.stdin)
    except Exception:
        return 0
    return handle_event(payload if isinstance(payload, dict) else {})


if __name__ == "__main__":
    raise SystemExit(main())
