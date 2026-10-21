use std::fs;

use symposiast::model::*;

fn fixture(root: &std::path::Path) -> Bundle {
    fs::write(
        root.join("source.rs"),
        "/// Construct a parser.\npub struct Parser;\n",
    )
    .unwrap();
    Bundle {
        schema_version: SCHEMA_VERSION,
        example_inventory: None,
        release: Release {
            name: "clap".into(),
            version: "4.5.0".into(),
            checksum: "0".repeat(64),
            repository: None,
            repository_commit: None,
            rust_version: None,
            features: Default::default(),
            collected_at: "0".into(),
        },
        sources: vec![Source {
            id: "e0001".into(),
            kind: "rustdoc".into(),
            path: "source.rs".into(),
            url: "https://example.invalid/source.rs".into(),
            sha256: sha256(b"/// Construct a parser.\npub struct Parser;\n"),
        }],
        analysis: Some(Analysis {
            example_coverage: vec![],
            summary: "Parser guidance".into(),
            findings: vec![Finding {
                id: "parser".into(),
                title: "Construct a parser".into(),
                guidance: "Use the parser.".into(),
                basis: Basis::Documented,
                features: vec![],
                citations: vec![Citation {
                    source_id: "e0001".into(),
                    start_line: 1,
                    end_line: 2,
                }],
            }],
            recipes: vec![Recipe {
                id: "minimal".into(),
                title: "Minimal".into(),
                description: "A small consumer".into(),
                code: "fn main() {}".into(),
                default_features: true,
                features: vec![],
                dependencies: Default::default(),
                citations: vec![Citation {
                    source_id: "e0001".into(),
                    start_line: 1,
                    end_line: 1,
                }],
            }],
        }),
        review: None,
        verification: vec![],
        warnings: vec![],
    }
}

#[test]
fn roundtrip_rejects_modified_evidence() {
    let root = tempfile::tempdir().unwrap();
    fixture(root.path()).save(root.path()).unwrap();
    Bundle::load(root.path()).unwrap();
    fs::write(root.path().join("source.rs"), "different source").unwrap();
    assert!(
        Bundle::load(root.path())
            .unwrap_err()
            .to_string()
            .contains("checksum")
    );
}

#[test]
fn citations_must_name_real_source_lines() {
    let root = tempfile::tempdir().unwrap();
    let mut bundle = fixture(root.path());
    bundle.analysis.as_mut().unwrap().findings[0].citations[0].end_line = 9;
    assert!(
        bundle
            .validate(root.path())
            .unwrap_err()
            .to_string()
            .contains("line range")
    );
    bundle.analysis.as_mut().unwrap().findings[0].citations[0].source_id = "invented".into();
    assert!(
        bundle
            .validate(root.path())
            .unwrap_err()
            .to_string()
            .contains("unknown source")
    );
}

#[test]
fn unknown_features_and_path_traversal_fail() {
    let root = tempfile::tempdir().unwrap();
    let mut bundle = fixture(root.path());
    bundle.analysis.as_mut().unwrap().recipes[0]
        .features
        .push("invented".into());
    assert!(
        bundle
            .validate(root.path())
            .unwrap_err()
            .to_string()
            .contains("unknown crate feature")
    );
    assert!(safe_path(root.path(), "../elsewhere").is_err());
    assert!(safe_path(root.path(), "/etc/passwd").is_err());
    assert!(safe_path(root.path(), "..\\elsewhere").is_err());
}

#[cfg(unix)]
#[test]
fn evidence_cannot_escape_via_symlink() {
    let root = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    std::os::unix::fs::symlink(outside.path(), root.path().join("escape")).unwrap();
    assert!(safe_path(root.path(), "escape/file").is_err());
    std::os::unix::fs::symlink(
        outside.path().join("not-created"),
        root.path().join("dangling"),
    )
    .unwrap();
    assert!(safe_path(root.path(), "dangling").is_err());
}

