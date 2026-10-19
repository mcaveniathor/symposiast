use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, ensure};
use serde::Serialize;

use crate::model::{Basis, Bundle, Citation, Recipe, safe_path, sha256};

const INSTALL_MARKER: &str = ".bundle-install.json";

fn write_manifest(directory: &Path, contents: &str) -> Result<()> {
    // Published Symposium 0.5.1 discovers the uppercase spelling; current
    // repository documentation uses the mixed-case spelling. Keep both views
    // identical until the released ecosystem has migrated.
    for name in ["Symposium.toml", "SYMPOSIUM.toml"] {
        fs::write(directory.join(name), contents)?;
    }
    Ok(())
}

#[derive(Serialize)]
struct Manifest {
    name: String,
    #[serde(rename = "depends-on")]
    depends_on: Vec<String>,
    skills: Vec<Skills>,
    installations: Vec<Installation>,
    hooks: Vec<Hook>,
    mcp_servers: Vec<McpServer>,
}

#[derive(Serialize)]
struct Skills {
    #[serde(rename = "depends-on")]
    depends_on: Vec<String>,
    source: SkillSource,
}

#[derive(Serialize)]
struct SkillSource {
    path: String,
}

#[derive(Serialize)]
struct Installation {
    name: String,
    executable: String,
}

#[derive(Serialize)]
struct Hook {
    name: String,
    event: String,
    format: String,
    command: String,
    args: Vec<String>,
}

#[derive(Serialize)]
struct McpServer {
    name: String,
    command: String,
    args: Vec<String>,
    env: Vec<Env>,
    #[serde(rename = "depends-on")]
    depends_on: Vec<String>,
}

#[derive(Serialize)]
struct Env {
    name: String,
    value: String,
}

fn identity(bundle: &Bundle) -> String {
    format!("{}-{}", bundle.release.name, bundle.release.version)
}

fn manifest(bundle: &Bundle, runtime_root: &Path, executable: &Path) -> Result<String> {
    let identity = identity(bundle);
    let portable_name: String = identity
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '-'
            }
        })
        .collect();
    // MCP names must fit agent configuration identifier restrictions. A digest
    // distinguishes versions whose punctuation normalizes to the same name.
    let id = format!("{}-{}", portable_name, &sha256(identity.as_bytes())[..8]);
    let runtime_name = format!("{id}-runtime");
    let exact = format!("{}=={}", bundle.release.name, bundle.release.version);
    let executable = executable
        .to_str()
        .context("executable path must be UTF-8")?
        .to_owned();
    let runtime_root = runtime_root
        .to_str()
        .context("bundle path must be UTF-8")?
        .to_owned();
    Ok(toml::to_string_pretty(&Manifest {
        name: id.clone(),
        // Keep the hook active after a dependency upgrade. Exact gating belongs
        // on the guidance and MCP entry, not on the whole plugin.
        depends_on: vec![bundle.release.name.clone()],
        skills: vec![Skills {
            depends_on: vec![exact.clone()],
            source: SkillSource {
                path: "skills".into(),
            },
        }],
        installations: vec![Installation {
            name: runtime_name.clone(),
            executable: executable.clone(),
        }],
        hooks: vec![Hook {
            name: format!("{id}-version-check"),
            event: "SessionStart".into(),
            format: "symposium".into(),
            command: runtime_name,
            args: vec!["hook".into(), runtime_root.clone()],
        }],
        mcp_servers: vec![McpServer {
            name: id,
            command: executable,
            args: vec!["serve".into(), runtime_root],
            env: vec![],
            depends_on: vec![exact],
        }],
    })?)
}

fn citations_markdown(bundle: &Bundle, citations: &[Citation]) -> String {
    citations
        .iter()
        .map(|citation| {
            let source = bundle
                .sources
                .iter()
                .find(|s| s.id == citation.source_id)
                .expect("validated citation");
            format!(
                "- [{} lines {}–{}](../evidence/{}.txt) ([source]({}))\n",
                source.id, citation.start_line, citation.end_line, source.id, source.url
            )
        })
        .collect()
}

