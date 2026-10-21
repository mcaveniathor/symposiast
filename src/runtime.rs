use std::{collections::BTreeSet, fs, path::Path};

use anyhow::{Context, Result, bail, ensure};
use serde::Deserialize;
use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};

use crate::model::{Bundle, Citation, safe_path};

const MAX_REQUEST_BYTES: usize = 1024 * 1024;
const MAX_RESULTS: u64 = 20;

fn rpc_error(id: Value, code: i64, message: impl Into<String>) -> Value {
    json!({"jsonrpc":"2.0", "id":id, "error":{"code":code,"message":message.into()}})
}

fn tools() -> Value {
    json!({"tools":[
        {"name":"search_guidance","description":"Search release-specific findings and recipes. Results include evidence and compile status.","inputSchema":{"type":"object","properties":{"query":{"type":"string","minLength":1,"maxLength":512},"limit":{"type":"integer","minimum":1,"maximum":20,"default":5}},"required":["query"],"additionalProperties":false}},
        {"name":"get_recipe","description":"Retrieve a complete Rust recipe with dependency features, release version, evidence, and compile status.","inputSchema":{"type":"object","properties":{"id":{"type":"string"}},"required":["id"],"additionalProperties":false}},
        {"name":"get_release_info","description":"Inspect the exact crate release, source provenance, review, verification, and coverage limitations.","inputSchema":{"type":"object","properties":{},"additionalProperties":false}}
    ]})
}

fn evidence(bundle: &Bundle, root: &Path, citations: &[Citation]) -> Result<Vec<Value>> {
    citations.iter().map(|citation| {
        let source = bundle.sources.iter().find(|source| source.id == citation.source_id).context("unknown source")?;
        let text = fs::read_to_string(safe_path(root, &source.path)?)?;
        let excerpt = text.lines().skip(citation.start_line - 1).take(citation.end_line - citation.start_line + 1).collect::<Vec<_>>().join("\n");
        let truncated = excerpt.chars().count() > 2048;
        Ok(json!({"source_id":source.id,"url":source.url,"sha256":source.sha256,"start_line":citation.start_line,"end_line":citation.end_line,"excerpt":excerpt.chars().take(2048).collect::<String>(),"excerpt_truncated":truncated}))
    }).collect()
}

fn verification(bundle: &Bundle, id: &str) -> Value {
    bundle.verification.iter().rev().find(|check| check.recipe_id == id).map(|check| json!(check)).unwrap_or_else(|| json!({"recipe_id":id,"status":"unrun","message":"No compilation check has run for this recipe."}))
}

fn tool_call(bundle: &Bundle, root: &Path, params: &Value) -> Result<Value> {
    let name = params
        .get("name")
        .and_then(Value::as_str)
        .context("tool name must be a string")?;
    let empty = json!({});
    let args = params
        .get("arguments")
        .unwrap_or(&empty)
        .as_object()
        .context("arguments must be an object")?;
    let exact_release = json!({"name":bundle.release.name,"version":bundle.release.version});
    match name {
        "search_guidance" => {
            ensure!(
                args.keys().all(|k| k == "query" || k == "limit"),
                "unknown search argument"
            );
            let query = args
                .get("query")
                .and_then(Value::as_str)
                .context("query must be a string")?;
            ensure!(
                !query.trim().is_empty() && query.chars().count() <= 512,
                "query must contain 1 to 512 characters"
            );
            let limit = args
                .get("limit")
                .map(|value| value.as_u64().context("limit must be an integer"))
                .transpose()?
                .unwrap_or(5);
            ensure!(
                (1..=MAX_RESULTS).contains(&limit),
                "limit must be between 1 and 20"
            );
            let terms: Vec<String> = query.split_whitespace().map(str::to_lowercase).collect();
            let score = |text: String| {
                let text = text.to_lowercase();
                terms
                    .iter()
                    .filter(|term| text.contains(term.as_str()))
                    .count()
            };
            let mut matches = Vec::new();
            if let Some(analysis) = &bundle.analysis {
                for finding in &analysis.findings {
                    let score = score(format!(
                        "{} {} {}",
                        finding.title,
                        finding.guidance,
                        finding.features.join(" ")
                    ));
                    if score > 0 {
                        matches.push((score, "finding", finding.id.as_str()));
                    }
                }
                for recipe in &analysis.recipes {
                    let score = score(format!(
                        "{} {} {}",
                        recipe.title,
                        recipe.description,
                        recipe.features.join(" ")
                    ));
                    if score > 0 {
                        matches.push((score, "recipe", recipe.id.as_str()));
                    }
                }
            }
            matches.sort_by(|left, right| {
                right
                    .0
                    .cmp(&left.0)
                    .then_with(|| left.1.cmp(right.1))
                    .then_with(|| left.2.cmp(right.2))
            });
            let total = matches.len();
            let mut results = Vec::new();
            for (score, kind, id) in matches.into_iter().take(limit as usize) {
                let analysis = bundle.analysis.as_ref().context("analysis unavailable")?;
                if kind == "finding" {
                    let finding = analysis
                        .findings
                        .iter()
                        .find(|finding| finding.id == id)
                        .context("finding unavailable")?;
                    results.push(json!({"kind":kind,"score":score,"id":id,"title":finding.title,"guidance":finding.guidance,"basis":finding.basis,"features":finding.features,"evidence":evidence(bundle,root,&finding.citations)?}));
                } else {
                    let recipe = analysis
                        .recipes
                        .iter()
                        .find(|recipe| recipe.id == id)
                        .context("recipe unavailable")?;
                    results.push(json!({"kind":kind,"score":score,"id":id,"title":recipe.title,"description":recipe.description,"default_features":recipe.default_features,"features":recipe.features,"verification":verification(bundle,id),"evidence":evidence(bundle,root,&recipe.citations)?}));
                }
            }
            Ok(
                json!({"release":exact_release,"results":results,"total_matches":total,"truncated":total > limit as usize}),
            )
        }
        "get_recipe" => {
            ensure!(
                args.keys().all(|key| key == "id"),
                "unknown recipe argument"
            );
            let id = args
                .get("id")
                .and_then(Value::as_str)
                .context("id must be a string")?;
            let recipe = bundle
                .analysis
                .as_ref()
                .and_then(|analysis| analysis.recipes.iter().find(|recipe| recipe.id == id))
                .context("recipe not found")?;
            Ok(
                json!({"release":exact_release,"recipe":recipe,"verification":verification(bundle,id),"validation_scope":"Compiled means cargo check succeeded; runtime behavior has not been tested.","evidence":evidence(bundle,root,&recipe.citations)?}),
            )
        }
        "get_release_info" => {
            ensure!(args.is_empty(), "get_release_info takes no arguments");
            Ok(
                json!({"release":bundle.release,"sources":bundle.sources,"example_inventory":bundle.example_inventory,"example_coverage":bundle.analysis.as_ref().map(|analysis| &analysis.example_coverage),"summary":bundle.analysis.as_ref().map(|analysis| &analysis.summary),"review":bundle.review,"verification":bundle.verification,"warnings":bundle.warnings}),
            )
        }
        _ => bail!("unknown tool: {name}"),
    }
}

