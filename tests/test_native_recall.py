"""Native Hermes callback contract -> real Rust CLI -> real loopback HTTP -> context.

No cloud calls or live credentials. Binary checks are mandatory in Rust CI and
explicitly skipped in the Python-only matrix, like the existing integration suite.
"""
from collections import Counter
import importlib.util
import json
import os
from pathlib import Path
import sys
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

import pytest

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location("memhooks_native_test", ROOT / "hooks/hermes/__init__.py",
                                            submodule_search_locations=[str(ROOT / "hooks/hermes")])
native = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = native
SPEC.loader.exec_module(native)
runtime = sys.modules[SPEC.name + ".recall_runtime"]
transport = sys.modules[SPEC.name + ".hindsight_recall"]


@pytest.fixture
def server():
    class Handler(BaseHTTPRequestHandler):
        def do_POST(self):
            body = self.rfile.read(int(self.headers["Content-Length"]))
            self.server.calls.append((self.path, dict(self.headers), json.loads(body)))
            if self.server.delay:
                time.sleep(self.server.delay)
            self.send_response(self.server.status)
            if self.server.status == 302:
                self.send_header("Location", self.server.url + "/do-not-follow")
            self.end_headers()
            try:
                data = self.server.response
                self.wfile.write(data if isinstance(data, bytes) else json.dumps(data).encode())
            except (BrokenPipeError, ConnectionResetError):
                pass

        def log_message(self, *_args):
            pass

    http = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    http.url = f"http://127.0.0.1:{http.server_port}"
    http.calls, http.delay, http.status = [], 0, 200
    http.response = {"results": [{"id": "fact-1", "text": "Use idempotency keys.",
                                 "type": "experience", "tags": ["project:test"], "document_id": "incident-7"}]}
    thread = threading.Thread(target=http.serve_forever, kwargs={"poll_interval": 0.02}, daemon=True)
    thread.start()
    try:
        yield http
    finally:
        http.shutdown()
        http.server_close()
        thread.join(timeout=2)


@pytest.fixture
def project(tmp_path, monkeypatch):
    monkeypatch.delenv("MEMHOOKS_ROOT", raising=False)
    root = tmp_path / "project"
    (root / ".git").mkdir(parents=True)
    (root / "src").mkdir()
    (root / "src/code.py").write_text("pass\n")
    hook(root, "recall_queries: [root-cue]\n")
    return root


def hook(path, routing=""):
    (path / "MEMHOOKS.md").write_text("---\nschema: memhooks/v2\n" + routing + "---\n")


def settings(project, server, binary=sys.executable):
    return {"enabled": True, "project_root": str(project), "binary": str(binary), "roles": ["reviewer"],
            "maintain": False, "timeout_seconds": 3, "max_queries": 3, "max_context_chars": 12000,
            "hindsight": {"base_url": server.url, "bank": "test-bank", "required_tags": ["project:test"]}}


def plan(project, target=None, **changes):
    return {"plan_version": "memhooks/plan-v1", "root": str(project), "target": str(target or project),
            "sources": [str(project / "MEMHOOKS.md")], "guidance": [], "omitted_queries": [],
            "effective_queries": [{"query": "What mattered?", "source": str(project / "MEMHOOKS.md"),
                                   "priority": None, "tags": [], "resources": [], "entities": [], "backends": {}}],
            **changes}


def unpack(response):
    assert response and "context" in response
    text = response["context"]
    assert text.startswith(runtime.PREFIX)
    return json.loads(text[len(runtime.PREFIX):])


def make_runtime(cfg, profile=lambda: "profile-a", secret=lambda _name: "test-token"):
    return runtime.RecallRuntime(lambda: cfg, profile, secret)


@pytest.fixture
def binary():
    suffix = ".exe" if os.name == "nt" else ""
    path = Path(os.getenv("MEMHOOKS_REVIEW_BIN", ROOT / "target/debug" / ("memhooks" + suffix)))
    if not path.is_file():
        if os.getenv("MEMHOOKS_REQUIRE_BINARY") == "1":
            pytest.fail("compiled MemHooks binary is required")
        pytest.skip("Rust executable unavailable in Python-only compatibility job")
    return path.resolve()


