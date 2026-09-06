---
schema: memhooks/v1
inherits: true

# Optional: memory namespace/bank/peer/session hint when the backend has one.
# bank: project-name

# Optional descriptive scope.
# scope: backend/auth

# Optional memory categories. Empty means no type restriction.
# Hindsight: world | experience | observation
memory_types: []

# Optional connection emphasis. Empty means no preference.
# Hindsight: semantic | temporal | entity | causal
connection_types: []

# Existing standing answers / synthesized resources worth reading first.
mental_models: []
knowledge_pages: []

# Specific questions whose answers matter when working in this directory.
# String entries remain valid. Use the structured form when routing metadata
# is genuinely known.
recall_queries:
  - "What architectural decisions govern this subsystem, and why were they made?"
  - query: "What previous failures, rejected approaches, or important gotchas should be remembered before changing it?"
    memory_types: []
    connection_types: []
    entities: []

# Named entities that should sharpen retrieval.
# Legacy/untyped: - Authentication
# Typed only when known:
#   - name: OpenAI
#     type: ORG
entities: []

# Backend-independent relevance hints; use native metadata filters when supported.
tags: []

# Obsolete or misleading memories that should not enter current context.
exclude: []

# Advisory only: public | internal | private
sensitivity: private
---

# Retrieval guidance

Use direct recall for concrete decisions, events, and implementation facts.
Use deeper memory reasoning only when synthesis is actually required.

If this turn establishes a durable non-obvious decision, failure, constraint,
or rejected approach that future work here could miss, record one concise
future-retrieval question in this hook (or use the runtime's MemHooks note
helper). When genuinely known, preserve its memory category, relevant connection
emphasis, and typed entities as routing metadata. Do not guess merely to fill
fields. Store the actual fact in the memory backend, not here.
