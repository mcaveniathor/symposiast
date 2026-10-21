use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail, ensure};
use clap::{Parser, Subcommand};
use symposiast::{
    adapters::{self, AgentKind, AgentOptions},
    collect,
    model::{Analysis, Bundle},
    plugin, registry, runtime,
    transaction::Transaction,
    verify::{self, VerifyOptions},
};

#[derive(Parser)]
#[command(
    version,
    about = "Build release-specific Rust crate guidance for Symposium"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Collect a frozen release and its source evidence, without running an agent.
    Collect {
        name: String,
        #[arg(long)]
        version: Option<String>,
        #[arg(long)]
        output: PathBuf,
    },
    /// Collect, analyze, verify, and render a Symposium plugin.
    Build {
        name: String,
        #[arg(long)]
        version: Option<String>,
        #[arg(long)]
        output: PathBuf,
        #[arg(long, value_enum, default_value = "codex")]
        agent: AgentKind,
        #[arg(long)]
        agent_executable: Option<PathBuf>,
        #[arg(long, value_enum)]
        reviewer: Option<AgentKind>,
        #[arg(long)]
        reviewer_executable: Option<PathBuf>,
        /// Import structured analysis instead of invoking an agent.
        #[arg(long)]
        analysis: Option<PathBuf>,
        #[arg(long, default_value_t = 600)]
        timeout: u64,
        /// Render with explicit unverified labels; do not run Cargo.
        #[arg(long)]
        skip_verify: bool,
    },
    /// Analyze an existing collected bundle, then verify and render it.
    Analyze {
        bundle: PathBuf,
        #[arg(long, value_enum, default_value = "codex")]
        agent: AgentKind,
        #[arg(long)]
        agent_executable: Option<PathBuf>,
        #[arg(long, value_enum)]
        reviewer: Option<AgentKind>,
        #[arg(long)]
        reviewer_executable: Option<PathBuf>,
        #[arg(long)]
        analysis: Option<PathBuf>,
        #[arg(long, default_value_t = 600)]
        timeout: u64,
        #[arg(long)]
        skip_verify: bool,
    },
    /// Recheck recipes against their pinned release and update the plugin.
    Verify {
        bundle: PathBuf,
        #[arg(long, default_value_t = 300)]
        timeout: u64,
        #[arg(long)]
        offline: bool,
    },
    /// Validate evidence and render the bundle as a Symposium plugin.
    Render { bundle: PathBuf },
    /// Export compiled guidance as a portable, skills-only Symposium registry entry.
    ExportRegistry {
        bundle: PathBuf,
        /// New plugin directory inside a registry; existing paths are never replaced.
        #[arg(long)]
        output: PathBuf,
    },
    /// Install a durable plugin copy into the local Symposium plugin source.
    Install {
        bundle: PathBuf,
        #[arg(long)]
        symposium_home: Option<PathBuf>,
    },
    /// Serve the bundle through the Model Context Protocol over stdio.
    Serve { bundle: PathBuf },
    /// Report dependency versions unsupported by this bundle.
    Check {
        bundle: PathBuf,
        #[arg(long, default_value = ".")]
        workspace: PathBuf,
    },
    /// Handle a Symposium SessionStart event from stdin.
    Hook { bundle: PathBuf },
    /// Fetch a new release into a separate output directory, preserving the old bundle.
    Refresh {
        bundle: PathBuf,
        #[arg(long)]
        output: PathBuf,
        #[arg(long, value_enum, default_value = "codex")]
        agent: AgentKind,
        #[arg(long)]
        agent_executable: Option<PathBuf>,
        #[arg(long, default_value_t = 600)]
        timeout: u64,
    },
}

struct AnalysisOptions {
    agent: AgentOptions,
    reviewer: Option<AgentOptions>,
    imported: Option<PathBuf>,
    skip_verify: bool,
}

fn options(
    agent: AgentKind,
    executable: Option<PathBuf>,
    reviewer: Option<AgentKind>,
    reviewer_executable: Option<PathBuf>,
    imported: Option<PathBuf>,
    timeout_seconds: u64,
    skip_verify: bool,
) -> Result<AnalysisOptions> {
    ensure!(timeout_seconds > 0, "timeout must be positive");
    ensure!(
        reviewer.is_some() || reviewer_executable.is_none(),
        "--reviewer-executable requires --reviewer"
    );
    Ok(AnalysisOptions {
        agent: AgentOptions {
            kind: agent,
            executable,
            timeout_seconds,
        },
        reviewer: reviewer.map(|kind| AgentOptions {
            kind,
            executable: reviewer_executable,
            timeout_seconds,
        }),
        imported,
        skip_verify,
    })
}

async fn analyze(root: &Path, options: AnalysisOptions) -> Result<()> {
    Bundle::load(root)?;
    let transaction = Transaction::new(root)?;
    if let Err(error) = analyze_staged(transaction.path(), options).await {
        let failed = transaction.preserve_failure();
        return Err(error.context(format!(
            "previous bundle preserved; failed run retained at {}",
            failed.display()
        )));
    }
    transaction.commit(|root| plugin::render(&Bundle::load(root)?, root))?;
    println!("Plugin written to {}", root.join("plugin").display());
    Ok(())
}

