"""Regression cases from the September 2026 review, plus real-binary integration.

The Rust CI job builds the binary and requires these CLI checks to execute.
The Python-only compatibility matrix explicitly skips CLI-dependent checks.
"""
from __future__ import annotations

import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys

import pytest


@pytest.fixture
def repository() -> Path:
    return Path(os.environ.get("MEMHOOKS_REVIEW_REPO", Path(__file__).resolve().parents[1])).resolve()


@pytest.fixture
def loader_path(repository: Path) -> Path:
    path = repository / "hooks" / "hermes" / "memhooks_pre_llm.py"
    assert path.is_file(), "Set MEMHOOKS_REVIEW_REPO to the MemHooks checkout."
    return path


@pytest.fixture
def adapter(loader_path: Path, monkeypatch):
    monkeypatch.setenv("MEMHOOKS_MAX_CHARS", "24000")
    monkeypatch.setenv("MEMHOOKS_MAX_GUIDANCE_CHARS", "6000")
    monkeypatch.setenv("MEMHOOKS_RESOLVE_TIMEOUT", "5")
    spec = importlib.util.spec_from_file_location("memhooks_review_adapter", loader_path)
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def query_plan(count: int = 150) -> dict:
    return {
        "root": "/repo",
        "target": "/repo/src",
        "sources": ["/repo/MEMHOOKS.md"],
        "effective_queries": [
            {
                "source": "/repo/MEMHOOKS.md",
                "query": f"What happened in module {index}?",
                "priority": 0.5,
                "roles": [],
                "entities": [],
                "resources": [],
                "tags": [],
                "backends": {},
            }
            for index in range(count)
        ],
    }


def test_emitted_context_respects_actual_character_budget(adapter):
    context = adapter.build_context(query_plan())
    assert len(context) <= adapter.MAX_TOTAL_CHARS, (
        f"emitted={len(context)}, budget={adapter.MAX_TOTAL_CHARS}"
    )


def test_fallback_itself_respects_character_budget(adapter):
    plan = {
        "root": "/repo",
        "target": "/repo/" + "segment/" * 100,
        "sources": ["/repo/" + "segment/" * i + "MEMHOOKS.md" for i in range(1, 101)],
    }
    bounded = adapter.bounded_plan(adapter.runtime_routing_view(plan))
    compact = json.dumps(bounded, ensure_ascii=False, separators=(",", ":"))
    assert len(compact) <= adapter.MAX_TOTAL_CHARS


@pytest.mark.parametrize("payload", ["[]", "null"])
def test_nonobject_event_degrades_to_empty_hook_response(loader_path: Path, payload: str):
    result = subprocess.run(
        [sys.executable, str(loader_path)], input=payload, text=True,
        capture_output=True, timeout=10, check=False,
    )
    assert result.returncode == 0, result.stderr
    assert json.loads(result.stdout) == {}


def test_nonstring_cwd_degrades_to_empty_hook_response(loader_path: Path):
    result = subprocess.run(
        [sys.executable, str(loader_path)],
        input=json.dumps({"hook_event_name": "pre_llm_call", "cwd": 17}),
        text=True, capture_output=True, timeout=10, check=False,
    )
    assert result.returncode == 0, result.stderr
    assert json.loads(result.stdout) == {}


def test_bad_budget_config_does_not_crash_protocol(loader_path: Path):
    env = dict(os.environ, MEMHOOKS_MAX_CHARS="invalid")
    result = subprocess.run(
        [sys.executable, str(loader_path)], input="{}", env=env,
        text=True, capture_output=True, timeout=10, check=False,
    )
    assert result.returncode == 0, result.stderr
    assert json.loads(result.stdout) == {}


