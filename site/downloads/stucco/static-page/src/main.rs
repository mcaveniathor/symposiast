use std::{
    fs,
    path::{Component, Path, PathBuf},
};
use stucco::prelude::*;

// Match the site's URL mount point; use "/" for a site served at the root.
const BASE_PATH: &str = "/demo/";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("stucco-example"));
    let bundle = Bundle::new(Preset::Slate).prefix(&format!("{BASE_PATH}_stucco/"));
    let user_text = "A <careful> welcome & a fresh start.";
    let html = Page::new(&bundle, "Built with Stucco")
        .main(
            Container::new().child(
                Stack::new()
                    .child(Heading::new(1, "Built with Stucco"))
                    .child(Text::new(user_text))
                    .child(Link::new(
                        "Stucco repository",
                        "https://github.com/mcaveniathor/stucco",
                    )),
            ),
        )
        .render();

    assert!(html.contains(r#"<main id="main">"#));
    assert!(html.contains(r##"href="#main""##));
    assert!(html.contains("A &lt;careful&gt; welcome &amp; a fresh start."));
    assert!(!html.contains("<careful>"));
    assert!(html.contains(bundle.stylesheet_url()));

    fs::create_dir_all(&output)?;
    let paths = bundle.paths();
    for url in &paths {
        // The deployment prefix is a URL, not a directory inside the output.
        let relative = url
            .strip_prefix(BASE_PATH)
            .expect("asset stays under site prefix");
        let relative = Path::new(relative);
        assert!(
            relative
                .components()
                .all(|part| matches!(part, Component::Normal(_)))
        );
        let destination = output.join(relative);
        fs::create_dir_all(destination.parent().expect("asset has a parent"))?;
        let asset = bundle.get(url).expect("listed asset exists");
        fs::write(&destination, &*asset.bytes)?;
        assert_eq!(fs::read(&destination)?, asset.bytes.as_ref());
    }
    fs::write(output.join("index.html"), &html)?;
    assert_eq!(fs::read_to_string(output.join("index.html"))?, html);
    assert!(!paths.is_empty());
    println!(
        "Wrote {} and {} assets; serve it at {BASE_PATH}",
        output.display(),
        paths.len()
    );
    Ok(())
}
