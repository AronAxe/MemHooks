# Security policy

MemHooks processes repository-controlled routing metadata and may place retrieved context into an AI agent's working context. Treating repository text as untrusted data is therefore a core security boundary, not an optional convention.

## Reporting a vulnerability

Please report security vulnerabilities privately rather than opening a public issue with exploit details.

Use GitHub's private vulnerability reporting / Security Advisory flow for this repository when available. If that interface is unavailable, contact the repository owner privately through GitHub before publishing technical exploit details.

Include, when possible:

- affected MemHooks version;
- affected runtime/integration (reference CLI, Hermes hook, or another adapter);
- reproduction steps;
- expected versus actual trust boundary;
- whether the issue permits data loss, path escape, prompt-authority escalation, command execution, or unauthorized memory access/writes.

## Security boundaries

A `MEMHOOKS.md` file is repository-controlled input. It must never, by itself:

- acquire system/developer prompt authority;
- grant tool permissions or credentials;
- authorize arbitrary command execution;
- authorize memory writes/deletes;
- escape the canonical project root;
- bypass the host runtime's normal access controls.

Provider-native data under `backends.<provider>` is opaque configuration carried for an authorized adapter. The generic MemHooks core does not execute it.

The reference maintainer uses project-root containment, exclusive file locking, and atomic replacement when changing hook files.

## Supported versions

Security fixes are applied to the latest published release. Users should upgrade to the newest patch release before reporting an issue already fixed in a later version.
