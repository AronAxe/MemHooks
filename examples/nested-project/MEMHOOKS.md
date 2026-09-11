---
schema: memhooks/v2
scope: project-root
inherits: true

recall_queries:
  - "What are the current architectural boundaries and non-negotiable project constraints?"
  - "Which major approaches were previously rejected, and why?"

entities:
  - Example Project

resources:
  - name: Architecture/System overview
    kind: architecture
  - name: Decisions/Current architecture
    kind: decision-log

tags:
  - project:example

exclude:
  - abandoned v0 prototype

sensitivity: private

backends:
  hindsight:
    bank: example-project
  mem0:
    filters:
      user_id: example-project-agent
---

# Retrieval guidance

Prefer established project context and concrete past decisions. Do not resurrect abandoned v0 assumptions merely because they are semantically similar.
