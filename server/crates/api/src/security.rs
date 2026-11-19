//! Local-mode security (PLAN "MCP interface", DOMAIN_MODEL "Identity & access").
//!
//! - The server binds to 127.0.0.1 only (see `main.rs`).
//! - `Host` must be localhost: blocks DNS-rebinding attacks.
//! - `Origin`, when present, must be an allowed local origin: blocks cross-site browser calls.
//! - Protected routes need `Authorization: Bearer <per-install token>`.

use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use rand::RngCore;
use std::path::Path;

use crate::AppState;

pub struct LocalSecurity {
    token: String,
    allowed_hosts: Vec<String>,
    allowed_origins: Vec<String>,
}

impl LocalSecurity {
    pub fn new(token: String, port: u16, extra_origins: &[String]) -> Self {
        let allowed_hosts = vec![format!("localhost:{port}"), format!("127.0.0.1:{port}")];
        let mut allowed_origins: Vec<String> = allowed_hosts.iter().map(|h| format!("http://{h}")).collect();
        allowed_origins.extend(extra_origins.iter().cloned());
        Self { token, allowed_hosts, allowed_origins }
    }

    pub fn host_allowed(&self, host: Option<&str>) -> bool {
        host.is_some_and(|h| self.allowed_hosts.iter().any(|a| a == h))
    }

    pub fn origin_allowed(&self, origin: Option<&str>) -> bool {
        origin.is_none_or(|o| self.allowed_origins.iter().any(|a| a == o))
    }

    pub fn token_valid(&self, headers: &HeaderMap) -> bool {
        let Some(value) = headers.get(header::AUTHORIZATION).and_then(|v| v.to_str().ok()) else {
            return false;
        };
        let Some(given) = value.strip_prefix("Bearer ") else { return false };
        constant_time_eq(given.as_bytes(), self.token.as_bytes())
    }
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

/// Read the per-install token from `<data_dir>/local_token`, creating it on first run.
pub fn load_or_create_token(data_dir: &Path) -> std::io::Result<String> {
    let path = data_dir.join("local_token");
    if let Ok(existing) = std::fs::read_to_string(&path) {
        let trimmed = existing.trim();
        if !trimmed.is_empty() {
            return Ok(trimmed.to_string());
        }
    }
    std::fs::create_dir_all(data_dir)?;
    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    let token: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    std::fs::write(&path, &token)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
    }
    Ok(token)
}

pub async fn check_host_and_origin(State(state): State<AppState>, request: Request, next: Next) -> Response {
    let headers = request.headers();
    let host = headers.get(header::HOST).and_then(|v| v.to_str().ok());
    let origin = headers.get(header::ORIGIN).and_then(|v| v.to_str().ok());
    if !state.security.host_allowed(host) {
        return (StatusCode::FORBIDDEN, "host not allowed").into_response();
    }
    if !state.security.origin_allowed(origin) {
        return (StatusCode::FORBIDDEN, "origin not allowed").into_response();
    }
    next.run(request).await
}

pub async fn require_token(State(state): State<AppState>, request: Request, next: Next) -> Response {
    if !state.security.token_valid(request.headers()) {
        return (StatusCode::UNAUTHORIZED, "missing or invalid local token").into_response();
    }
    next.run(request).await
}