def test_real_native_event_maintains_cue_then_retrieves_memories(project, server, binary):
    cfg = settings(project, server, binary)
    cfg["maintain"] = True
    instance = make_runtime(cfg)
    instance.post_tool_call(session_id="s1", task_id="native-task", status="ok", tool_name="read_file",
                            args={"path": "src/code.py"})
    local = project / "src/MEMHOOKS.md"
    assert local.is_file() and "src/code.py" in local.read_text()
    response = instance.pre_llm_call(session_id="s1", turn_id="t2", user_message="Continue")
    data = unpack(response)
    assert data["memories"][0]["text"] == "Use idempotency keys."
    assert data["memories"][0]["origin"]["target"] == str(project / "src")
    assert len(data["memories"]) == 1  # same fact selected by two different cues
    assert len(data["memories"][0]["sources"]) == 2
    assert len(response["context"]) <= cfg["max_context_chars"]
    assert "test-token" not in response["context"]
    assert server.calls and all(call[0] == "/v1/default/banks/test-bank/memories/recall" for call in server.calls)
    assert all(call[1]["Authorization"] == "Bearer test-token" for call in server.calls)
    assert all(call[2]["tags_match"] == "all_strict" for call in server.calls)


def test_real_roles_and_child_override_before_retrieval(project, server, binary):
    hook(project, "recall_queries: [same]\n")
    hook(project / "src", "recall_queries:\n  - query: same\n    when:\n      roles: [architect]\n")
    instance = make_runtime(settings(project, server, binary))
    data = unpack(instance.pre_llm_call(session_id="s", active_files=["src/code.py"], active_roles=["architect"]))
    assert data["queries_executed"] == 0 and not server.calls  # event cannot replace host roles


def test_real_invalid_child_never_falls_back_to_root(project, server, binary):
    hook(project / "src", "recall_queries:\n  - query: bad\n    priority: 99\n")
    instance = make_runtime(settings(project, server, binary))
    assert instance.pre_llm_call(session_id="s", active_files=["src/code.py"]) is None
    assert not server.calls


def test_real_sibling_exclusions_stay_separate(project, server, binary):
    hook(project, "")
    for name, excluded in (("left", "idempotency"), ("right", "obsolete")):
        directory = project / name
        directory.mkdir()
        (directory / "code.py").write_text("pass")
        hook(directory, f"scope: {name}\nexclude: [{excluded}]\nrecall_queries: [question]\n")
    result = unpack(make_runtime(settings(project, server, binary)).pre_llm_call(
        session_id="s", active_files=["left/code.py", "right/code.py"]))
    assert len(result["memories"]) == 1
    assert result["memories"][0]["origin"]["scope"] == "right"
    assert result["issues"]["excluded"] == 1


def test_real_delete_event_prunes_local_generated_cue(project, server, binary):
    cfg = settings(project, server, binary)
    cfg["maintain"] = True
    instance = make_runtime(cfg)
    event = {"session_id": "s", "status": "ok", "args": {"path": "src/code.py"}}
    instance.post_tool_call(**event)
    (project / "src/code.py").unlink()
    instance.post_tool_call(**event)
    assert "memhooks:auto" not in (project / "src/MEMHOOKS.md").read_text()


def test_register_is_network_free_and_uses_native_hook_contract(monkeypatch):
    class Context:
        def __init__(self):
            self.hooks = {}
        def get_config(self, key, default):
            assert key == "runtime"
            return default
        def register_hook(self, name, callback):
            self.hooks[name] = callback
    monkeypatch.setattr(native, "profile_home", lambda: "test-profile")
    monkeypatch.setattr(native, "scoped_secret", lambda _name: pytest.fail("no credential read when disabled"))
    ctx = Context()
    native.register(ctx)
    assert set(ctx.hooks) == {"pre_llm_call", "post_tool_call", "on_session_finalize", "on_session_reset"}
    assert ctx.hooks["pre_llm_call"](session_id="s", turn_id="t", future_field=True) is None
    ctx.hooks["post_tool_call"](session_id="s", status="ok", args={"path": "x"})


def test_session_profile_isolation_and_finalize_not_turn_end(project, server, monkeypatch):
    current = ["profile-a"]
    instance = make_runtime(settings(project, server), lambda: current[0])
    instance.post_tool_call(session_id="a", task_id="task", status="ok", args={"path": "src/code.py"})
    selected = []
    monkeypatch.setattr(runtime.routing, "resolve_plan", lambda _bin, target, *_args: selected.append(target) or plan(project, target))
    instance.pre_llm_call(session_id="a")
    assert selected[-1] == project / "src"
    instance.pre_llm_call(session_id="b")
    assert selected[-1] == project
    current[0] = "profile-b"
    instance.pre_llm_call(session_id="a")
    assert selected[-1] == project
    current[0] = "profile-a"
    instance.clear_session(old_session_id="a", new_session_id="c")
    instance.pre_llm_call(session_id="a")
    assert selected[-1] == project