fn handle_request(bundle: &Bundle, root: &Path, request: Value) -> Option<Value> {
    let id = request.get("id").cloned().unwrap_or(Value::Null);
    if !request.is_object()
        || request.get("jsonrpc").and_then(Value::as_str) != Some("2.0")
        || request.get("method").and_then(Value::as_str).is_none()
        || !(id.is_null() || id.is_string() || id.is_i64() || id.is_u64())
    {
        return Some(rpc_error(Value::Null, -32600, "Invalid JSON-RPC request"));
    }
    // Notifications never receive a response, including cancellation and the
    // initialized notification. Tool execution is read-only and synchronous.
    request.get("id")?;
    let method = request["method"].as_str().unwrap();
    let params = request.get("params").cloned().unwrap_or_else(|| json!({}));
    if !params.is_object() {
        return Some(rpc_error(id, -32602, "params must be an object"));
    }
    let result = match method {
        "initialize" => {
            let Some(requested) = params.get("protocolVersion").and_then(Value::as_str) else {
                return Some(rpc_error(id, -32602, "protocolVersion must be a string"));
            };
            let version = match requested {
                "2024-11-05" | "2025-03-26" | "2025-06-18" | "2025-11-25" => requested,
                _ => "2025-03-26",
            };
            json!({"protocolVersion":version,"capabilities":{"tools":{"listChanged":false}},"serverInfo":{"name":"symposiast","version":env!("CARGO_PKG_VERSION")},"instructions":format!("Guidance applies to {} {} only. Check evidence and verification status before using a recipe.",bundle.release.name,bundle.release.version)})
        }
        "ping" => json!({}),
        "tools/list" => tools(),
        "tools/call" => match tool_call(bundle, root, &params) {
            Ok(value) => {
                json!({"content":[{"type":"text","text":value.to_string()}],"isError":false})
            }
            Err(error) => {
                json!({"content":[{"type":"text","text":error.to_string()}],"isError":true})
            }
        },
        _ => return Some(rpc_error(id, -32601, format!("Method not found: {method}"))),
    };
    Some(json!({"jsonrpc":"2.0","id":id,"result":result}))
}

/// Serve the bundle through newline-delimited MCP stdio. Stdout is protocol only.
pub async fn serve(root: &Path) -> Result<()> {
    let bundle = Bundle::load(root)?;
    let mut input = BufReader::new(tokio::io::stdin());
    let mut output = tokio::io::stdout();
    loop {
        let mut line = Vec::new();
        let length = (&mut input)
            .take(MAX_REQUEST_BYTES as u64 + 1)
            .read_until(b'\n', &mut line)
            .await?;
        if length == 0 {
            break;
        }
        if line.iter().all(u8::is_ascii_whitespace) {
            continue;
        }
        let response = if length > MAX_REQUEST_BYTES {
            Some(rpc_error(Value::Null, -32600, "Request exceeds 1 MiB"))
        } else {
            match serde_json::from_slice(&line) {
                Ok(request) => handle_request(&bundle, root, request),
                Err(_) => Some(rpc_error(Value::Null, -32700, "Parse error")),
            }
        };
        if let Some(response) = response {
            output.write_all(response.to_string().as_bytes()).await?;
            output.write_all(b"\n").await?;
            output.flush().await?;
        }
        if length > MAX_REQUEST_BYTES {
            break;
        }
    }
    Ok(())
}

