---
schema: memhooks/v1
scope: backend/auth
inherits: true

memory_types:
  - world
  - experience
  - observation

connection_types:
  - entity
  - causal
  - temporal
  - semantic

recall_queries:
  - query: "Why was the current token refresh architecture chosen, and what alternatives were rejected?"
    memory_types:
      - world
      - experience
    connection_types:
      - causal
      - semantic
    entities:
      - authentication
      - refresh token

  - query: "What previous bugs or production failures involved token rotation, session expiry, or authentication state?"
    memory_types:
      - experience
    connection_types:
      - temporal
      - causal
      - entity
    entities:
      - authentication
      - refresh token
      - session expiry

entities:
  - authentication
  - refresh token
  - session expiry

tags:
  - subsystem:auth

exclude:
  - deprecated cookie-only prototype
---

# Retrieval guidance

Before changing authentication behavior, recall the design rationale and incident history. If the evidence disagrees, use the current memory backend's deeper reasoning/synthesis operation rather than guessing.
