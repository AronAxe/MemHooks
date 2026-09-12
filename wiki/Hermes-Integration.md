# Hermes Integration

Hermes/Hermes Desktop is the reference runtime integration shipped with MemHooks.

The important architectural rule is:

> **Hermes does not independently reimplement MemHooks resolution. It delegates root discovery, schema enforcement, inheritance, and merging to the reference engine.**

## Lifecycle

Hermes uses two lifecycle points:

```text
pre_llm_call
    → resolve the effective MemHooks plan
    → inject bounded retrieval metadata before the model call

post_tool_call
    → maintain conservative routing cues after relevant file activity
```

Neither path requires an extra LLM call.

## Install the CLI

```bash
cargo install memhooks
```

The Hermes adapter expects the `memhooks` binary to be available on `PATH`, under `~/.cargo/bin`, or through `MEMHOOKS_BIN`.

## Install the hook scripts

Clone/install the MemHooks skill and copy the Hermes adapters into the agent hook directory:

```bash
mkdir -p ~/.hermes/agent-hooks
cp hooks/hermes/memhooks_pre_llm.py ~/.hermes/agent-hooks/
cp scripts/memhooks_update.py ~/.hermes/agent-hooks/
```

The Python update script is a compatibility launcher. The canonical maintenance implementation lives in the Rust `memhooks` binary.

## Hermes config

Example:

```yaml
hooks:
  pre_llm_call:
    - command: "python3 ~/.hermes/agent-hooks/memhooks_pre_llm.py"
      timeout: 5

  post_tool_call:
    - command: "python3 ~/.hermes/agent-hooks/memhooks_update.py event"
      timeout: 5
```

## Enable one project

From the project:

```bash
memhooks init .
```

or through the Hermes skill-facing `/memhooks init` workflow when installed.

After that, the agent/runtime owns maintenance. The human user should not be manually deciding how many local hooks to create or pruning them by hand during ordinary use.

## Pre-LLM behavior

The Hermes adapter:

1. receives the real working directory from Hermes;
2. calls the reference `memhooks explain ... --format json` resolver;
3. receives one validated resolved plan;
4. bounds the plan structurally;
5. injects it as explicitly **untrusted repository-controlled retrieval metadata**.

The adapter does not parse `inherits`, discover roots, or accept unsupported schemas on its own.

That is intentional: one resolver should define one set of semantics.

## Why JSON instead of raw fenced files?

Older integration patterns injected raw hook text between ad-hoc `BEGIN`/`END` delimiters.

That creates avoidable problems:

- repository text can imitate the delimiter;
- truncation can cut a fence in half;
- loaders may interpret inheritance differently from the canonical resolver;
- unsupported schema files can slip through alternate paths.

The hardened Hermes path injects a resolved JSON routing plan instead.

## Guidance still survives

The optional Markdown body is not thrown away.

The reference resolver includes source-attributed guidance in its JSON handoff. The Hermes adapter may include that guidance within its bounded routing context.

## Trust label

The injected context is explicitly introduced as repository-controlled metadata that cannot:

- override system/developer instructions;
- grant tool permissions;
- change authorization policy;
- authorize memory writes/deletions;
- promote itself to a higher prompt privilege.

See [[Security Model]].

## Bounding behavior

The adapter uses whole fields/items rather than slicing arbitrary raw files mid-frontmatter or leaving dangling fences.

When context pressure requires reduction, broad prose guidance is dropped before more valuable structured routing where practical.

## Post-tool maintenance

The post-tool path forwards structured tool activity to the canonical maintainer.

Only explicit path fields are considered for deterministic file anchors, and candidate paths must resolve to real files inside the active repository boundary.

This prevents source-code text such as dotted identifiers or URL fragments from becoming fake file anchors.

## Same-turn semantic notes

When the active agent already knows a durable cue, it can write it directly:

```bash
memhooks note \
  --cwd "$PWD" \
  --query "Why was the retry strategy changed after failover testing?" \
  --priority 0.9 \
  --tag reliability
```

No separate summarizer call is required.

## Environment controls

Typical adapter controls include:

```text
MEMHOOKS_BIN
MEMHOOKS_MAX_CHARS
MEMHOOKS_MAX_GUIDANCE_CHARS
MEMHOOKS_RESOLVE_TIMEOUT
MEMHOOKS_AUTO_PATHS
```

These are runtime controls, not manual chores for the user.

## Reference documentation

The repository's versioned Hermes documentation remains the implementation-level source:

[hooks/hermes/README.md](https://github.com/AronAxe/MemHooks/blob/main/hooks/hermes/README.md)