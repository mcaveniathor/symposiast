use std::sync::Arc;
use axum::{
    body::{to_bytes, Body},
    extract::State,
    http::{header::CONTENT_TYPE, Request, StatusCode},
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tower::ServiceExt;

struct AppState { prefix: String }

#[derive(Deserialize)]
struct Input { name: String }

#[derive(Serialize)]
struct Output { greeting: String }

async fn greet(
    State(state): State<Arc<AppState>>,
    Json(input): Json<Input>,
) -> (StatusCode, Json<Output>) {
    (StatusCode::CREATED, Json(Output {
        greeting: format!("{}, {}", state.prefix, input.name),
    }))
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let state = Arc::new(AppState { prefix: "Hello".into() });
    let app = Router::new().route("/greet", post(greet)).with_state(state);
    let request = Request::builder()
        .method("POST")
        .uri("/greet")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"name":"Ada"}"#))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    assert_eq!(response.headers()[CONTENT_TYPE], "application/json");
    let body = to_bytes(response.into_body(), 1024).await.unwrap();
    assert_eq!(body.as_ref(), br#"{"greeting":"Hello, Ada"}"#);
}
