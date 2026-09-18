# Native Hermes + Hindsight recall

This opt-in integration closes one complete loop:

```text
successful post_tool_call(args with explicit file paths)
  -> profile/session-local recent file activity
  -> memhooks event (existing Rust cue maintenance)
next pre_llm_call
  -> memhooks explain (existing Rust validation/inheritance/roles)
  -> authorized Hindsight Recall requests
  -> source-attributed, bounded memories returned as {context: ...}
```

It is a general Hermes plugin, not a replacement memory provider. The core remains
backend-neutral; only `hindsight_recall.py` interprets the concrete HTTP transport.
Nothing calls Retain, Reflect, or a separate LLM for query generation or summary.
The configured Hindsight service still has its own retrieval/model costs.

## Install and explicitly enable

Requires Python 3.11+ for current Hermes (the adapter modules also test on 3.10),
the MemHooks 0.6.0 CLI, and an existing Hindsight bank with useful stored memories.
No Hindsight Python SDK is needed. This plugin is a repository addition, **not a new
crate release**. Installing the Cargo binary alone does not install this plugin.

From a trusted MemHooks checkout:

```bash
DEST="${HERMES_HOME:-$HOME/.hermes}/plugins/memhooks-recall"
mkdir -p "$DEST"
cp hooks/hermes/{plugin.yaml,__init__.py,recall_runtime.py,hindsight_recall.py,memhooks_pre_llm.py} "$DEST/"
hermes plugins enable memhooks-recall
```

In the active profile's `config.yaml`, merge this entry with existing settings
rather than replacing the entire `plugins` mapping:

```yaml
plugins:
  entries:
    memhooks-recall:
      settings:
        runtime:
          enabled: true
          project_root: /absolute/path/to/your/project
          binary: /absolute/path/to/.cargo/bin/memhooks
          roles: [reviewer]
          allowed_sensitivities: [public]
          maintain: true
          max_queries: 3
          max_context_chars: 12000
          timeout_seconds: 5
          hindsight:
            base_url: https://your-hindsight-host.example
            bank: your-project-bank
            api_key_env: HINDSIGHT_API_KEY
            required_tags: [project:your-project]
            budget: low
            max_tokens: 1024
```

Set the named credential using Hermes' **active-profile secret configuration**.
Never put it in `MEMHOOKS.md`, this plugin's YAML, or a repository. The plugin uses
`agent.secret_scope.get_secret` on each callback; it does not cache credentials or
fall back to another profile's process environment. Missing/unscoped credentials
stop retrieval. `base_url` must be the service origin/base prefix, not a recall URL.

For an intentionally unauthenticated local server, use a literal loopback address
such as `http://127.0.0.1:8888` and explicitly set `allow_anonymous: true` under
`hindsight`. Plain HTTP to hostnames (including `localhost`) and non-loopback
addresses is rejected. Remote servers require HTTPS and a scoped credential.

`project_root` binds this plugin instance/profile to **one local project**. Native
Hermes payloads need not expose a working directory, so the plugin never guesses
from a gateway worker's process cwd. Relative file paths are relative to this root,
or an explicit in-root event `cwd`. Use separate profiles/configurations for other
projects. Remote/container tool paths require a host-side local mapping; they are
not guessed. Run `memhooks init /absolute/path/to/your/project` once to opt the
project in. A root hook must exist and validate; `inherits: false` may still remove its cues
from a child scope without disabling that child.

Remove the older **MemHooks-specific** `pre_llm_call`/`post_tool_call` shell entries
when switching to this plugin; preserve unrelated hooks. Existing Hermes memory
providers are not disabled or reconfigured, so review their own automatic prefetch
settings to avoid duplicate retrieval. Memory writes remain that provider's job.

## What runs automatically

The plugin registers native `post_tool_call`, `pre_llm_call`, `on_session_finalize`
and `on_session_reset` callbacks. Only successful tool events (`status: ok`) with
explicit path fields are recorded. Blocked/failed/cancelled calls, shell commands,
raw diffs, file contents and tool-result prose are never mined for filenames.
`args` is the native Hermes tool-input field; the plugin translates validated paths
to the existing Rust maintainer's `tool_input` event format. Explicit deletion paths
can prune generated cues through that same Rust path.

Recent paths stay in memory, keyed by current profile, canonical project root and
session ID. Subagents with separate session IDs stay separate. No prompts, memories
or credentials enter this activity store. It keeps at most 64 paths per session,
128 sessions total, and expires inactive entries after one hour. Finalization/reset
clears the matching session; ordinary `on_session_end` is **not** finalization in
Hermes, so the plugin does not erase useful activity at every user-turn boundary.
Restarting the host starts with root/cwd routing until new tool activity arrives.

Hermes' `pre_llm_call` is a **user-turn** hook, not every internal model API attempt.
Activity therefore affects the next user turn. Hosts can optionally supply explicit
`active_files` for that turn. Host-configured roles alone select role-filtered cues;
repo text and event `active_roles` cannot replace this plugin's configured roles.
No stable session ID means no plugin injection, rather than cross-session fallback.

