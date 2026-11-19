---
id: 8
title: Kit archive format, file hashes and `kit pack`
milestone: M1
status: proposed
depends_on:
- 6
size: M
area:
- server/crates/kits
- server/crates/cli
attempts: 0
branch: null
issue: 8
issue_synced_at: 2026-11-19T23:10:04.693668Z
issue_body_hash: 15bf844a9d093214
issue_status: proposed
---

## Goal
PLAN §11 Package format: a kit is published as a `.tgz` of its folder plus a generated
`.kit/metadata.json` with the sha256 of every file. A KB pins an exact version **and hash**
(invariant 6), so M1 needs a deterministic kit hash. Add `kits::archive` with
`file_hashes(dir)`, `kit_sha256(dir)`, `pack(dir, out)` and `unpack(archive, dest)` (verifying
hashes), and wire `kit pack`.

## Acceptance criteria
- [ ] `file_hashes` lists every file under the kit folder (sorted, `/`-separated relative paths),
      skipping `.kit/`, `.git/`, `.DS_Store` — test.
- [ ] `kit_sha256` is stable across runs and machines (independent of file order and mtimes), and
      changes when any file's content changes — tests with a temp copy of `kits/_test`.
- [ ] `pack` refuses an invalid kit and writes `<id-name>-<version>.tgz` containing the files and
      `.kit/metadata.json` (`id`, `version`, `sha256`, `files`) — test.
- [ ] `pack` is reproducible: packing twice yields byte-identical archives — test.
- [ ] `unpack` rejects an archive whose file doesn't match its listed hash, with an absolute path,
      or with `..` components — tests.
- [ ] `kit pack <dir> [-o out]` works; `kit validate` behaviour unchanged — CLI test.
- [ ] `just check` passes.

## Notes
Libraries are a second package type but are out of scope here (M3). Signing is M8. Keep
dependencies small (`sha2`, `tar`, `flate2`).

## Log
- 2026-11-19 18:10 — issue #8 opened
