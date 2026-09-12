# Changelog

## 0.5.1 — 2026-09-12

- Unified reading and writing around one normative `memhooks/v2` data path: the Rust reference maintainer now writes semantic notes and automatic file anchors directly into YAML frontmatter consumed by the resolver.
- Added `memhooks init`, `memhooks note`, and `memhooks event` to the reference CLI/library; the Python maintainer is now only a compatibility launcher.
- Added exclusive hook-file locking and atomic same-directory replacement for deterministic maintenance.
- Restricted automatic path discovery to explicit path-bearing tool-input fields and existing files contained by the canonical project root; arbitrary file contents are no longer regex-mined for path-looking strings.
- Unified canonical root semantics across resolver, maintainer, and bundled runtime adapters: explicit containing `MEMHOOKS_ROOT`, otherwise nearest Git root as a hard boundary, with hooked-ancestor fallback only outside Git.
- Made same-trimmed-text queries local overrides across inheritance so adapters do not issue the same retrieval twice with contradictory metadata.
- Included source-attributed Markdown `guidance` in human and JSON `memhooks explain` output; JSON now serializes the resolved structure instead of manually mirroring selected fields.
- Reworked the Hermes pre-LLM adapter to delegate schema/root/inheritance resolution to the Rust engine and inject one structurally bounded JSON plan explicitly labelled as untrusted repository-controlled data.
- Removed raw BEGIN/END repository-text fences and mid-file truncation from the bundled Hermes adapter.
- Replaced deprecated `serde_yaml` with maintained `serde_yaml_ng`; added Saphyr marked YAML parsing for precise diagnostic source locations.
- Made structured query/entity/resource parsing tolerant enough to emit targeted diagnostics for malformed entries such as `quer:` instead of aborting the entire file with an internal enum-deserialization error.
- Added `MH024` for nonexistent target paths plus `MH025`–`MH027` for malformed query/entity/resource entries.
- Hardened SARIF with validation-root-relative artifact URIs and driver rule/help metadata.
- Fixed doctest examples and added an explicit `cargo test --doc` CI gate.
- Added a committed `Cargo.lock`, locked Rust CI, a real Rust 1.78 MSRV job, Python 3.10/3.12 syntax/tests, Ruff, pinned GitHub Action SHAs, read-only workflow permissions, and CI concurrency control.
- Added `references/**`, `SECURITY.md`, `CONTRIBUTING.md`, and `Cargo.lock` to the crate package; fixed GitHub funding metadata.
- Added `.gitignore` for Rust/Python build artifacts and MemHooks lock files.
- Added a root `MEMHOOKS.md` so the repository dogfoods its own protocol and trust/release invariants.
- Updated README, Agent Skill, normative format spec, CLI/Rust/Hermes/troubleshooting documentation, templates, tests, and release guidance to match the hardened implementation.
- Protocol schema remains `memhooks/v2`.

## 0.5.0 — 2026-09-11

- Introduced `memhooks/v2` as a deliberately backend-neutral protocol core.
- Removed provider-specific concepts from the universal schema, including top-level `bank`, `memory_types`, `connection_types`, `mental_models`, and `knowledge_pages`.
- Added opaque `backends.<provider>` namespaces for provider-native retrieval controls; the core preserves and structurally merges provider mappings without interpreting their internal keys.
- Defined deterministic provider merge semantics: nested mappings recursively merge, while more-local scalar/list values replace parent values; query-local provider configuration overlays resolved scope configuration.
- Added backend-neutral `resources` cues with optional open `kind` and `salience` metadata.
- Kept weighted retrieval (`priority`), role routing (`when.roles`), generic entities/salience, tags, exclusions, scope, sensitivity, and filesystem inheritance in the core.
- Made the reference resolver reject unsupported schemas during resolution rather than silently ignoring unknown provider-specific data.
- Updated the validator to reject provider-native fields placed directly in the v2 core while leaving provider namespace internals opaque to generic validation.
- Updated `memhooks explain` JSON/human output to expose generic resources and resolved backend namespaces rather than Hindsight-shaped fields.
- Reworked the zero-LLM maintainer around v2: removed `--memory-type`/`--connection-type`, added generic resources/tags plus opaque `--backends` JSON, and deep-merges repeated provider hints.
- Added first-class provider references for Hindsight, OpenViking, Honcho, **Mem0**, and generic/unknown backends.
- Added current Mem0 v3-style guidance for filters/entity scope, `top_k`, `threshold`, `rerank`, and Graph Memory retrieval while keeping those controls inside `backends.mem0`.
- Reframed documentation ownership: normal users enable MemHooks; agents/runtimes create, maintain, prune, and size local retrieval cues automatically.
- Updated templates, nested examples, Agent Skill instructions, Hermes integration docs, quickstart, protocol guide, Rust API docs, CLI docs, troubleshooting, and publishing guidance for v2.
- Bumped the Rust crate and Agent Skill version to `0.5.0`.

## 0.4.1 — 2026-09-11

