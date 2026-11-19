//! Configuration from environment variables (see `.env.example`).

use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Config {
    /// Port on 127.0.0.1 (`PORT`, default 3000).
    pub port: u16,
    /// Folder for the database, local token and files (`DATA_DIR`, default `./data`).
    pub data_dir: PathBuf,
    /// Database URL (`DATABASE_URL`, default `sqlite://<data_dir>/mnemo.db`).
    pub database_url: String,
    /// Extra allowed browser origins, comma separated (`ALLOWED_ORIGINS`), e.g. the Vite dev server.
    pub extra_origins: Vec<String>,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let port = std::env::var("PORT").ok().map(|p| p.parse()).transpose()?.unwrap_or(3000);
        let data_dir = std::env::var("DATA_DIR").map(PathBuf::from).unwrap_or_else(|_| "data".into());
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| format!("sqlite://{}", data_dir.join("mnemo.db").display()));
        let extra_origins = std::env::var("ALLOWED_ORIGINS")
            .map(|v| v.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
            .unwrap_or_else(|_| vec!["http://localhost:5173".into(), "http://127.0.0.1:5173".into()]);
        Ok(Self { port, data_dir, database_url, extra_origins })
    }
}
