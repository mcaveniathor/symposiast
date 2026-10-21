use std::{collections::BTreeMap, fs, path::Path, process::Command};

use serde_json::Value;
use symposiast::{
    model::{Bundle, CheckStatus, Review, sha256},
    registry,
};

fn copy_tree(source: &Path, target: &Path) {
    fs::create_dir_all(target).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_dir() {
            // Runtime renders and Cargo caches are not part of the portable pilot.
            if entry.file_name() != "target" && entry.file_name() != "plugin" {
                copy_tree(&entry.path(), &target.join(entry.file_name()));
            }
        } else {
            fs::copy(entry.path(), target.join(entry.file_name())).unwrap();
        }
    }
}

fn fixture(root: &Path) -> Bundle {
    copy_tree(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("plugins/stucco/bundle"),
        root,
    );
    Bundle::load(root).unwrap()
}

fn files(root: &Path) -> BTreeMap<String, Vec<u8>> {
    fn visit(root: &Path, path: &Path, result: &mut BTreeMap<String, Vec<u8>>) {
        for entry in fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_dir() {
                visit(root, &entry.path(), result);
            } else {
                result.insert(
                    entry
                        .path()
                        .strip_prefix(root)
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .into(),
                    fs::read(entry.path()).unwrap(),
                );
            }
        }
    }
    let mut result = BTreeMap::new();
    visit(root, root, &mut result);
    result
}

#[test]
fn cli_exports_a_relocatable_deterministic_pilot_without_touching_the_bundle() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    let bundle = fixture(&source);
    let before = files(&source);
    let output = temp.path().join("registry/stucco");
    let result = Command::new(env!("CARGO_BIN_EXE_symposiast"))
        .args(["export-registry"])
        .arg(&source)
        .arg("--output")
        .arg(&output)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(files(&source), before);
    let second = temp.path().join("second");
    registry::export(&source, &second).unwrap();
    assert_eq!(files(&output), files(&second));

    let relocated = temp.path().join("relocated");
    fs::rename(&output, &relocated).unwrap();
    fs::remove_dir_all(&source).unwrap();
    let manifest_text = fs::read_to_string(relocated.join("Symposium.toml")).unwrap();
    assert_eq!(
        manifest_text,
        fs::read_to_string(relocated.join("SYMPOSIUM.toml")).unwrap()
    );
    let manifest: toml::Value = toml::from_str(&manifest_text).unwrap();
    assert_eq!(manifest["depends-on"][0].as_str(), Some("stucco==0.3.0"));
    assert_eq!(
        manifest["skills"][0]["source"]["path"].as_str(),
        Some("skills")
    );
    for runtime_field in ["hooks", "installations", "mcp_servers"] {
        assert!(manifest.get(runtime_field).is_none());
    }
    let skill = fs::read_dir(relocated.join("skills"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let references = skill.join("references");
    let provenance: Value =
        serde_json::from_slice(&fs::read(references.join("provenance.json")).unwrap()).unwrap();
    assert_eq!(provenance["bundle_sha256"], sha256(&before["bundle.json"]));
    assert!(provenance["review"].is_null());
    assert_eq!(
        provenance["sources"].as_array().unwrap().len(),
        bundle.sources.len()
    );
    for source in provenance["sources"].as_array().unwrap() {
        assert_eq!(
            source["sha256"],
            sha256(&fs::read(references.join(source["path"].as_str().unwrap())).unwrap())
        );
    }
    for recipe in provenance["recipes"].as_array().unwrap() {
        assert_eq!(recipe["status"], "compiled");
        assert_eq!(
            recipe["lockfile_sha256"],
            sha256(&fs::read(references.join(recipe["lockfile_path"].as_str().unwrap())).unwrap())
        );
    }
    assert_eq!(
        provenance["sources"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|s| s["kind"] == "license")
            .count(),
        2
    );
    for (path, bytes) in files(&relocated) {
        let text = String::from_utf8(bytes).unwrap();
        assert!(!text.contains(source.to_str().unwrap()));
        if path.ends_with(".md") {
            for link in text
                .split("](")
                .skip(1)
                .filter_map(|part| part.split(')').next())
            {
                if !link.contains("://") {
                    assert!(
                        relocated.join(&path).parent().unwrap().join(link).is_file(),
                        "broken link in {path}: {link}"
                    );
                }
            }
        }
    }
}

#[test]
fn export_rejects_unverified_stale_and_rejected_proposals_before_publication() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("bundle");
    let original = fixture(&source);
    for case in [
        "missing-check",
        "failed-check",
        "missing-lock-record",
        "stale",
        "rejected-review",
    ] {
        let mut bundle = original.clone();
        match case {
            "missing-check" => {
                bundle.verification.pop();
            }
            "failed-check" => bundle.verification[0].status = CheckStatus::Failed,
            "missing-lock-record" => bundle.verification[0].lockfile_sha256 = None,
            "stale" => bundle.analysis.as_mut().unwrap().recipes[0]
                .code
                .push_str("\n// edited"),
            "rejected-review" => {
                bundle.review = Some(Review {
                    approved: false,
                    issues: vec!["Revise recipe".into()],
                })
            }
            _ => unreachable!(),
        }
        bundle.save(&source).unwrap();
        let output = temp.path().join(case);
        assert!(
            registry::export(&source, &output).is_err(),
            "accepted {case}"
        );
        assert!(!output.exists());
    }
    original.save(&source).unwrap();
    fs::write(
        source.join("verification/static-page/Cargo.lock"),
        "changed",
    )
    .unwrap();
    assert!(registry::export(&source, &temp.path().join("changed-lock")).is_err());
}

