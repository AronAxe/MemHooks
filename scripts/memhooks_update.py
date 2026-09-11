#!/usr/bin/env python3
"""Zero-LLM MemHooks maintainer.

The maintainer is primarily for agent/runtime use, not manual hook authoring.

Jobs:
1. ``event`` (default): inspect a runtime tool event and maintain small path-based
   retrieval anchors without an LLM call.
2. ``note``: preserve one semantic retrieval cue already discovered by the current
   agent, including generic routing metadata and optional opaque backend hints.
3. ``init``: enable MemHooks at a repository root.

The script never writes memory content. It writes retrieval-routing metadata only.
"""

from __future__ import annotations

import argparse
import copy
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
SKIP_PARTS = {
    ".git",
    ".hg",
    ".svn",
    ".venv",
    "venv",
    "node_modules",
    "dist",
    "build",
    "__pycache__",
    ".idea",
    ".vscode",
    ".pytest_cache",
    ".mypy_cache",
}
PATH_KEYS = {
    "path",
    "file",
    "filename",
    "file_path",
    "filepath",
    "target",
    "source",
    "destination",
    "dest",
    "output",
    "output_path",
    "workdir",
    "cwd",
}
FILE_TOKEN = re.compile(
    r"(?<![\w:/.-])([A-Za-z0-9_.~/-]+\.[A-Za-z0-9_.-]{1,12})(?![\w.-])"
)


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
    git = _git_root(cwd)
    if git and (git / HOOK_FILENAME).is_file():
        return git
    hits = [directory for directory in (cwd, *cwd.parents) if (directory / HOOK_FILENAME).is_file()]
    return hits[-1] if hits else None


def _safe_relative(candidate: str, cwd: Path, root: Path) -> Path | None:
    candidate = candidate.strip().strip("'\"`[](){}<>,;:")
    if not candidate or candidate.startswith(("http://", "https://", "git@")):
        return None
    candidate = os.path.expandvars(os.path.expanduser(candidate))
    path = Path(candidate)
    if not path.is_absolute():
        path = cwd / path
    try:
        path = path.resolve(strict=False)
        relative = path.relative_to(root)
    except Exception:
        return None
    if not relative.parts or any(part in SKIP_PARTS for part in relative.parts):
        return None
    if path.name == HOOK_FILENAME:
        return None
    if path.exists() and path.is_dir():
        return None
    if not path.suffix and not path.exists():
        return None
    return relative


def _strings(value: Any) -> Iterable[tuple[str | None, str]]:
    if isinstance(value, dict):
        for key, nested in value.items():
            if isinstance(nested, str):
                yield str(key).lower(), nested
            else:
                yield from _strings(nested)
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
        candidates.extend(match.group(1) for match in FILE_TOKEN.finditer(text))
        if any(character in text for character in " /\\"):
            try:
                candidates.extend(
                    token for token in shlex.split(text) if "/" in token or "\\" in token
                )
            except Exception:
                pass
        for raw in candidates:
            relative = _safe_relative(raw, cwd, root)
            if relative is not None:
                found.add(relative)
    return found