@pytest.mark.parametrize("status", ["blocked", "error", "cancelled", None])
def test_unsuccessful_tools_do_not_create_activity(project, server, status):
    instance = make_runtime(settings(project, server))
    instance.post_tool_call(session_id="s", status=status, args={"path": "src/code.py"})
    assert not instance.sessions


def test_explicit_paths_only_and_root_containment(project, tmp_path):
    outside = tmp_path / "private.txt"
    outside.write_text("secret")
    link = project / "src/link.txt"
    link.symlink_to(outside)
    args = {"content": 'path: "src/code.py"', "command": "cat src/code.py", "path": str(link)}
    assert runtime.explicit_files(args, project, project) == []
    assert runtime.explicit_files({"file_path": "src/code.py"}, project, project) == [str(project / "src/code.py")]


@pytest.mark.parametrize("native_options,issue", [
    ({"bank": "somebody-elses-bank"}, "bank_not_authorized"),
    ({"strategy": "reflect"}, "unsupported_strategy"),
    ({"base_url": "https://attacker.invalid"}, "unsupported_provider_options"),
    ({"api_key": "repo-selected-secret"}, "unsupported_provider_options"),
    ({"memory_types": ["invented"]}, "invalid_memory_types"),
])
def test_repo_cannot_expand_backend_authority(project, server, monkeypatch, native_options, issue):
    value = plan(project)
    value["effective_queries"][0]["backends"] = {"hindsight": native_options}
    monkeypatch.setattr(runtime.routing, "resolve_plan", lambda *_args: value)
    data = unpack(make_runtime(settings(project, server)).pre_llm_call(session_id="s"))
    assert data["issues"][issue] == 1 and not server.calls


def test_sensitivity_requires_host_opt_in(project, server, monkeypatch):
    monkeypatch.setattr(runtime.routing, "resolve_plan", lambda *_args: plan(project, sensitivity="private"))
    cfg = settings(project, server)
    instance = make_runtime(cfg)
    assert unpack(instance.pre_llm_call(session_id="s"))["issues"]["sensitivity_not_authorized"] == 1
    assert not server.calls
    cfg["allowed_sensitivities"] = ["private"]
    assert unpack(instance.pre_llm_call(session_id="s"))["memories"]


def test_response_filters_and_provenance(project, server, monkeypatch):
    good = server.response["results"][0]
    server.response["results"] = [good, good, {**good, "id": "wrongtag", "tags": []},
                                 {**good, "id": "old", "metadata": {"note": "obsolete"}},
                                 {**good, "id": "wrongtype", "type": "world"}]
    value = plan(project, exclude=["obsolete"])
    value["effective_queries"][0]["backends"] = {"hindsight": {"memory_types": ["experience"]}}
    monkeypatch.setattr(runtime.routing, "resolve_plan", lambda *_args: value)
    data = unpack(make_runtime(settings(project, server)).pre_llm_call(session_id="s"))
    assert len(data["memories"]) == 1
    assert data["memories"][0]["document_id"] == "incident-7"
    assert data["issues"] == {"tag_mismatch": 1, "excluded": 1, "type_mismatch": 1}


def test_priority_query_cap_and_no_whole_conversation_upload(project, server, monkeypatch):
    value = plan(project)
    value["effective_queries"] = [{**value["effective_queries"][0], "query": f"q{i}", "priority": i / 10} for i in range(7)]
    monkeypatch.setattr(runtime.routing, "resolve_plan", lambda *_args: value)
    data = unpack(make_runtime(settings(project, server)).pre_llm_call(session_id="s", user_message="DO NOT UPLOAD ME"))
    assert [call[2]["query"] for call in server.calls] == ["q6", "q5", "q4"]
    assert data["queries_executed"] == 3 and data["issues"]["query_limit"] == 4
    assert "DO NOT UPLOAD ME" not in json.dumps(server.calls)


@pytest.mark.parametrize("budget", [256, 500, 1200, 12000])
def test_complete_memory_budget_with_unicode(budget):
    records = [{"id": str(i), "text": "完整的记忆 🦉" * 100} for i in range(20)]
    output = {"schema": "memhooks/memories-v1", "queries_executed": 1, "issues": Counter(),
              "memories": records, "guidance": [{"source": "/repo", "value": "x" * 100000}]}
    context = runtime.pack_context(output, budget)
    assert len(context) <= budget
    if context:
        data = json.loads(context[len(runtime.PREFIX):])
        assert all(item in records for item in data["memories"])


