---
id: 10
title: Install kit versions into the store (immutable)
milestone: M1
status: proposed
depends_on:
- 8
size: M
area:
- server/crates/store
attempts: 0
branch: null
issue: 10
issue_synced_at: 2026-11-19T23:10:13.140351Z
issue_body_hash: 8fc967dd5b6edf32
issue_status: proposed
---

## Goal
Invariant 6: published kit versions are immutable. Installing a local kit folder validates it,
computes its hash and stores `(kit_id, version, sha256, manifest, files)` in `kits` /
`kit_versions` (PLAN §11 Installing). Add `Store::install_kit`, `Store::get_kit_version`,
`Store::list_kits`.

## Acceptance criteria
- [ ] Installing `kits/_test` creates one `kits` row (`source = local`) and one `kit_versions` row
      with the `kit_sha256` from 0008 and the manifest JSON — store test.
- [ ] Installing the same version with the same hash is a no-op returning the existing row — test.
- [ ] Installing the same version with a **different** hash fails with `KitVersionConflict` and
      leaves the stored row unchanged — test.
- [ ] An invalid kit is refused with its problems; nothing is written — test.
- [ ] `latest_version` follows the highest semver installed, not the latest install — test.
- [ ] `get_kit_version` returns a `KitDef` (0001) built from the stored manifest — test.
- [ ] `just check` passes.

## Notes
Store depends on `mnemo-kits` for validation and hashing; keep SQL portable (store CLAUDE.md).
No new migration should be needed; if one is, add `0002_*.sql`.

## Log
- 2026-11-19 18:10 — issue #10 opened