@pytest.fixture
def binary(repository: Path) -> Path:
    configured = os.environ.get("MEMHOOKS_REVIEW_BIN")
    suffix = ".exe" if os.name == "nt" else ""
    path = Path(configured).expanduser() if configured else repository / "target" / "debug" / f"memhooks{suffix}"
    if not path.is_file():
        if os.getenv("MEMHOOKS_REQUIRE_BINARY") == "1":
            pytest.fail("Rust integration job requires a built MemHooks binary")
        pytest.skip("Rust CLI not available in Python-only compatibility job")
    return path.resolve()


@pytest.fixture
def project(tmp_path: Path, monkeypatch) -> Path:
    monkeypatch.delenv("MEMHOOKS_ROOT", raising=False)
    monkeypatch.delenv("MEMHOOKS_AUTO_PATHS", raising=False)
    root = tmp_path / "project"
    (root / ".git").mkdir(parents=True)
    (root / "src").mkdir()
    return root


def run_cli(binary: Path, cwd: Path, *args: str, payload: dict | None = None):
    return subprocess.run(
        [str(binary), *map(str, args)], cwd=cwd,
        input=json.dumps(payload) if payload is not None else None,
        text=True, capture_output=True, timeout=10, check=False,
    )


def enable(root: Path, routing: str = "") -> Path:
    hook = root / "MEMHOOKS.md"
    hook.write_text(f"---\nschema: memhooks/v2\n{routing}---\n", encoding="utf-8")
    return hook


def test_cli_init_from_nested_default_targets_repository_root(binary, project):
    result = run_cli(binary, project / "src", "init")
    assert result.returncode == 0, result.stderr
    assert (project / "MEMHOOKS.md").is_file()
    assert not (project / "src" / "MEMHOOKS.md").exists()


def test_cli_note_default_cwd_finds_enabled_root(binary, project):
    hook = enable(project)
    result = run_cli(binary, project / "src", "note", "--query", "Nested relative note")
    assert result.returncode == 0, result.stderr
    assert "Nested relative note" in hook.read_text(encoding="utf-8")


def test_cli_validate_all_from_nested_detects_root_error(binary, project):
    (project / "MEMHOOKS.md").write_text("---\nschema: memhooks/v1\n---\n", encoding="utf-8")
    result = run_cli(binary, project / "src", "validate", "--all", "--format", "json")
    assert result.returncode != 0, result.stdout
    assert any(d.get("code") == "MH001" for d in json.loads(result.stdout))


def test_cli_explain_does_not_execute_malformed_role_routing(binary, project):
    enable(project, 'recall_queries:\n  - query: "Restricted query"\n    when:\n      roles: reviewer\n')
    invalid = run_cli(binary, project, "validate", str(project), "--format", "json")
    assert invalid.returncode != 0, "Fixture must be invalid before testing explain."
    result = run_cli(binary, project, "explain", str(project), "--role", "feature_dev", "--format", "json")
    assert result.returncode != 0, "explain accepted a query whose role metadata failed validation"


def test_cli_note_rejects_invalid_backend_shape_without_modification(binary, project):
    hook = enable(project)
    before = hook.read_bytes()
    result = run_cli(binary, project, "note", "--cwd", str(project), "--query", "Bad provider", "--backends", '{"mem0":42}')
    assert result.returncode != 0, result.stdout
    assert hook.read_bytes() == before


def test_cli_note_rejects_invalid_entity_without_modification(binary, project):
    hook = enable(project)
    before = hook.read_bytes()
    result = run_cli(binary, project, "note", "--cwd", str(project), "--query", "Bad entity", "--entity", '{"name":[]}')
    assert result.returncode != 0, result.stdout
    assert hook.read_bytes() == before


def test_cli_event_accepts_parentheses_in_paths(binary, project):
    enable(project)
    target = project / "app" / "(auth)" / "login.py"
    target.parent.mkdir(parents=True)
    target.write_text("pass\n", encoding="utf-8")
    result = run_cli(binary, project, "event", payload={
        "hook_event_name": "post_tool_call", "cwd": str(project),
        "tool_name": "write_file", "tool_input": {"path": "app/(auth)/login.py"},
    })
    assert result.returncode == 0, result.stderr
    assert (target.parent / "MEMHOOKS.md").is_file()


