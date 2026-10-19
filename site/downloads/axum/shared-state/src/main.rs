use std::sync::Arc;
use axum::{
    body::{to_bytes, Body},
    extract::State,
    http::{Request, StatusCode},
    routing::get,
    Router,
};
use tower::ServiceExt;

struct AppState {
    greeting: String,
}

async fn greeting(State(state): State<Arc<AppState>>) -> String {
    state.greeting.clone()
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let state = Arc::new(AppState { greeting: "Hello from shared state".into() });
    let app = Router::new().route("/", get(greeting)).with_state(state);
    let request = Request::builder().uri("/").body(Body::empty()).unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024).await.unwrap();
    assert_eq!(body.as_ref(), b"Hello from shared state");
}
