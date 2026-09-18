"""Focused release-note and managed-wiki lifecycle regressions; no network required."""
import importlib.util
import json
import subprocess
import sys
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[1]


def load_script(name):
    spec = importlib.util.spec_from_file_location(name, ROOT / "scripts" / f"{name}.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


notes = load_script("release_notes")
wiki = load_script("sync_wiki")


@pytest.mark.parametrize("requested", ["0.6.0", "[0.6.0]", "v0.6.0", "[v0.6.0]"])
@pytest.mark.parametrize("older", ["0.5.1", "[0.5.1]"])
def test_mixed_release_heading_styles(requested, older):
    section = f"## {requested} - 2026-09-18\n\n### Fixed\nOnly this release.\n"
    text = f"# Changelog\n## [Unreleased]\nFuture work.\n\n{section}\n## {older}\nOld work.\n"
    assert notes.extract_release_notes(text, "v0.6.0") == section


def test_exact_prerelease_build_and_last_section_without_newline():
    text = "## [1.0.0-rc.1+build.7]\nRC\n## 1.0.0\nStable\n## 0.9.0\nLast"
    assert notes.extract_release_notes(text, "1.0.0-rc.1+build.7") == "## [1.0.0-rc.1+build.7]\nRC\n"
    assert notes.extract_release_notes(text, "0.9.0") == "## 0.9.0\nLast\n"


@pytest.mark.parametrize("fence", ["```", "~~~~"])
def test_fenced_headings_do_not_end_or_duplicate_release(fence):
    section = f"## 0.6.0\n{fence}md\n## [0.6.0]\n## 0.5.1\n{fence}\nEnd.\n"
    assert notes.extract_release_notes(section + "## 0.5.1\nOld\n", "0.6.0") == section


@pytest.mark.parametrize("boundary", ["## Appendix", "# References", "## [Unreleased]"])
def test_nonrelease_top_level_sections_are_not_included(boundary):
    assert notes.extract_release_notes(f"## [0.6.0]\nNew\n{boundary}\nOther", "0.6.0") == "## [0.6.0]\nNew\n"


@pytest.mark.parametrize("text,version", [
    ("## 0.6.0\nA", "0.6.1"),
    ("## [0.6.0]\nA\n## 0.6.0\nB", "0.6.0"),
    ("## 0.6.0-rc.1\nA", "0.6.0"),
    ("## 0.6.0\nA", "0.6"),
])
def test_missing_ambiguous_and_invalid_versions_fail(text, version):
    with pytest.raises(ValueError):
        notes.extract_release_notes(text, version)


def test_real_changelog_mixed_style_regression():
    text = (ROOT / "CHANGELOG.md").read_text(encoding="utf-8")
    current = notes.extract_release_notes(text, "0.6.0")
    assert current.startswith("## [0.6.0]")
    assert "## 0.5.1" not in current and "## 0.5.0" not in current
    assert "### Migration" in current
    previous = notes.extract_release_notes(text, "0.5.1")
    assert previous.startswith("## 0.5.1") and "## 0.5.0" not in previous


def test_extractor_cli_does_not_clobber_output_on_missing_release(tmp_path):
    changelog = tmp_path / "CHANGELOG.md"
    changelog.write_text("## 0.6.0\nNew\n## 0.5.1\nOld", encoding="utf-8")
    output = tmp_path / "notes.md"
    command = [sys.executable, str(ROOT / "scripts/release_notes.py"),
               "0.6.0", "--changelog", str(changelog), "--output", str(output)]
    assert subprocess.run(command, check=False).returncode == 0
    before = output.read_bytes()
    command[2] = "9.9.9"
    result = subprocess.run(command, capture_output=True, text=True, check=False)
    assert result.returncode == 1 and "found 0" in result.stderr
    assert output.read_bytes() == before


@pytest.fixture
def directories(tmp_path):
    source, destination = tmp_path / "source", tmp_path / "destination"
    source.mkdir()
    destination.mkdir()
    return source, destination


def put(root, name, content="page"):
    path = root / name
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")
    return path


def snapshot(root):
    return {str(p.relative_to(root)): p.read_bytes() for p in root.rglob("*") if p.is_file()}


def test_add_update_delete_rename_and_preserve_manual_pages(directories):
    source, destination = directories
    put(source, "Old.md", "old")
    put(source, "Home.md", "home")
    put(source, "images/logo.svg", "logo")
    put(destination, "Manual.md", "untouched")
    put(destination, ".git/config", "untouched git")
    wiki.sync_pages(source, destination)
    (source / "Old.md").rename(source / "Renamed.md")
    (source / "images/logo.svg").unlink()
    put(source, "Home.md", "new home")
    result = wiki.sync_pages(source, destination)
    assert result == {"written": ["Home.md", "Renamed.md"], "removed": ["Old.md", "images/logo.svg"]}
    assert not (destination / "Old.md").exists()
    assert (destination / "Manual.md").read_text() == "untouched"
    assert (destination / ".git/config").read_text() == "untouched git"
    manifest = json.loads((destination / wiki.MANIFEST).read_text())
    assert set(manifest["files"]) == {"Home.md", "Renamed.md"}
    before = snapshot(destination)
    assert wiki.sync_pages(source, destination) == {"written": [], "removed": []}
    assert snapshot(destination) == before


def test_empty_source_removes_only_managed_pages(directories):
    source, destination = directories
    managed = put(source, "Managed.md")
    put(destination, "Manual.md")
    wiki.sync_pages(source, destination)
    managed.unlink()
    assert wiki.sync_pages(source, destination)["removed"] == ["Managed.md"]
    assert (destination / "Manual.md").exists()


def test_missing_remote_managed_page_is_harmless(directories):
    source, destination = directories
    put(source, "Gone.md")
    wiki.sync_pages(source, destination)
    (source / "Gone.md").unlink()
    (destination / "Gone.md").unlink()
    assert wiki.sync_pages(source, destination)["removed"] == []


@pytest.mark.parametrize("remove", [False, True])
def test_remote_manual_edit_blocks_entire_update_or_deletion(directories, remove):
    source, destination = directories
    put(source, "Edited.md", "managed")
    wiki.sync_pages(source, destination)
    put(destination, "Edited.md", "manual changes")
    put(source, "A-new.md", "must not get written before conflict")
    if remove:
        (source / "Edited.md").unlink()
    else:
        put(source, "Edited.md", "new managed")
    before = snapshot(destination)
    with pytest.raises(ValueError, match="conflict"):
        wiki.sync_pages(source, destination)
    assert snapshot(destination) == before


def test_unmanaged_collision_refused_but_identical_page_can_be_adopted(directories):
    source, destination = directories
    put(source, "Shared.md", "source")
    put(destination, "Shared.md", "manual")
    with pytest.raises(ValueError, match="conflict"):
        wiki.sync_pages(source, destination)
    assert not (destination / wiki.MANIFEST).exists()
    put(destination, "Shared.md", "source")
    assert wiki.sync_pages(source, destination)["written"] == []
    assert "Shared.md" in json.loads((destination / wiki.MANIFEST).read_text())["files"]


def test_preview_does_not_write_delete_or_create_manifest(directories):
    source, destination = directories
    put(source, "New.md")
    put(destination, "Old.md", "old")
    before = snapshot(destination)
    result = wiki.sync_pages(source, destination,
                            previous={"Old.md": wiki.digest(b"old")}, dry_run=True)
    assert result == {"written": ["New.md"], "removed": ["Old.md"]}
    assert snapshot(destination) == before


@pytest.mark.parametrize("name", ["../escape.md", "/outside.md", ".git/config", ".GIT/config",
                                  "a/../../escape", "a\\escape.md", "a//b.md", wiki.MANIFEST])
def test_unsafe_manifest_paths_fail_before_changes(directories, name):
    source, destination = directories
    put(source, "A.md")
    put(destination, wiki.MANIFEST, json.dumps({"version": 1, "files": {name: wiki.digest(b"x")}}))
    before = snapshot(destination)
    with pytest.raises(ValueError):
        wiki.sync_pages(source, destination)
    assert snapshot(destination) == before


@pytest.mark.parametrize("data", [[], {"version": 2, "files": {}}, {"version": 1},
                                  {"version": 1, "files": {"A.md": "bad"}}])
def test_invalid_manifest_is_not_treated_as_unowned(directories, data):
    source, destination = directories
    put(destination, wiki.MANIFEST, json.dumps(data))
    before = snapshot(destination)
    with pytest.raises((TypeError, ValueError)):
        wiki.sync_pages(source, destination)
    assert snapshot(destination) == before


@pytest.mark.parametrize("location", ["source-file", "destination-file", "parent", "manifest"])
def test_symlinks_cannot_redirect_writes_or_deletes(directories, location, tmp_path):
    source, destination = directories
    outside = tmp_path / "outside"
    outside.mkdir()
    sentinel = put(outside, "Page.md", "untouched")
    if location == "source-file":
        (source / "Page.md").symlink_to(sentinel)
    elif location == "destination-file":
        put(source, "Page.md", "new")
        (destination / "Page.md").symlink_to(sentinel)
    elif location == "parent":
        put(source, "linked/Page.md", "new")
        (destination / "linked").symlink_to(outside, target_is_directory=True)
    else:
        (destination / wiki.MANIFEST).symlink_to(sentinel)
    with pytest.raises(ValueError, match="symlink"):
        wiki.sync_pages(source, destination)
    assert sentinel.read_text() == "untouched"


def commit(repo, message):
    wiki.git(repo, "add", "-A")
    wiki.git(repo, "-c", "user.name=Test", "-c", "user.email=test@example.invalid",
             "commit", "--allow-empty", "-qm", message)
    return wiki.git(repo, "rev-parse", "HEAD").decode().strip()


def test_legacy_bootstrap_tracks_older_deleted_pages_without_claiming_manual(tmp_path):
    repo, remote = tmp_path / "repo", tmp_path / "remote"
    for directory in (repo, remote):
        directory.mkdir()
        wiki.git(directory, "init", "-q")
    source = repo / "wiki"
    put(source, "Old.md", "old content")
    put(source, "Home.md", "initial")
    first = commit(repo, "initial wiki")
    put(remote, "Old.md", "old content")
    put(remote, "Home.md", "initial")
    commit(remote, f"docs: sync MemHooks wiki from {first}")
    (source / "Old.md").unlink()
    put(source, "Home.md", "updated")
    second = commit(repo, "delete old page")
    put(remote, "Home.md", "updated")
    commit(remote, f"docs: sync MemHooks wiki from {second}")
    put(remote, "Manual.md", "manual")
    commit(remote, "manual page")
    ownership = wiki.legacy_manifest(repo, remote)
    assert ownership == {"Old.md": wiki.digest(b"old content"), "Home.md": wiki.digest(b"updated")}
    command = [sys.executable, str(ROOT / "scripts/sync_wiki.py"), str(remote), "--repo", str(repo)]
    result = subprocess.run(command, capture_output=True, text=True, check=False)
    assert result.returncode == 0, result.stderr
    assert json.loads(result.stdout)["removed"] == ["Old.md"]
    assert (remote / "Manual.md").read_text() == "manual"
    assert (remote / wiki.MANIFEST).exists()


def test_bootstrap_does_not_guess_ownership_without_recorded_sync(tmp_path):
    repo, remote = tmp_path / "repo", tmp_path / "remote"
    for directory in (repo, remote):
        directory.mkdir()
        wiki.git(directory, "init", "-q")
    put(repo, "wiki/Home.md", "home")
    commit(repo, "source")
    put(remote, "Possibly-old.md", "unknown origin")
    commit(remote, "manual initialization")
    previous = wiki.legacy_manifest(repo, remote)
    assert previous == {}
    wiki.sync_pages(repo / "wiki", remote, previous=previous)
    assert (remote / "Possibly-old.md").read_text() == "unknown origin"


def test_missing_legacy_source_commit_fails_without_writing(tmp_path):
    repo, remote = tmp_path / "repo", tmp_path / "remote"
    for directory in (repo, remote):
        directory.mkdir()
        wiki.git(directory, "init", "-q")
    put(repo, "wiki/Home.md")
    commit(repo, "source")
    put(remote, "Manual.md")
    commit(remote, "docs: sync MemHooks wiki from " + "1" * 40)
    before = snapshot(remote)
    result = subprocess.run([sys.executable, str(ROOT / "scripts/sync_wiki.py"), str(remote),
                             "--repo", str(repo)], capture_output=True, text=True, check=False)
    assert result.returncode == 1
    assert snapshot(remote) == before
