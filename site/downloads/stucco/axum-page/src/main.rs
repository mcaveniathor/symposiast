use axum::{
    Router,
    body::{Body, to_bytes},
    http::{
        Request, StatusCode,
        header::{CACHE_CONTROL, CONTENT_TYPE},
    },
    routing::get,
};
use stucco::prelude::*;
use tower::ServiceExt;

async fn index(page: PageCx) -> Document {
    page.title("Hello, Stucco").main(
        Container::new().child(
            Stack::new()
                .child(Heading::new(1, "Hello, Stucco"))
                .child(Text::new("Rust <HTML> & a real stylesheet.")),
        ),
    )
}

fn request(uri: &str) -> Request<Body> {
    Request::builder().uri(uri).body(Body::empty()).unwrap()
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bundle = Bundle::new(Preset::Slate);
    let stylesheet_url = bundle.stylesheet_url().to_owned();
    let stylesheet = bundle
        .get(&stylesheet_url)
        .expect("theme stylesheet exists");
    // Call stucco last so it provides assets, middleware and PageCx to every route.
    let app = Router::new().route("/", get(index)).stucco(bundle);

    let response = app.clone().oneshot(request("/")).await?;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[CONTENT_TYPE], "text/html; charset=utf-8");
    let html = String::from_utf8(to_bytes(response.into_body(), 1024 * 1024).await?.to_vec())?;
    assert!(html.contains(r#"<main id="main">"#));
    assert!(html.contains(r##"href="#main""##));
    assert!(html.contains("Rust &lt;HTML&gt; &amp; a real stylesheet."));
    assert!(html.contains(&stylesheet_url));

    let response = app.oneshot(request(&stylesheet_url)).await?;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[CONTENT_TYPE], "text/css; charset=utf-8");
    assert!(
        response.headers()[CACHE_CONTROL]
            .to_str()?
            .contains("immutable")
    );
    let css = to_bytes(response.into_body(), 1024 * 1024).await?;
    assert_eq!(css.as_ref(), stylesheet.bytes.as_ref());
    println!("Verified HTML and {stylesheet_url} through in-memory requests.");
    Ok(())
}
