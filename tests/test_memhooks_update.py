import os
import subprocess
import sys
from pathlib import Path

SCRIPT = Path(__file__).resolve().parents[1] / "scripts" / "memhooks_update.py"


def make_fake_memhooks(tmp_path):
    binary = tmp_path / "memhooks"
    binary.write_text(
        "#!/usr/bin/env python3\n"
        "import sys\n"
        "data = sys.stdin.read()\n"
        "print('ARGS=' + '|'.join(sys.argv[1:]))\n"
        "print('STDIN=' + data)\n",
        encoding="utf-8",
    )
    binary.chmod(0o755)
    return binary


def run_launcher(*args, env=None, input_text=""):
    return subprocess.run(
        [sys.executable, str(SCRIPT), *args],
        input=input_text,
        capture_output=True,
        text=True,
        check=False,
        env=env,
    )


def test_launcher_forwards_note_arguments_to_reference_binary(tmp_path):
    binary = make_fake_memhooks(tmp_path)
    env = os.environ.copy()
    env["MEMHOOKS_BIN"] = str(binary)

    result = run_launcher(
        "note",
        "--cwd",
        "/tmp/project",
        "--query",
        "Why did auth change?",
        "--priority",
        "0.9",
        env=env,
    )

    assert result.returncode == 0
    assert "ARGS=note|--cwd|/tmp/project|--query|Why did auth change?|--priority|0.9" in result.stdout


def test_launcher_defaults_to_event_and_preserves_stdin(tmp_path):
    binary = make_fake_memhooks(tmp_path)
    env = os.environ.copy()
    env["MEMHOOKS_BIN"] = str(binary)
    payload = '{"hook_event_name":"post_tool_call"}'

    result = run_launcher(env=env, input_text=payload)

    assert result.returncode == 0
    assert "ARGS=event" in result.stdout
    assert f"STDIN={payload}" in result.stdout


def test_launcher_fails_clearly_when_binary_is_unavailable(tmp_path):
    env = os.environ.copy()
    env.pop("MEMHOOKS_BIN", None)
    env["PATH"] = str(tmp_path)

    result = run_launcher("init", ".", env=env)

    assert result.returncode == 127
    assert "cargo install memhooks" in result.stderr