#[test]
fn export_preserves_existing_destinations() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("bundle");
    fixture(&source);
    let output = temp.path().join("entry");
    registry::export(&source, &output).unwrap();
    fs::write(output.join("notes.txt"), "human edits").unwrap();
    let before = files(&output);
    assert!(
        registry::export(&source, &output)
            .unwrap_err()
            .to_string()
            .contains("already exists")
    );
    assert_eq!(files(&output), before);
    let file = temp.path().join("file");
    fs::write(&file, "keep").unwrap();
    assert!(registry::export(&source, &file).is_err());
    assert_eq!(fs::read_to_string(file).unwrap(), "keep");
}

#[cfg(unix)]
#[test]
fn export_rejects_symlink_destinations_and_evidence() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("bundle");
    let bundle = fixture(&source);
    let output = temp.path().join("link");
    std::os::unix::fs::symlink(temp.path().join("absent"), &output).unwrap();
    assert!(registry::export(&source, &output).is_err());
    assert!(output.is_symlink());
    let evidence = source.join(&bundle.sources[0].path);
    let outside = temp.path().join("evidence");
    fs::rename(&evidence, &outside).unwrap();
    std::os::unix::fs::symlink(&outside, evidence).unwrap();
    assert!(registry::export(&source, &temp.path().join("entry")).is_err());
}

#[test]
fn registry_preserves_example_inventory_and_skip_reasons() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    let mut bundle = fixture(&source);
    let inventory = symposiast::examples::inventory(&bundle.sources, &source).unwrap();
    assert!(!inventory.examples.is_empty());
    bundle.analysis.as_mut().unwrap().example_coverage = inventory
        .examples
        .iter()
        .map(|example| symposiast::examples::ExampleCoverage {
            example_id: example.id.clone(),
            disposition: symposiast::examples::Disposition::Skipped,
            recipe_ids: vec![],
            reason: "Fixture records an explicit skip for export validation.".into(),
        })
        .collect();
    bundle.example_inventory = Some(inventory.clone());
    bundle.validate(&source).unwrap();
    bundle.save(&source).unwrap();
    let output = temp.path().join("export");
    registry::export(&source, &output).unwrap();
    let skill = fs::read_dir(output.join("skills"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    fs::remove_dir_all(&source).unwrap();
    let data: Value =
        serde_json::from_slice(&fs::read(skill.join("references/examples.json")).unwrap()).unwrap();
    assert_eq!(data["inventory"], serde_json::to_value(&inventory).unwrap());
    assert_eq!(
        data["coverage"].as_array().unwrap().len(),
        inventory.examples.len()
    );
    assert!(
        data["coverage"]
            .as_array()
            .unwrap()
            .iter()
            .all(|entry| entry["disposition"] == "skipped")
    );
    let provenance: Value =
        serde_json::from_slice(&fs::read(skill.join("references/provenance.json")).unwrap())
            .unwrap();
    assert_eq!(provenance["example_inventory"], data["inventory"]);
    assert_eq!(provenance["example_coverage"], data["coverage"]);
    let report = fs::read_to_string(skill.join("references/examples.md")).unwrap();
    assert!(report.contains("Skipped: Fixture records an explicit skip"));
    for example in &inventory.examples {
        assert!(
            skill
                .join(format!(
                    "references/evidence/{}.txt",
                    example.citation.source_id
                ))
                .exists()
        );
    }
}

#[test]
#[cfg(unix)]
fn ci_adapter_accounts_for_real_inventory_with_optional_repository_evidence() {
    for published_only in [false, true] {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("source");
        let mut bundle = fixture(&source);
        if published_only {
            bundle
                .sources
                .retain(|source| source.url.starts_with("https://docs.rs/"));
        }
        // Model collection with and without the optional repository snapshot.
        bundle.example_inventory = None;
        bundle.analysis = None;
        bundle.review = None;
        bundle.verification.clear();
        bundle.save(&source).unwrap();
        let result = Command::new(env!("CARGO_BIN_EXE_symposiast"))
            .arg("analyze")
            .arg(&source)
            .args(["--agent", "custom", "--agent-executable"])
            .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/ci/stucco-adapter.py"))
            .arg("--skip-verify")
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let generated = Bundle::load(&source).unwrap();
        let inventory = generated.example_inventory.unwrap();
        let analysis = generated.analysis.unwrap();
        assert!(!inventory.examples.is_empty());
        assert_eq!(analysis.example_coverage.len(), inventory.examples.len());
        assert!(
            analysis
                .example_coverage
                .iter()
                .any(|coverage| coverage.disposition == symposiast::examples::Disposition::Adapted)
        );
        assert!(
            analysis
                .example_coverage
                .iter()
                .any(|coverage| coverage.disposition == symposiast::examples::Disposition::Skipped)
        );
        assert_eq!(analysis.recipes.len(), 1);
        assert_eq!(analysis.recipes[0].id, "semantic-page");
    }
}
