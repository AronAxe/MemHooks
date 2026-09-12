---
schema: memhooks/v2
inherits: true

# Optional descriptive scope. This is backend-neutral metadata.
# scope: backend/auth

# Concrete questions whose answers matter when an agent works here.
recall_queries:
  - "What architectural decisions govern this subsystem, and why were they made?"
  - query: "What previous failures, rejected approaches, or important gotchas should be remembered before changing it?"
    # Optional retrieval importance. 0.0 = lowest, 1.0 = highest.
    priority: 0.8
    # Optional role routing. Role names are project/runtime-defined.
    when:
      roles: [reviewer, refactor]
    # Query-local generic cues supplement scope-level cues.
    entities: []
    resources: []
    tags: []
    # Provider-native controls belong only here.
    backends: {}

# Named retrieval cues.
# Simple: - Authentication
# Structured:
#   - name: OpenAI
#     type: ORG
#     salience: 0.9
entities: []

# Named existing resources worth retrieving/reading when relevant.
# Simple: - auth-architecture
# Structured:
#   - name: outage-postmortem
#     kind: postmortem
#     salience: 0.9
resources: []

# Backend-neutral relevance/routing labels.
tags: []

# Obsolete or misleading context that should not enter the active task.
exclude: []

# Advisory only; host policy remains authoritative.
sensitivity: private

# Opaque provider namespaces. MemHooks preserves/merges these but does not
# interpret their internal fields.
backends: {}
# Example provider mappings, when genuinely needed:
# backends:
#   hindsight:
#     bank: project-memory
#     memory_types: [experience]
#     connection_types: [causal, temporal]
#     strategy: reflect
#   mem0:
#     filters:
#       user_id: project-agent
#     top_k: 8
#     rerank: true
#   openviking: {}
#   honcho: {}
---

# Retrieval guidance

Use direct recall for concrete decisions, events, and implementation facts.
Use deeper synthesis only when the active backend exposes it and the task needs it.

This file is agent/runtime-maintained routing metadata, not memory content. Keep
retrieval cues concise enough to function as an index. When a durable cue is
learned, preserve only metadata that is actually known. Put provider-native
controls under `backends.<provider>`; never promote one backend's vocabulary
into the universal MemHooks core.
