# Weighted Retrieval and Roles

Two core MemHooks features help a runtime decide **which retrieval requests matter most** and **when they apply**:

- query `priority`;
- `when.roles`.

They are intentionally simple and backend-neutral.

## Query priority

A structured recall query may declare:

```yaml
recall_queries:
  - query: "Which previous failures could make this refactor unsafe?"
    priority: 0.95
```

Valid explicit range:

```text
0.0 .. 1.0
```

Priority means:

> **When context budget is limited, how costly is it to omit this recall request?**

It does not mean:

- semantic similarity;
- memory confidence;
- truth probability;
- provider relevance;
- a universal numeric ranking formula.

MemHooks does not mandate `score × priority` or any other single equation.

## No magic default

If `priority` is omitted, the protocol does not silently invent `0.5`, `1.0`, or another universal value.

An unweighted query remains an ordinary applicable retrieval request.

This matters because absence of metadata should not pretend the agent made a quantitative judgment it never made.

## Priority versus backend ranking

Suppose a backend searches for two effective queries:

```text
Query A priority: 0.95
Query B priority: 0.40
```

Within Query A's results, the memory backend may return relevance scores such as:

```text
0.91, 0.84, 0.70
```

Those numbers answer different questions:

```text
MemHooks priority
  → importance of satisfying the recall request

backend relevance
  → quality of a particular returned result for that search
```

Do not collapse them into one concept.

## Entity and resource salience

Entities/resources can separately carry `salience`:

```yaml
entities:
  - name: Authentication
    salience: 0.9

resources:
  - name: outage-postmortem
    kind: postmortem
    salience: 1.0
```

Salience describes the importance of the **cue**, not the whole query.

So it is perfectly valid to have:

```text
query priority:       0.7
entity salience:      0.95
backend result score: 0.82
```

Those numbers do not conflict.

## Role routing

A structured query may be restricted to one or more runtime/project roles:

```yaml
recall_queries:
  - query: "Which rejected architecture options should be reconsidered?"
    when:
      roles: [architect, reviewer]
```

Role names are deliberately open strings. MemHooks does not impose a universal taxonomy such as `coder`, `reviewer`, `planner`, etc.

Your runtime decides which roles exist.

## Matching semantics

When active roles are known, matching is exact-string OR matching.

Declared:

```yaml
roles: [reviewer, architect]
```

Active:

```text
reviewer
```

→ query applies.

Active:

```text
tester
```

→ query does not apply.

## Unknown role is not a non-match

If the runtime has **no role information**, role-restricted queries are preserved.

This is deliberate.

```text
role unknown
≠
known role does not match
```

Dropping restricted queries merely because an adapter lacks a role concept would silently lose retrieval intent.

## Runtime aliases

If a host runtime has role aliases, normalize them **before** passing active roles to the resolver.

For example, a host might decide:

```text
code-reviewer → reviewer
security-review → reviewer + security
```

That alias policy belongs to the host, not the MemHooks protocol.

## Priority + roles together

```yaml
recall_queries:
  - query: "Which prior security failures affect this design?"
    priority: 1.0
    when:
      roles: [reviewer, security]
```

This says:

> When the runtime knows the agent is acting as `reviewer` or `security`, this recall request is extremely important under context pressure.

It does **not** say anything about which backend must be used or how that backend ranks individual memories.

## Practical budgeting

A reasonable runtime can use a policy roughly like:

1. enforce security/access rules first;
2. apply role routing;
3. preserve higher-priority recall requests when not all can be satisfied;
4. use entity/resource salience as cue hints;
5. let each memory backend rank results natively;
6. deduplicate returned evidence;
7. stop when more retrieval is unlikely to change the task.

MemHooks provides the intent. The runtime owns the context-budget algorithm.