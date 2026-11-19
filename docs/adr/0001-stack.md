# 0001. Rust server, React client, SQLite first

- Status: accepted
- Date: 2026-11-18

## Context
The platform needs a fast, safe server with a long life (MCP, engines, sandboxed plugins later)
and a dashboard. It runs locally first and must be hostable later (PLAN: Tech stack, Local now).

## Decision
- Server: Rust (edition 2024), axum + tokio, sqlx. One binary `mnemo`.
- Storage: SQLite locally (WAL, foreign keys on), PostgreSQL later behind the same store interface.
- Client: React + TypeScript + Vite, managed with **pnpm** (npm 10.9 failed to resolve the
  Vitest 4/5 peer graph). TypeScript 5.9 until `openapi-typescript` supports newer versions.
- API types: the server's OpenAPI (utoipa) generates the client's types (`just gen-api`); CI
  fails on drift.

## Consequences
Single language for server, CLI, engines and devloop. Two SQL dialects to support later, so
queries stay portable. Client tooling versions are pinned where the ecosystem lags.
