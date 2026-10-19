use stucco::actions::ButtonLink;
use stucco::data::Card;
use stucco::layout::{Cluster, Grid, Stack};
use stucco::{Measure, Space, Variant, el};

use crate::shell::Site;

pub(crate) fn page(site: &Site) -> String {
    let hero = el::section().class("hero").aria("labelledby", "hero-title").child(
        el::div().class("hero-copy")
            .child(el::p().class("eyebrow").child(el::span().class("status-dot").aria("hidden", "true")).text("RUST CRATES. RELEASE-SPECIFIC KNOWLEDGE."))
            .child(el::h1().id("hero-title").text("Guidance for the ").child(el::em().text("release")).text(" you use."))
            .child(el::p().class("hero-description").text("Turn a Rust crate’s source, changelog, documentation, and examples into skills, MCP retrieval, and version checks for Symposium."))
            .child(Cluster::new().space(Space::S3).class("hero-actions")
                .child(ButtonLink::new("Get started →", site.url("getting-started.html")).variant(Variant::Primary).class("button-primary"))
                .child(ButtonLink::new("Explore examples", site.url("examples.html")).class("button-secondary")))
            .child(el::p().class("hero-footnote").text("Bring Claude Code, Codex, or your own agent."))
            .child(el::p().class("built-with").text("This site is built with ")
                .child(el::a().href("https://github.com/mcaveniathor/stucco").text("Stucco"))
                .text(". ")
                .child(el::a().href(site.url("examples.html#stucco-021")).text("Explore its plugin and examples →"))),
    ).child(
        el::div().class("release-note").aria("label", "Example release dossier")
            .child(el::div().class("note-topline").child(el::span().text("RELEASE DOSSIER")).child(el::span().text("001")))
            .child(el::h2().text("clap").child(el::span().text("4.6.7")))
            .child(el::p().class("note-subtitle").text("A snapshot you can trace."))
            .child(el::dl().class("dossier-facts")
                .child(el::div().child(el::dt().text("Evidence")).child(el::dd().text("Published crate + pinned repository")))
                .child(el::div().child(el::dt().text("Guidance")).child(el::dd().text("Citations to source lines")))
                .child(el::div().child(el::dt().text("Recipes")).child(el::dd().text("3 compiled examples")))
                .child(el::div().child(el::dt().text("Coverage")).child(el::dd().text("Exact release, explicit features"))))
            .child(el::div().class("note-stamp").child(el::span().aria("hidden", "true").text("✓")).text(" VALIDATED PILOT")),
    );

    let terminal = el::section().class("quickstart").aria("labelledby", "quickstart-title")
        .child(el::div().class("section-intro")
            .child(el::p().class("eyebrow").text("FROM CRATE TO PLUGIN"))
            .child(el::h2().id("quickstart-title").text("One crate. A useful starting point."))
            .child(el::p().text("Resolve the latest stable release, collect its evidence, and generate a plugin with your configured agent."))
            .child(el::a().class("text-link").href(site.url("getting-started.html")).text("Read the installation guide →")))
        .child(el::div().class("terminal")
            .child(el::div().class("terminal-heading").child(el::span().text("TERMINAL")).child(el::span().text("claude-code / codex / custom")))
            .child(el::pre().attr("tabindex", "0").child(el::code().text("symposiast build clap \\\n  --agent claude-code \\\n  --output .symposiast/clap\n\nsymposiast install .symposiast/clap")))
            .child(el::p().class("terminal-note").text("Use --agent codex to run with Codex. Agent authentication is managed by its CLI.")));

    let artifacts = el::section().class("section artifacts").aria("labelledby", "artifacts-title")
        .child(el::div().class("section-heading").child(el::p().class("eyebrow").text("THREE WAYS TO USE THE KNOWLEDGE"))
            .child(el::h2().id("artifacts-title").text("Built for your development loop.")))
        .child(Grid::new().min(Measure::Xs).space(Space::S4).class("artifact-grid").children([
            ("01", "Skills", "Release-specific guidance", "Idioms, feature choices, and complete recipes with citations back to the collected evidence.", "getting-started.html#inspect-the-result"),
            ("02", "MCP retrieval", "Answers close to the code", "Search guidance and retrieve recipes from a local server. Every response carries the release context.", "reference.html#mcp-retrieval"),
            ("03", "Version checks", "A visible boundary", "A session hook checks Cargo.lock and reports when the plugin’s release differs from the project’s dependency.", "reference.html#version-checks-and-hooks"),
        ].map(|(number, label, title, description, path)| {
            Card::new().class("artifact-card").child(Stack::new().space(Space::S3)
                .child(el::p().class("card-label").child(el::span().text(number)).text(label))
                .child(el::h3().text(title))
                .child(el::p().text(description))
                .child(el::a().class("text-link").href(site.url(path)).text(format!("About {} →", label.to_lowercase()))))
        })));

    let process = el::section().class("section process").aria("labelledby", "process-title")
        .child(el::div().class("section-heading")
            .child(el::p().class("eyebrow").text("FOLLOW THE EVIDENCE"))
            .child(el::h2().id("process-title").text("A chain you can inspect."))
            .child(el::p().text("Keep the source material, the analysis, and the checks together. Inspect each step before putting the plugin to work.")))
        .child(el::ol().class("process-list").children([
            ("Collect", "Resolve an exact release. Verify the archive checksum and gather sources from its published crate and pinned repository."),
            ("Analyze", "Give your agent a frozen evidence bundle. Record findings, recipes, features, and source-line citations."),
            ("Verify", "Compile recipe projects against the exact crate version. Preserve lockfiles and check results for inspection."),
            ("Install", "Package the guidance for Symposium, with local retrieval and a hook that checks the project’s version."),
        ].into_iter().enumerate().map(|(i, (title, description))| {
            el::li().child(el::span().class("step-number").text(format!("0{}", i + 1)))
                .child(el::h3().text(title)).child(el::p().text(description))
        })));

    let pilots = el::section().class("section pilots").aria("labelledby", "pilots-title")
        .child(el::div().class("section-intro")
            .child(el::p().class("eyebrow").text("START WITH REAL EXAMPLES"))
            .child(el::h2().id("pilots-title").text("Small recipes. Concrete checks."))
            .child(el::p().text("The pilots use imported analyses. Recipes compile against their exact releases, with additional parser, request, and rendering checks."))
            .child(el::a().class("text-link").href(site.url("examples.html")).text("Explore the checked recipes →")))
        .child(el::div().class("pilot-list")
            .child(pilot(site, "clap", "4.6.7", "Parsers, subcommands, validation", "Validated", "examples.html#clap-467"))
            .child(pilot(site, "axum", "0.8.9", "Routes, shared state, typed JSON", "Validated", "examples.html#axum-089"))
            .child(pilot(site, "stucco", "0.2.1", "Static pages, themed assets, Axum", "Validated", "examples.html#stucco-021"))
            .child(el::div().class("pilot-row planned")
                .child(el::div().child(el::h3().text("bevy")).child(el::p().text("Game and ECS workflows")))
                .child(el::span().class("status-label").text("Planned"))));

    site.page("Symposiast", "Generate release-specific Rust crate skills, MCP retrieval, and version checks for the Symposium ecosystem.", "home", (hero, terminal, artifacts, process, pilots))
}

fn pilot(
    site: &Site,
    name: &str,
    version: &str,
    description: &str,
    status: &str,
    path: &str,
) -> impl stucco::Render + 'static {
    el::a()
        .class("pilot-row")
        .href(site.url(path))
        .child(
            el::div()
                .child(el::h3().text(name).child(el::span().text(version)))
                .child(el::p().text(description)),
        )
        .child(el::span().class("status-label").text(status))
}
