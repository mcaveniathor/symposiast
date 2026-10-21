use std::{
    fs,
    path::{Component, Path, PathBuf},
};
use stucco::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("stucco-example"));
    let bundle = Bundle::new(Preset::Slate);
    let html = Page::new(&bundle, "Hello, Stucco")
        .main(
            Container::new().child(
                Stack::new()
                    .child(Heading::new(1, "Hello, Stucco"))
                    .child(Text::new("A <careful> welcome & a fresh start.")),
            ),
        )
        .render();
    assert!(html.contains(r#"<main id="main">"#));
    assert!(html.contains(r##"href="#main""##));
    assert!(html.contains("A &lt;careful&gt; welcome &amp; a fresh start."));
    assert!(html.contains(bundle.stylesheet_url()));
    for path in bundle.paths() {
        let relative = Path::new(path.trim_start_matches('/'));
        assert!(
            relative
                .components()
                .all(|part| matches!(part, Component::Normal(_)))
        );
        let destination = output.join(relative);
        fs::create_dir_all(destination.parent().expect("asset has a parent"))?;
        let asset = bundle.get(&path).expect("listed asset exists");
        fs::write(&destination, &*asset.bytes)?;
        assert_eq!(fs::read(&destination)?, asset.bytes.as_ref());
    }
    fs::write(output.join("index.html"), &html)?;
    assert_eq!(fs::read_to_string(output.join("index.html"))?, html);
    println!(
        "Wrote {} with matching theme assets; serve this directory at /.",
        output.display()
    );
    Ok(())
}
