use axum::{extract::Extension, routing::get, Router};
use tower::{limit::ConcurrencyLimitLayer, ServiceBuilder};
use tower_http::trace::TraceLayer;

#[derive(Clone)]
struct AppTag(&'static str);

async fn handler(Extension(tag): Extension<AppTag>) -> String {
    format!("Application: {}", tag.0)
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing_subscriber::filter::LevelFilter::DEBUG)
        .init();
    let app = Router::new()
        .route("/", get(handler).layer(ConcurrencyLimitLayer::new(64)))
        .layer(ServiceBuilder::new()
            .layer(TraceLayer::new_for_http())
            .layer(Extension(AppTag("example"))));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, app).await
}
