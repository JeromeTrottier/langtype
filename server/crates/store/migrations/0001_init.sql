-- Mnemo initial schema (DOMAIN_MODEL.md). Multi-user from day one; local mode uses one user.
-- IDs are UUIDv7 strings. Timestamps are UTC RFC 3339 strings. JSON is stored as TEXT.
-- Never edit this file once applied; add a new migration instead.

-- Identity & access ---------------------------------------------------------

CREATE TABLE users (
    id              TEXT PRIMARY KEY,
    email           TEXT UNIQUE,
    display_name    TEXT NOT NULL,
    locale          TEXT NOT NULL DEFAULT 'en',
    native_language TEXT NOT NULL DEFAULT 'en',
    created_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE workspaces (
    id         TEXT PRIMARY KEY,
    name       TEXT NOT NULL,
    kind       TEXT NOT NULL CHECK (kind IN ('personal', 'team')),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE memberships (
    user_id        TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    workspace_id   TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    workspace_role TEXT NOT NULL CHECK (workspace_role IN ('owner', 'admin', 'member')),
    created_at     TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    PRIMARY KEY (user_id, workspace_id)
);

-- Kits & libraries ----------------------------------------------------------

CREATE TABLE kits (
    id             TEXT PRIMARY KEY,               -- '@scope/name'
    latest_version TEXT,
    source         TEXT NOT NULL CHECK (source IN ('local', 'git', 'registry')),
    created_at     TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE kit_versions (
    kit_id       TEXT NOT NULL REFERENCES kits(id),
    version      TEXT NOT NULL,
    sha256       TEXT NOT NULL,
    manifest     TEXT NOT NULL,                    -- JSON
    files        TEXT NOT NULL DEFAULT '{}',       -- JSON: path -> sha256
    yanked       INTEGER NOT NULL DEFAULT 0,
    installed_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    PRIMARY KEY (kit_id, version)
);

CREATE TABLE libraries (
    id            TEXT NOT NULL,                   -- '@scope/name'
    version       TEXT NOT NULL,
    sha256        TEXT NOT NULL,
    license       TEXT NOT NULL,
    attribution   TEXT NOT NULL,
    record_schema TEXT NOT NULL,                   -- JSON Schema
    search_fields TEXT NOT NULL DEFAULT '[]',      -- JSON array
    installed_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    PRIMARY KEY (id, version)
);

CREATE TABLE library_records (
    library_id      TEXT NOT NULL,
    library_version TEXT NOT NULL,
    record_id       TEXT NOT NULL,                 -- stable across versions
    data            TEXT NOT NULL,                 -- JSON
    rank            INTEGER,
    removed         INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (library_id, library_version, record_id),
    FOREIGN KEY (library_id, library_version) REFERENCES libraries(id, version)
);

-- Knowledge bases -----------------------------------------------------------

CREATE TABLE knowledge_bases (
    id             TEXT PRIMARY KEY,
    workspace_id   TEXT NOT NULL REFERENCES workspaces(id),
    name           TEXT NOT NULL,
    kit_id         TEXT NOT NULL,
    kit_version    TEXT NOT NULL,
    kit_sha256     TEXT NOT NULL,
    settings       TEXT NOT NULL DEFAULT '{}',     -- JSON
    status         TEXT NOT NULL DEFAULT 'active'
                   CHECK (status IN ('active', 'paused', 'archived', 'upgrading')),
    schema_version INTEGER NOT NULL DEFAULT 1,
    created_at     TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at     TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    FOREIGN KEY (kit_id, kit_version) REFERENCES kit_versions(kit_id, version)
);

CREATE TABLE kb_members (
    kb_id      TEXT NOT NULL REFERENCES knowledge_bases(id) ON DELETE CASCADE,
    user_id    TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    access     TEXT NOT NULL CHECK (access IN ('owner', 'editor', 'viewer')),
    kit_roles  TEXT NOT NULL DEFAULT '[]',         -- JSON array of role ids
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    PRIMARY KEY (kb_id, user_id)
);

CREATE TABLE agent_connections (
    id           TEXT PRIMARY KEY,
    kb_id        TEXT NOT NULL REFERENCES knowledge_bases(id) ON DELETE CASCADE,
    user_id      TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    client       TEXT NOT NULL,
    scopes       TEXT NOT NULL DEFAULT '["read","write"]',
    token_hash   TEXT,
    last_seen_at TEXT,
    revoked_at   TEXT,
    created_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    FOREIGN KEY (kb_id, user_id) REFERENCES kb_members(kb_id, user_id) ON DELETE CASCADE
);

CREATE TABLE entities (
    id            TEXT PRIMARY KEY,
    kb_id         TEXT NOT NULL REFERENCES knowledge_bases(id) ON DELETE CASCADE,
    type          TEXT NOT NULL,
    data          TEXT NOT NULL,                   -- JSON, valid against the kit type
    owner_user_id TEXT REFERENCES users(id),
    revealed_to   TEXT NOT NULL DEFAULT '[]',      -- JSON array of role ids / user ids
    status        TEXT CHECK (status IN ('open', 'resolved', 'superseded')),
    superseded_by TEXT REFERENCES entities(id),
    valid_until   TEXT,
    library_ref   TEXT,                            -- JSON {library_id, record_id, library_version}
    source        TEXT NOT NULL CHECK (source IN
                  ('setup', 'starter', 'library', 'member', 'agent', 'activity', 'engine', 'import')),
    version       INTEGER NOT NULL DEFAULT 1,
    created_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    deleted_at    TEXT
);
CREATE INDEX entities_kb_type ON entities(kb_id, type) WHERE deleted_at IS NULL;

CREATE TABLE relations (
    id          TEXT PRIMARY KEY,
    kb_id       TEXT NOT NULL REFERENCES knowledge_bases(id) ON DELETE CASCADE,
    type        TEXT NOT NULL,
    from_entity TEXT NOT NULL REFERENCES entities(id),
    to_entity   TEXT NOT NULL REFERENCES entities(id),
    data        TEXT,
    created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);
CREATE INDEX relations_from ON relations(kb_id, from_entity);
CREATE INDEX relations_to ON relations(kb_id, to_entity);

CREATE TABLE engine_state (
    kb_id      TEXT NOT NULL REFERENCES knowledge_bases(id) ON DELETE CASCADE,
    engine     TEXT NOT NULL,
    entity_id  TEXT NOT NULL REFERENCES entities(id),
    key        TEXT NOT NULL DEFAULT '',
    state      TEXT NOT NULL,                      -- JSON
    version    INTEGER NOT NULL DEFAULT 1,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    PRIMARY KEY (kb_id, engine, entity_id, key)
);

CREATE TABLE overrides (
    id                   TEXT PRIMARY KEY,
    scope                TEXT NOT NULL CHECK (scope IN ('kb', 'workspace')),
    scope_id             TEXT NOT NULL,
    target               TEXT NOT NULL CHECK (target IN ('skill', 'setting', 'briefing')),
    name                 TEXT NOT NULL,
    content              TEXT NOT NULL,
    based_on_kit_version TEXT NOT NULL,
    updated_at           TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

-- Sessions & activities -----------------------------------------------------

CREATE TABLE sessions (
    id            TEXT PRIMARY KEY,
    kb_id         TEXT NOT NULL REFERENCES knowledge_bases(id) ON DELETE CASCADE,
    user_id       TEXT NOT NULL REFERENCES users(id),
    role          TEXT,
    channel       TEXT NOT NULL CHECK (channel IN ('mcp', 'dashboard')),
    connection_id TEXT REFERENCES agent_connections(id),
    status        TEXT NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'closed', 'abandoned')),
    title         TEXT,
    started_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    ended_at      TEXT
);

CREATE TABLE briefings (
    session_id     TEXT PRIMARY KEY REFERENCES sessions(id) ON DELETE CASCADE,
    role           TEXT,
    sections       TEXT NOT NULL,                  -- JSON
    token_estimate INTEGER NOT NULL,
    generated_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE recaps (
    session_id    TEXT PRIMARY KEY REFERENCES sessions(id) ON DELETE CASCADE,
    summary       TEXT NOT NULL,
    next_steps    TEXT,
    changeset_ids TEXT NOT NULL DEFAULT '[]',
    event_ids     TEXT NOT NULL DEFAULT '[]',
    created_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE assignments (
    id          TEXT PRIMARY KEY,
    kb_id       TEXT NOT NULL REFERENCES knowledge_bases(id) ON DELETE CASCADE,
    activity    TEXT NOT NULL,
    items       TEXT,                              -- JSON array of entity ids
    note        TEXT,
    assigned_by TEXT NOT NULL,                     -- JSON Actor
    assigned_to TEXT NOT NULL REFERENCES users(id),
    status      TEXT NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'done', 'dismissed', 'expired')),
    due_at      TEXT,
    created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE activity_runs (
    id            TEXT PRIMARY KEY,
    kb_id         TEXT NOT NULL REFERENCES knowledge_bases(id) ON DELETE CASCADE,
    user_id       TEXT NOT NULL REFERENCES users(id),
    activity      TEXT NOT NULL,
    assignment_id TEXT REFERENCES assignments(id),
    items         TEXT NOT NULL DEFAULT '[]',      -- JSON array of entity ids served
    results       TEXT NOT NULL DEFAULT '[]',      -- JSON array of raw item results
    batch_ids     TEXT NOT NULL DEFAULT '[]',      -- idempotency keys of received batches
    summary       TEXT,
    status        TEXT NOT NULL DEFAULT 'started' CHECK (status IN ('started', 'completed', 'abandoned')),
    started_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    ended_at      TEXT
);

-- Change tracking & events --------------------------------------------------

CREATE TABLE changesets (
    id              TEXT PRIMARY KEY,
    kb_id           TEXT NOT NULL REFERENCES knowledge_bases(id) ON DELETE CASCADE,
    actor           TEXT NOT NULL,                 -- JSON Actor
    channel         TEXT NOT NULL,
    session_id      TEXT REFERENCES sessions(id),
    activity_run_id TEXT REFERENCES activity_runs(id),
    summary         TEXT NOT NULL,
    status          TEXT NOT NULL DEFAULT 'applied' CHECK (status IN ('draft', 'applied', 'discarded')),
    preview         TEXT,                          -- JSON, for drafts
    reverts         TEXT REFERENCES changesets(id),
    expires_at      TEXT,
    at              TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);
CREATE INDEX changesets_kb_at ON changesets(kb_id, at);

CREATE TABLE changes (
    changeset_id TEXT NOT NULL REFERENCES changesets(id) ON DELETE CASCADE,
    seq          INTEGER NOT NULL,
    target       TEXT NOT NULL CHECK (target IN ('entity', 'relation', 'engine_state', 'settings')),
    target_id    TEXT NOT NULL,
    op           TEXT NOT NULL CHECK (op IN ('create', 'update', 'delete')),
    before       TEXT,                             -- JSON
    after        TEXT,                             -- JSON
    PRIMARY KEY (changeset_id, seq)
);
CREATE INDEX changes_target ON changes(target, target_id);

CREATE TABLE events (
    id              TEXT PRIMARY KEY,
    kb_id           TEXT NOT NULL REFERENCES knowledge_bases(id) ON DELETE CASCADE,
    type            TEXT NOT NULL,
    class           TEXT NOT NULL CHECK (class IN ('story', 'evidence', 'bookkeeping')),
    payload         TEXT NOT NULL DEFAULT '{}',    -- JSON
    subjects        TEXT NOT NULL DEFAULT '[]',    -- JSON array of entity ids
    actor           TEXT NOT NULL,                 -- JSON Actor
    channel         TEXT NOT NULL,
    session_id      TEXT REFERENCES sessions(id),
    activity_run_id TEXT REFERENCES activity_runs(id),
    changeset_id    TEXT REFERENCES changesets(id),
    visibility      TEXT,                          -- JSON, defaults to subjects' visibility
    at              TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);
CREATE INDEX events_kb_at ON events(kb_id, at);

-- Search ----------------------------------------------------------------------

CREATE TABLE search_entries (
    id          TEXT PRIMARY KEY,
    kb_id       TEXT NOT NULL REFERENCES knowledge_bases(id) ON DELETE CASCADE,
    target      TEXT NOT NULL CHECK (target IN ('entity', 'entity_version', 'event', 'recap')),
    target_id   TEXT NOT NULL,
    text        TEXT NOT NULL,
    is_current  INTEGER NOT NULL DEFAULT 1,
    valid_from  TEXT NOT NULL,
    valid_to    TEXT,
    status      TEXT,
    visibility  TEXT NOT NULL DEFAULT '{}'         -- JSON visibility inputs
);
CREATE INDEX search_entries_kb_current ON search_entries(kb_id, is_current);

CREATE VIRTUAL TABLE search_fts USING fts5(
    text,
    content = 'search_entries',
    content_rowid = 'rowid',
    tokenize = 'unicode61 remove_diacritics 0'
);

CREATE TABLE embedding_jobs (
    search_entry_id TEXT NOT NULL REFERENCES search_entries(id) ON DELETE CASCADE,
    model_id        TEXT NOT NULL,
    status          TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'done', 'failed')),
    attempts        INTEGER NOT NULL DEFAULT 0,
    updated_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    PRIMARY KEY (search_entry_id, model_id)
);
