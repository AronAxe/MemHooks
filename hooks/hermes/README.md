# Hermes runtime integration

Hermes exposes lifecycle hooks, so MemHooks does not have to depend on the model remembering that `MEMHOOKS.md` exists.

The v0.5.1 integration has one important rule:

> **Hermes does not implement a second MemHooks parser or maintainer. It delegates both reading and writing semantics to the Rust reference engine.**

```text
pre_llm_call   -> Python adapter -> memhooks explain --format json
post_tool_call -> memhooks event
```

Neither path makes an extra LLM call.

## User experience

The user normally does only this:

1. install the `memhooks` binary;
2. install/configure the Hermes adapter;
3. enable MemHooks once for a project.

After that, the agent/runtime maintains local routing cues. The user is not expected to decide how many hook files exist, prune them manually, or hand-tune their wording.

## Install

Install the canonical reference binary:

```bash
cargo install memhooks
```

Install the Agent Skill and pre-LLM adapter:

```bash
git clone https://github.com/AronAxe/MemHooks.git ~/.hermes/skills/memhooks
mkdir -p ~/.hermes/agent-hooks
cp ~/.hermes/skills/memhooks/hooks/hermes/memhooks_pre_llm.py ~/.hermes/agent-hooks/
```

Then configure Hermes:

```yaml
hooks:
  pre_llm_call:
    - command: "python3 ~/.hermes/agent-hooks/memhooks_pre_llm.py"
      timeout: 5
  post_tool_call:
    - command: "memhooks event"
      timeout: 5
```

If the shell used by Hermes cannot find Cargo-installed binaries, set `MEMHOOKS_BIN` to the exact installed `memhooks` path for the pre-LLM adapter and use that exact path in the `post_tool_call` command.

`scripts/memhooks_update.py` remains available only as a compatibility launcher for older configurations. It `exec`s the same `memhooks` binary and contains no independent parsing/root/persistence logic.

## Enable a project

Preferred user-facing Agent Skill command:

```text
/memhooks init
```

Low-level equivalent:

```bash
memhooks init /path/to/project
```

This creates the root `MEMHOOKS.md` using `schema: memhooks/v2` at the canonical project root.

## Pre-LLM retrieval routing

[`memhooks_pre_llm.py`](memhooks_pre_llm.py) receives Hermes' `pre_llm_call` JSON payload and:

1. reads Hermes' real working directory;
2. calls `memhooks explain <cwd> --format json`;
3. accepts routing only when the reference resolver succeeds;
4. constructs one bounded JSON routing payload;
5. returns it as Hermes context explicitly labelled **untrusted repository-controlled retrieval metadata**.

This means the Rust reference resolver owns:

- canonical root discovery;
- `memhooks/v2` schema enforcement;
- root → leaf discovery;
- `inherits: false`;
- same-text local query override;
- role-preserving resolution;
- generic/provider namespace merging;
- source provenance;
- Markdown body guidance.

The adapter does **not** regex YAML fields such as `inherits`, so a provider-native nested key cannot alter core inheritance.

### Trust boundary

The adapter no longer injects raw files between repository-selectable `BEGIN`/`END` delimiters. Resolved routing is JSON data prefaced with an explicit trust statement. Repository text cannot escape a delimiter and masquerade as adapter/system instructions.

Size limiting removes/truncates complete JSON fields/items while always preserving a syntactically closed payload. It does not cut a hook or YAML frontmatter mid-file.

The loader does not contact Hindsight, Mem0, OpenViking, Honcho, or another memory backend. Backend selection/retrieval remains a runtime/model-adapter concern.

## Post-tool maintenance

`memhooks event` reads one `post_tool_call` JSON payload from stdin.

Automatic maintenance is deliberately conservative:

