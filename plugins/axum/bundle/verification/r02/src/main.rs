use axum::{extract::Path, http::{Method, StatusCode, Uri}, routing::{get, post}, Router};

async fn wildcard(Path(rest): Path<String>) -> String {
    rest
}

async fn outer_fallback() -> (StatusCode, &'static str) {
    (StatusCode::NOT_FOUND, "outer fallback")
}

async fn method_fallback(method: Method, uri: Uri) -> (StatusCode, String) {
    (StatusCode::METHOD_NOT_ALLOWED, format!("{method} is not allowed for {uri}"))
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let api = Router::new()
        .route("/users", get(|| async { "users" }))
        .fallback(|| async { (StatusCode::NOT_FOUND, "API fallback") });
    let shared = Router::new().route("/users", get(|| async { "shared users" }));
    let methods = get(|| async { "GET" })
        .merge(post(|| async { "POST" }))
        .fallback(method_fallback);
    let app = Router::new()
        .nest("/api", api)
        .nest("/shared", shared)
        .route("/foo/{*rest}", get(wildcard))
        .route("/foo/bar", get(|| async { "static route" }))
        .route("/foo", get(|| async { "bare prefix" }))
        .route("/foo/", get(|| async { "trailing slash" }))
        .route("/methods", methods)
        .fallback(outer_fallback);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, app).await
}
