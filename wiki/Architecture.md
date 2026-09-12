# Architecture

MemHooks is intentionally small enough that the reference implementation can remain auditable.

The important architectural property is **one model, one resolver, one set of semantics**.

## Component map

```text
MEMHOOKS.md
   │
   ├── parser ────────────────┐
   │                          │
   ├── validator              │
   │                          │
   └── resolver               │
          │                   │
          ├── effective plan  │
          │       │           │
          │       └── adapter/runtime → memory backend
          │
          └── explain CLI

maintainer (init/note/event)
   │
   └── writes the same frontmatter model ↑
```

## Parser

Responsibilities:

- locate YAML frontmatter;
- deserialize `memhooks/v2` core fields;
- preserve unknown/future fields where safe;
- keep optional Markdown body guidance separate;
- retain source location information needed for diagnostics.

Malformed structured entries should not automatically destroy linting for the entire file. The parser/model should preserve enough raw structure for the validator to issue targeted diagnostics.

## Model

The model represents backend-neutral protocol concepts:

- hook frontmatter;
- recall queries;
- role conditions;
- entities;
- resources;
- opaque backend maps.

Provider-native values are represented as opaque YAML mappings rather than becoming Rust fields in the universal core.

That is a feature: adding a Mem0 or Hindsight option should not require teaching the generic model what that option means.

## Resolver

Responsibilities:

- verify the target exists;
- find the root;
- build the root→leaf chain;
- enforce supported schema;
- apply `inherits: false`;
- merge core fields;
- apply same-query local override semantics;
- structurally merge backend namespaces;
- preserve provenance/guidance;
- expose effective queries after optional role filtering.

Adapters should reuse the resolver instead of implementing their own near-copy.

## Validator

Responsibilities:

- validate supported schema;
- report unknown/malformed core fields;
- validate priority/salience ranges;
- validate roles/entities/resources;
- validate backend namespace shape;
- surface duplicates/conflicts;
- emit human, JSON, and SARIF diagnostics.

Provider-internal validation is deliberately out of generic scope.

## Real YAML spans

Diagnostics should point to the actual offending YAML node—not the first textual line containing a matching word.

This matters especially for:

- repeated `priority` keys in different sequence items;
- repeated `salience` keys;
- words such as `schema:` appearing in Markdown examples;
- SARIF/code-scanning annotations.

The hardened parser/validator uses a YAML AST with source markers for diagnostic anchoring instead of scanning the entire source text heuristically.

## Maintainer

The maintainer lives in the Rust reference implementation and exposes:

```text
init
note
event
```

Responsibilities:

- initialize `memhooks/v2` frontmatter;
- upsert semantic recall queries;
- preserve unknown/provider fields across rewrites;
- maintain conservative file-resource anchors;
- share the resolver's root semantics;
- lock concurrent updates;
- write atomically.

The Python helper remains only as a runtime compatibility launcher where useful.

## CLI

The CLI is intentionally thin:

```text
memhooks validate
memhooks explain
memhooks init
memhooks note
memhooks event
```

`explain --format json` serializes the resolved structure rather than maintaining a manually duplicated field mirror. That reduces drift when the resolved model evolves.

## Runtime adapter

The adapter is responsible for things MemHooks should not decide globally:

- active role identification;
- memory backend selection;
- credentials;
- provider-specific configuration interpretation;
- retrieval execution;
- context budget;
- prompt placement/trust;
- provider result provenance.

The adapter should consume the reference resolved plan rather than reparsing repository files independently.

## Memory backend

The backend stores/retrieves the actual memories.

MemHooks does not require it to expose a specific ontology. The adapter translates generic routing intent plus the selected provider namespace into the backend's real API.

## Why not make MemHooks a daemon?

A daemon would add lifecycle, installation, privilege, failure, and synchronization complexity to a problem that can remain mostly deterministic and on-demand.

The reference design therefore favors:

```text
resolve when needed
maintain after relevant activity
exit
```

rather than a permanently running service.

## CI as architecture enforcement

The hardened CI checks more than compilation:

- rustfmt;
- Clippy with warnings denied;
- ordinary Rust tests;
- doctests;
- Rustdoc warnings as errors;
- repository MemHooks validation;
- declared Rust MSRV;
- Python 3.10 + 3.12 syntax/lint/tests;
- package manifest/publishability checks.

The goal is to prevent the reference implementation from making claims the test suite never exercises.

## Package boundary

The published crate includes the source and documentation needed to understand the protocol/reference implementation, including the normative/provider references.

This matters because docs.rs and crates.io users should not receive a package whose README links to important files that were omitted from the crate.

## Design principle

The architecture should make it hard for these to diverge:

```text
what writes hooks
what reads hooks
what validates hooks
what adapters consume
what documentation claims
```

The fewer independent interpretations of the protocol exist, the less likely MemHooks is to fail at the seams.