---
schema: memhooks/v1
inherits: true

# Optional: memory namespace/bank/peer/session hint when the backend has one.
# bank: project-name

# Optional descriptive scope.
# scope: backend/auth

# Existing stable summaries/pages/mental models worth retrieving first.
knowledge_pages: []

# Optional scope-wide memory/fact-type hints. Empty means no type filter.
# Hindsight supports: world | experience | observation
memory_types: []

# Specific questions whose answers matter when working in this directory.
# String entries remain valid. Use the structured form when type/entity routing
# is known and useful.
recall_queries:
  - "What architectural decisions govern this subsystem, and why were they made?"
  - query: "What previous failures, rejected approaches, or important gotchas should be remembered before changing it?"
    memory_types: []
    entities: []

# Named entities that should sharpen retrieval.
# Legacy: - Authentication
# Typed:  - name: Authentication
#           type: COMPONENT
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
helper). When clear, preserve its memory type and typed entities as routing
metadata. Store the actual fact in the memory backend, not here.
