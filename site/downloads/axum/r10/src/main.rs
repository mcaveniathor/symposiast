use axum::{routing::get, Form, Router};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct Payload {
    value: String,
}

async fn echo(Form(payload): Form<Payload>) -> Form<Payload> {
    Form(payload)
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let app = Router::new().route("/form", get(echo).post(echo));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, app).await
}