def external_hook(project: Path) -> tuple[Path, Path]:
    outside = project.parent / "external-MEMHOOKS.md"
    outside.write_text('---\nschema: memhooks/v2\nrecall_queries: [external-private-cue]\n---\n', encoding="utf-8")
    alias = project / "src" / "MEMHOOKS.md"
    try:
        alias.symlink_to(outside)
    except (OSError, NotImplementedError) as exc:
        pytest.skip(f"Symlinks unavailable in this test environment: {exc}")
    return outside, alias


def test_cli_resolution_does_not_read_external_symlinked_hook(binary, project):
    enable(project)
    external_hook(project)
    result = run_cli(binary, project, "explain", str(project / "src"), "--format", "json")
    assert "external-private-cue" not in result.stdout


def test_cli_note_does_not_copy_external_symlinked_hook(binary, project):
    enable(project)
    outside, alias = external_hook(project)
    before = outside.read_bytes()
    result = run_cli(binary, project, "note", "--cwd", str(project / "src"), "--query", "Local cue")
    assert result.returncode != 0
    assert alias.is_symlink()
    assert outside.read_bytes() == before


@pytest.mark.parametrize("budget", [1, 160, 200, 350, 1000, 24000])
def test_exact_budget_with_unicode_and_large_constraints(adapter, budget):
    adapter.MAX_TOTAL_CHARS = budget
    plan = query_plan(60)
    plan["exclude"] = ["禁止" * 1000]
    plan["guidance"] = [{"source": "/repo/MEMHOOKS.md", "value": "é🦉" * 9000}]
    context = adapter.build_context(plan)
    assert len(context) <= budget
    if context:
        value = json.loads(context[context.index("{"):])
        if value.get("recall_queries"):
            assert value["exclude"] == plan["exclude"]


@pytest.mark.parametrize("timeout", ["nan", "inf", "-1", "oops"])
def test_invalid_timeout_is_a_normal_empty_response(loader_path, timeout):
    result = subprocess.run([sys.executable, str(loader_path)], input="{}", text=True,
                            capture_output=True, check=False,
                            env=dict(os.environ, MEMHOOKS_RESOLVE_TIMEOUT=timeout))
    assert result.returncode == 0
    assert json.loads(result.stdout) == {}
    assert "TIMEOUT" in result.stderr


@pytest.mark.parametrize("path_arg", [None, ".", ".."])
def test_nested_init_path_spellings_agree(binary, project, path_arg):
    args = ["init"] + ([path_arg] if path_arg else [])
    result = run_cli(binary, project / "src", *args)
    assert result.returncode == 0, result.stderr
    assert (project / "MEMHOOKS.md").exists()
    assert not (project / "src/MEMHOOKS.md").exists()


def test_explicit_root_and_relative_note_choose_nearest_hook(binary, project, monkeypatch):
    enable(project)
    local = enable(project / "src")
    monkeypatch.setenv("MEMHOOKS_ROOT", "..")
    result = run_cli(binary, project / "src", "note", "--cwd", ".", "--query", "Local note")
    assert result.returncode == 0, result.stderr
    assert "Local note" in local.read_text()
    assert "Local note" not in (project / "MEMHOOKS.md").read_text()


def test_invalid_existing_hook_is_never_rewritten(binary, project):
    hook = enable(project, "recall_queries:\n  - query: bad\n    priority: 9.0\n")
    before = hook.read_bytes()
    for args in (("init",), ("note", "--query", "new")):
        result = run_cli(binary, project, *args)
        assert result.returncode != 0
        assert hook.read_bytes() == before