fn recipe_markdown(bundle: &Bundle, recipe: &Recipe) -> String {
    let status = bundle
        .verification
        .iter()
        .rev()
        .find(|check| check.recipe_id == recipe.id);
    let verification = status
        .map(|check| {
            format!(
                "{:?}: {} ({}).",
                check.status, check.message, check.rustc_version
            )
        })
        .unwrap_or_else(|| "Unrun: this recipe has not been compiled.".into());
    let mut dependencies = toml::Table::new();
    dependencies.insert(
        bundle.release.name.clone(),
        toml::Value::Table(toml::Table::from_iter([
            (
                "version".into(),
                toml::Value::String(format!("={}", bundle.release.version)),
            ),
            (
                "default-features".into(),
                toml::Value::Boolean(recipe.default_features),
            ),
            (
                "features".into(),
                toml::Value::Array(
                    recipe
                        .features
                        .iter()
                        .cloned()
                        .map(toml::Value::String)
                        .collect(),
                ),
            ),
        ])),
    );
    for (name, dep) in &recipe.dependencies {
        dependencies.insert(
            name.clone(),
            toml::Value::Table(toml::Table::from_iter([
                ("version".into(), toml::Value::String(dep.version.clone())),
                (
                    "default-features".into(),
                    toml::Value::Boolean(dep.default_features),
                ),
                (
                    "features".into(),
                    toml::Value::Array(
                        dep.features
                            .iter()
                            .cloned()
                            .map(toml::Value::String)
                            .collect(),
                    ),
                ),
            ])),
        );
    }
    let manifest = toml::to_string_pretty(&toml::Table::from_iter([(
        "dependencies".into(),
        toml::Value::Table(dependencies),
    )]))
    .expect("TOML values serialize");
    format!(
        "# {}\n\n{}\n\nRelease: {} {}.\n\nVerification: {}\nCompilation does not establish runtime behavior or idiomatic usage.\n\n```toml\n{}```\n\n```rust\n{}\n```\n\n## Evidence\n\n{}",
        recipe.title,
        recipe.description,
        bundle.release.name,
        bundle.release.version,
        verification,
        manifest,
        recipe.code,
        citations_markdown(bundle, &recipe.citations)
    )
}

