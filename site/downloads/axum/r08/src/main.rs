use axum::{
    error_handling::{HandleError, HandleErrorLayer},
    extract::Request,
    http::{Method, StatusCode, Uri},
    response::Response,
    routing::get,
    BoxError, Router,
};
use std::time::Duration;
use tower::ServiceBuilder;

async fn slow() -> &'static str {
    std::future::pending::<()>().await;
    "finished"
}

async fn handle_error(method: Method, uri: Uri, error: BoxError) -> (StatusCode, String) {
    let status = if error.is::<tower::timeout::error::Elapsed>() {
        StatusCode::REQUEST_TIMEOUT
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    };
    (status, format!("{method} {uri} could not be completed"))
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let fallible = tower::service_fn(|_req: Request| async {
        Err::<Response, anyhow::Error>(anyhow::anyhow!("demonstration failure"))
    });
    let app = Router::new()
        .route_service("/service", HandleError::new(fallible, |_error: anyhow::Error| async {
            (StatusCode::INTERNAL_SERVER_ERROR, "Service failed")
        }))
        .route("/slow", get(slow))
        .layer(ServiceBuilder::new()
            .layer(HandleErrorLayer::new(handle_error))
            .timeout(Duration::from_secs(1)));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, app).await
}
