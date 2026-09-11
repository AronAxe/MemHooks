# Hermes runtime hooks

Hermes exposes lifecycle hooks, so MemHooks does not have to depend on the model remembering that `MEMHOOKS.md` exists.

MemHooks uses two:

```text
pre_llm_call   -> load retrieval routing before the model sees the turn
post_tool_call -> maintain routing after relevant file activity
```

Both paths are ordinary Python scripts. **Neither makes an extra LLM call.**

## User experience

The user should normally do only two things:

1. install/configure the Hermes hooks;
2. enable MemHooks once for a project (`/memhooks init`).

After that, the **agent/runtime** maintains local routing cues. The user is not expected to manually decide how many hook files exist or how they are worded.

## 1. Pre-LLM loader

[`memhooks_pre_llm.py`](memhooks_pre_llm.py):

1. receives Hermes' shell-hook JSON payload on stdin;
2. reads Hermes' real `cwd`;
3. finds the workspace root;
4. walks root → active directory;
5. reads every applicable `MEMHOOKS.md`;
6. honors `inherits: false`;
7. returns `{"context": "..."}`;
8. Hermes injects that routing context before the model call.

The loader deliberately does not call Hindsight, Mem0, OpenViking, Honcho, or another memory backend. Backend selection/retrieval remains a model/runtime adapter concern.

## 2. Zero-LLM maintainer

[`../../scripts/memhooks_update.py`](../../scripts/memhooks_update.py) handles automatic routing maintenance.

On Hermes `post_tool_call`, it inspects tool input, extracts project file paths, and maintains a bounded auto-managed block in the corresponding local `MEMHOOKS.md`.

Example generated cue:

```text
Before substantive work here, recall prior decisions, constraints,
failures, fixes, rejected approaches, and unresolved issues involving:
- `backend/auth/refresh.py`
```

That is a *retrieval cue*, not a memory. Actual facts remain in the configured memory backend.

The script writes only inside a tree already opted in with an ancestor `MEMHOOKS.md`.

### Why deterministic anchors stay semantically conservative

A path hook can know **which files changed**. It cannot reliably know:

- how important a future recall query will be;
- which agent role it applies to;
- which named entities/resources matter;
- which provider will be active later;
- what provider-native search controls are appropriate.

Therefore deterministic anchors do not guess those details.

### Same-turn semantic notes

When the current agent has already discovered a durable retrieval cue during normal reasoning, it can preserve that cue with the same deterministic helper—without a second model call.

Simple:

```bash
python3 ~/.hermes/agent-hooks/memhooks_update.py note \
  --cwd "$PWD" \
  --query "Why was refresh-token rotation split into two stages, and what alternatives were rejected?"
```

Structured generic cue:

```bash
python3 ~/.hermes/agent-hooks/memhooks_update.py note \
  --cwd "$PWD" \
  --query "Why did authentication change after the outage?" \
  --priority 1.0 \
  --role reviewer \
  --entity '{"name":"Authentication","type":"CONCEPT","salience":0.95}' \
  --resource '{"name":"auth-postmortem","kind":"postmortem","salience":0.9}' \
  --tag security
```

Provider-native hints, only when genuinely known:

```bash
python3 ~/.hermes/agent-hooks/memhooks_update.py note \
  --cwd "$PWD" \
  --query "Why did authentication change after the outage?" \
  --backends '{"mem0":{"top_k":8,"rerank":true},"hindsight":{"memory_types":["experience"]}}'
```

`--backends` must be a JSON object whose provider values are objects. The maintainer **preserves and deep-merges** those mappings; it does not interpret provider-specific keys.

Additional supported generic options:

- repeatable `--role`
- repeatable `--entity`
- repeatable `--resource`
- repeatable `--tag`
- one optional `--priority 0.0..1.0`
- one optional `--backends '{...}'`

The semantic-note block is stored as bounded JSON inside `MEMHOOKS.md`. The model call already in progress supplies the semantic insight; no extra summarizer/model pass is started.

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

Hermes may ask for consent the first time it sees a new shell hook.

## Enable a project

Preferred user-facing command:

```text
/memhooks init
```

Low-level equivalent:

```bash
python3 ~/.hermes/agent-hooks/memhooks_update.py init /path/to/project
```

This creates the root `MEMHOOKS.md` using `schema: memhooks/v2`. From then on, post-tool maintenance can create/update more specific local routing files as the agent works.

## Memory-provider mapping

The loaded `memhooks/v2` core stays backend-neutral. Provider-specific configuration belongs only under:

```yaml
backends:
  hindsight: {...}
  mem0: {...}
  openviking: {...}
  honcho: {...}
```

The active model/runtime should read the matching provider reference:

- `references/memory-systems/01-hindsight.md`
- `references/memory-systems/02-openviking.md`
- `references/memory-systems/03-honcho.md`
- `references/memory-systems/04-mem0.md`
- `references/memory-systems/99-generic-or-unknown.md`

Namespace presence does not select a backend; Hermes/runtime configuration does.

## What is guaranteed

With both hooks installed and the project initialized:

- applicable routing files are loaded before the LLM call;
- touched paths create/refresh local recall anchors after tool calls;
- semantic notes can preserve known priority/roles/entities/resources/tags/provider hints;
- deterministic maintenance does not guess provider semantics;
- no extra LLM call is spent on loading or deterministic maintenance;
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

Those bounds are runtime-maintenance controls. They are not tasks for the user to manually enforce.

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
