import subprocess
import sys
from pathlib import Path

SCRIPT = Path(__file__).resolve().parents[1] / "scripts" / "memhooks_update.py"


def run_cli(*args, cwd=None):
    return subprocess.run(
        [sys.executable, str(SCRIPT), *args],
        cwd=cwd,
        capture_output=True,
        text=True,
        check=False,
    )


def make_repo(tmp_path):
    repo = tmp_path / "repo"
    repo.mkdir()
    subprocess.run(["git", "init"], cwd=repo, capture_output=True, check=True)
    return repo


def test_init_seeds_backend_neutral_v2_fields(tmp_path):
    repo = make_repo(tmp_path)

    result = run_cli("init", str(repo))
    assert result.returncode == 0

    text = (repo / "MEMHOOKS.md").read_text(encoding="utf-8")
    assert "schema: memhooks/v2" in text
    assert "entities: []" in text
    assert "resources: []" in text
    assert "backends: {}" in text
    assert "memory_types" not in text
    assert "connection_types" not in text


def test_note_persists_generic_cues_and_opaque_backend_namespaces(tmp_path):
    repo = make_repo(tmp_path)
    assert run_cli("init", str(repo)).returncode == 0

    result = run_cli(
        "note",
        "--cwd",
        str(repo),
        "--query",
        "Why did auth change after the outage?",
        "--entity",
        "Authentication",
        "--resource",
        '{"name":"auth-design","kind":"architecture-note","salience":0.8}',
        "--tag",
        "security",
        "--backends",
        '{"hindsight":{"memory_types":["experience"],"connection_types":["causal","temporal"]},"mem0":{"top_k":8,"rerank":true}}',
    )
    assert result.returncode == 0

    text = (repo / "MEMHOOKS.md").read_text(encoding="utf-8")
    assert '"Authentication"' in text
    assert '"resources": [' in text
    assert '"auth-design"' in text
    assert '"kind": "architecture-note"' in text
    assert '"security"' in text
    assert '"hindsight": {' in text
    assert '"memory_types": [' in text
    assert '"experience"' in text
    assert '"mem0": {' in text
    assert '"top_k": 8' in text
    assert '"rerank": true' in text


def test_note_persists_priority_roles_and_entity_salience(tmp_path):
    repo = make_repo(tmp_path)
    assert run_cli("init", str(repo)).returncode == 0

    result = run_cli(
        "note",
        "--cwd",
        str(repo),
        "--query",
        "Which security boundaries are non-negotiable?",
        "--priority",
        "1.0",
        "--role",
        "reviewer",
        "--role",
        "architect",
        "--entity",
        '{"name":"Authentication","type":"CONCEPT","salience":0.95}',
    )
    assert result.returncode == 0

    text = (repo / "MEMHOOKS.md").read_text(encoding="utf-8")
    assert '"priority": 1.0' in text
    assert '"when": {' in text
    assert '"reviewer"' in text
    assert '"architect"' in text
    assert '"salience": 0.95' in text


def test_repeated_note_enriches_generic_and_backend_metadata(tmp_path):
    repo = make_repo(tmp_path)
    assert run_cli("init", str(repo)).returncode == 0

    query = "Why did auth change?"
    assert run_cli(
        "note",
        "--cwd",
        str(repo),
        "--query",
        query,
        "--priority",
        "0.7",
        "--role",
        "reviewer",
        "--tag",
        "auth",
        "--backends",
        '{"mem0":{"filters":{"user_id":"alice"},"threshold":0.2}}',
    ).returncode == 0

    assert run_cli(
        "note",
        "--cwd",
        str(repo),
        "--query",
        query,
        "--priority",
        "0.9",
        "--role",
        "architect",
        "--entity",
        '{"name":"OpenAI","type":"ORG","salience":0.8}',
        "--resource",
        "auth-postmortem",
        "--backends",
        '{"mem0":{"rerank":true,"top_k":5},"hindsight":{"memory_types":["experience"]}}',
    ).returncode == 0

    text = (repo / "MEMHOOKS.md").read_text(encoding="utf-8")
    assert '"priority": 0.9' in text
    assert '"reviewer"' in text
    assert '"architect"' in text
    assert '"auth"' in text
    assert '"filters": {' in text
    assert '"user_id": "alice"' in text
    assert '"threshold": 0.2' in text
    assert '"rerank": true' in text
    assert '"top_k": 5' in text
    assert '"hindsight": {' in text
    assert '"experience"' in text
    assert '"type": "ORG"' in text
    assert '"salience": 0.8' in text
    assert '"auth-postmortem"' in text


def test_invalid_priority_salience_and_backend_shape_are_rejected(tmp_path):
    repo = make_repo(tmp_path)
    assert run_cli("init", str(repo)).returncode == 0

    bad_priority = run_cli(
        "note",
        "--cwd",
        str(repo),
        "--query",
        "bad priority",
        "--priority",
        "1.5",
    )
    assert bad_priority.returncode != 0

    bad_entity_salience = run_cli(
        "note",
        "--cwd",
        str(repo),
        "--query",
        "bad entity salience",
        "--entity",
        '{"name":"OpenAI","salience":-0.1}',
    )
    assert bad_entity_salience.returncode != 0
    assert "salience" in bad_entity_salience.stderr.lower()

    bad_resource_salience = run_cli(
        "note",
        "--cwd",
        str(repo),
        "--query",
        "bad resource salience",
        "--resource",
        '{"name":"design","salience":1.2}',
    )
    assert bad_resource_salience.returncode != 0
    assert "salience" in bad_resource_salience.stderr.lower()

    bad_backends = run_cli(
        "note",
        "--cwd",
        str(repo),
        "--query",
        "bad backend shape",
        "--backends",
        '{"mem0":"not-an-object"}',
    )
    assert bad_backends.returncode != 0
    assert "backend" in bad_backends.stderr.lower()


def test_markdown_bullet_notes_are_normalized_to_structured_json(tmp_path):
    repo = make_repo(tmp_path)
    (repo / "MEMHOOKS.md").write_text(
        """---
schema: memhooks/v2
inherits: true
---

<!-- memhooks:notes:start -->
## In-session retrieval cues

- Why did the old approach fail?

Durable facts belong in the memory backend.
<!-- memhooks:notes:end -->
""",
        encoding="utf-8",
    )

    result = run_cli(
        "note",
        "--cwd",
        str(repo),
        "--query",
        "What replaced the old approach?",
        "--tag",
        "architecture",
    )
    assert result.returncode == 0

    text = (repo / "MEMHOOKS.md").read_text(encoding="utf-8")
    assert "```json" in text
    assert '"query": "Why did the old approach fail?"' in text
    assert '"query": "What replaced the old approach?"' in text
    assert '"architecture"' in text