@pytest.mark.parametrize("response", [b"not json", [], {"results": [None]},
                                      {"results": [{"id": "a", "text": "b", "tags": "project:test"}]},
                                      b"x" * (transport.MAX_RESPONSE_BYTES + 1)])
def test_malformed_or_oversized_backend_response_is_safe(server, response):
    server.response = response
    results, status = transport.recall(server.url, "test-bank", "test-token", {"query": "q"}, 2)
    assert not results and status == "backend_error"


@pytest.mark.parametrize("code,expected", [(401, "unauthorized"), (403, "unauthorized"), (500, "backend_error"), (302, "backend_error")])
def test_http_errors_and_redirects_are_not_retried(server, code, expected):
    server.status = code
    results, status = transport.recall(server.url, "test-bank", "test-token", {"query": "q"}, 2)
    assert not results and status == expected and len(server.calls) == 1


def test_hard_worker_timeout(server):
    server.delay = 1
    started = time.monotonic()
    results, status = transport.recall(server.url, "test-bank", "test-token", {"query": "q"}, 0.15)
    assert not results and status == "timeout"
    assert time.monotonic() - started < 0.8


@pytest.mark.parametrize("url", ["http://example.com", "http://localhost:8888", "https://user:password@example.com",
                                "https://example.com?api_key=secret", "file:///tmp/memory"])
def test_unsafe_backend_urls_rejected(url):
    with pytest.raises(ValueError):
        transport.endpoint(url, "bank")


def test_missing_or_unscoped_credentials_fail_closed(project, server, monkeypatch, caplog):
    monkeypatch.setattr(runtime.routing, "resolve_plan", lambda *_args: plan(project))
    cfg = settings(project, server)
    assert make_runtime(cfg, secret=lambda _name: "").pre_llm_call(session_id="s") is None
    def unavailable(_name):
        raise RuntimeError("do not log SECRET-TOKEN")
    assert make_runtime(cfg, secret=unavailable).pre_llm_call(session_id="s") is None
    assert "SECRET-TOKEN" not in caplog.text and not server.calls
    cfg["hindsight"]["allow_anonymous"] = True
    assert unpack(make_runtime(cfg, secret=lambda _name: "").pre_llm_call(session_id="s"))["memories"]


def test_state_has_bounded_sessions_files_and_expiry(project, server, monkeypatch):
    instance = make_runtime(settings(project, server))
    for i in range(150):
        instance._files(("profile", str(project), str(i)), [str(n) for n in range(100)])
    assert len(instance.sessions) == 128
    assert all(len(entry[1]) == 64 for entry in instance.sessions.values())
    now = time.monotonic()
    monkeypatch.setattr(runtime.time, "monotonic", lambda: now + 3601)
    instance._files(("profile", str(project), "new"))
    assert len(instance.sessions) == 1


def test_credentials_are_resolved_in_current_profile_each_call(project, server, monkeypatch):
    monkeypatch.setattr(runtime.routing, "resolve_plan", lambda *_args: plan(project))
    profile = ["alpha"]
    instance = make_runtime(settings(project, server), lambda: profile[0], lambda _name: profile[0] + "-token")
    instance.pre_llm_call(session_id="same-id")
    profile[0] = "beta"
    instance.pre_llm_call(session_id="same-id")
    assert [call[1]["Authorization"] for call in server.calls] == ["Bearer alpha-token", "Bearer beta-token"]


def test_unsafe_cwd_and_absent_session_never_retrieve(project, server, monkeypatch, tmp_path):
    monkeypatch.setattr(runtime.routing, "resolve_plan", lambda *_args: pytest.fail("resolver should not run"))
    instance = make_runtime(settings(project, server))
    assert instance.pre_llm_call(session_id="s", cwd=str(tmp_path)) is None
    assert instance.pre_llm_call() is None
    assert not server.calls


def test_real_inheritance_cut_keeps_local_recall_without_parent_cues(project, server, binary):
    hook(project / "src", "inherits: false\nrecall_queries: [local-only]\n")
    response = make_runtime(settings(project, server, binary)).pre_llm_call(
        session_id="s", active_files=["src/code.py"])
    data = unpack(response)
    assert data["queries_executed"] == 1
    assert server.calls[0][2]["query"] == "local-only"
    assert data["memories"][0]["sources"] == [str(project / "src/MEMHOOKS.md")]
