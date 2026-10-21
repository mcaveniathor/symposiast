use axum::{
    body::Body,
    http::{HeaderMap, StatusCode, Uri},
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
    Extension, Router,
};

#[derive(Clone)]
struct ResponseTag;

async fn composed() -> impl IntoResponse {
    (StatusCode::OK, [("x-example", "tuple")], Extension(ResponseTag), "Hello")
}

async fn raw() -> Response {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header("x-example", "builder")
        .body(Body::from("not found"))
        .unwrap()
}

async fn choice(uri: Uri) -> Response {
    match uri.query() {
        Some("format=html") => Html("<p>Hello</p>").into_response(),
        Some("redirect=true") => Redirect::to("/").into_response(),
        _ => "Hello".into_response(),
    }
}

fn validate(headers: &HeaderMap) -> Result<(), StatusCode> {
    if headers.contains_key("x-name") {
        Ok(())
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

async fn create(headers: HeaderMap) -> Result<StatusCode, StatusCode> {
    validate(&headers)?;
    Ok(StatusCode::CREATED)
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let app = Router::new()
        .route("/", get(composed))
        .route("/raw", get(raw))
        .route("/choice", get(choice))
        .route("/create", post(create));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, app).await
}
