# store

SQLite now, PostgreSQL later — keep SQL portable (no SQLite-only functions outside migrations).
- Never edit an applied migration; add `migrations/NNNN_name.sql`.
- Every query on KB data filters by `kb_id`. Writes happen inside one transaction per changeset.
- Tests use `Store::open("sqlite::memory:")`.
