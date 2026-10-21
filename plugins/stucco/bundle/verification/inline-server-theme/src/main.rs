use stucco::theme::{Preset, Scheme};
use stucco::{Bundle, Delivery, Page, el};

fn main() {
    let bundle = Bundle::new(Preset::Slate);
    let html = Page::new(&bundle, "Account")
        .server_theme(Some(Scheme::Dark))
        .delivery(Delivery::Inline)
        .body(el::main().child(el::h1().text("Account")))
        .render();
    assert!(html.contains("data-theme=\"dark\""));
    assert!(html.contains("<style"));
    assert!(!html.contains("rel=\"stylesheet\""));
    assert!(html.contains("<h1>Account</h1>"));
    println!("{html}");
}
