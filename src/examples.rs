//! Documentation examples are evidence candidates, not executable doctests.

use std::{collections::BTreeSet, fs, path::Path};

use anyhow::{Context, Result, ensure};
use proc_macro2::{Delimiter, TokenStream, TokenTree};
use pulldown_cmark::{CodeBlockKind, Event, Parser, Tag, TagEnd};
use serde::{Deserialize, Serialize};

use crate::model::{Analysis, Citation, Source, safe_path, sha256};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ExampleInventory {
    pub examples: Vec<DocExample>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DocExample {
    pub id: String,
    pub citation: Citation,
    /// Original fence info, including ignore, compile_fail, edition, and target annotations.
    pub info: String,
    /// Rustdoc hidden lines are preserved; this is not a generated consumer program.
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExampleCoverage {
    pub example_id: String,
    pub disposition: Disposition,
    pub recipe_ids: Vec<String>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Disposition {
    Adapted,
    Supporting,
    Skipped,
}

/// Reproducible from the frozen evidence. No macro expansion or external includes.
pub fn inventory(sources: &[Source], root: &Path) -> Result<ExampleInventory> {
    let mut inventory = ExampleInventory {
        examples: vec![],
        warnings: vec!["Example coverage is limited to collected Markdown code blocks and literal Rust doc attributes/comments. Unlabelled blocks are treated as Rust only in Rust documentation. Macros, cfg_attr, external includes, and generated documentation are not expanded; standalone example programs are still ordinary evidence.".into()],
    };
    for source in sources {
        let text = fs::read_to_string(safe_path(root, &source.path)?)?;
        let path = source.url.split(['?', '#']).next().unwrap_or(&source.url);
        if path.ends_with(".rs") || source.path.ends_with(".rs") {
            match text.parse::<TokenStream>() {
                Ok(tokens) => rust_docs(tokens, source, &mut inventory),
                Err(_) => inventory.warnings.push(format!(
                    "{}: Rust source could not be tokenized (possibly truncated); its documentation examples were not inventoried.", source.id
                )),
            }
        } else if path.ends_with(".md")
            || path.ends_with(".mdx")
            || matches!(
                source.kind.as_str(),
                "readme" | "documentation" | "changelog"
            )
        {
            let lines: Vec<_> = (1..=text.lines().count()).map(|n| (n, n)).collect();
            markdown(&text, &lines, false, source, &mut inventory.examples);
        }
    }
    Ok(inventory)
}

fn rust_info(info: &str, rustdoc: bool) -> bool {
    let tags: Vec<_> = info
        .split([',', ' ', '\t'])
        .filter(|s| !s.is_empty())
        .collect();
    if tags.is_empty() {
        return rustdoc;
    }
    tags.contains(&"rust")
        || tags.iter().all(|tag| {
            matches!(
                *tag,
                "ignore"
                    | "no_run"
                    | "should_panic"
                    | "compile_fail"
                    | "test_harness"
                    | "standalone_crate"
            ) || tag.starts_with("ignore-")
                || matches!(
                    *tag,
                    "edition2015" | "edition2018" | "edition2021" | "edition2024"
                )
                || (tag.len() == 5
                    && tag.starts_with('E')
                    && tag[1..].bytes().all(|b| b.is_ascii_digit()))
        })
}

fn markdown(
    text: &str,
    lines: &[(usize, usize)],
    rustdoc: bool,
    source: &Source,
    examples: &mut Vec<DocExample>,
) {
    let mut block: Option<DocExample> = None;
    let line_starts: Vec<_> = std::iter::once(0)
        .chain(text.match_indices('\n').map(|(i, _)| i + 1))
        .collect();
    for (event, range) in Parser::new(text).into_offset_iter() {
        match event {
            Event::Start(Tag::CodeBlock(kind)) => {
                let info = match kind {
                    CodeBlockKind::Fenced(info) if rust_info(&info, rustdoc) => info.to_string(),
                    CodeBlockKind::Indented if rustdoc => String::new(),
                    _ => continue,
                };
                let start = line_starts.partition_point(|offset| *offset <= range.start) - 1;
                let end_offset = range.end.saturating_sub(1);
                let end = line_starts.partition_point(|offset| *offset <= end_offset) - 1;
                let (Some(first), Some(last)) = (lines.get(start), lines.get(end)) else {
                    continue;
                };
                block = Some(DocExample {
                    id: String::new(),
                    citation: Citation {
                        source_id: source.id.clone(),
                        start_line: first.0,
                        end_line: last.1,
                    },
                    info,
                    code: String::new(),
                });
            }
            Event::Text(code) => {
                if let Some(block) = &mut block {
                    block.code.push_str(&code);
                }
            }
            Event::End(TagEnd::CodeBlock) => {
                if let Some(mut block) = block.take() {
                    // The ordinal distinguishes several blocks in one escaped doc attribute.
                    block.id = format!(
                        "doc-{}-{}",
                        &sha256(source.id.as_bytes())[..16],
                        examples.len() + 1
                    );
                    examples.push(block);
                }
            }
            _ => {}
        }
    }
}

fn rust_docs(tokens: TokenStream, source: &Source, inventory: &mut ExampleInventory) {
    // Iterative traversal avoids adding a recursive walk of untrusted source trees.
    let mut pending = vec![tokens];
    while let Some(tokens) = pending.pop() {
        let tokens: Vec<_> = tokens.into_iter().collect();
        let mut text = String::new();
        let mut lines = Vec::new();
        let mut index = 0;
        while index < tokens.len() {
            let mut group_index = index + 1;
            if matches!(tokens.get(group_index), Some(TokenTree::Punct(p)) if p.as_char() == '!') {
                group_index += 1;
            }
            if matches!(&tokens[index], TokenTree::Punct(p) if p.as_char() == '#')
                && let Some(TokenTree::Group(group)) = tokens.get(group_index)
                && group.delimiter() == Delimiter::Bracket
            {
                let attr: Vec<_> = group.stream().into_iter().collect();
                if matches!(attr.first(), Some(TokenTree::Ident(name)) if name == "doc")
                    && matches!(attr.get(1), Some(TokenTree::Punct(p)) if p.as_char() == '=')
                {
                    if let [_, _, TokenTree::Literal(literal)] = attr.as_slice()
                        && let Ok(value) = syn::parse_str::<syn::LitStr>(&literal.to_string())
                    {
                        let span = literal.span();
                        let raw = span.source_text().unwrap_or_default();
                        let is_comment = raw.starts_with("///")
                            || raw.starts_with("//!")
                            || raw.starts_with("/**")
                            || raw.starts_with("/*!");
                        let value = value.value();
                        let star_margin = raw.starts_with("/*")
                            && value
                                .lines()
                                .skip(1)
                                .filter(|line| !line.trim().is_empty())
                                .all(|line| line.trim_start().starts_with('*'));
                        for (offset, line) in value.split('\n').enumerate() {
                            // Conventional leading stars in block documentation are decoration.
                            let line = if star_margin {
                                line.trim_start().strip_prefix('*').unwrap_or(line)
                            } else {
                                line
                            };
                            let line = if is_comment {
                                line.strip_prefix(' ').unwrap_or(line)
                            } else {
                                line
                            };
                            text.push_str(line);
                            text.push('\n');
                            lines.push(if is_comment {
                                let line = span.start().line + offset;
                                (line, line)
                            } else {
                                // Escapes can change line counts: cite the complete literal.
                                (span.start().line, span.end().line)
                            });
                        }
                    } else {
                        markdown(&text, &lines, true, source, &mut inventory.examples);
                        text.clear();
                        lines.clear();
                        inventory.warnings.push(format!(
                            "{}:{}: nonliteral doc attribute was not expanded.",
                            source.id,
                            group.span().start().line
                        ));
                    }
                }
                index = group_index + 1;
                continue;
            }
            markdown(&text, &lines, true, source, &mut inventory.examples);
            text.clear();
            lines.clear();
            if let TokenTree::Group(group) = &tokens[index] {
                pending.push(group.stream());
            }
            index += 1;
        }
        markdown(&text, &lines, true, source, &mut inventory.examples);
    }
}

pub fn validate_coverage(inventory: Option<&ExampleInventory>, analysis: &Analysis) -> Result<()> {
    let Some(inventory) = inventory else {
        ensure!(
            analysis.example_coverage.is_empty(),
            "example coverage requires an inventory"
        );
        return Ok(());
    };
    let mut covered = BTreeSet::new();
    for coverage in &analysis.example_coverage {
        let example = inventory
            .examples
            .iter()
            .find(|example| example.id == coverage.example_id)
            .context("coverage references an unknown documentation example")?;
        ensure!(
            covered.insert(&coverage.example_id),
            "duplicate documentation example coverage"
        );
        ensure!(
            !coverage.reason.trim().is_empty(),
            "example coverage requires a reason"
        );
        ensure!(
            coverage.recipe_ids.is_empty() == (coverage.disposition == Disposition::Skipped),
            "adapted/supporting examples require recipes; skipped examples must not name recipes"
        );
        let mut recipes = BTreeSet::new();
        for id in &coverage.recipe_ids {
            ensure!(recipes.insert(id), "duplicate recipe in example coverage");
            let recipe = analysis
                .recipes
                .iter()
                .find(|recipe| &recipe.id == id)
                .context("example coverage references an unknown recipe")?;
            ensure!(
                recipe.citations.iter().any(|citation| {
                    citation.source_id == example.citation.source_id
                        && citation.start_line <= example.citation.end_line
                        && citation.end_line >= example.citation.start_line
                }),
                "recipe {id} must cite the documentation example it uses"
            );
        }
    }
    ensure!(
        covered.len() == inventory.examples.len(),
        "analysis must account for every inventoried documentation example"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn extract(filename: &str, text: &str) -> ExampleInventory {
        let root = tempfile::tempdir().unwrap();
        fs::write(root.path().join("evidence.txt"), text).unwrap();
        inventory(
            &[Source {
                id: "source".into(),
                kind: "documentation".into(),
                path: "evidence.txt".into(),
                url: format!("https://example.invalid/{filename}"),
                sha256: sha256(text.as_bytes()),
            }],
            root.path(),
        )
        .unwrap()
    }

    #[test]
    fn markdown_fences_preserve_locations_annotations_and_hidden_lines() {
        let inventory = extract(
            "README.md",
            "# Intro\n\n```rust,ignore-x86_64,edition2024\n# use demo::Parser;\nParser::new();\n```\n\n~~~compile_fail,E0308\nlet value: u8 = \"bad\";\n~~~\n\n```sh\necho ignored\n```\n\n```\nunlabelled Markdown is ambiguous\n```\n",
        );
        assert_eq!(inventory.examples.len(), 2);
        let example = &inventory.examples[0];
        assert_eq!(example.citation.start_line, 3);
        assert_eq!(example.citation.end_line, 6);
        assert_eq!(example.info, "rust,ignore-x86_64,edition2024");
        assert_eq!(example.code, "# use demo::Parser;\nParser::new();\n");
        assert_eq!(inventory.examples[1].info, "compile_fail,E0308");
    }

    #[test]
    fn rust_docs_include_line_block_and_literal_attributes_but_not_strings() {
        let text = r####"//! Crate docs.
//!
//! ```
//! # let hidden = 1;
//! assert_eq!(hidden, 1);
//! ```
const TEXT: &str = r###"
/// ```rust
/// fake();
/// ```
"###;
// ```rust
// ordinary_comment();
// ```
/**
 * ```no_run
 * block_example();
 * ```
 */
pub struct Thing;
#[doc = "```rust\nattribute_example();\n```"]
pub fn item() {}
"####;
        let inventory = extract("src/lib.rs", text);
        assert_eq!(inventory.examples.len(), 3, "{inventory:#?}");
        let first = &inventory.examples[0];
        assert_eq!((first.citation.start_line, first.citation.end_line), (3, 6));
        assert!(first.code.contains("# let hidden = 1;"));
        let block = &inventory.examples[1];
        assert_eq!(
            (block.citation.start_line, block.citation.end_line),
            (16, 18)
        );
        assert_eq!(block.info, "no_run");
        assert!(block.code.contains("block_example();"));
        let attr = &inventory.examples[2];
        assert_eq!((attr.citation.start_line, attr.citation.end_line), (21, 21));
        assert_eq!(attr.code, "attribute_example();\n");
    }

    #[test]
    fn markdown_lists_and_quotes_keep_source_lines() {
        let inventory = extract(
            "guide.md",
            "- Example:\n\n  ```rust\n  list_example();\n  ```\n\n> ```rust\n> quote_example();\n> ```\n",
        );
        assert_eq!(inventory.examples.len(), 2);
        assert_eq!(inventory.examples[0].code, "list_example();\n");
        assert_eq!(inventory.examples[0].citation.start_line, 3);
        assert_eq!(inventory.examples[1].code, "quote_example();\n");
        assert_eq!(inventory.examples[1].citation.start_line, 7);
    }

    #[test]
    fn incomplete_rust_and_external_docs_report_omissions() {
        let inventory = extract(
            "lib.rs",
            "/// ```rust\n/// example();\n/// ```\npub fn truncated() {",
        );
        assert!(inventory.examples.is_empty());
        assert!(
            inventory
                .warnings
                .iter()
                .any(|warning| warning.contains("could not be tokenized"))
        );
        let inventory = extract(
            "lib.rs",
            "#![doc = include_str!(\"outside.md\")]\npub struct Item;",
        );
        assert!(inventory.examples.is_empty());
        assert!(
            inventory
                .warnings
                .iter()
                .any(|warning| warning.contains("nonliteral doc attribute"))
        );
    }

    #[test]
    fn rustdoc_indented_blocks_and_distinct_items_are_separate() {
        let inventory = extract(
            "lib.rs",
            "/// Example:\n///\n///     indented();\npub struct First;\n/// ```rust\n/// second();\n/// ```\npub struct Second;",
        );
        assert_eq!(inventory.examples.len(), 2);
        assert_eq!(inventory.examples[0].code, "indented();\n");
        assert_ne!(inventory.examples[0].id, inventory.examples[1].id);
    }
}