#[test]
fn editing_recipe_invalidates_compilation_record() {
    let root = tempfile::tempdir().unwrap();
    let mut bundle = fixture(root.path());
    let recipe = &bundle.analysis.as_ref().unwrap().recipes[0];
    bundle.verification.push(RecipeCheck {
        recipe_id: recipe.id.clone(),
        input_sha256: recipe_fingerprint(&bundle, recipe).unwrap(),
        status: CheckStatus::Compiled,
        rustc_version: "fixture".into(),
        log_path: "verification/minimal/check.log".into(),
        lockfile_sha256: None,
        message: "fixture result".into(),
    });
    bundle.validate(root.path()).unwrap();
    bundle.analysis.as_mut().unwrap().recipes[0].code = "fn main() { invalid(); }".into();
    assert!(
        bundle
            .validate(root.path())
            .unwrap_err()
            .to_string()
            .contains("stale verification")
    );
}

#[test]
fn changing_resolved_dependencies_invalidates_compilation_record() {
    let root = tempfile::tempdir().unwrap();
    let mut bundle = fixture(root.path());
    let recipe = &bundle.analysis.as_ref().unwrap().recipes[0];
    fs::create_dir_all(root.path().join("verification/minimal")).unwrap();
    fs::write(
        root.path().join("verification/minimal/Cargo.lock"),
        "original lockfile",
    )
    .unwrap();
    bundle.verification.push(RecipeCheck {
        recipe_id: recipe.id.clone(),
        input_sha256: recipe_fingerprint(&bundle, recipe).unwrap(),
        status: CheckStatus::Compiled,
        rustc_version: "fixture".into(),
        log_path: "verification/minimal/check.log".into(),
        lockfile_sha256: Some(sha256(b"original lockfile")),
        message: "fixture result".into(),
    });
    bundle.validate(root.path()).unwrap();
    fs::write(
        root.path().join("verification/minimal/Cargo.lock"),
        "changed dependencies",
    )
    .unwrap();
    assert!(
        bundle
            .validate(root.path())
            .unwrap_err()
            .to_string()
            .contains("lockfile changed")
    );
}

fn inventoried_fixture(root: &std::path::Path) -> Bundle {
    let mut bundle = fixture(root);
    let text = "/// ```rust,ignore\n/// first();\n/// ```\n/// ```compile_fail\n/// invalid();\n/// ```\npub struct Parser;\n";
    fs::write(root.join("source.rs"), text).unwrap();
    bundle.sources[0].sha256 = sha256(text.as_bytes());
    bundle.example_inventory =
        Some(symposiast::examples::inventory(&bundle.sources, root).unwrap());
    let examples = &bundle.example_inventory.as_ref().unwrap().examples;
    bundle.analysis.as_mut().unwrap().example_coverage = vec![
        symposiast::examples::ExampleCoverage {
            example_id: examples[0].id.clone(),
            disposition: symposiast::examples::Disposition::Adapted,
            recipe_ids: vec!["minimal".into()],
            reason: "Adapt the documented construction into a complete consumer.".into(),
        },
        symposiast::examples::ExampleCoverage {
            example_id: examples[1].id.clone(),
            disposition: symposiast::examples::Disposition::Skipped,
            recipe_ids: vec![],
            reason: "Intentional compiler error.".into(),
        },
    ];
    bundle
}

#[test]
fn inventory_roundtrips_and_requires_complete_valid_coverage() {
    use symposiast::examples::Disposition;
    let root = tempfile::tempdir().unwrap();
    let bundle = inventoried_fixture(root.path());
    bundle.validate(root.path()).unwrap();
    bundle.save(root.path()).unwrap();
    assert_eq!(
        Bundle::load(root.path()).unwrap().example_inventory,
        bundle.example_inventory
    );
    for case in [
        "missing",
        "duplicate",
        "unknown-example",
        "unknown-recipe",
        "empty-reason",
        "skipped-with-recipe",
        "adapted-without-recipe",
        "uncited",
        "duplicate-recipe",
        "tampered-inventory",
    ] {
        let mut changed = bundle.clone();
        let analysis = changed.analysis.as_mut().unwrap();
        match case {
            "missing" => {
                analysis.example_coverage.pop();
            }
            "duplicate" => analysis
                .example_coverage
                .push(analysis.example_coverage[0].clone()),
            "unknown-example" => analysis.example_coverage[0].example_id = "unknown".into(),
            "unknown-recipe" => analysis.example_coverage[0].recipe_ids[0] = "unknown".into(),
            "empty-reason" => analysis.example_coverage[0].reason = " ".into(),
            "skipped-with-recipe" => {
                analysis.example_coverage[0].disposition = Disposition::Skipped
            }
            "adapted-without-recipe" => analysis.example_coverage[0].recipe_ids.clear(),
            "uncited" => {
                analysis.recipes[0].citations[0] = Citation {
                    source_id: "e0001".into(),
                    start_line: 7,
                    end_line: 7,
                }
            }
            "duplicate-recipe" => analysis.example_coverage[0]
                .recipe_ids
                .push("minimal".into()),
            "tampered-inventory" => {
                changed.example_inventory.as_mut().unwrap().examples[0].code = "invented();".into()
            }
            _ => unreachable!(),
        }
        assert!(changed.validate(root.path()).is_err(), "accepted {case}");
    }
    let mut supporting = bundle;
    supporting.analysis.as_mut().unwrap().example_coverage[0].disposition = Disposition::Supporting;
    supporting.validate(root.path()).unwrap();
}

