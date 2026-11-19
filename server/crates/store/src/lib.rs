//! Storage for Mnemo. M0 provides the SQLite connection, migrations and the local user.

use mnemo_core::{UserId, WorkspaceId};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions};
use std::str::FromStr;

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),
    #[error("migration error: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),
    #[error("invalid stored id: {0}")]
    BadId(#[from] uuid::Error),
}

pub type Result<T> = std::result::Result<T, StoreError>;

/// Handle to the database.
#[derive(Clone, Debug)]
pub struct Store {
    pool: SqlitePool,
}

/// The single user and personal workspace used in local mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LocalIdentity {
    pub user_id: UserId,
    pub workspace_id: WorkspaceId,
}

impl Store {
    /// Open (creating if needed) a SQLite database and apply migrations.
    /// `url` is e.g. `sqlite://data/mnemo.db` or `sqlite::memory:`.
    pub async fn open(url: &str) -> Result<Self> {
        let options = SqliteConnectOptions::from_str(url)?
            .create_if_missing(true)
            .foreign_keys(true)
            .journal_mode(SqliteJournalMode::Wal);
        // In-memory databases are per connection, so keep a single one.
        let max = if url.contains(":memory:") { 1 } else { 8 };
        let pool = SqlitePoolOptions::new().max_connections(max).connect_with(options).await?;
        sqlx::migrate!("./migrations").run(&pool).await?;
        Ok(Self { pool })
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Return the local user and workspace, creating them on first run.
    pub async fn ensure_local_identity(&self) -> Result<LocalIdentity> {
        let mut tx = self.pool.begin().await?;
        let existing: Option<(String, String)> = sqlx::query_as(
            "SELECT m.user_id, m.workspace_id FROM memberships m
             JOIN workspaces w ON w.id = m.workspace_id
             WHERE w.kind = 'personal' AND m.workspace_role = 'owner'
             ORDER BY m.created_at LIMIT 1",
        )
        .fetch_optional(&mut *tx)
        .await?;

        let identity = match existing {
            Some((user, workspace)) => {
                LocalIdentity { user_id: user.parse()?, workspace_id: workspace.parse()? }
            }
            None => {
                let identity = LocalIdentity { user_id: UserId::new(), workspace_id: WorkspaceId::new() };
                sqlx::query("INSERT INTO users (id, display_name) VALUES (?, 'Local user')")
                    .bind(identity.user_id.to_string())
                    .execute(&mut *tx)
                    .await?;
                sqlx::query("INSERT INTO workspaces (id, name, kind) VALUES (?, 'My space', 'personal')")
                    .bind(identity.workspace_id.to_string())
                    .execute(&mut *tx)
                    .await?;
                sqlx::query(
                    "INSERT INTO memberships (user_id, workspace_id, workspace_role) VALUES (?, ?, 'owner')",
                )
                .bind(identity.user_id.to_string())
                .bind(identity.workspace_id.to_string())
                .execute(&mut *tx)
                .await?;
                identity
            }
        };
        tx.commit().await?;
        Ok(identity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn migrations_create_the_full_schema() {
        let store = Store::open("sqlite::memory:").await.unwrap();
        let tables: Vec<(String,)> =
            sqlx::query_as("SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name")
                .fetch_all(store.pool())
                .await
                .unwrap();
        let names: Vec<&str> = tables.iter().map(|t| t.0.as_str()).collect();
        for expected in [
            "users",
            "workspaces",
            "memberships",
            "kits",
            "kit_versions",
            "libraries",
            "library_records",
            "knowledge_bases",
            "kb_members",
            "agent_connections",
            "entities",
            "relations",
            "engine_state",
            "overrides",
            "sessions",
            "briefings",
            "recaps",
            "assignments",
            "activity_runs",
            "changesets",
            "changes",
            "events",
            "search_entries",
            "search_fts",
            "embedding_jobs",
        ] {
            assert!(names.contains(&expected), "missing table {expected}");
        }
    }

    #[tokio::test]
    async fn local_identity_is_created_once() {
        let store = Store::open("sqlite::memory:").await.unwrap();
        let first = store.ensure_local_identity().await.unwrap();
        let second = store.ensure_local_identity().await.unwrap();
        assert_eq!(first, second);
    }
}
