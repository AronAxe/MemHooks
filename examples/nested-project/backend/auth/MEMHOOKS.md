---
schema: memhooks/v2
scope: backend/auth
inherits: true

recall_queries:
  - query: "Why was the current token refresh architecture chosen, and what alternatives were rejected?"
    priority: 0.9
    entities:
      - authentication
      - refresh token
    tags: [architecture]
    backends:
      hindsight:
        memory_types: [world, experience]
        connection_types: [causal, semantic]
      mem0:
        top_k: 8
        rerank: true

  - query: "What previous bugs or production failures involved token rotation, session expiry, or authentication state?"
    priority: 1.0
    entities:
      - authentication
      - refresh token
      - session expiry
    tags: [incident, reliability]
    backends:
      hindsight:
        memory_types: [experience]
        connection_types: [temporal, causal, entity]
      mem0:
        top_k: 12
        threshold: 0.1

entities:
  - authentication
  - refresh token
  - session expiry

resources:
  - name: auth-incident-history
    kind: postmortem-index
    salience: 0.9

tags:
  - subsystem:auth

exclude:
  - deprecated cookie-only prototype
---

# Retrieval guidance

Before changing authentication behavior, recall the design rationale and incident history. If evidence disagrees, use the active provider's deeper reasoning/synthesis capability when available rather than guessing.
