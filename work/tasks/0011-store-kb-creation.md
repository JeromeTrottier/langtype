---
id: 11
title: Create knowledge bases from a kit version
milestone: M1
status: proposed
depends_on: [10, 3]
size: M
area: [server/crates/store]
attempts: 0
branch: null
---

## Goal
PLAN M1 "KB creation from a kit": `Store::create_kb(workspace, name, kit_id, version, settings,
creator)` pins the exact kit version and hash, validates settings against the kit's
`settings` definitions (defaults applied), and makes the creator an `owner` KbMember
(DOMAIN_MODEL §3, §6). Plus `get_kb`, `list_kbs(user)`, `set_kb_status`, `kb_member(kb, user)`.

## Acceptance criteria
- [ ] Created KB stores `kit_id`, `kit_version`, `kit_sha256` equal to the installed version —
      store test.
- [ ] Creating from an unknown kit or version fails; from a yanked version fails — tests.
- [ ] Settings: missing values get defaults; wrong types are rejected with field errors (uses 0003)
      — tests.
- [ ] The creator becomes an `owner` KbMember; `list_kbs(user)` only returns KBs they're a member of
      — tests.
- [ ] A KB always has at least one owner: removing/demoting the last owner fails — test.
- [ ] `set_kb_status` moves between `active | paused | archived | upgrading` — test.
- [ ] Every query filters by `kb_id` (review check) and all inserts run in one transaction — test
      that a failure mid-way leaves no KB row.
- [ ] `just check` passes.

## Notes
Workspace membership is required to create a KB in that workspace (local mode: the local user's
personal workspace). KB name is NFC-normalised and non-empty.

## Log
