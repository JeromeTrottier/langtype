use axum::{body::Body, http::Request};
use http_body_util::BodyExt;
use mnemo_api::{AppState, router, security::LocalSecurity};
use mnemo_store::Store;
use std::sync::Arc;
use tower::ServiceExt;

const TOKEN: &str = "test-token";

async fn app() -> axum::Router {
    let store = Store::open("sqlite::memory:").await.unwrap();
    let identity = store.ensure_local_identity().await.unwrap();
    let security = Arc::new(LocalSecurity::new(TOKEN.into(), 3000, &["http://localhost:5173".into()]));
    router(AppState { store, identity, security })
}

fn get(path: &str) -> axum::http::request::Builder {
    Request::get(path).header("host", "localhost:3000")
}

#[tokio::test]
async fn health_is_public() {
    let res = app().await.oneshot(get("/api/v1/health").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(res.status(), 200);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "ok");
}

#[tokio::test]
async fn foreign_host_is_rejected() {
    let req = Request::get("/api/v1/health").header("host", "evil.example:3000").body(Body::empty()).unwrap();
    assert_eq!(app().await.oneshot(req).await.unwrap().status(), 403);
}

#[tokio::test]
async fn foreign_origin_is_rejected() {
    let req = get("/api/v1/health").header("origin", "https://evil.example").body(Body::empty()).unwrap();
    assert_eq!(app().await.oneshot(req).await.unwrap().status(), 403);
}

#[tokio::test]
async fn dev_server_origin_is_allowed() {
    let req = get("/api/v1/health").header("origin", "http://localhost:5173").body(Body::empty()).unwrap();
    assert_eq!(app().await.oneshot(req).await.unwrap().status(), 200);
}

#[tokio::test]
async fn protected_route_needs_the_token() {
    let missing = get("/api/v1/whoami").body(Body::empty()).unwrap();
    assert_eq!(app().await.oneshot(missing).await.unwrap().status(), 401);

    let wrong = get("/api/v1/whoami").header("authorization", "Bearer nope").body(Body::empty()).unwrap();
    assert_eq!(app().await.oneshot(wrong).await.unwrap().status(), 401);

    let ok =
        get("/api/v1/whoami").header("authorization", format!("Bearer {TOKEN}")).body(Body::empty()).unwrap();
    assert_eq!(app().await.oneshot(ok).await.unwrap().status(), 200);
}

#[test]
fn token_is_created_once_and_reused() {
    let dir = tempfile::tempdir().unwrap();
    let first = mnemo_api::security::load_or_create_token(dir.path()).unwrap();
    let second = mnemo_api::security::load_or_create_token(dir.path()).unwrap();
    assert_eq!(first, second);
    assert_eq!(first.len(), 64);
}
