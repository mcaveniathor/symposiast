use std::{
    collections::BTreeMap,
    fs,
    path::{Component, Path},
};

use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bundle {
    pub schema_version: u32,
    pub release: Release,
    pub sources: Vec<Source>,
    pub analysis: Option<Analysis>,
    pub review: Option<Review>,
    #[serde(default)]
    pub verification: Vec<RecipeCheck>,
    #[serde(default)]
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Release {
    pub name: String,
    pub version: String,
    pub checksum: String,
    pub repository: Option<String>,
    pub repository_commit: Option<String>,
    pub rust_version: Option<String>,
    pub features: BTreeMap<String, Vec<String>>,
    pub collected_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub id: String,
    pub kind: String,
    pub path: String,
    pub url: String,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Analysis {
    pub summary: String,
    pub findings: Vec<Finding>,
    pub recipes: Vec<Recipe>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Finding {
    pub id: String,
    pub title: String,
    pub guidance: String,
    pub basis: Basis,
    pub features: Vec<String>,
    pub citations: Vec<Citation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Basis {
    Documented,
    Inferred,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Citation {
    pub source_id: String,
    pub start_line: usize,
    pub end_line: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Recipe {
    pub id: String,
    pub title: String,
    pub description: String,
    /// A complete Rust src/main.rs file; examples may contain #[test] functions.
    pub code: String,
    pub default_features: bool,
    pub features: Vec<String>,
    /// Additional registry dependencies, never paths or git sources.
    pub dependencies: BTreeMap<String, Dependency>,
    pub citations: Vec<Citation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Dependency {
    pub version: String,
    pub features: Vec<String>,
    pub default_features: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Review {
    pub approved: bool,
    pub issues: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecipeCheck {
    pub recipe_id: String,
    #[serde(default)]
    pub input_sha256: String,
    pub status: CheckStatus,
    pub rustc_version: String,
    pub log_path: String,
    pub lockfile_sha256: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CheckStatus {
    Compiled,
    Failed,
    Unrun,
}

pub fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub fn recipe_fingerprint(bundle: &Bundle, recipe: &Recipe) -> Result<String> {
    Ok(sha256(&serde_json::to_vec(&(
        &bundle.release.name,
        &bundle.release.version,
        &bundle.release.checksum,
        recipe,
    ))?))
}

pub fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 100
        && value
            .bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-' || c == b'_')
}

pub fn safe_path(root: &Path, relative: &str) -> Result<std::path::PathBuf> {
    let path = Path::new(relative);
    ensure!(
        !relative.is_empty()
            && !relative.contains('\\')
            && path.components().all(|c| matches!(c, Component::Normal(_))),
        "unsafe bundle path: {relative}"
    );
    let canonical_root = root.canonicalize()?;
    let mut current = canonical_root.clone();
    for component in path.components() {
        current.push(component);
        match fs::symlink_metadata(&current) {
            Ok(metadata) => ensure!(
                !metadata.file_type().is_symlink(),
                "bundle path contains a symlink: {relative}"
            ),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    Ok(canonical_root.join(path))
}

impl Bundle {
    pub fn load(root: &Path) -> Result<Self> {
        let bundle: Self = serde_json::from_slice(
            &fs::read(root.join("bundle.json")).context("read bundle.json")?,
        )?;
        bundle.validate(root)?;
        Ok(bundle)
    }

    pub fn save(&self, root: &Path) -> Result<()> {
        fs::create_dir_all(root)?;
        let mut file = tempfile::NamedTempFile::new_in(root)?;
        serde_json::to_writer_pretty(&mut file, self)?;
        file.persist(root.join("bundle.json"))?;
        Ok(())
    }

    pub fn validate(&self, root: &Path) -> Result<()> {
        ensure!(
            self.schema_version == SCHEMA_VERSION,
            "unsupported bundle schema {}",
            self.schema_version
        );
        ensure!(valid_id(&self.release.name), "invalid crate name");
        semver::Version::parse(&self.release.version).context("invalid crate version")?;
        let mut lengths = BTreeMap::new();
        for source in &self.sources {
            ensure!(valid_id(&source.id), "invalid source ID");
            let bytes = fs::read(safe_path(root, &source.path)?)?;
            ensure!(
                sha256(&bytes) == source.sha256,
                "source checksum mismatch: {}",
                source.id
            );
            let text = std::str::from_utf8(&bytes).context("source must be UTF-8")?;
            ensure!(
                lengths
                    .insert(source.id.as_str(), text.lines().count())
                    .is_none(),
                "duplicate source ID"
            );
        }
        let citations = |refs: &[Citation]| -> Result<()> {
            ensure!(!refs.is_empty(), "every finding and recipe needs evidence");
            for citation in refs {
                let count = lengths
                    .get(citation.source_id.as_str())
                    .context("citation references an unknown source")?;
                ensure!(
                    citation.start_line > 0
                        && citation.end_line >= citation.start_line
                        && citation.end_line <= *count,
                    "citation line range is invalid for {}",
                    citation.source_id
                );
            }
            Ok(())
        };
        if let Some(analysis) = &self.analysis {
            ensure!(
                !analysis.summary.trim().is_empty(),
                "analysis summary is empty"
            );
            ensure!(
                !analysis.findings.is_empty() && !analysis.recipes.is_empty(),
                "analysis needs findings and recipes"
            );
            let mut ids = std::collections::BTreeSet::new();
            for finding in &analysis.findings {
                ensure!(
                    valid_id(&finding.id) && ids.insert(finding.id.as_str()),
                    "invalid or duplicate finding ID"
                );
                ensure!(
                    !finding.title.trim().is_empty() && !finding.guidance.trim().is_empty(),
                    "finding text is empty"
                );
                citations(&finding.citations)?;
                for feature in &finding.features {
                    ensure!(
                        self.release.features.contains_key(feature),
                        "unknown finding feature: {feature}"
                    );
                }
            }
            ids.clear();
            for recipe in &analysis.recipes {
                ensure!(
                    valid_id(&recipe.id) && ids.insert(recipe.id.as_str()),
                    "invalid or duplicate recipe ID"
                );
                ensure!(
                    !recipe.code.trim().is_empty() && !recipe.title.trim().is_empty(),
                    "recipe code or title is empty"
                );
                citations(&recipe.citations)?;
                for feature in &recipe.features {
                    ensure!(
                        self.release.features.contains_key(feature),
                        "unknown crate feature: {feature}"
                    );
                }
                for (name, dep) in &recipe.dependencies {
                    ensure!(
                        valid_id(name) && name != &self.release.name,
                        "invalid or duplicate recipe dependency: {name}"
                    );
                    semver::VersionReq::parse(&dep.version)
                        .context("invalid dependency version requirement")?;
                }
            }
            ids.clear();
            for check in &self.verification {
                ensure!(
                    ids.insert(check.recipe_id.as_str()),
                    "duplicate verification record"
                );
                let recipe = analysis
                    .recipes
                    .iter()
                    .find(|recipe| recipe.id == check.recipe_id)
                    .context("verification references unknown recipe")?;
                safe_path(root, &check.log_path)?;
                if check.status == CheckStatus::Compiled {
                    ensure!(
                        check.input_sha256 == recipe_fingerprint(self, recipe)?,
                        "stale verification for {}; re-run verify",
                        recipe.id
                    );
                    if let Some(expected) = &check.lockfile_sha256 {
                        let lockfile =
                            safe_path(root, &format!("verification/{}/Cargo.lock", recipe.id))?;
                        let bytes = fs::read(lockfile)
                            .context("compiled recipe lockfile is missing; re-run verify")?;
                        ensure!(
                            sha256(&bytes) == *expected,
                            "compiled recipe lockfile changed for {}; re-run verify",
                            recipe.id
                        );
                    }
                }
            }
        } else if !self.verification.is_empty() {
            bail!("verification requires analysis");
        }
        Ok(())
    }
}