- only values under explicit path-bearing tool-input keys are considered;
- arbitrary file contents/free text are not regex-mined for path-looking strings;
- a candidate must resolve to an existing file;
- the file must be contained by the canonical project root;
- deterministic anchors are written directly to YAML frontmatter as a structured recall query with generic `resources` of `kind: file`;
- no priority, roles, semantic entities, or provider settings are guessed from a touched path.

The maintainer uses an exclusive sibling lock and atomic same-directory replacement so concurrent tool calls do not silently lose each other's updates and an interrupted write does not truncate the hook.

## Same-turn semantic notes

When the current agent has already learned a durable retrieval cue during normal reasoning, it can preserve it without another model call:

```bash
memhooks note \
  --cwd "$PWD" \
  --query "Why did authentication change after the outage?" \
  --priority 1.0 \
  --role reviewer \
  --entity '{"name":"Authentication","type":"CONCEPT","salience":0.95}' \
  --resource '{"name":"auth-postmortem","kind":"postmortem","salience":0.9}' \
  --tag security \
  --backends '{"mem0":{"top_k":8,"rerank":true},"hindsight":{"memory_types":["experience"]}}'
```

The structured cue is written into the **same YAML frontmatter read by `memhooks resolve/explain`**. There is no body JSON note store.

Repeated notes with the same query text enrich the existing structured query while preserving unknown/future fields and opaque provider data.

## Canonical root behavior

Hermes loading and maintenance use the same root rules as the reference library:

1. a configured containing `MEMHOOKS_ROOT` wins;
2. otherwise the nearest Git root is a hard boundary;
3. only outside Git may the highest ancestor with `MEMHOOKS.md` become root.

A stray `~/MEMHOOKS.md` therefore cannot capture writes from a Git repository that has not opted in at its own root.

## Provider mapping

The loaded `memhooks/v2` core stays backend-neutral. Provider-specific configuration belongs only under:

```yaml
backends:
  hindsight: {...}
  mem0: {...}
  openviking: {...}
  honcho: {...}
```

Provider reference guides:

- `references/memory-systems/01-hindsight.md`
- `references/memory-systems/02-openviking.md`
- `references/memory-systems/03-honcho.md`
- `references/memory-systems/04-mem0.md`
- `references/memory-systems/99-generic-or-unknown.md`

Namespace presence does not select a backend; Hermes/runtime configuration and authorization do.

## What is guaranteed

With the integration installed and the project initialized:

- only resolver-validated v2 routing is injected before the LLM call;
- body guidance remains available with source provenance;
- automatic file anchors are written to normative frontmatter and are visible to the resolver;
- semantic notes are written to normative frontmatter and are visible to the resolver;
- deterministic maintenance does not parse arbitrary file contents for anchors;
- root semantics match the resolver;
- writes are locked and atomic;
- no extra LLM call is spent on loading or deterministic maintenance;
- no memory is created/rewritten merely because MemHooks ran.

## Bounds

Loader defaults:

- `MEMHOOKS_MAX_CHARS=24000`
- `MEMHOOKS_MAX_GUIDANCE_CHARS=6000`
- `MEMHOOKS_RESOLVE_TIMEOUT=5`
- `MEMHOOKS_BIN=/exact/path/to/memhooks` optionally selects the reference binary
- `MEMHOOKS_ROOT=/explicit/workspace/root` optionally overrides root detection when it contains the target

Maintainer default:

- `MEMHOOKS_AUTO_PATHS=12` automatic file resources per local auto-query

These are runtime controls, not manual user chores.

## Test manually

Loader:

```bash
printf '%s' '{"hook_event_name":"pre_llm_call","cwd":"'"$(pwd)"'"}' \
  | python3 ~/.hermes/agent-hooks/memhooks_pre_llm.py
```

Maintainer:

```bash
printf '%s' '{"hook_event_name":"post_tool_call","cwd":"'"$(pwd)"'","tool_name":"write_file","tool_input":{"path":"src/example.py"}}' \
  | memhooks event
```

Inspect exactly what the adapter/runtime will receive:

```bash
memhooks explain "$PWD" --format json
```