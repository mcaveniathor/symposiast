use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd, html};
use std::collections::HashMap;
use stucco::{Raw, el};

use crate::{DOCUMENTS, shell::Site};

pub(crate) fn page(
    site: &Site,
    slug: &str,
    title: &str,
    description: &str,
    source: &str,
) -> String {
    let (body, headings) = render(site, source);
    let contents = || {
        el::ul().children(headings.iter().map(|(level, id, label)| {
            el::li()
                .class(if *level == HeadingLevel::H3 {
                    "toc-nested"
                } else {
                    ""
                })
                .child(el::a().href(format!("#{id}")).text(label.clone()))
        }))
    };
    let toc = el::nav()
        .class("page-toc")
        .aria("label", "On this page")
        .child(el::p().class("eyebrow").text("ON THIS PAGE"))
        .child(contents());
    let mobile_toc = el::details()
        .class("mobile-toc")
        .child(el::summary().text("On this page"))
        .child(el::nav().aria("label", "On this page").child(contents()));
    let position = DOCUMENTS
        .iter()
        .position(|(id, _, _)| *id == slug)
        .unwrap_or(0);
    let next = DOCUMENTS.get(position + 1);
    let mut end = el::nav()
        .class("doc-pagination")
        .aria("label", "Documentation pages");
    if position > 0 {
        let (previous, label, _) = DOCUMENTS[position - 1];
        end = end.child(
            el::a()
                .href(site.url(&format!("{previous}.html")))
                .child(el::span().text("← PREVIOUS"))
                .child(el::strong().text(label)),
        );
    }
    if let Some((next, label, _)) = next {
        end = end.child(
            el::a()
                .class("next-page")
                .href(site.url(&format!("{next}.html")))
                .child(el::span().text("NEXT →"))
                .child(el::strong().text(*label)),
        );
    }
    site.page(title, description, slug,
        el::div().class("docs-layout")
            .child(el::div().class("docs-body")
                .child(el::p().class("eyebrow doc-eyebrow").text(format!("DOCUMENTATION / 0{}", position + 1)))
                .child(mobile_toc)
                // Content is trusted, reviewed Markdown checked into this repository.
                .child(el::article().class("prose").child(Raw::trusted(body)))
                .child(end))
            .child(el::aside().class("docs-aside").child(toc)
                .child(el::div().class("aside-note").child(el::p().class("eyebrow").text("EXACT RELEASES"))
                    .child(el::p().text("Check the version, features, and verification status before using a recipe.")))),
    )
}

fn render(site: &Site, source: &str) -> (String, Vec<(HeadingLevel, String, String)>) {
    let options = Options::ENABLE_TABLES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_HEADING_ATTRIBUTES;
    let mut events: Vec<_> = Parser::new_ext(source, options).collect();
    let mut headings = Vec::new();
    let mut used = HashMap::<String, usize>::new();
    for index in 0..events.len() {
        let Event::Start(Tag::Heading { level, .. }) = &events[index] else {
            continue;
        };
        let level = *level;
        let mut label = String::new();
        for event in events.iter().skip(index + 1) {
            match event {
                Event::End(TagEnd::Heading(_)) => break,
                Event::Text(text) | Event::Code(text) => label.push_str(text),
                Event::SoftBreak | Event::HardBreak => label.push(' '),
                _ => {}
            }
        }
        let Event::Start(Tag::Heading { id, .. }) = &mut events[index] else {
            unreachable!()
        };
        let raw_id = id
            .as_ref()
            .map(|value| value.to_string())
            .unwrap_or_else(|| slugify(&label));
        let count = used.entry(raw_id.clone()).or_default();
        let unique_id = if *count == 0 {
            raw_id
        } else {
            format!("{raw_id}-{count}")
        };
        *count += 1;
        *id = Some(unique_id.clone().into());
        if matches!(level, HeadingLevel::H2 | HeadingLevel::H3) {
            headings.push((level, unique_id, label));
        }
    }
    for event in &mut events {
        match event {
            Event::Start(Tag::Link { dest_url, .. })
            | Event::Start(Tag::Image { dest_url, .. })
                if !dest_url.starts_with('#')
                    && !dest_url.contains(':')
                    && !dest_url.starts_with("//") =>
            {
                let value = dest_url.strip_prefix('/').unwrap_or(dest_url);
                let (path, fragment) = value
                    .split_once('#')
                    .map_or((value, None), |(path, fragment)| (path, Some(fragment)));
                let path = path
                    .strip_suffix(".md")
                    .map_or_else(|| path.to_owned(), |stem| format!("{stem}.html"));
                let destination = match fragment {
                    Some(fragment) => format!("{path}#{fragment}"),
                    None => path,
                };
                *dest_url = site.url(&destination).into();
            }
            _ => {}
        }
    }
    let mut body = String::new();
    html::push_html(&mut body, events.into_iter());
    // Keyboard readers can scroll long examples even when JavaScript is off.
    body = body.replace("<pre>", "<pre tabindex=\"0\">");
    (body, headings)
}

fn slugify(value: &str) -> String {
    let mut slug = String::new();
    for character in value.to_lowercase().chars() {
        if character.is_alphanumeric() || character == '_' || character == '-' {
            slug.push(character);
        } else if character.is_whitespace() && !slug.ends_with('-') && !slug.is_empty() {
            slug.push('-');
        }
    }
    let slug = slug.trim_end_matches('-');
    if slug.is_empty() {
        "section".into()
    } else {
        slug.into()
    }
}
