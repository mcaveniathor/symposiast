use std::{
    collections::BTreeMap,
    fs,
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

use serde_json::{Value, json};
use symposiast::{
    model::{
        Analysis, Basis, Bundle, CheckStatus, Citation, Finding, Recipe, RecipeCheck, Release,
        SCHEMA_VERSION, Source, recipe_fingerprint, sha256,
    },
    plugin, runtime,
};

fn fixture(root: &Path) -> Bundle {
    fs::create_dir_all(root.join("sources")).unwrap();
    fs::create_dir_all(root.join("checks")).unwrap();
    let text = "Use derive to parse a command line.\nEnable the derive feature.\n";
    fs::write(root.join("sources/guide.txt"), text).unwrap();
    fs::write(root.join("checks/parse.log"), "example compilation passed").unwrap();
    let citations = vec![Citation {
        source_id: "guide".into(),
        start_line: 1,
        end_line: 2,
    }];
    let mut bundle = Bundle {
        schema_version: SCHEMA_VERSION,
        release: Release {
            name: "clap".into(),
            version: "4.5.0".into(),
            checksum: "a".repeat(64),
            repository: Some("https://github.com/clap-rs/clap".into()),
            repository_commit: None,
            rust_version: Some("1.74".into()),
            features: BTreeMap::from([("derive".into(), vec![])]),
            collected_at: "2026-10-19T00:00:00Z".into(),
        },
        sources: vec![Source {
            id: "guide".into(),
            kind: "documentation".into(),
            path: "sources/guide.txt".into(),
            url: "https://docs.rs/clap/4.5.0/clap/".into(),
            sha256: sha256(text.as_bytes()),
        }],
        analysis: Some(Analysis {
            summary: "Parse CLI arguments with the documented derive interface.".into(),
            findings: vec![Finding {
                id: "derive-parsing".into(),
                title: "Parse CLI arguments".into(),
                guidance: "Use the derive interface for typed parsing.".into(),
                basis: Basis::Documented,
                features: vec!["derive".into()],
                citations: citations.clone(),
            }],
            recipes: vec![Recipe {
                id: "parse-cli".into(),
                title: "Parse a CLI".into(),
                description: "A minimal argument parser.".into(),
                code: "fn main() {}".into(),
                default_features: true,
                features: vec!["derive".into()],
                dependencies: BTreeMap::new(),
                citations,
            }],
        }),
        review: None,
        verification: vec![],
        warnings: vec!["Fixture covers only argument parsing.".into()],
    };
    bundle.verification.push(RecipeCheck {
        recipe_id: "parse-cli".into(),
        status: CheckStatus::Compiled,
        rustc_version: "rustc fixture".into(),
        log_path: "checks/parse.log".into(),
        lockfile_sha256: None,
        input_sha256: recipe_fingerprint(&bundle, &bundle.analysis.as_ref().unwrap().recipes[0])
            .unwrap(),
        message: "cargo check passed".into(),
    });
    bundle.save(root).unwrap();
    bundle
}

fn invoke(command: &str, root: &Path, input: &str) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_symposiast"))
        .arg(command)
        .arg(root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

fn lock(root: &Path, versions: &[&str]) {
    let text = versions
        .iter()
        .map(|version| format!("[[package]]\nname = \"clap\"\nversion = \"{version}\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\nchecksum = \"{}\"\n\n", "a".repeat(64)))
        .collect::<String>();
    fs::write(root.join("Cargo.lock"), format!("version = 3\n\n{text}")).unwrap();
}

#[test]
fn plugin_gates_guidance_exactly_but_keeps_version_hook_active() {
    let temp = tempfile::tempdir().unwrap();
    let bundle = fixture(temp.path());
    plugin::render(&bundle, temp.path()).unwrap();
    assert_eq!(
        fs::read(temp.path().join("plugin/Symposium.toml")).unwrap(),
        fs::read(temp.path().join("plugin/SYMPOSIUM.toml")).unwrap()
    );
    let value: toml::Value =
        toml::from_str(&fs::read_to_string(temp.path().join("plugin/Symposium.toml")).unwrap())
            .unwrap();
    assert_eq!(value["depends-on"][0].as_str(), Some("clap"));
    assert_eq!(
        value["skills"][0]["depends-on"][0].as_str(),
        Some("clap==4.5.0")
    );
    assert_eq!(
        value["mcp_servers"][0]["depends-on"][0].as_str(),
        Some("clap==4.5.0")
    );
    assert!(
        value["mcp_servers"][0]["name"]
            .as_str()
            .unwrap()
            .bytes()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == b'-' || ch == b'_')
    );
    assert_eq!(value["hooks"][0]["event"].as_str(), Some("SessionStart"));
    assert!(value["hooks"][0].get("predicates").is_none());
    let skill = fs::read_dir(temp.path().join("plugin/skills"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let guide = fs::read_to_string(skill.join("SKILL.md")).unwrap();
    assert!(guide.contains("references/recipes/parse-cli.md"));
    assert!(guide.contains("runtime behavior has not been tested"));
    assert_eq!(
        fs::read_to_string(skill.join("references/evidence/guide.txt")).unwrap(),
        fs::read_to_string(temp.path().join("sources/guide.txt")).unwrap()
    );
}

#[test]
fn installed_bundle_survives_removal_of_original_and_has_absolute_commands() {
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("research");
    let home = root.path().join("symposium");
    let mut bundle = fixture(&source);
    let project = source.join("verification/parse-cli");
    fs::create_dir_all(project.join("src")).unwrap();
    fs::write(
        project.join("Cargo.toml"),
        "[package]\nname = \"recipe-parse-cli\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
    )
    .unwrap();
    fs::write(project.join("src/main.rs"), "fn main() {}\n").unwrap();
    let lockfile = "version = 3\n[[package]]\nname = \"recipe-parse-cli\"\nversion = \"0.0.0\"\n";
    fs::write(project.join("Cargo.lock"), lockfile).unwrap();
    bundle.verification[0].lockfile_sha256 = Some(sha256(lockfile.as_bytes()));
    bundle.save(&source).unwrap();
    let installed =
        plugin::install(&source, &home, Path::new(env!("CARGO_BIN_EXE_symposiast"))).unwrap();
    assert_eq!(installed, home.join("plugins/clap-4.5.0"));
    plugin::install(&source, &home, Path::new(env!("CARGO_BIN_EXE_symposiast"))).unwrap();
    fs::remove_dir_all(&source).unwrap();
    Bundle::load(&installed.join("bundle")).unwrap();
    let manifest: toml::Value =
        toml::from_str(&fs::read_to_string(installed.join("Symposium.toml")).unwrap()).unwrap();
    assert!(Path::new(manifest["mcp_servers"][0]["command"].as_str().unwrap()).is_absolute());
    assert_eq!(
        manifest["mcp_servers"][0]["args"][1].as_str(),
        installed.join("bundle").to_str()
    );
    assert_eq!(
        manifest["hooks"][0]["args"][1].as_str(),
        installed.join("bundle").to_str()
    );
    assert!(installed.join("bundle/checks/parse.log").is_file());
    assert_eq!(
        fs::read_to_string(installed.join("bundle/verification/parse-cli/Cargo.lock")).unwrap(),
        lockfile
    );
    assert!(
        installed
            .join("bundle/verification/parse-cli/Cargo.toml")
            .is_file()
    );
    assert!(
        installed
            .join("bundle/verification/parse-cli/src/main.rs")
            .is_file()
    );
    let result = invoke(
        "serve",
        &installed.join("bundle"),
        "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"tools/call\",\"params\":{\"name\":\"get_recipe\",\"arguments\":{\"id\":\"parse-cli\"}}}\n",
    );
    assert!(result.status.success());
    let response: Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(response["result"]["isError"], false);
}

#[test]
fn install_preserves_an_unowned_existing_plugin() {
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("research");
    fixture(&source);
    let home = root.path().join("symposium");
    let target = home.join("plugins/clap-4.5.0");
    fs::create_dir_all(&target).unwrap();
    fs::write(target.join("user-file"), "preserve this").unwrap();
    assert!(plugin::install(&source, &home, Path::new(env!("CARGO_BIN_EXE_symposiast"))).is_err());
    assert_eq!(
        fs::read_to_string(target.join("user-file")).unwrap(),
        "preserve this"
    );
}

#[test]
fn rejected_review_cannot_replace_an_existing_rendered_plugin() {
    let root = tempfile::tempdir().unwrap();
    let mut bundle = fixture(root.path());
    plugin::render(&bundle, root.path()).unwrap();
    let manifest = fs::read(root.path().join("plugin/Symposium.toml")).unwrap();
    bundle.review = Some(symposiast::model::Review {
        approved: false,
        issues: vec!["The recipe needs revision.".into()],
    });
    bundle.save(root.path()).unwrap();
    assert!(plugin::render(&bundle, root.path()).is_err());
    assert_eq!(
        fs::read(root.path().join("plugin/Symposium.toml")).unwrap(),
        manifest
    );
    let destination = root.path().join("symposium");
    assert!(
        plugin::install(
            root.path(),
            &destination,
            Path::new(env!("CARGO_BIN_EXE_symposiast"))
        )
        .is_err()
    );
    assert!(!destination.join("plugins/clap-4.5.0").exists());
}

#[test]
fn versions_check_ancestor_lockfile_and_partial_coverage() {
    let root = tempfile::tempdir().unwrap();
    let bundle = fixture(root.path());
    let nested = root.path().join("workspace/member/src");
    fs::create_dir_all(&nested).unwrap();
    assert!(
        runtime::check_versions(&bundle, &nested)
            .unwrap()
            .is_empty()
    );
    lock(root.path(), &["4.5.0"]);
    assert!(
        runtime::check_versions(&bundle, &nested)
            .unwrap()
            .is_empty()
    );
    lock(root.path(), &["4.5.0", "4.6.0", "3.2.0"]);
    let warnings = runtime::check_versions(&bundle, &nested).unwrap();
    assert_eq!(warnings.len(), 1);
    assert!(warnings[0].contains("Partial guidance coverage"));
    assert!(warnings[0].contains("3.2.0, 4.6.0"));
    lock(root.path(), &["4.6.0"]);
    assert!(
        runtime::check_versions(&bundle, &nested).unwrap()[0].contains("Guidance version mismatch")
    );
    fs::write(root.path().join("Cargo.lock"), "this is not TOML").unwrap();
    assert!(runtime::check_versions(&bundle, &nested).is_err());
}

#[test]
fn matching_versions_with_patches_or_wrong_checksums_are_not_claimed_covered() {
    let root = tempfile::tempdir().unwrap();
    let bundle = fixture(root.path());
    let cases = [
        ("", "local/path source"),
        (
            "source = \"git+https://example.invalid/repo#commit\"\n",
            "Git source",
        ),
        (
            "source = \"registry+https://example.invalid/index\"\n",
            "another registry",
        ),
        (
            "source = \"registry+https://github.com/rust-lang/crates.io-index\"\nchecksum = \"wrong\"\n",
            "checksum different",
        ),
    ];
    for (fields, expected) in cases {
        fs::write(
            root.path().join("Cargo.lock"),
            format!("version = 3\n[[package]]\nname = \"clap\"\nversion = \"4.5.0\"\n{fields}"),
        )
        .unwrap();
        let warnings = runtime::check_versions(&bundle, root.path()).unwrap();
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains(expected), "{}", warnings[0]);
    }
}

#[test]
fn mcp_retrieves_cited_evidence_and_handles_protocol_and_tool_errors() {
    let root = tempfile::tempdir().unwrap();
    fixture(root.path());
    let messages = [
        json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"test","version":"1"}}}),
        json!({"jsonrpc":"2.0","method":"notifications/initialized"}),
        json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}),
        json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"search_guidance","arguments":{"query":"derive","limit":1}}}),
        json!({"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"get_recipe","arguments":{"id":"parse-cli"}}}),
        json!({"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"search_guidance","arguments":{"query":"derive","limit":0}}}),
        json!({"jsonrpc":"2.0","id":6,"method":"absent"}),
        json!({"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":"get_release_info","arguments":{}}}),
    ];
    let input = messages
        .iter()
        .map(|message| format!("{message}\n"))
        .collect::<String>()
        + "not json\n";
    let output = invoke("serve", root.path(), &input);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let responses: Vec<Value> = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(responses.len(), 8, "notifications must not receive replies");
    assert_eq!(responses[0]["result"]["protocolVersion"], "2025-03-26");
    assert_eq!(responses[1]["result"]["tools"].as_array().unwrap().len(), 3);
    let search: Value = serde_json::from_str(
        responses[2]["result"]["content"][0]["text"]
            .as_str()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(search["results"].as_array().unwrap().len(), 1);
    assert_eq!(search["truncated"], true);
    assert_eq!(search["results"][0]["evidence"][0]["start_line"], 1);
    assert!(
        search["results"][0]["evidence"][0]["excerpt"]
            .as_str()
            .unwrap()
            .contains("Enable the derive feature")
    );
    let recipe: Value = serde_json::from_str(
        responses[3]["result"]["content"][0]["text"]
            .as_str()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(recipe["verification"]["status"], "compiled");
    assert_eq!(recipe["recipe"]["features"][0], "derive");
    assert_eq!(responses[4]["result"]["isError"], true);
    assert_eq!(responses[5]["error"]["code"], -32601);
    assert_eq!(responses[6]["result"]["isError"], false);
    assert_eq!(responses[7]["error"]["code"], -32700);
}

#[test]
fn hook_warns_in_canonical_shape_and_never_blocks_on_bad_input() {
    let root = tempfile::tempdir().unwrap();
    fixture(root.path());
    let input = json!({"SessionStart":{"session_id":null,"cwd":root.path()}}).to_string();
    lock(root.path(), &["4.6.0"]);
    let output = invoke("hook", root.path(), &input);
    assert!(output.status.success());
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(
        value["SessionStart"]["additionalContext"]
            .as_str()
            .unwrap()
            .contains("4.6.0")
    );
    lock(root.path(), &["4.5.0"]);
    let output = invoke("hook", root.path(), &input);
    assert!(output.status.success());
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout).unwrap(),
        json!({})
    );
    let output = invoke("hook", root.path(), "invalid json");
    assert!(output.status.success());
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout).unwrap(),
        json!({})
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("Version check unavailable"));
}
