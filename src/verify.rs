use std::{fs, path::Path, process::Stdio, time::Duration};

use anyhow::{Context, Result, ensure};
use tokio::{process::Command, time::timeout};

use crate::{
    adapters::ProcessGroup,
    model::{Bundle, CheckStatus, Recipe, RecipeCheck, recipe_fingerprint, safe_path, sha256},
};

#[derive(Debug, Clone)]
pub struct VerifyOptions {
    pub timeout_seconds: u64,
    pub offline: bool,
}

impl Default for VerifyOptions {
    fn default() -> Self {
        Self {
            timeout_seconds: 300,
            offline: false,
        }
    }
}

/// Cargo builds can execute dependency build scripts and procedural macros.
/// This reduces credential inheritance; it is not an operating-system sandbox.
fn cargo_command(directory: &Path) -> Command {
    let mut command = Command::new("cargo");
    command.current_dir(directory).env_clear();
    for name in [
        "PATH",
        "HOME",
        "USERPROFILE",
        "SYSTEMROOT",
        "TEMP",
        "TMP",
        "CARGO_HOME",
        "RUSTUP_HOME",
        "RUSTUP_TOOLCHAIN",
        "HTTP_PROXY",
        "HTTPS_PROXY",
        "ALL_PROXY",
        "NO_PROXY",
        "http_proxy",
        "https_proxy",
        "all_proxy",
        "no_proxy",
        "SSL_CERT_FILE",
        "SSL_CERT_DIR",
        "CARGO_HTTP_CAINFO",
    ] {
        if let Some(value) = std::env::var_os(name) {
            command.env(name, value);
        }
    }
    command
        .env("CARGO_BUILD_JOBS", "2")
        .env("CARGO_TERM_COLOR", "never")
        .env("CARGO_INCREMENTAL", "0");
    command.kill_on_drop(true).stdin(Stdio::null());
    #[cfg(unix)]
    command.process_group(0);
    command
}

async fn status(mut command: Command) -> Result<std::process::ExitStatus> {
    let mut child = command.spawn()?;
    let _group = ProcessGroup(child.id());
    Ok(child.wait().await?)
}

fn manifest(bundle: &Bundle, recipe: &Recipe) -> Result<String> {
    let mut deps = toml::map::Map::new();
    let mut primary = toml::map::Map::new();
    primary.insert(
        "version".into(),
        toml::Value::String(format!("={}", bundle.release.version)),
    );
    primary.insert(
        "default-features".into(),
        toml::Value::Boolean(recipe.default_features),
    );
    primary.insert(
        "features".into(),
        toml::Value::Array(
            recipe
                .features
                .iter()
                .cloned()
                .map(toml::Value::String)
                .collect(),
        ),
    );
    deps.insert(bundle.release.name.clone(), toml::Value::Table(primary));
    for (name, dependency) in &recipe.dependencies {
        let mut spec = toml::map::Map::new();
        spec.insert(
            "version".into(),
            toml::Value::String(dependency.version.clone()),
        );
        spec.insert(
            "default-features".into(),
            toml::Value::Boolean(dependency.default_features),
        );
        spec.insert(
            "features".into(),
            toml::Value::Array(
                dependency
                    .features
                    .iter()
                    .cloned()
                    .map(toml::Value::String)
                    .collect(),
            ),
        );
        deps.insert(name.clone(), toml::Value::Table(spec));
    }
    let mut package = toml::map::Map::new();
    package.insert(
        "name".into(),
        toml::Value::String(format!("recipe-{}", recipe.id)),
    );
    package.insert("version".into(), toml::Value::String("0.0.0".into()));
    package.insert("edition".into(), toml::Value::String("2024".into()));
    package.insert("publish".into(), toml::Value::Boolean(false));
    let mut root = toml::map::Map::new();
    root.insert("package".into(), toml::Value::Table(package));
    root.insert("workspace".into(), toml::Value::Table(Default::default()));
    root.insert("dependencies".into(), toml::Value::Table(deps));
    Ok(toml::to_string_pretty(&root)?)
}

