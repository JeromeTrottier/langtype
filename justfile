# Mnemo development commands. `just --list` to see them all.

set shell := ["bash", "-cu"]

default:
    @just --list

# Server on 127.0.0.1:3000 and the dashboard dev server on :5173.
dev:
    trap 'kill 0' EXIT; cargo run -p mnemo-api -- serve & pnpm --dir client dev & wait

# Run only the server.
serve:
    cargo run -p mnemo-api -- serve

# Format Rust and client code.
fmt:
    cargo fmt --all
    pnpm --dir client run format

# Regenerate the client's API types from the server's OpenAPI.
gen-api:
    cargo run -q -p mnemo-api -- openapi > client/src/api/openapi.json
    pnpm --dir client run gen:api

# Fail if the generated API types are out of date.
check-api:
    cargo run -q -p mnemo-api -- openapi > /tmp/mnemo-openapi.json
    diff -q /tmp/mnemo-openapi.json client/src/api/openapi.json || (echo "API changed: run 'just gen-api'" && exit 1)

# Validate every kit (the invalid fixture must fail).
check-kits:
    cargo run -q -p mnemo-cli --bin kit -- validate kits/_test
    ! cargo run -q -p mnemo-cli --bin kit -- validate kits/_test/invalid > /dev/null

# Quick checks used by the agents' Stop hook.
check-fast:
    cargo fmt --all --check
    cargo clippy --workspace --all-targets -q -- -D warnings
    cargo test --workspace -q
    pnpm --dir client run typecheck
    pnpm --dir client run test

# Everything CI checks. Must pass before work is handed off.
check: check-fast check-kits check-api
    pnpm --dir client run lint
    pnpm --dir client run format:check
    pnpm --dir client run build

# Build and show the development agents' status.
agents-status:
    cargo run -q -p devloop -- status

# Live feed of what the development agents are doing.
agents-watch *args:
    cargo run -q -p devloop -- watch {{args}}

# Validation screenshots for pull requests (see client/scripts/screenshot.mjs).
[positional-arguments]
screenshot *args:
    node client/scripts/screenshot.mjs "$@"

# Install devloop to run at login (builds a release binary first).
agents-install:
    cargo build -q -p devloop --release
    ./target/release/devloop install
