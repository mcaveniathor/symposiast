use axum::{extract::Request, routing::get, Router, ServiceExt};
use tower::{util::MapRequestLayer, Layer};

fn rewrite(mut request: Request) -> Request {
    if request.uri().path() == "/legacy" {
        let target = match request.uri().query() {
            Some(query) => format!("/?{query}"),
            None => "/".to_owned(),
        };
        *request.uri_mut() = target.parse().expect("valid path with the existing query");
    }
    request
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let app: Router = Router::new().route("/", get(|| async { "Current endpoint" }));
    let wrapped = MapRequestLayer::new(rewrite).layer(app);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, wrapped.into_make_service()).await
}