def _read(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except (FileNotFoundError, OSError):
        return ""


def _replace_block(text: str, start: str, end: str, block: str) -> str:
    if start in text and end in text:
        before = text.split(start, 1)[0].rstrip()
        after = text.split(end, 1)[1].lstrip("\n")
        pieces = [before, block.rstrip()]
        if after:
            pieces.append(after.rstrip())
        return "\n\n".join(piece for piece in pieces if piece) + "\n"
    base = text.rstrip()
    return base + ("\n\n" if base else "") + block.rstrip() + "\n"


def _ensure_base(path: Path) -> str:
    text = _read(path)
    if text.strip():
        return text
    return (
        "---\n"
        "schema: memhooks/v2\n"
        "inherits: true\n"
        "entities: []\n"
        "resources: []\n"
        "backends: {}\n"
        "---\n\n"
        "# MemHooks\n"
    )


def _parse_auto_paths(text: str) -> list[str]:
    if AUTO_START not in text or AUTO_END not in text:
        return []
    body = text.split(AUTO_START, 1)[1].split(AUTO_END, 1)[0]
    return re.findall(r"^- `([^`]+)`\s*$", body, flags=re.M)


def _auto_block(paths: list[str]) -> str:
    listing = "\n".join(f"- `{path}`" for path in paths)
    return (
        f"{AUTO_START}\n"
        "## Auto-maintained recall anchors\n\n"
        "Before substantive work here, recall prior **decisions, constraints, "
        "failures, fixes, rejected approaches, and unresolved issues** involving:\n"
        f"{listing}\n\n"
        "These are retrieval cues, not memory contents. This deterministic path "
        "maintainer does not guess semantic priority, agent roles, entities, "
        "resources, or backend-specific controls. The active agent may add a "
        "semantic cue when those details are actually known.\n"
        f"{AUTO_END}"
    )


def _update_directory(root: Path, relative_files: Iterable[Path]) -> int:
    grouped: dict[Path, list[str]] = {}
    for relative in relative_files:
        grouped.setdefault(relative.parent, []).append(relative.as_posix())

    writes = 0
    for relative_directory, additions in grouped.items():
        directory = root / relative_directory
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


def _normalize_entity(raw: Any) -> dict[str, Any] | str | None:
    if isinstance(raw, str):
        name = " ".join(raw.strip().split())
        return name or None
    if not isinstance(raw, dict):
        return None
    name = " ".join(str(raw.get("name") or raw.get("text") or "").strip().split())
    if not name:
        return None
    entity: dict[str, Any] = {"name": name}
    entity_type = str(raw.get("type") or "").strip()
    if entity_type:
        entity["type"] = entity_type
    if "salience" in raw:
        salience = _valid_unit_interval(raw.get("salience"))
        if salience is not None:
            entity["salience"] = salience
    return entity


def _normalize_resource(raw: Any) -> dict[str, Any] | str | None:
    if isinstance(raw, str):
        name = " ".join(raw.strip().split())
        return name or None
    if not isinstance(raw, dict):
        return None
    name = " ".join(str(raw.get("name") or raw.get("text") or "").strip().split())
    if not name:
        return None
    resource: dict[str, Any] = {"name": name}
    kind = str(raw.get("kind") or "").strip()
    if kind:
        resource["kind"] = kind
    if "salience" in raw:
        salience = _valid_unit_interval(raw.get("salience"))
        if salience is not None:
            resource["salience"] = salience
    return resource


def _normalize_backends(value: Any) -> dict[str, dict[str, Any]]:
    if not isinstance(value, dict):
        return {}
    normalized: dict[str, dict[str, Any]] = {}
    for provider, config in value.items():
        provider_name = str(provider).strip()
        if provider_name and isinstance(config, dict):
            normalized[provider_name] = copy.deepcopy(config)
    return normalized


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
            " ".join(str(role).strip().split())
            for role in (when.get("roles") or [])
            if " ".join(str(role).strip().split())
        ]
        if roles:
            note["when"] = {"roles": list(dict.fromkeys(roles))}

    entities = []
    for raw in value.get("entities") or []:
        entity = _normalize_entity(raw)
        if entity is not None and entity not in entities:
            entities.append(entity)
    if entities:
        note["entities"] = entities

    resources = []
    for raw in value.get("resources") or []:
        resource = _normalize_resource(raw)
        if resource is not None and resource not in resources:
            resources.append(resource)
    if resources:
        note["resources"] = resources

    tags = [
        " ".join(str(tag).strip().split())
        for tag in (value.get("tags") or [])
        if " ".join(str(tag).strip().split())
    ]
    if tags:
        note["tags"] = list(dict.fromkeys(tags))

    backends = _normalize_backends(value.get("backends"))
    if backends:
        note["backends"] = backends

    return note


def _notes_from(text: str) -> list[dict[str, Any]]:
    if NOTES_START not in text or NOTES_END not in text:
        return []

    body = text.split(NOTES_START, 1)[1].split(NOTES_END, 1)[0]
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
        "These are routing cues only. `priority`, `when.roles`, entities, resources, "
        "and tags are backend-neutral. Provider-native controls belong only under "
        "the cue's `backends.<provider>` object. Durable facts remain in the memory "
        "backend rather than in this file.\n"
        f"{NOTES_END}"
    )


def _parse_entity_arg(raw: str) -> dict[str, Any] | str:
    raw = raw.strip()
    if not raw:
        raise ValueError("empty entity")
    if raw.startswith("{"):
        parsed = json.loads(raw)
        entity = _normalize_entity(parsed)
        if entity is None:
            raise ValueError("entity JSON requires a non-empty name/text")
        if "salience" in parsed and _valid_unit_interval(parsed.get("salience")) is None:
            raise ValueError("entity salience must be between 0.0 and 1.0")
        return entity
    return " ".join(raw.split())


def _parse_resource_arg(raw: str) -> dict[str, Any] | str:
    raw = raw.strip()
    if not raw:
        raise ValueError("empty resource")
    if raw.startswith("{"):
        parsed = json.loads(raw)
        resource = _normalize_resource(parsed)
        if resource is None:
            raise ValueError("resource JSON requires a non-empty name/text")
        if "salience" in parsed and _valid_unit_interval(parsed.get("salience")) is None:
            raise ValueError("resource salience must be between 0.0 and 1.0")
        return resource
    return " ".join(raw.split())


def _parse_backends_arg(raw: str) -> dict[str, dict[str, Any]]:
    try:
        parsed = json.loads(raw)
    except json.JSONDecodeError as exc:
        raise ValueError(f"backends must be valid JSON: {exc}") from exc
    if not isinstance(parsed, dict):
        raise ValueError("backends JSON must be an object")
    normalized = _normalize_backends(parsed)
    if len(normalized) != len(parsed):
        raise ValueError("each backend namespace must map to an object")
    return normalized


def _deep_merge_dict(target: dict[str, Any], overlay: dict[str, Any]) -> dict[str, Any]:
    result = copy.deepcopy(target)
    for key, value in overlay.items():
        if isinstance(result.get(key), dict) and isinstance(value, dict):
            result[key] = _deep_merge_dict(result[key], value)
        else:
            result[key] = copy.deepcopy(value)
    return result


