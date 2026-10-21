use axum::{
    Router,
    body::{Body, to_bytes},
    extract::Query,
    http::{
        Request, StatusCode,
        header::{CACHE_CONTROL, CONTENT_TYPE, VARY},
    },
    response::Response,
    routing::get,
};
use std::collections::HashMap;
use stucco::prelude::*;
use tower::ServiceExt;

fn greeting(name: &str) -> impl Render {
    el::div()
        .id("greeting")
        .child(Text::new(format!("Hello, {name}!")))
}

async fn index(page: PageCx, Query(params): Query<HashMap<String, String>>) -> Response {
    let name = params.get("q").map(String::as_str).unwrap_or("visitor");
    page.respond()
        .fragment("greeting", || greeting(name))
        .page(|| {
            page.title("Greeting").main(Stack::of((
                Heading::new(1, "Greeting"),
                Form::get("/").child(Stack::of((
                    Field::new("Name", Input::text("q").value(name)),
                    Button::new("Greet").submit(),
                ))),
                greeting(name),
            )))
        })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bundle = Bundle::new(Preset::Slate);
    let stylesheet_url = bundle.stylesheet_url().to_owned();
    let stylesheet = bundle
        .get(&stylesheet_url)
        .expect("theme stylesheet exists");
    // Call stucco last to attach assets, middleware, and PageCx to all routes.
    let app = Router::new().route("/", get(index)).stucco(bundle);
    for target in [None, Some("greeting"), Some("unknown")] {
        let mut request = Request::builder().uri("/?q=%3CAda%3E");
        if let Some(target) = target {
            request = request
                .header("Stucco-Request", "fragment")
                .header("Stucco-Target", target);
        }
        let response = app.clone().oneshot(request.body(Body::empty())?).await?;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()[CONTENT_TYPE], "text/html; charset=utf-8");
        assert_eq!(response.headers()[VARY], "Stucco-Request, Stucco-Target");
        if target == Some("greeting") {
            assert_eq!(response.headers()[CACHE_CONTROL], "no-store");
        }
        let html = String::from_utf8(to_bytes(response.into_body(), 1024 * 1024).await?.to_vec())?;
        assert!(html.contains("Hello, &lt;Ada&gt;!"));
        assert_eq!(html.contains("<html"), target != Some("greeting"));
        assert_eq!(html.contains("<form"), target != Some("greeting"));
    }
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(&stylesheet_url)
                .body(Body::empty())?,
        )
        .await?;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[CONTENT_TYPE], "text/css; charset=utf-8");
    assert!(
        response.headers()[CACHE_CONTROL]
            .to_str()?
            .contains("immutable")
    );
    assert_eq!(
        to_bytes(response.into_body(), 1024 * 1024).await?.as_ref(),
        stylesheet.bytes.as_ref()
    );
    println!("Verified full pages, fragments, fallback, escaping, and stylesheet bytes.");
    if std::env::args().any(|argument| argument == "--serve") {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
        axum::serve(listener, app).await?;
    }
    Ok(())
}