def test_hook_and_lock_symlinks_are_rejected(binary, project):
    hook = enable(project)
    original = hook.read_bytes()
    outside = project.parent / "outside.lock"
    outside.write_text("do not touch")
    lock = project / ".MEMHOOKS.md.lock"
    try:
        lock.symlink_to(outside)
    except OSError:
        pytest.skip("symlink creation unavailable")
    result = run_cli(binary, project, "note", "--query", "bad lock")
    assert result.returncode != 0
    assert hook.read_bytes() == original
    assert outside.read_text() == "do not touch"
    assert lock.is_symlink()


def test_validate_all_reports_hook_symlinks(binary, project):
    enable(project)
    external_hook(project)
    result = run_cli(binary, project, "validate", "--all", "--format", "json")
    assert result.returncode != 0
    assert any(item["code"] == "MH029" for item in json.loads(result.stdout))


def run_real_loader(binary, loader_path, project, **fields):
    payload = {"hook_event_name": "pre_llm_call", "cwd": str(project), **fields}
    result = subprocess.run([sys.executable, str(loader_path)], input=json.dumps(payload),
                            text=True, capture_output=True, check=False, timeout=20,
                            env=dict(os.environ, MEMHOOKS_BIN=str(binary)))
    assert result.returncode == 0, result.stderr
    response = json.loads(result.stdout)
    if not response:
        return response
    context = response["context"]
    assert len(context) <= 24000
    return json.loads(context[context.index("{"):])


def test_real_adapter_targets_active_files_and_applies_roles(binary, loader_path, project):
    enable(project, "recall_queries: [root-cue]\n")
    enable(project / "src", 'recall_queries:\n  - query: local-review\n    when:\n      roles: [reviewer]\n')
    (project / "src/code.py").write_text("pass")
    value = run_real_loader(binary, loader_path, project,
                            active_files=["src/code.py"], active_roles=["reviewer"])
    assert {q["query"] for q in value["recall_queries"]} == {"root-cue", "local-review"}
    filtered = run_real_loader(binary, loader_path, project,
                               active_files=["src/code.py"], active_roles=["developer"])
    assert {q["query"] for q in filtered["recall_queries"]} == {"root-cue"}
    assert any(q["reason"] == "role_mismatch" for q in filtered["omitted_queries"])


def test_real_adapter_preserves_sibling_constraints(binary, loader_path, project):
    enable(project)
    paths = []
    for name in ("left", "right"):
        directory = project / name
        directory.mkdir()
        enable(directory, f'scope: {name}\nexclude: ["{name}-excluded"]\nrecall_queries: ["{name}-query"]\n')
        (directory / "code.py").write_text("pass")
        paths.append(f"{name}/code.py")
    value = run_real_loader(binary, loader_path, project, active_files=paths)
    assert len(value["contexts"]) == 2
    for query in value["recall_queries"]:
        context = value["contexts"][query["contexts"][0]]
        assert query["query"] == f'{context["scope"]}-query'
        assert context["exclude"] == [f'{context["scope"]}-excluded']


def test_active_child_override_is_not_resurrected_by_cwd(binary, loader_path, project):
    enable(project, 'recall_queries:\n  - query: same\n    tags: [old]\n')
    enable(project / "src", 'recall_queries:\n  - query: same\n    tags: [new]\n')
    (project / "src/code.py").write_text("pass")
    value = run_real_loader(binary, loader_path, project, active_files=["src/code.py"])
    assert len(value["recall_queries"]) == 1
    assert value["recall_queries"][0]["tags"] == ["new"]
    assert any(item["reason"] == "overridden" for item in value["omitted_queries"])


def test_invalid_active_child_does_not_fall_back_to_root(binary, loader_path, project):
    enable(project, "recall_queries: [root-cue]\n")
    enable(project / "src", 'recall_queries:\n  - query: invalid\n    priority: 2\n')
    (project / "src/code.py").write_text("pass")
    assert run_real_loader(binary, loader_path, project, active_files=["src/code.py"]) == {}