pub async fn verify(bundle: &mut Bundle, root: &Path, options: &VerifyOptions) -> Result<()> {
    ensure!(
        options.timeout_seconds > 0,
        "verification timeout must be positive"
    );
    bundle.validate(root)?;
    let recipes = bundle
        .analysis
        .as_ref()
        .context("bundle has no analysis")?
        .recipes
        .clone();
    let rustc = timeout(
        Duration::from_secs(15),
        Command::new("rustc")
            .arg("--version")
            .kill_on_drop(true)
            .output(),
    )
    .await
    .context("rustc --version timed out")?
    .context("Rust is required to verify recipes")?;
    ensure!(rustc.status.success(), "rustc --version failed");
    let rustc_version = String::from_utf8_lossy(&rustc.stdout).trim().to_string();
    bundle.verification.clear();
    bundle.save(root)?;
    for recipe in recipes {
        let relative = format!("verification/{}", recipe.id);
        let directory = safe_path(root, &relative)?;
        fs::create_dir_all(safe_path(root, &format!("{relative}/src"))?)?;
        let manifest_text = manifest(bundle, &recipe)?;
        let manifest_path = safe_path(root, &format!("{relative}/Cargo.toml"))?;
        let lock_path = safe_path(root, &format!("{relative}/Cargo.lock"))?;
        if fs::read_to_string(&manifest_path).ok().as_deref() != Some(&manifest_text)
            && lock_path.exists()
        {
            fs::remove_file(&lock_path)?;
        }
        fs::write(manifest_path, manifest_text)?;
        fs::write(
            safe_path(root, &format!("{relative}/src/main.rs"))?,
            &recipe.code,
        )?;
        let log_path = format!("{relative}/check.log");
        let log_file = fs::File::create(safe_path(root, &log_path)?)?;
        let mut check = RecipeCheck {
            recipe_id: recipe.id.clone(),
            input_sha256: recipe_fingerprint(bundle, &recipe)?,
            status: CheckStatus::Failed,
            rustc_version: rustc_version.clone(),
            log_path,
            lockfile_sha256: None,
            message: String::new(),
        };
        let run = async {
            if !directory.join("Cargo.lock").exists() {
                let mut lock = cargo_command(&directory);
                lock.arg("generate-lockfile");
                if options.offline {
                    lock.arg("--offline");
                }
                lock.stdout(log_file.try_clone()?)
                    .stderr(log_file.try_clone()?);
                let result = status(lock)
                    .await
                    .context("start Cargo lockfile resolution")?;
                ensure!(result.success(), "Cargo lockfile resolution failed");
            }
            let mut command = cargo_command(&directory);
            command
                .args(["check", "--locked", "--all-targets"])
                .env("CARGO_TARGET_DIR", safe_path(root, "verification/target")?);
            if options.offline {
                command.arg("--offline");
            }
            command
                .stdout(log_file.try_clone()?)
                .stderr(log_file.try_clone()?);
            let result = status(command).await.context("start Cargo example check")?;
            ensure!(result.success(), "cargo check failed");
            Ok::<(), anyhow::Error>(())
        };
        match timeout(Duration::from_secs(options.timeout_seconds), run).await {
            Ok(Ok(())) => {
                check.status = CheckStatus::Compiled;
                check.message =
                    "cargo check --locked --all-targets passed; runtime behavior was not tested"
                        .into();
            }
            Ok(Err(error)) => check.message = format!("{error:#}"),
            Err(_) => {
                check.message = format!("verification exceeded {} seconds", options.timeout_seconds)
            }
        }
        if let Ok(bytes) = fs::read(directory.join("Cargo.lock")) {
            check.lockfile_sha256 = Some(sha256(&bytes));
        }
        eprintln!("{}: {:?}", recipe.id, check.status);
        bundle.verification.push(check);
        bundle.save(root)?;
    }
    ensure!(
        bundle
            .verification
            .iter()
            .all(|check| check.status == CheckStatus::Compiled),
        "one or more recipes failed verification; see bundle.json and verification/*/check.log"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Release, SCHEMA_VERSION};
    #[test]
    fn consumer_manifest_pins_primary_and_keeps_feature_selection() {
        let bundle = Bundle {
            schema_version: SCHEMA_VERSION,
            example_inventory: None,
            release: Release {
                name: "clap".into(),
                version: "4.5.0".into(),
                checksum: "x".into(),
                repository: None,
                repository_commit: None,
                rust_version: None,
                features: Default::default(),
                collected_at: "0".into(),
            },
            sources: vec![],
            analysis: None,
            review: None,
            verification: vec![],
            warnings: vec![],
        };
        let recipe = Recipe {
            id: "basic".into(),
            title: "Basic".into(),
            description: "".into(),
            code: "fn main() {}".into(),
            default_features: false,
            features: vec!["derive".into()],
            dependencies: Default::default(),
            citations: vec![],
        };
        let value: toml::Value = toml::from_str(&manifest(&bundle, &recipe).unwrap()).unwrap();
        assert_eq!(
            value["dependencies"]["clap"]["version"].as_str(),
            Some("=4.5.0")
        );
        assert_eq!(
            value["dependencies"]["clap"]["default-features"].as_bool(),
            Some(false)
        );
        assert!(value.get("workspace").is_some());
    }
}