## Supported Hindsight mapping and authority

The host selects one fixed bank and endpoint. Optional host `required_tags` are
sent as `tags_match: all_strict` and rechecked on returned results. An empty host
list authorizes retrieval across that configured bank. Tags are not a substitute
for server-side authentication/authorization: use credentials restricted to the
intended bank where the deployment supports that policy.

The initial executable adapter supports `backends.hindsight` keys:

| Key | Behavior |
|---|---|
| `bank` | Must equal the host-selected bank; another bank is rejected. |
| `strategy` | Only absent/`recall` is supported. `reflect` is reported, not silently executed or substituted. |
| `memory_types` | Nonempty subset of `world`, `experience`, `observation`; passed as `types` and rechecked on results. |

Other Hindsight namespace keys cause an explicit `unsupported_provider_options`
outcome. Broader conventions documented in the provider reference are not all
implemented by this first adapter. In particular there is no direct Mental Model
or Knowledge Page lookup. Other provider namespaces do not select another backend.

Generic entity/resource names and tags sharpen the natural-language query; they do
not become authorization tags. Query text plus hints is capped at 2000 UTF-8 bytes;
over-limit queries are skipped, not cut into a different question. Hindsight also
enforces its own token limit and may reject a query within this outer byte limit.
The full conversation, user prompt, file bodies and Markdown guidance are not sent
to the memory service. Higher-priority cues run first, with at most three calls by
default and no retries. Duplicate facts are collapsed within the same target and
routing constraints, retaining contributing hook-source paths.

A nonempty scope `sensitivity` label must be explicitly listed in the host's
`allowed_sensitivities`; the default list is empty. Add `private` or another label
only when that host/provider/context is authorized for it. Labels are hints, not
server permissions. Unlabeled hooks are covered by the explicit project/bank grant.
Generic `scope` is preserved as provenance; it does not implicitly select a tenant.
Sibling scopes retain separate exclusions and provenance. `exclude` entries are
matched as case-insensitive literal substrings against the full returned record
before it enters context (not semantic classifiers or glob expressions).

## Bounds, diagnostics and failures

The shared timeout covers resolution and backend calls. Each HTTP request runs in
a short-lived subprocess that is killed/waited on at its remaining deadline,
including slow DNS, response headers or body reads. Credentials use stdin, never
process arguments or disk. Redirects and ambient HTTP proxies are disabled. HTTP
responses are capped at 1 MiB; malformed/oversized data is discarded. Authentication
failures stop further calls, rather than trying another bank or weaker scope.

`max_queries` allows 1–16; `timeout_seconds` allows 0.1–30; `max_context_chars` allows
256–100000. The Hindsight `budget` is `low`, `mid` or `high`, and its `max_tokens`
allows 1–4096. These are host controls, not repository overrides.

The emitted `memhooks/memories-v1` JSON contains `memories`, `guidance`,
`queries_executed`, and counted `issues`. Memories include their ID, unchanged text,
category, available document/time provenance, contributing hook sources, and
bank/target/scope/sensitivity/exclusions. The trust wrapper is included in the
character limit. Whole memories are selected; an oversized fact is omitted rather
than clipped into a misleading claim. Guidance gets remaining space after memories.

Provider failures do not inject fabricated results or silently switch back to a
broader retrieval plan. Available successful memories can still be returned with
failure counts. Logs contain only outcome codes/exception classes—not credentials,
queries, returned facts or raw server errors. Context is untrusted reference data;
labeling it does not make prompt-injection attacks impossible. The plugin itself
never executes retrieved text and does not grant tools any additional permissions.

## Verification

```bash
cargo build --locked
MEMHOOKS_REQUIRE_BINARY=1 python3 -m pytest -q tests/test_native_recall.py
hermes plugins doctor "${HERMES_HOME:-$HOME/.hermes}/plugins/memhooks-recall" --ci
```

The automated suite uses native-shaped hook payloads, a real compiled Rust CLI and
a real loopback HTTP contract server. It tests file activity/maintenance, inheritance,
roles, exclusions, provenance, deduplication, profile/session isolation, cleanup,
query/context bounds, credentials, redirects, malformed responses and hard timeouts.
It does **not** claim a live Hindsight deployment or full installed Hermes process
was exercised by that suite. Run Plugin Doctor and a normal turn against the
operator's configured Hermes/Hindsight environment as the deployment smoke test.
Correct routing/retrieval is not yet a measured improvement in agent task outcomes.

## Upstream contracts checked

- Hermes observer hooks: https://github.com/NousResearch/hermes-agent/blob/c62bd9f2078a946108f1c9d9b24bf118963277ef/website/docs/developer-guide/observer-hooks.md
- Hermes native plugin/config contract: https://github.com/NousResearch/hermes-agent/blob/c62bd9f2078a946108f1c9d9b24bf118963277ef/website/docs/developer-guide/plugins/index.md
- Hindsight Recall: https://hindsight.vectorize.io/developer/api/recall
- HTTP endpoint and response schema: https://hindsight.vectorize.io/api-reference (`POST /v1/default/banks/{bank_id}/memories/recall`)