#[derive(Deserialize)]
struct Lockfile {
    #[serde(default)]
    package: Vec<LockedPackage>,
}
#[derive(Deserialize)]
struct LockedPackage {
    name: String,
    version: String,
    source: Option<String>,
    checksum: Option<String>,
}

/// Compare against the closest ancestor lockfile without resolving or networking.
pub fn check_versions(bundle: &Bundle, workspace: &Path) -> Result<Vec<String>> {
    let workspace = workspace
        .canonicalize()
        .with_context(|| format!("inspect workspace {}", workspace.display()))?;
    let directory = if workspace.is_file() {
        workspace.parent().context("workspace has no parent")?
    } else {
        workspace.as_path()
    };
    let Some(lock_path) = directory
        .ancestors()
        .map(|dir| dir.join("Cargo.lock"))
        .find(|path| path.is_file())
    else {
        return Ok(vec![]);
    };
    let lock: Lockfile = toml::from_str(&fs::read_to_string(&lock_path)?)
        .with_context(|| format!("parse {}", lock_path.display()))?;
    let packages: Vec<_> = lock
        .package
        .iter()
        .filter(|package| package.name == bundle.release.name)
        .collect();
    let versions: BTreeSet<_> = packages
        .iter()
        .map(|package| package.version.as_str())
        .collect();
    if versions.is_empty() {
        return Ok(vec![]);
    }
    let mismatches: Vec<_> = versions
        .iter()
        .copied()
        .filter(|version| *version != bundle.release.version)
        .collect();
    let mut warnings = Vec::new();
    if !mismatches.is_empty() {
        let partial = if versions.contains(bundle.release.version.as_str()) {
            "Partial guidance coverage: "
        } else {
            "Guidance version mismatch: "
        };
        warnings.push(format!(
        "{partial}{} guidance covers {} only; Cargo.lock resolves unsupported versions {}. Do not apply this bundle to those versions. Build matching guidance with `symposiast build {} --version <resolved-version> --output <new-bundle-directory>`.",
        bundle.release.name,
        bundle.release.version,
        mismatches.join(", "),
        bundle.release.name
        ));
    }
    let mut provenance = BTreeSet::new();
    for package in packages
        .iter()
        .filter(|package| package.version == bundle.release.version)
    {
        match package.source.as_deref() {
            Some(
                "registry+https://github.com/rust-lang/crates.io-index"
                | "registry+https://index.crates.io/"
                | "sparse+https://index.crates.io/",
            ) => {
                if package
                    .checksum
                    .as_ref()
                    .is_some_and(|checksum| checksum != &bundle.release.checksum)
                {
                    provenance.insert("a checksum different from the analyzed crates.io release");
                }
            }
            None => {
                provenance.insert("a local/path source");
            }
            Some(source) if source.starts_with("git+") => {
                provenance.insert("a Git source");
            }
            Some(_) => {
                provenance.insert("another registry or source");
            }
        }
    }
    for origin in provenance {
        warnings.push(format!("Source mismatch: Cargo.lock resolves {} {} with {origin}. This guidance covers the published crates.io artifact only; verify the source before applying it.",bundle.release.name,bundle.release.version));
    }
    Ok(warnings)
}

fn hook_response(bundle: &Bundle, payload: &Value) -> Result<Value> {
    let Some(event) = payload.get("SessionStart") else {
        return Ok(json!({}));
    };
    let cwd = event
        .get("cwd")
        .and_then(Value::as_str)
        .context("SessionStart cwd is unavailable")?;
    let warnings = check_versions(bundle, Path::new(cwd))?;
    if warnings.is_empty() {
        Ok(json!({}))
    } else {
        Ok(json!({"SessionStart":{"additionalContext":warnings.join("\n")}}))
    }
}

/// A warning hook must never block the coding session, including when input or
/// the installed bundle cannot be inspected. Diagnostics go only to stderr.
pub async fn hook(root: &Path) -> Result<()> {
    let result = async {
        let mut input = String::new();
        tokio::io::stdin()
            .take(MAX_REQUEST_BYTES as u64 + 1)
            .read_to_string(&mut input)
            .await?;
        ensure!(input.len() <= MAX_REQUEST_BYTES, "hook input exceeds 1 MiB");
        let payload: Value = serde_json::from_str(&input).context("parse hook input")?;
        let bundle = Bundle::load(root)?;
        hook_response(&bundle, &payload)
    }
    .await;
    let response = match result {
        Ok(value) => value,
        Err(error) => {
            eprintln!("Version check unavailable: {error:#}");
            json!({})
        }
    };
    let mut output = tokio::io::stdout();
    output.write_all(response.to_string().as_bytes()).await?;
    output.write_all(b"\n").await?;
    Ok(())
}
