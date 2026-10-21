use axum::{extract::rejection::JsonRejection, http::{HeaderMap, StatusCode}, routing::post, Json, Router};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct Payload {
    username: String,
}

#[derive(Serialize)]
struct ErrorBody {
    message: String,
}

async fn echo(
    _headers: HeaderMap,
    payload: Result<Json<Payload>, JsonRejection>,
) -> Result<Json<Payload>, (StatusCode, Json<ErrorBody>)> {
    payload.map_err(|rejection| {
        let status = rejection.status();
        let body = ErrorBody { message: rejection.body_text() };
        (status, Json(body))
    })
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let app = Router::new().route("/echo", post(echo));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, app).await
}
