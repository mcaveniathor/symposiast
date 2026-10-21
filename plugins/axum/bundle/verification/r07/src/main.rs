use axum::{
    extract::{Extension, Request},
    http::{header::AUTHORIZATION, StatusCode},
    middleware::{self, Next},
    response::Response,
    routing::get,
    Router,
};

#[derive(Clone)]
struct CurrentUser {
    name: String,
}

async fn auth(mut req: Request, next: Next) -> Result<Response, StatusCode> {
    // Demonstration only: replace this fixed token check with real authentication.
    let authorized = req.headers().get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        == Some("Bearer demo-token");
    if !authorized {
        return Err(StatusCode::UNAUTHORIZED);
    }
    req.extensions_mut().insert(CurrentUser { name: "demo".to_owned() });
    Ok(next.run(req).await)
}

async fn profile(Extension(user): Extension<CurrentUser>) -> String {
    format!("Hello, {}", user.name)
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let app = Router::new()
        .route("/profile", get(profile))
        .route_layer(middleware::from_fn(auth));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, app).await
}