async fn analyze_staged(root: &Path, options: AnalysisOptions) -> Result<()> {
    let mut bundle = Bundle::load(root)?;
    let analysis: Analysis = match options.imported {
        Some(path) => serde_json::from_slice(&fs::read(path).context("read imported analysis")?)
            .context("parse imported analysis")?,
        None => adapters::analyze(&bundle, root, &options.agent).await?,
    };
    bundle.analysis = Some(analysis);
    bundle.review = None;
    bundle.verification.clear();
    bundle.validate(root)?;
    bundle.save(root)?;
    if let Some(reviewer) = options.reviewer {
        let review = adapters::review(&bundle, root, &reviewer).await?;
        let approved = review.approved && review.issues.is_empty();
        bundle.review = Some(review);
        bundle.save(root)?;
        ensure!(
            approved,
            "review raised unresolved issues; inspect bundle.json before rendering"
        );
    }
    if !options.skip_verify {
        verify::verify(
            &mut bundle,
            root,
            &VerifyOptions {
                timeout_seconds: options.agent.timeout_seconds,
                offline: false,
            },
        )
        .await?;
    }
    plugin::render(&bundle, root)?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    match Cli::parse().command {
        Commands::Collect {
            name,
            version,
            output,
        } => {
            let bundle = collect::collect(&name, version.as_deref(), &output).await?;
            bundle.save(&output)?;
            println!(
                "Collected {} {} into {}",
                bundle.release.name,
                bundle.release.version,
                output.display()
            );
            for warning in &bundle.warnings {
                eprintln!("Note: {warning}");
            }
        }
        Commands::Build {
            name,
            version,
            output,
            agent,
            agent_executable,
            reviewer,
            reviewer_executable,
            analysis,
            timeout,
            skip_verify,
        } => {
            let opts = options(
                agent,
                agent_executable,
                reviewer,
                reviewer_executable,
                analysis,
                timeout,
                skip_verify,
            )?;
            collect::collect(&name, version.as_deref(), &output)
                .await?
                .save(&output)?;
            analyze(&output, opts).await?;
        }
        Commands::Analyze {
            bundle,
            agent,
            agent_executable,
            reviewer,
            reviewer_executable,
            analysis,
            timeout,
            skip_verify,
        } => {
            analyze(
                &bundle,
                options(
                    agent,
                    agent_executable,
                    reviewer,
                    reviewer_executable,
                    analysis,
                    timeout,
                    skip_verify,
                )?,
            )
            .await?;
        }
        Commands::Verify {
            bundle,
            timeout,
            offline,
        } => {
            let transaction = Transaction::new(&bundle)?;
            // Verification is allowed to refresh stale checks after recipe edits.
            let mut data: Bundle =
                serde_json::from_slice(&fs::read(transaction.path().join("bundle.json"))?)?;
            data.verification.clear();
            data.validate(transaction.path())?;
            if let Err(error) = verify::verify(
                &mut data,
                transaction.path(),
                &VerifyOptions {
                    timeout_seconds: timeout,
                    offline,
                },
            )
            .await
            {
                let failed = transaction.preserve_failure();
                return Err(error.context(format!(
                    "previous bundle preserved; failed checks at {}",
                    failed.display()
                )));
            }
            transaction.commit(|root| plugin::render(&Bundle::load(root)?, root))?;
        }
        Commands::Render { bundle } => {
            plugin::render(&Bundle::load(&bundle)?, &bundle)?;
        }
        Commands::ExportRegistry { bundle, output } => {
            registry::export(&bundle, &output)?;
            println!("Registry entry written to {}", output.display());
        }
        Commands::Install {
            bundle,
            symposium_home,
        } => {
            let home = symposium_home
                .or_else(|| std::env::var_os("SYMPOSIUM_HOME").map(PathBuf::from))
                .or_else(|| std::env::var_os("HOME").map(|p| PathBuf::from(p).join(".symposium")))
                .context("set --symposium-home or SYMPOSIUM_HOME")?;
            let path = plugin::install(&bundle, &home, &std::env::current_exe()?)?;
            println!(
                "Installed {}. Run cargo agents sync in the consuming workspace.",
                path.display()
            );
        }
        Commands::Serve { bundle } => runtime::serve(&bundle).await?,
        Commands::Check { bundle, workspace } => {
            println!(
                "{}",
                serde_json::to_string_pretty(&runtime::check_versions(
                    &Bundle::load(&bundle)?,
                    &workspace
                )?)?
            );
        }
        Commands::Hook { bundle } => runtime::hook(&bundle).await?,
        Commands::Refresh {
            bundle,
            output,
            agent,
            agent_executable,
            timeout,
        } => {
            let old = Bundle::load(&bundle)?;
            if output.exists() && output.canonicalize()? == bundle.canonicalize()? {
                bail!("refresh needs a new output directory");
            }
            let new = collect::collect(&old.release.name, None, &output).await?;
            new.save(&output)?;
            eprintln!(
                "Refreshing {}: {} -> {}",
                old.release.name, old.release.version, new.release.version
            );
            analyze(
                &output,
                options(agent, agent_executable, None, None, None, timeout, false)?,
            )
            .await?;
        }
    }
    Ok(())
}
