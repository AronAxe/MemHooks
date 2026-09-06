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


def test_init_seeds_typed_routing_fields(tmp_path):
    repo = tmp_path / "repo"
    repo.mkdir()
    subprocess.run(["git", "init"], cwd=repo, capture_output=True, check=True)

    result = run_cli("init", str(repo))
    assert result.returncode == 0

    text = (repo / "MEMHOOKS.md").read_text(encoding="utf-8")
    assert "memory_types: []" in text
    assert "connection_types: []" in text
    assert "entities: []" in text


def test_note_persists_memory_connection_and_entity_types(tmp_path):
    repo = tmp_path / "repo"
    repo.mkdir()
    subprocess.run(["git", "init"], cwd=repo, capture_output=True, check=True)
    assert run_cli("init", str(repo)).returncode == 0

    result = run_cli(
        "note",
        "--cwd",
        str(repo),
        "--query",
        "Why did auth change after the outage?",
        "--memory-type",
        "experience",
        "--connection-type",
        "causal",
        "--connection-type",
        "temporal",
        "--entity",
        "Authentication",
        "--entity",
        '{"name":"OpenAI","type":"ORG"}',
    )
    assert result.returncode == 0

    text = (repo / "MEMHOOKS.md").read_text(encoding="utf-8")
    assert '"memory_types": [' in text
    assert '"experience"' in text
    assert '"connection_types": [' in text
    assert '"causal"' in text
    assert '"temporal"' in text
    assert '"name": "OpenAI"' in text
    assert '"type": "ORG"' in text
    assert '"Authentication"' in text


def test_repeated_note_enriches_instead_of_erasing_metadata(tmp_path):
    repo = tmp_path / "repo"
    repo.mkdir()
    subprocess.run(["git", "init"], cwd=repo, capture_output=True, check=True)
    assert run_cli("init", str(repo)).returncode == 0

    query = "Why did auth change?"
    assert run_cli(
        "note",
        "--cwd",
        str(repo),
        "--query",
        query,
        "--memory-type",
        "experience",
        "--connection-type",
        "causal",
    ).returncode == 0

    assert run_cli(
        "note",
        "--cwd",
        str(repo),
        "--query",
        query,
        "--connection-type",
        "temporal",
        "--entity",
        '{"name":"OpenAI","type":"ORG"}',
    ).returncode == 0

    text = (repo / "MEMHOOKS.md").read_text(encoding="utf-8")
    assert '"experience"' in text
    assert '"causal"' in text
    assert '"temporal"' in text
    assert '"type": "ORG"' in text


def test_legacy_bullet_notes_migrate_to_structured_json(tmp_path):
    repo = tmp_path / "repo"
    repo.mkdir()
    subprocess.run(["git", "init"], cwd=repo, capture_output=True, check=True)
    (repo / "MEMHOOKS.md").write_text(
        """---
schema: memhooks/v1
inherits: true
---

<!-- memhooks:notes:start -->
## In-session retrieval cues

- Why did the old approach fail?

Keep these as questions/cues; durable facts belong in the memory backend.
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
        "--memory-type",
        "world",
    )
    assert result.returncode == 0

    text = (repo / "MEMHOOKS.md").read_text(encoding="utf-8")
    assert "```json" in text
    assert '"query": "Why did the old approach fail?"' in text
    assert '"query": "What replaced the old approach?"' in text
    assert '"world"' in text
