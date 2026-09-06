# Hermes runtime hooks

Hermes exposes real lifecycle hooks, so MemHooks does not have to depend on the model remembering that `MEMHOOKS.md` exists.

MemHooks uses two of them:

```text
pre_llm_call   -> load routing before the model sees the turn
post_tool_call -> maintain routing after files are touched
```

Both paths are ordinary Python scripts. **Neither makes an LLM call.**

## 1. Pre-LLM loader

[`memhooks_pre_llm.py`](memhooks_pre_llm.py):

1. receives Hermes' shell-hook JSON payload on stdin;
2. reads Hermes' real `cwd`;
3. finds the workspace root;
4. walks root -> active directory;
5. reads every applicable `MEMHOOKS.md`;
6. honors `inherits: false`;
7. returns `{"context": "..."}`;
8. Hermes injects that context before the model call.

This guarantees the model sees the retrieval cues. It does not itself call Hindsight/OpenViking/Honcho because that would couple the loader to one memory API.

## 2. Zero-LLM maintainer

[`../../scripts/memhooks_update.py`](../../scripts/memhooks_update.py) handles the other half of the problem: **who keeps the routing file useful?**

On Hermes `post_tool_call`, it reads the tool input, extracts project file paths, and maintains a bounded auto-managed block in the corresponding directory's `MEMHOOKS.md`.

Example generated cue:

```text
Before substantive work here, recall prior decisions, constraints,
failures, fixes, rejected approaches, and unresolved issues involving:
- `backend/auth/refresh.py`
```

That is deliberately a *cue*, not a stored memory. The actual facts remain in the configured memory backend.

The script only auto-writes inside a tree that has already opted in with an ancestor `MEMHOOKS.md`.

### Why auto-anchors stay untyped

A deterministic path hook can know **which files changed**, but it cannot reliably determine whether the relevant Hindsight memory is a `world` fact, `experience`, or `observation`; whether the useful connection emphasis is semantic, temporal, entity, or causal; or what an entity's type should be.

MemHooks deliberately does **not guess** those classifications. Automatic file-path anchors remain untyped.

### Same-turn semantic notes

When the current agent has already discovered the semantic reason something matters during its normal turn, it can record one future retrieval question with the same deterministic script.

Simple note:

```bash
python3 ~/.hermes/agent-hooks/memhooks_update.py note \
  --cwd "$PWD" \
  --query "Why was refresh-token rotation split into two stages, and what alternatives were rejected?"
```

Structured note when the classifications are genuinely known:

```bash
python3 ~/.hermes/agent-hooks/memhooks_update.py note \
  --cwd "$PWD" \
  --query "Why did the authentication design change after the outage?" \
  --memory-type experience \
  --connection-type causal \
  --connection-type temporal \
  --entity 'Authentication' \
  --entity '{"name":"OpenAI","type":"ORG"}'
```

Supported Hindsight-compatible memory categories are:

```text
world | experience | observation
```

Supported MemHooks connection-emphasis values are:

```text
semantic | temporal | entity | causal
```

`--entity` accepts either a plain entity name or a JSON object containing `name` (or Hindsight-style `text`) and an optional explicit `type`.

The note block is stored as a bounded JSON array inside `MEMHOOKS.md`, so this routing metadata survives maintenance. Existing older markdown-bullet notes are still understood and are migrated when the block is next written.

This uses the model call that is already happening. It does **not** start a second summarizer/model pass.

## Install

```bash
mkdir -p ~/.hermes/agent-hooks
cp hooks/hermes/memhooks_pre_llm.py ~/.hermes/agent-hooks/
cp scripts/memhooks_update.py ~/.hermes/agent-hooks/
chmod +x ~/.hermes/agent-hooks/memhooks_pre_llm.py ~/.hermes/agent-hooks/memhooks_update.py
```

Then add:

```yaml
hooks:
  pre_llm_call:
    - command: "python3 ~/.hermes/agent-hooks/memhooks_pre_llm.py"
      timeout: 5
  post_tool_call:
    - command: "python3 ~/.hermes/agent-hooks/memhooks_update.py event"
      timeout: 5
```

Hermes asks for consent the first time it sees each new shell hook.

## Enable a project

MemHooks does not spray files into every repository merely because it is globally installed. Opt in once:

```bash
python3 ~/.hermes/agent-hooks/memhooks_update.py init /path/to/project
```

That creates the root `MEMHOOKS.md`. From then on, `post_tool_call` can create/update more specific hook files in directories that are actually touched.

## What is guaranteed

With both hooks installed and the project initialized:

- applicable routing files are loaded before the LLM call;
- touched file paths produce/refresh local recall anchors after tool calls;
- semantic notes can preserve known memory category, connection emphasis, and typed entities;
- deterministic maintenance never guesses those semantic classifications;
- no extra LLM call is spent on either operation;
- no memory is created/rewritten merely because MemHooks ran.

## Bounds

Loader defaults:

- `MEMHOOKS_MAX_CHARS=24000`
- `MEMHOOKS_MAX_FILE_CHARS=12000`
- `MEMHOOKS_FILENAME=MEMHOOKS.md`
- `MEMHOOKS_ROOT=/explicit/workspace/root` optionally overrides root detection

Maintainer defaults:

- `MEMHOOKS_AUTO_PATHS=12` auto file anchors per local hook
- `MEMHOOKS_MAX_NOTES=16` semantic note cues per local hook

## Test manually

Loader:

```bash
printf '%s' '{"hook_event_name":"pre_llm_call","cwd":"'"$(pwd)"'","extra":{}}' \
  | python3 ~/.hermes/agent-hooks/memhooks_pre_llm.py
```

Maintainer:

```bash
printf '%s' '{"hook_event_name":"post_tool_call","cwd":"'"$(pwd)"'","tool_name":"write_file","tool_input":{"path":"src/example.py"}}' \
  | python3 ~/.hermes/agent-hooks/memhooks_update.py event
```