def add_note(
    cwd: Path,
    query: str,
    priority: float | None = None,
    roles: list[str] | None = None,
    entities: list[dict[str, Any] | str] | None = None,
    resources: list[dict[str, Any] | str] | None = None,
    tags: list[str] | None = None,
    backends: dict[str, dict[str, Any]] | None = None,
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

    target_directory = next(
        (directory for directory in (cwd, *cwd.parents) if (directory / HOOK_FILENAME).is_file()),
        cwd,
    )
    try:
        target_directory.relative_to(root)
    except ValueError:
        target_directory = root

    hook = target_directory / HOOK_FILENAME
    text = _ensure_base(hook)
    notes = _notes_from(text)

    incoming_payload: dict[str, Any] = {
        "query": query,
        "entities": entities or [],
        "resources": resources or [],
        "tags": tags or [],
        "backends": backends or {},
    }
    if priority is not None:
        incoming_payload["priority"] = priority
    if normalized_roles:
        incoming_payload["when"] = {"roles": normalized_roles}

    incoming = _normalize_note(incoming_payload)
    if incoming is None:
        return 2

    replaced = False
    for index, existing in enumerate(notes):
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

        for key in ("entities", "resources", "tags"):
            if key not in incoming:
                continue
            prior = list(merged.get(key) or [])
            for item in incoming[key]:
                if item not in prior:
                    prior.append(item)
            if prior:
                merged[key] = prior

        if "backends" in incoming:
            merged["backends"] = _deep_merge_dict(
                dict(merged.get("backends") or {}), incoming["backends"]
            )

        notes[index] = _normalize_note(merged) or incoming
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
        "schema: memhooks/v2\n"
        "inherits: true\n"
        "entities: []\n"
        "resources: []\n"
        "backends: {}\n"
        "---\n\n"
        "# MemHooks\n\n"
        f"Recall prior decisions, constraints, failures, fixes, rejected approaches, "
        f"and unresolved issues concerning the `{name}` project before making "
        "substantive changes.\n\n"
        "The agent/runtime should maintain concise retrieval cues as the project "
        "evolves. Provider-native routing belongs under `backends.<provider>` and "
        "should only be added when the active backend and control are actually known.\n",
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
    parser = argparse.ArgumentParser(
        description="Maintain MEMHOOKS.md retrieval cues with zero extra LLM calls."
    )
    sub = parser.add_subparsers(dest="cmd")
    sub.add_parser("event", help="read a hook event JSON object from stdin")

    init_parser = sub.add_parser("init", help="enable MemHooks in this repository")
    init_parser.add_argument("path", nargs="?", default=".")

    note_parser = sub.add_parser("note", help="add one semantic retrieval cue")
    note_parser.add_argument("--cwd", default=".")
    note_parser.add_argument("--query", required=True)
    note_parser.add_argument(
        "--priority",
        type=float,
        default=None,
        help="optional retrieval priority from 0.0 to 1.0",
    )
    note_parser.add_argument(
        "--role",
        action="append",
        default=[],
        help="optional applicable agent role; repeat as needed",
    )
    note_parser.add_argument(
        "--entity",
        action="append",
        default=[],
        help=(
            "optional entity name or JSON object, e.g. "
            "'{\"name\":\"OpenAI\",\"type\":\"ORG\",\"salience\":0.9}'"
        ),
    )
    note_parser.add_argument(
        "--resource",
        action="append",
        default=[],
        help=(
            "optional named resource or JSON object, e.g. "
            "'{\"name\":\"auth-design\",\"kind\":\"architecture-note\",\"salience\":0.8}'"
        ),
    )
    note_parser.add_argument(
        "--tag",
        action="append",
        default=[],
        help="optional backend-neutral tag; repeat as needed",
    )
    note_parser.add_argument(
        "--backends",
        default="{}",
        help=(
            "optional JSON object of opaque provider-native hints, e.g. "
            "'{\"mem0\":{\"top_k\":8},\"hindsight\":{\"memory_types\":[\"experience\"]}}'"
        ),
    )

    args = parser.parse_args()
    command = args.cmd or "event"

    if command == "init":
        return init(Path(args.path).expanduser().resolve())

    if command == "note":
        if args.priority is not None and _valid_unit_interval(args.priority) is None:
            parser.error("--priority must be between 0.0 and 1.0")
        try:
            entities = [_parse_entity_arg(raw) for raw in args.entity]
            resources = [_parse_resource_arg(raw) for raw in args.resource]
            backends = _parse_backends_arg(args.backends)
        except (ValueError, json.JSONDecodeError) as exc:
            print(f"Invalid MemHooks note metadata: {exc}", file=sys.stderr)
            return 2
        return add_note(
            Path(args.cwd).expanduser().resolve(),
            args.query,
            priority=args.priority,
            roles=args.role,
            entities=entities,
            resources=resources,
            tags=args.tag,
            backends=backends,
        )

    try:
        payload = json.load(sys.stdin)
    except Exception:
        return 0
    return handle_event(payload if isinstance(payload, dict) else {})


if __name__ == "__main__":
    raise SystemExit(main())