/// Materialize a Symposium plugin whose skills carry their own evidence files.
pub fn render(bundle: &Bundle, root: &Path) -> Result<()> {
    bundle.validate(root)?;
    ensure!(
        bundle
            .review
            .as_ref()
            .is_none_or(|review| review.approved && review.issues.is_empty()),
        "review has unresolved issues; resolve them before rendering"
    );
    let root = root.canonicalize()?;
    let analysis = bundle
        .analysis
        .as_ref()
        .context("analyzed guidance is required to render a plugin")?;
    let staging = tempfile::Builder::new()
        .prefix(".plugin-render-")
        .tempdir_in(&root)?;
    let plugin = staging.path();
    // Installed skill names share one namespace, even when a workspace uses
    // multiple releases. Keep the crate prefix and a stable release identity.
    let prefix: String = bundle
        .release
        .name
        .replace('_', "-")
        .chars()
        .take(49)
        .collect();
    let skill_name = format!(
        "{}-{}-guide",
        prefix.trim_end_matches('-'),
        &sha256(identity(bundle).as_bytes())[..8]
    );
    let skill = plugin.join("skills").join(&skill_name);
    let references = skill.join("references");
    for child in ["findings", "recipes", "evidence"] {
        fs::create_dir_all(references.join(child))?;
    }
    // JSON strings are valid YAML quoted scalars and escape arbitrary titles.
    let description = format!(
        "Use {} {} for the tasks covered in its release guidance; consult cited findings, recipe verification status, and required features.",
        bundle.release.name, bundle.release.version
    );
    let mut index = format!(
        "---\nname: {}\ndescription: {}\ndepends-on: {}\n---\n\n# {} {}\n\n{}\n\nUse this guidance only with the stated release. Check the project's Cargo.lock before applying it. A compiled recipe passed cargo check; runtime behavior has not been tested. Read each recipe's feature and dependency requirements.\n\n## Guidance\n\n",
        serde_json::to_string(&skill_name)?,
        serde_json::to_string(&description)?,
        serde_json::to_string(&format!(
            "{}=={}",
            bundle.release.name, bundle.release.version
        ))?,
        bundle.release.name,
        bundle.release.version,
        analysis.summary
    );
    for finding in &analysis.findings {
        let basis = match finding.basis {
            Basis::Documented => "Documented",
            Basis::Inferred => "Inferred from the cited sources",
        };
        let features = if finding.features.is_empty() {
            "No additional features stated".to_owned()
        } else {
            finding.features.join(", ")
        };
        fs::write(
            references
                .join("findings")
                .join(format!("{}.md", finding.id)),
            format!(
                "# {}\n\n{}\n\nBasis: {}.\n\nFeatures: {}.\n\n## Evidence\n\n{}",
                finding.title,
                finding.guidance,
                basis,
                features,
                citations_markdown(bundle, &finding.citations)
            ),
        )?;
        index.push_str(&format!(
            "- [{}](references/findings/{}.md) — {}.\n",
            finding.title, finding.id, basis
        ));
    }
    index.push_str("\n## Recipes\n\n");
    for recipe in &analysis.recipes {
        fs::write(
            references.join("recipes").join(format!("{}.md", recipe.id)),
            recipe_markdown(bundle, recipe),
        )?;
        let status = bundle
            .verification
            .iter()
            .rev()
            .find(|check| check.recipe_id == recipe.id)
            .map(|check| format!("{:?}", check.status))
            .unwrap_or_else(|| "Unrun".into());
        index.push_str(&format!(
            "- [{}](references/recipes/{}.md) — {}.\n",
            recipe.title, recipe.id, status
        ));
    }
    for source in &bundle.sources {
        fs::copy(
            safe_path(&root, &source.path)?,
            references
                .join("evidence")
                .join(format!("{}.txt", source.id)),
        )?;
    }
    if !bundle.warnings.is_empty() {
        index.push_str("\n## Coverage limitations\n\n");
        for warning in &bundle.warnings {
            index.push_str(&format!("- {warning}\n"));
        }
    }
    fs::write(skill.join("SKILL.md"), index)?;
    write_manifest(plugin, &manifest(bundle, &root, &std::env::current_exe()?)?)?;
    let target = root.join("plugin");
    ensure!(
        !target.is_symlink(),
        "refusing to replace a symlink plugin directory"
    );
    let backup = tempfile::Builder::new()
        .prefix(".plugin-previous-")
        .tempdir_in(&root)?;
    let previous = backup.path().join("plugin");
    if target.exists() {
        fs::rename(&target, &previous)?;
    }
    if let Err(error) = fs::rename(plugin, &target) {
        if previous.exists() {
            fs::rename(&previous, &target)
                .context("restore previous plugin after failed render")?;
        }
        return Err(error.into());
    }
    Ok(())
}

fn copy_tree(source: &Path, target: &Path) -> Result<()> {
    ensure!(
        !fs::symlink_metadata(source)?.file_type().is_symlink(),
        "refusing to copy a symlink: {}",
        source.display()
    );
    if source.is_dir() {
        fs::create_dir_all(target)?;
        for entry in fs::read_dir(source)? {
            let entry = entry?;
            copy_tree(&entry.path(), &target.join(entry.file_name()))?;
        }
    } else {
        fs::copy(source, target)?;
    }
    Ok(())
}

