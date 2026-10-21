//! Static documentation rendered with Stucco and checked-in Markdown.

mod landing;
mod markdown;
mod shell;

use std::{fs, io, path::Path};
use stucco::{Asset, el, register_asset};

use shell::Site;

/// Registered alongside Stucco's layout, action, and typography assets.
pub static SITE_STYLE: Asset = Asset {
    name: "symposiast-site",
    css: Some(include_str!("../assets/site.css")),
    behavior: None,
    deps: &[],
};
register_asset!(SITE_STYLE);

pub(crate) const DOCUMENTS: [(&str, &str, &str); 6] = [
    (
        "getting-started",
        "Getting started",
        "Use shared guidance or generate a release-specific Symposium plugin.",
    ),
    (
        "maintainers",
        "For maintainers",
        "Pilot guidance for your crate, review it with users, and maintain it across releases.",
    ),
    (
        "examples",
        "Examples",
        "Explore checked clap, axum, Bevy, and Stucco recipes, with their exact versions and features.",
    ),
    (
        "adapters",
        "Agent adapters",
        "Use Claude Code, Codex, or a custom executable to analyze crate evidence.",
    ),
    (
        "automation",
        "Automation",
        "Generate portable crate guidance in GitHub Actions after a release or on demand.",
    ),
    (
        "reference",
        "Reference",
        "Commands, bundle contents, MCP tools, version checks, and current boundaries.",
    ),
];

/// Normalize a local URL base. A project site and a root site are both supported.
pub fn normalize_base(base: &str) -> io::Result<String> {
    if !base.starts_with('/')
        || !base
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"/-_ .".contains(&c))
        || base.contains(' ')
        || base.contains("//")
        || base.split('/').any(|part| part == "." || part == "..")
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "base must be an absolute URL path, such as /symposiast/ or /",
        ));
    }
    let trimmed = base.trim_matches('/');
    Ok(if trimmed.is_empty() {
        "/".into()
    } else {
        format!("/{trimmed}/")
    })
}

/// Build all pages and assets; returned paths are relative to `out`.
pub fn build(out: &Path, base: &str) -> io::Result<Vec<String>> {
    let site = Site::new(normalize_base(base)?);
    let source = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = vec![
        ("index.html".to_owned(), landing::page(&site).into_bytes()),
        ("404.html".to_owned(), not_found(&site).into_bytes()),
        (
            "assets/favicon.svg".to_owned(),
            include_bytes!("../assets/favicon.svg").to_vec(),
        ),
        (
            "assets/site.js".to_owned(),
            include_bytes!("../assets/site.js").to_vec(),
        ),
        (".nojekyll".to_owned(), Vec::new()),
    ];
    for (slug, title, description) in DOCUMENTS {
        let content = fs::read_to_string(source.join(format!("content/{slug}.md")))?;
        files.push((
            format!("{slug}.html"),
            markdown::page(&site, slug, title, description, &content).into_bytes(),
        ));
    }
    for path in site.bundle.paths() {
        let asset = site.bundle.get(&path).expect("listed bundle asset exists");
        let local = path
            .strip_prefix(&site.base)
            .expect("bundle asset uses site base");
        files.push((local.to_owned(), asset.bytes.to_vec()));
    }
    let downloads = source.join("downloads");
    if downloads.exists() {
        collect_downloads(&downloads, "examples", &mut files)?;
    }
    let mut written = Vec::with_capacity(files.len());
    for (path, contents) in files {
        let target = out.join(&path);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(target, contents)?;
        written.push(path);
    }
    Ok(written)
}

fn collect_downloads(
    source: &Path,
    prefix: &str,
    files: &mut Vec<(String, Vec<u8>)>,
) -> io::Result<()> {
    let mut entries = fs::read_dir(source)?.collect::<io::Result<Vec<_>>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let name = entry.file_name().into_string().map_err(|_| {
            io::Error::new(io::ErrorKind::InvalidData, "download filename is not UTF-8")
        })?;
        if name == "target" || name.starts_with('.') {
            continue;
        }
        let path = format!("{prefix}/{name}");
        let kind = entry.file_type()?;
        if kind.is_dir() {
            collect_downloads(&entry.path(), &path, files)?;
        } else if kind.is_file()
            && (name == "Cargo.toml"
                || name == "Cargo.lock"
                || name == "coverage.json"
                || name.ends_with(".rs"))
        {
            files.push((path, fs::read(entry.path())?));
        } else if !kind.is_file() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "downloads must be regular files or directories",
            ));
        }
    }
    Ok(())
}

fn not_found(site: &Site) -> String {
    site.page(
        "Page not found",
        "This page could not be found. Return to the Symposiast documentation.",
        "404",
        el::section()
            .class("not-found")
            .child(el::p().class("eyebrow").text("404 / PAGE NOT FOUND"))
            .child(el::h1().text("A missing reference."))
            .child(el::p().text("The page may have moved, or the address may be incomplete."))
            .child(
                el::a()
                    .class("text-link")
                    .href(site.url(""))
                    .text("Return to Symposiast →"),
            ),
    )
}
