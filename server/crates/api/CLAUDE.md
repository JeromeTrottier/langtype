# api

The `mnemo` binary. Handlers are thin: parse, call core/store, map errors.
- Every route goes through `check_host_and_origin`; every non-public route through `require_token`
  (ADR 0004). Add a test for each new route's auth.
- Every route is in `ApiDoc` (utoipa); run `just gen-api` after API changes.