#[test]
fn legacy_bundles_do_not_claim_example_coverage() {
    let root = tempfile::tempdir().unwrap();
    let mut bundle = fixture(root.path());
    bundle.save(root.path()).unwrap();
    let json: serde_json::Value =
        serde_json::from_slice(&fs::read(root.path().join("bundle.json")).unwrap()).unwrap();
    assert!(json.get("example_inventory").is_none());
    assert!(
        Bundle::load(root.path())
            .unwrap()
            .example_inventory
            .is_none()
    );
    bundle.analysis.as_mut().unwrap().example_coverage = inventoried_fixture(root.path())
        .analysis
        .unwrap()
        .example_coverage;
    bundle.sources[0].sha256 = sha256(&fs::read(root.path().join("source.rs")).unwrap());
    assert!(
        bundle
            .validate(root.path())
            .unwrap_err()
            .to_string()
            .contains("requires an inventory")
    );
}

#[test]
fn cli_import_upgrades_legacy_inventory_and_renders_coverage_atomically() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("bundle");
    fs::create_dir(&root).unwrap();
    let mut bundle = inventoried_fixture(&root);
    let analysis = bundle.analysis.take().unwrap();
    bundle.example_inventory = None;
    bundle.save(&root).unwrap();
    let before = fs::read(root.join("bundle.json")).unwrap();
    let input = temp.path().join("analysis.json");
    let mut incomplete = analysis.clone();
    incomplete.example_coverage.clear();
    fs::write(&input, serde_json::to_vec(&incomplete).unwrap()).unwrap();
    let analyze = || {
        std::process::Command::new(env!("CARGO_BIN_EXE_symposiast"))
            .arg("analyze")
            .arg(&root)
            .arg("--analysis")
            .arg(&input)
            .arg("--skip-verify")
            .output()
            .unwrap()
    };
    let failed = analyze();
    assert!(!failed.status.success());
    assert!(
        String::from_utf8_lossy(&failed.stderr).contains("every inventoried documentation example")
    );
    assert_eq!(fs::read(root.join("bundle.json")).unwrap(), before);
    fs::write(&input, serde_json::to_vec(&analysis).unwrap()).unwrap();
    let success = analyze();
    assert!(
        success.status.success(),
        "{}",
        String::from_utf8_lossy(&success.stderr)
    );
    let saved = Bundle::load(&root).unwrap();
    assert_eq!(saved.example_inventory.unwrap().examples.len(), 2);
    assert!(saved.verification.is_empty());
    let skill = fs::read_dir(root.join("plugin/skills"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let report = fs::read_to_string(skill.join("references/examples.md")).unwrap();
    assert!(report.contains("Adapted:"));
    assert!(report.contains("Skipped: Intentional compiler error."));
    assert!(report.contains("recipes/minimal.md"));
    let json: serde_json::Value =
        serde_json::from_slice(&fs::read(skill.join("references/examples.json")).unwrap()).unwrap();
    assert_eq!(json["inventory"]["examples"].as_array().unwrap().len(), 2);
    assert_eq!(json["coverage"].as_array().unwrap().len(), 2);
}
