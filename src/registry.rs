//! Portable, skills-only registry exports; no local executable or bundle paths.

use std::{fs, path::Path};

use anyhow::{Context, Result, ensure};
use serde_json::json;

use crate::{
    model::{Bundle, CheckStatus, safe_path, sha256},
    plugin,
};

/// Export one compiled release into a new directory within a Symposium registry.
/// The source bundle is read-only and an existing destination is never replaced.
pub fn export(root: &Path, output: &Path) -> Result<()> {
    let bundle = Bundle::load(root)?;
    plugin::validate_guidance(&bundle, root)?;
    let analysis = bundle.analysis.as_ref().expect("validated analysis");
    for recipe in &analysis.recipes {
        ensure!(
            bundle.verification.iter().any(|check| {
                check.recipe_id == recipe.id
                    && check.status == CheckStatus::Compiled
                    && check.lockfile_sha256.is_some()
            }),
            "registry export requires current compilation and a recorded lockfile for recipe {}; run verify first",
            recipe.id
        );
    }
    ensure!(
        !output.try_exists()? && !output.is_symlink(),
        "registry output already exists; export to a new directory and review the diff before replacing an entry"
    );
    let parent = output
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    fs::create_dir_all(parent)?;
    let staging = tempfile::Builder::new()
        .prefix(".registry-export-")
        .tempdir_in(parent)?;
    let skill = plugin::render_skills(&bundle, root, staging.path())?;
    let references = skill.join("references");
    fs::create_dir(references.join("lockfiles"))?;
    let mut recipes = Vec::new();
    for check in &bundle.verification {
        let path = format!("lockfiles/{}.lock", check.recipe_id);
        fs::copy(
            safe_path(
                root,
                &format!("verification/{}/Cargo.lock", check.recipe_id),
            )?,
            references.join(&path),
        )?;
        recipes.push(json!({
            "recipe_id": check.recipe_id,
            "input_sha256": check.input_sha256,
            "status": check.status,
            "rustc_version": check.rustc_version,
            "lockfile_sha256": check.lockfile_sha256,
            "lockfile_path": path,
        }));
    }
    let sources: Vec<_> = bundle
        .sources
        .iter()
        .map(|source| {
            json!({
                "id": source.id,
                "kind": source.kind,
                "url": source.url,
                "sha256": source.sha256,
                "path": format!("evidence/{}.txt", source.id),
            })
        })
        .collect();
    fs::write(
        references.join("provenance.json"),
        serde_json::to_vec_pretty(&json!({
            "schema_version": 1,
            "generator": {"name": "symposiast", "version": env!("CARGO_PKG_VERSION")},
            "bundle_sha256": sha256(&fs::read(root.join("bundle.json"))?),
            "release": bundle.release,
            "review": bundle.review,
            "sources": sources,
            "example_inventory": bundle.example_inventory,
            "example_coverage": analysis.example_coverage,
            "recipes": recipes,
            "warnings": bundle.warnings,
        }))?,
    )?;
    let index_path = skill.join("SKILL.md");
    let mut index = fs::read_to_string(&index_path)?;
    index.push_str("\n## Provenance\n\n[Release, evidence hashes, review status, and compilation records](references/provenance.json). Paths in that record are relative to its directory. Dependency lockfiles are included alongside the records.\n");
    fs::write(index_path, index)?;

    // Keep the whole entry gated to the release: this export has no drift hook.
    let identity = format!("{}-{}", bundle.release.name, bundle.release.version);
    let name = format!(
        "symposiast-{}-{}",
        bundle.release.name,
        &sha256(identity.as_bytes())[..8]
    );
    let exact = format!("{}=={}", bundle.release.name, bundle.release.version);
    let manifest = toml::to_string_pretty(&toml::toml! {
        name = name
        depends-on = [exact]
        [[skills]]
        source.path = "skills"
    })?;
    for filename in ["Symposium.toml", "SYMPOSIUM.toml"] {
        fs::write(staging.path().join(filename), &manifest)?;
    }
    fs::rename(staging.path(), output).context("publish registry export")?;
    Ok(())
}
