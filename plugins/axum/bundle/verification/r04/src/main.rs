use axum::{extract::{Path, Query}, routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
struct Pagination {
    page: usize,
    per_page: usize,
}

#[derive(Serialize)]
struct User {
    id: Uuid,
    username: String,
}

#[derive(Serialize)]
struct Page {
    user_id: Uuid,
    page: usize,
    per_page: usize,
}

async fn user(Path(id): Path<Uuid>) -> Json<User> {
    // Illustrative lookup result.
    Json(User { id, username: "example".to_owned() })
}

async fn things(Path(user_id): Path<Uuid>, Query(p): Query<Pagination>) -> Json<Page> {
    Json(Page { user_id, page: p.page, per_page: p.per_page })
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let app = Router::new()
        .route("/users/{id}", get(user))
        .route("/users/{id}/things", get(things));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, app).await
}