def test_auto_scopes_and_delete_rename_lifecycle(binary, project):
    enable(project)
    for name in ("top.py", "src/old.py"):
        (project / name).write_text("pass")
    event = {"hook_event_name": "post_tool_call", "cwd": str(project),
             "tool_input": {"paths": ["top.py", "src/old.py"]}}
    assert run_cli(binary, project, "event", payload=event).returncode == 0
    resolved = run_cli(binary, project, "explain", "src", "--format", "json")
    auto = [q for q in json.loads(resolved.stdout)["effective_queries"] if "memhooks:auto" in q["tags"]]
    assert len(auto) == 2
    (project / "src/old.py").rename(project / "src/new.py")
    event["tool_input"] = {"source": "src/old.py", "destination": "src/new.py"}
    assert run_cli(binary, project, "event", payload=event).returncode == 0
    text = (project / "src/MEMHOOKS.md").read_text()
    assert "src/new.py" in text and "src/old.py" not in text
    (project / "src/new.py").unlink()
    event["tool_input"] = {"path": "src/new.py"}
    assert run_cli(binary, project, "event", payload=event).returncode == 0
    assert "memhooks:auto" not in (project / "src/MEMHOOKS.md").read_text()


def test_prune_and_remove_have_nonmutating_previews(binary, project):
    hook = enable(project, 'recall_queries:\n  - query: manual\n  - query: generated\n'
                  '    tags: ["memhooks:auto"]\n    resources:\n      - name: missing.py\n        kind: file\n')
    before = hook.read_bytes()
    preview = run_cli(binary, project, "prune", "--all", "--dry-run")
    assert preview.returncode == 0, preview.stderr
    assert json.loads(preview.stdout)["resources_removed"] == 1
    assert hook.read_bytes() == before
    assert not (project / ".MEMHOOKS.md.lock").exists()
    preview = run_cli(binary, project, "remove", "--query", "manual", "--dry-run")
    assert preview.returncode == 0 and json.loads(preview.stdout)["changed"]
    assert hook.read_bytes() == before
    assert run_cli(binary, project, "prune", "--all").returncode == 0
    assert "manual" in hook.read_text() and "generated" not in hook.read_text()
    assert run_cli(binary, project, "remove", "--query", "manual").returncode == 0
    assert "manual" not in hook.read_text()


@pytest.mark.parametrize("malformed", [[], None, {"plan_version": "old"},
    {"plan_version": "memhooks/plan-v1", "sources": 42},
    {"plan_version": "memhooks/plan-v1", "root": "/repo", "target": "/repo", "sources": [],
     "effective_queries": [], "omitted_queries": [], "guidance": [42]}])
def test_malformed_resolver_output_is_not_injected(adapter, monkeypatch, malformed):
    from types import SimpleNamespace
    monkeypatch.setattr(adapter.subprocess, "run", lambda *args, **kwargs:
                        SimpleNamespace(returncode=0, stdout=json.dumps(malformed), stderr=""))
    assert adapter.resolve_plan("unused", Path("/repo")) is None


def test_dangling_local_hook_is_not_skipped_when_writing(binary, project):
    root_hook = enable(project)
    before = root_hook.read_bytes()
    try:
        (project / "src/MEMHOOKS.md").symlink_to(project / "missing.md")
    except OSError:
        pytest.skip("symlink creation unavailable")
    result = run_cli(binary, project / "src", "note", "--query", "Must not move to root")
    assert result.returncode != 0
    assert root_hook.read_bytes() == before


def test_non_json_backend_values_report_error_not_panic(binary, project):
    enable(project, "backends:\n  custom:\n    ? [a, b]\n    : value\nrecall_queries: [test]\n")
    result = run_cli(binary, project, "explain", "--format", "json")
    assert result.returncode != 0
    assert "panicked" not in result.stderr
    assert "JSON" in result.stderr