/// Copy a complete bundle into Symposium's built-in user plugin registry.
/// Existing unrelated directories are never replaced. The runtime binary is
/// referenced by its absolute installed path; Symposium does not acquire it.
pub fn install(root: &Path, symposium_home: &Path, executable: &Path) -> Result<PathBuf> {
    let bundle = Bundle::load(root)?;
    ensure!(
        bundle.analysis.is_some(),
        "analyzed guidance is required to install a plugin"
    );
    let executable = executable
        .canonicalize()
        .context("locate the runtime executable")?;
    ensure!(executable.is_file(), "runtime executable is not a file");
    let registry = symposium_home.join("plugins");
    fs::create_dir_all(&registry)?;
    let registry = registry.canonicalize()?;
    let target = registry.join(identity(&bundle));
    let marker = serde_json::json!({"schema_version": 1, "crate": bundle.release.name, "version": bundle.release.version});
    if target.exists() || target.is_symlink() {
        ensure!(
            !target.is_symlink() && target.is_dir(),
            "installation target is not an owned directory: {}",
            target.display()
        );
        let existing: serde_json::Value = serde_json::from_slice(
            &fs::read(target.join(INSTALL_MARKER))
                .context("refusing to replace a plugin without an ownership marker")?,
        )?;
        ensure!(
            existing == marker,
            "refusing to replace an unrelated plugin at {}",
            target.display()
        );
    }
    let staging = tempfile::Builder::new()
        .prefix(".bundle-stage-")
        .tempdir_in(&registry)?;
    let staged_bundle = staging.path().join("bundle");
    fs::create_dir_all(&staged_bundle)?;
    for source in &bundle.sources {
        ensure!(
            source.path != "bundle.json" && !source.path.starts_with("plugin/"),
            "reserved source path: {}",
            source.path
        );
        let destination = staged_bundle.join(&source.path);
        fs::create_dir_all(destination.parent().context("source needs a parent")?)?;
        fs::copy(safe_path(root, &source.path)?, destination)?;
    }
    for check in &bundle.verification {
        if check.log_path.is_empty() {
            continue;
        }
        let source = safe_path(root, &check.log_path)?;
        if source.is_file() {
            let destination = staged_bundle.join(&check.log_path);
            fs::create_dir_all(destination.parent().context("log needs a parent")?)?;
            fs::copy(source, destination)?;
        }
    }
    for recipe in &bundle.analysis.as_ref().expect("checked analysis").recipes {
        let lock_checksum = bundle
            .verification
            .iter()
            .find(|check| check.recipe_id == recipe.id)
            .and_then(|check| check.lockfile_sha256.as_deref());
        for filename in ["Cargo.toml", "Cargo.lock", "src/main.rs"] {
            let relative = format!("verification/{}/{filename}", recipe.id);
            let source = safe_path(root, &relative)?;
            if !source.is_file() {
                ensure!(
                    filename != "Cargo.lock" || lock_checksum.is_none(),
                    "missing recorded recipe lockfile: {relative}"
                );
                continue;
            }
            if filename == "Cargo.lock"
                && let Some(expected) = lock_checksum
            {
                ensure!(
                    sha256(&fs::read(&source)?) == expected,
                    "recipe lockfile checksum mismatch: {}",
                    recipe.id
                );
            }
            let destination = staged_bundle.join(&relative);
            fs::create_dir_all(destination.parent().context("recipe file needs a parent")?)?;
            fs::copy(source, destination)?;
        }
    }
    bundle.save(&staged_bundle)?;
    render(&bundle, &staged_bundle)?;
    copy_tree(
        &staged_bundle.join("plugin/skills"),
        &staging.path().join("skills"),
    )?;
    fs::remove_dir_all(staged_bundle.join("plugin"))?;
    write_manifest(
        staging.path(),
        &manifest(&bundle, &target.join("bundle"), &executable)?,
    )?;
    fs::write(
        staging.path().join(INSTALL_MARKER),
        serde_json::to_vec_pretty(&marker)?,
    )?;
    // Keep a recoverable old directory until the new tree has been placed.
    let backup = tempfile::Builder::new()
        .prefix(".bundle-backup-")
        .tempdir_in(&registry)?;
    let old = backup.path().join("previous");
    if target.exists() {
        fs::rename(&target, &old)?;
    }
    if let Err(error) = fs::rename(staging.path(), &target) {
        if old.exists() {
            fs::rename(&old, &target)
                .context("restore previous plugin after failed installation")?;
        }
        return Err(error.into());
    }
    Ok(target)
}