- Published `memhooks` 0.4.1 to crates.io and verified the public install path with `cargo install memhooks --version 0.4.1` plus an installed-binary version check.
- Prepared the Rust reference implementation for publication on crates.io.
- Added crates.io package metadata: homepage, docs.rs URL, README, keywords, categories, and a lean explicit package include set.
- Added direct installation instructions for `cargo install memhooks` and `cargo add memhooks`.
- Updated the README to explain the CLI/library split in plain language and to use package-safe absolute artwork URLs.
- Added a documentation index, quickstart, CLI reference, Rust API guide, agent integration guide, protocol guide, troubleshooting reference, and crates.io publishing guide under `docs/`.
- Exposed the Rust library guide as crate-level Rustdoc so docs.rs receives a maintained landing page.
- Added Rustdoc warnings-as-errors and `cargo publish --dry-run` to CI so documentation and package publishability are continuously verified.
- Kept the MemHooks protocol itself unchanged: `memhooks/v1`, weighted retrieval, entity salience, role routing, inheritance, and runtime behavior are identical to v0.4.0.

## 0.4.0 — 2026-09-11

- Added weighted retrieval for structured `recall_queries` with optional `priority` values from `0.0` to `1.0`.
- Added optional entity `salience` values from `0.0` to `1.0`, kept semantically separate from query priority and backend relevance/confidence scores.
- Added role-based query routing through extensible `when.roles` conditions. Role names remain runtime/project-defined rather than a closed MemHooks taxonomy.
- Preserved backward compatibility: string-only queries/entities and structured entries without priority, salience, or role conditions remain valid under `memhooks/v1`.
- Added the Rust reference library for parsing, resolving, role-filtering, and validating MemHooks inheritance.
- Added the `memhooks` CLI with `validate` and `explain` commands.
- Added human-readable, JSON, and SARIF validation output plus CI-friendly exit codes.
- Added repository-wide discovery that respects Git ignore rules and source-aware diagnostics for schema/routing mistakes.
- Added regression coverage for weighted queries, salient entities, inheritance cutoffs, role routing, and legacy syntax.
- Added GitHub Actions CI covering Rust formatting, Clippy, Rust tests, repository validation, and the existing Python regression suite.

## 0.3.0 — 2026-09-07

- Added first-class `memory_types` routing metadata with Hindsight-compatible `world`, `experience`, and `observation` categories.
- Added `connection_types` routing hints for `semantic`, `temporal`, `entity`, and `causal` knowledge connections, kept explicitly separate from memory and entity types.
- Added structured `recall_queries` that can carry query-local memory categories, connection emphasis, and entities.
- Added typed entity routing with `{name, type}` while keeping legacy string entities valid.
- Kept entity typing open rather than inventing a universal taxonomy: preserve explicit backend/user types; do not guess unknown types.
- Added `mental_models` as a distinct higher-level retrieval target alongside `knowledge_pages`; neither is treated as a raw memory type.
- Extended `memhooks_update.py note` with `--memory-type`, `--connection-type`, and repeatable `--entity` arguments.
- Semantic notes are now stored as bounded JSON routing records so type metadata survives maintenance; old markdown bullet notes remain readable and migrate on the next write.
- Clarified Hindsight's documented ontology: each selected memory type runs the full four-strategy recall pipeline independently.
- Deterministic path auto-anchors remain intentionally untyped rather than guessing semantic classification without an LLM.

## 0.2.1 — 2026-08-30

- Added the Hermes slash-command bootstrap `/memhooks init` for one-time per-project opt-in.
- Documented that Hermes automatically exposes installed skills as slash commands, so MemHooks does not patch Hermes' built-in command registry.
- `/memhooks init` delegates to the existing deterministic `scripts/memhooks_update.py init` initializer and creates the root `MEMHOOKS.md` for the active project.
- Clarified that only the project root needs explicit initialization; downstream `MEMHOOKS.md` files can then be created and maintained automatically from tool activity.

## 0.2.0 — 2026-08-29

- Added `scripts/memhooks_update.py`, a zero-LLM routing-file maintainer.
- Added `init`, `event`, and `note` modes.
- Added deterministic `post_tool_call` auto-anchors so touched code areas acquire future recall cues without a model pass.
- Added same-turn semantic notes for important decisions/failures/constraints without an extra LLM call.
- Kept memory retention itself out of scope: MemHooks writes retrieval metadata, not memories.
- Updated Hermes installation to wire both `pre_llm_call` loading and `post_tool_call` maintenance.

## 0.1.1 — 2026-08-29

- Added a real Hermes `pre_llm_call` shell hook that deterministically loads applicable `MEMHOOKS.md` files before every user-turn LLM call.
- Added root detection, root-to-leaf loading, `inherits: false`, and bounded context injection without an extra model call.
- Added Hermes hook installation/config documentation.
- Clarified the split between deterministic hook loading and memory-backend-specific retrieval.

## 0.1.0 — 2026-08-29

- Initial public skill/spec package.
- Root-to-leaf `MEMHOOKS.md` inheritance.
- Retrieval-only boundary.
- Hindsight, OpenViking, and Honcho mappings.
- Generic self-adaptation guidance for unlisted memory systems.
- Hermes / Agent Skills compatible `SKILL.md` layout.
