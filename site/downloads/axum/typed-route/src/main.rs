use axum::{
    body::{to_bytes, Body},
    extract::Path,
    http::{Request, StatusCode},
    routing::get,
    Router,
};
use tower::ServiceExt;

async fn user(Path(id): Path<u64>) -> String {
    format!("user:{id}")
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let app = Router::new().route("/users/{id}", get(user));
    let request = Request::builder().uri("/users/42").body(Body::empty()).unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024).await.unwrap();
    assert_eq!(body.as_ref(), b"user:42");
}
