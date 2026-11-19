//! Mnemo HTTP server (M0: health, OpenAPI, local security).

pub mod config;
pub mod security;

use axum::{Json, Router, extract::State, middleware, routing::get};
use mnemo_store::{LocalIdentity, Store};
use serde::Serialize;
use std::sync::Arc;
use utoipa::{OpenApi, ToSchema};

/// Shared state for request handlers.
#[derive(Clone)]
pub struct AppState {
    pub store: Store,
    pub identity: LocalIdentity,
    pub security: Arc<security::LocalSecurity>,
}

#[derive(Serialize, ToSchema)]
pub struct Health {
    /// Always "ok" when the server is up.
    status: &'static str,
    /// Server version.
    version: &'static str,
    /// Database schema version this build expects.
    schema_version: u32,
}

#[derive(Serialize, ToSchema)]
pub struct WhoAmI {
    /// The local user's id.
    user_id: String,
    /// The local user's personal workspace id.
    workspace_id: String,
    /// "local" in local mode.
    auth_mode: &'static str,
}

/// Server health. Public: needs no token.
#[utoipa::path(get, path = "/api/v1/health", responses((status = 200, body = Health)))]
pub async fn health() -> Json<Health> {
    Json(Health {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
        schema_version: mnemo_core::SCHEMA_VERSION,
    })
}

/// The identity the request acts as. Requires the local token.
#[utoipa::path(
    get,
    path = "/api/v1/whoami",
    responses((status = 200, body = WhoAmI), (status = 401, description = "Missing or wrong token")),
    security(("local_token" = []))
)]
pub async fn whoami(State(state): State<AppState>) -> Json<WhoAmI> {
    Json(WhoAmI {
        user_id: state.identity.user_id.to_string(),
        workspace_id: state.identity.workspace_id.to_string(),
        auth_mode: "local",
    })
}

#[derive(OpenApi)]
#[openapi(
    info(title = "Mnemo API", description = "Structured memory for Claude agents."),
    paths(health, whoami),
    components(schemas(Health, WhoAmI)),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "local_token",
            SecurityScheme::Http(HttpBuilder::new().scheme(HttpAuthScheme::Bearer).build()),
        );
    }
}

/// Build the application router.
pub fn router(state: AppState) -> Router {
    let protected = Router::new()
        .route("/api/v1/whoami", get(whoami))
        .layer(middleware::from_fn_with_state(state.clone(), security::require_token));

    Router::new()
        .route("/api/v1/health", get(health))
        .merge(protected)
        .layer(middleware::from_fn_with_state(state.clone(), security::check_host_and_origin))
        .with_state(state)
}
