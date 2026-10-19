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
