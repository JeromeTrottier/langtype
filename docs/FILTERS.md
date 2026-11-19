# Expressions and queries (spec v1)

One expression language and one query format, used everywhere: kit rules, activity `select` and
`map`, briefing sections, signals, views, MCP tools (`query_entities`, `search`) and the dashboard.
Changing this spec requires an ADR.

## 1. Expressions: CEL

Expressions are [CEL](https://cel.dev) (Common Expression Language): safe, side-effect free,
always terminating. Evaluated with a Rust CEL implementation.

### Variables in scope

| Variable | Type | Available in |
|---|---|---|
| `entity` | `{ id, type, data, status, created_at, updated_at, owner_user_id }` | filters, computed fields, `map`, visibility conditions |
| `state` | map: the engine state of `entity` for the engine in context (keyed by `key` when several) | filters, rules, `map` |
| `event` | `{ type, class, payload, subjects, at }` | rule triggers |
| `record` | library record data | `import_from` mappings |
| `role` | list of the caller's kit roles | filters, visibility |
| `user` | `{ id }` | filters, visibility |
| `now` | timestamp | everywhere |

### Platform functions

| Function | Returns |
|---|---|
| `days_since(t)` | Whole days between `t` and `now` (user's time zone for day boundaries) |
| `count(filter)` | Number of visible entities matching a filter (rules only) |
| `has_role(r)` | Whether the caller has kit role `r` |
| `normalize(s)` | NFC-normalised, case-folded string (accents kept) |
| `tts(text)` | An audio reference for presenters (text-to-speech) |

Standard CEL functions (`size`, `contains`, `startsWith`, `matches`, timestamps, durations) are
available.

## 2. Queries: the filter format

```yaml
type: [character, npc]                 # one entity type or a list (required)
where: "state.in_combat && entity.data.hp > 0"   # CEL predicate (optional)
include: { history: false }            # false (default): current state only
sort: [ { field: updated_at, dir: desc } ]       # fields: data.<name>, created_at, updated_at, state.<name>
limit: 20                              # 1..1000, default 50
```

The same object is the `filter` argument of MCP tools, written as JSON.

### Evaluation

1. **SQL step:** the part of `where` in the safe subset is compiled to SQL. The subset is:
   comparisons (`==`, `!=`, `<`, `<=`, `>`, `>=`) between a field path (`entity.data.x`,
   `entity.status`, `entity.updated_at`) and a literal; `&&`, `||`, `!`; `in` with a literal list;
   `has(entity.data.x)`.
2. **Memory step:** the rest of the predicate (engine `state`, functions) is evaluated on the SQL
   results, with a cap on rows scanned.
3. **Visibility:** results are filtered with the caller's role (DOMAIN_MODEL, Visibility).
4. **Sort and limit.**

### Validation

`kit validate` parses and type-checks every expression and filter against the kit's entity
types and the variables in scope, and rejects unknown fields, unknown types and wrong types.

## 3. Examples

```yaml
# Activity: items due for review
select: { type: item, where: "state.due <= now", sort: [ { field: state.due, dir: asc } ], limit: 15 }

# Briefing section: open threads, newest first
{ type: thread, where: "entity.status == 'open'", sort: [ { field: updated_at, dir: desc } ], limit: 10 }

# Rule condition
when: "event.type == 'quest_completed' && has(event.payload.faction)"
```
