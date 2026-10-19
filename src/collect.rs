//! Release-pinned, bounded collection of public crate evidence.

use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    fs,
    io::Read,
    path::{Component, Path},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result, ensure};
use flate2::read::GzDecoder;
use reqwest::Client;
use semver::Version;
use serde::Deserialize;

use crate::model::{Bundle, Release, SCHEMA_VERSION, Source, sha256, valid_id};

const MAX_DOWNLOAD: usize = 32 * 1024 * 1024;
const MAX_EXPANDED: u64 = 192 * 1024 * 1024;
const MAX_ARCHIVE_ENTRIES: usize = 30_000;
const MAX_CANDIDATE_BYTES: usize = 16 * 1024 * 1024;
const MAX_FILE_BYTES: usize = 48 * 1024;
const MAX_EVIDENCE_BYTES: usize = 320 * 1024;
const MAX_EVIDENCE_FILES: usize = 48;

#[derive(Debug, Deserialize)]
struct IndexRelease {
    name: String,
    vers: String,
    cksum: String,
    #[serde(default)]
    yanked: bool,
    #[serde(default)]
    features: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    features2: BTreeMap<String, Vec<String>>,
    rust_version: Option<String>,
}

#[derive(Debug)]
struct Candidate {
    path: String,
    kind: &'static str,
    text: String,
    url: String,
    published: bool,
    truncated: bool,
}

#[derive(Default)]
struct ArchiveContents {
    files: BTreeMap<String, Vec<u8>>,
    omitted: usize,
    truncated: BTreeSet<String>,
    special_files: usize,
}

/// Collect sources without running any downloaded code or checking out a branch.
/// Existing nonempty output directories are never replaced.
pub async fn collect(crate_name: &str, version: Option<&str>, root: &Path) -> Result<Bundle> {
    ensure!(
        valid_id(crate_name),
        "crate name must contain lowercase ASCII letters, digits, underscores, or hyphens"
    );
    ensure_empty(root)?;
    let client = Client::builder()
        .user_agent(concat!(
            "symposiast/",
            env!("CARGO_PKG_VERSION"),
            " (public crate documentation collector)"
        ))
        .https_only(true)
        .connect_timeout(Duration::from_secs(30))
        .timeout(Duration::from_secs(120))
        .build()?;
    let index_url = format!("https://index.crates.io/{}", sparse_path(crate_name));
    let index_bytes = download(&client, &index_url, MAX_DOWNLOAD).await?;
    let selected = select_release(std::str::from_utf8(&index_bytes)?, crate_name, version)?;
    let archive_url = format!(
        "https://static.crates.io/crates/{0}/{0}-{1}.crate",
        crate_name, selected.vers
    );
    let archive_bytes = download(&client, &archive_url, MAX_DOWNLOAD).await?;
    verify_checksum(&archive_bytes, &selected.cksum)?;
    let crate_prefix = format!("{crate_name}-{}", selected.vers);
    let published = read_archive(&archive_bytes, Some(&crate_prefix))
        .context("read verified published crate archive")?;
    let manifest = published
        .files
        .get("Cargo.toml")
        .context("published crate has no readable Cargo.toml")?;
    let manifest: toml::Value =
        toml::from_str(std::str::from_utf8(manifest)?).context("parse published Cargo.toml")?;
    let package = manifest
        .get("package")
        .context("published Cargo.toml has no package table")?;
    ensure!(
        package.get("name").and_then(toml::Value::as_str) == Some(crate_name),
        "published package name differs from registry record"
    );
    ensure!(
        package.get("version").and_then(toml::Value::as_str) == Some(selected.vers.as_str()),
        "published package version differs from registry record"
    );
    let repository = package
        .get("repository")
        .and_then(toml::Value::as_str)
        .map(str::to_owned);
    let (commit, package_path) = vcs_identity(&published)?;
    let companion_names: BTreeSet<String> = manifest
        .get("dependencies")
        .and_then(toml::Value::as_table)
        .into_iter()
        .flat_map(|dependencies| dependencies.iter())
        .map(|(name, dependency)| {
            dependency
                .get("package")
                .and_then(toml::Value::as_str)
                .unwrap_or(name)
                .to_owned()
        })
        .collect();
    let mut warnings = vec![
        "Collection is bounded to 48 UTF-8 evidence files, 48 KiB per file, and 320 KiB total; examples and API documentation are sampled, not exhaustive. Rendered documentation and external documentation sites are not crawled.".to_owned(),
    ];
    if published.omitted > 0 {
        warnings.push(format!("Published archive: {} eligible files omitted because they exceeded collection bounds or were not UTF-8 text.", published.omitted));
    }
    if published.special_files > 0 {
        warnings.push(format!("Published archive: {} link or special-file entries were rejected without following or extracting them.", published.special_files));
    }
    if published
        .files
        .get(".cargo_vcs_info.json")
        .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(bytes).ok())
        .and_then(|value| {
            value
                .pointer("/git/dirty")
                .and_then(serde_json::Value::as_bool)
        })
        == Some(true)
    {
        warnings.push("Published VCS metadata marks the working tree dirty: repository snapshot evidence may differ from the published crate; prefer published sources when they disagree.".to_owned());
    }
    let mut candidates = candidates_from(&published, true, |path| {
        format!(
            "https://docs.rs/crate/{crate_name}/{}/source/{path}",
            selected.vers
        )
    });

    if let (Some(repository), Some(commit)) = (&repository, &commit) {
        match github_repository(repository) {
            Some((owner, repo)) => {
                let snapshot_url = format!("https://codeload.github.com/{owner}/{repo}/tar.gz/{commit}");
                match download(&client, &snapshot_url, MAX_DOWNLOAD).await
                    .and_then(|bytes| read_archive(&bytes, None))
                {
                    Ok(snapshot) => {
                        if snapshot.special_files > 0 {
                            warnings.push(format!("Pinned repository archive: {} link or special-file entries were rejected without following or extracting them.", snapshot.special_files));
                        }
                        if snapshot.omitted > 0 {
                            warnings.push(format!("Pinned repository archive: {} eligible files omitted because they exceeded collection bounds or were not UTF-8 text.", snapshot.omitted));
                        }
                        candidates.extend(candidates_from(&snapshot, false, |path| {
                            format!("https://github.com/{owner}/{repo}/blob/{commit}/{path}")
                        }).into_iter().filter(|candidate| relevant_repository_path(&candidate.path, &package_path)
                            || companion_names.contains(candidate.path.split('/').next().unwrap_or(""))));
                    }
                    Err(error) => warnings.push(format!("Pinned repository snapshot unavailable; using published crate evidence: {error:#}")),
                }
            }
            None => warnings.push("Repository is not a supported public GitHub URL; repository-only documentation and examples were not collected.".to_owned()),
        }
    } else {
        warnings.push("No repository URL with a valid published VCS commit was available; repository-only documentation and examples were not collected. Unpinned repository HEAD was not substituted.".to_owned());
    }

    let chosen = choose_evidence(candidates, &mut warnings);
    ensure!(!chosen.is_empty(), "no usable crate evidence collected");
    // Delay writes until required network, integrity, and parsing checks succeed.
    ensure_empty(root)?;
    fs::create_dir_all(root.join("sources"))?;
    let mut sources = Vec::new();
    for (index, candidate) in chosen.into_iter().enumerate() {
        let id = format!("e{:04}", index + 1);
        let path = format!("sources/{id}.txt");
        let bytes = candidate.text.as_bytes();
        fs::write(root.join(&path), bytes)?;
        sources.push(Source {
            id,
            kind: candidate.kind.to_owned(),
            path,
            url: candidate.url,
            sha256: sha256(bytes),
        });
    }
    let mut features = selected.features;
    features.extend(selected.features2);
    let bundle = Bundle {
        schema_version: SCHEMA_VERSION,
        release: Release {
            name: crate_name.to_owned(),
            version: selected.vers,
            checksum: selected.cksum,
            repository,
            repository_commit: commit,
            rust_version: selected.rust_version.or_else(|| {
                package
                    .get("rust-version")
                    .and_then(toml::Value::as_str)
                    .map(str::to_owned)
            }),
            features,
            collected_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)?
                .as_secs()
                .to_string(),
        },
        sources,
        analysis: None,
        review: None,
        verification: Vec::new(),
        warnings,
    };
    bundle.validate(root)?;
    bundle.save(root)?;
    Ok(bundle)
}

fn ensure_empty(root: &Path) -> Result<()> {
    if root.exists() {
        ensure!(
            root.is_dir() && fs::read_dir(root)?.next().is_none(),
            "output already exists and is not empty: {}",
            root.display()
        );
    }
    Ok(())
}

fn sparse_path(name: &str) -> String {
    match name.len() {
        1 => format!("1/{name}"),
        2 => format!("2/{name}"),
        3 => format!("3/{}/{name}", &name[..1]),
        _ => format!("{}/{}/{name}", &name[..2], &name[2..4]),
    }
}

fn select_release(index: &str, crate_name: &str, requested: Option<&str>) -> Result<IndexRelease> {
    let requested = requested
        .map(Version::parse)
        .transpose()
        .context("--version must be an exact semantic version")?;
    let mut matching = Vec::new();
    for line in index.lines().filter(|line| !line.trim().is_empty()) {
        let release: IndexRelease =
            serde_json::from_str(line).context("parse crates.io index record")?;
        ensure!(
            release.name == crate_name,
            "unexpected package in registry index"
        );
        let version = Version::parse(&release.vers)?;
        if release.yanked {
            continue;
        }
        if let Some(requested) = &requested {
            if &version != requested {
                continue;
            }
        } else if !version.pre.is_empty() {
            continue;
        }
        matching.push((version, release));
    }
    matching
        .into_iter()
        .max_by(|a, b| a.0.cmp(&b.0))
        .map(|(_, release)| release)
        .context("no matching non-yanked crate release found (latest defaults to stable releases)")
}

fn verify_checksum(bytes: &[u8], expected: &str) -> Result<()> {
    ensure!(
        expected.len() == 64 && expected.bytes().all(|b| b.is_ascii_hexdigit()),
        "invalid registry checksum"
    );
    ensure!(
        sha256(bytes).eq_ignore_ascii_case(expected),
        "published crate checksum does not match crates.io index"
    );
    Ok(())
}

async fn download(client: &Client, url: &str, limit: usize) -> Result<Vec<u8>> {
    let mut response = client
        .get(url)
        .send()
        .await
        .with_context(|| format!("download {url}"))?
        .error_for_status()
        .with_context(|| format!("download {url}"))?;
    ensure!(
        response
            .content_length()
            .is_none_or(|size| size <= limit as u64),
        "download exceeds {} MiB limit: {url}",
        limit / 1024 / 1024
    );
    let mut output = Vec::new();
    while let Some(chunk) = response.chunk().await? {
        ensure!(
            output.len().saturating_add(chunk.len()) <= limit,
            "download exceeds size limit: {url}"
        );
        output.extend_from_slice(&chunk);
    }
    Ok(output)
}

fn archive_path(path: &Path) -> Result<(String, String)> {
    let path = path.to_str().context("archive path is not UTF-8")?;
    ensure!(
        !path.contains('\\') && !path.starts_with('/') && !path.contains('\0'),
        "unsafe archive path"
    );
    // Check the lexical spelling too: Path::components normalizes internal `.`.
    ensure!(
        path.split('/').all(|part| part != ".." && part != "."),
        "unsafe archive path: {path}"
    );
    ensure!(
        Path::new(path)
            .components()
            .all(|c| matches!(c, Component::Normal(_))),
        "unsafe archive path: {path}"
    );
    let path = path.trim_end_matches('/');
    let (prefix, relative) = path.split_once('/').unwrap_or((path, ""));
    ensure!(
        !prefix.is_empty() && !prefix.contains(':'),
        "unsafe archive root"
    );
    Ok((prefix.to_owned(), relative.to_owned()))
}

fn read_archive(bytes: &[u8], expected_prefix: Option<&str>) -> Result<ArchiveContents> {
    let decoder = GzDecoder::new(bytes);
    let mut archive = tar::Archive::new(decoder.take(MAX_EXPANDED + 1));
    let mut output = ArchiveContents::default();
    let mut root = expected_prefix.map(str::to_owned);
    let mut expanded = 0u64;
    let mut retained = 0usize;
    for (index, entry) in archive.entries()?.enumerate() {
        ensure!(
            index < MAX_ARCHIVE_ENTRIES,
            "archive contains too many entries"
        );
        let mut entry = entry?;
        let (prefix, relative) = archive_path(&entry.path()?)?;
        let entry_type = entry.header().entry_type();
        expanded = expanded
            .checked_add(entry.size())
            .context("archive size overflow")?;
        ensure!(
            expanded <= MAX_EXPANDED,
            "archive expanded size exceeds limit"
        );
        if !entry_type.is_file() && !entry_type.is_dir() {
            output.special_files += 1;
            continue;
        }
        if let Some(root) = &root {
            ensure!(
                &prefix == root,
                "archive contains unexpected root directory"
            );
        } else {
            root = Some(prefix);
        }
        if entry_type.is_dir() || relative.is_empty() {
            continue;
        }
        if classify(&relative).is_none() {
            continue;
        }
        if retained >= MAX_CANDIDATE_BYTES {
            output.omitted += 1;
            continue;
        }
        // Manifests must remain complete to parse package metadata correctly.
        let limit = if relative == "Cargo.toml" {
            512 * 1024
        } else {
            MAX_FILE_BYTES
        };
        let mut content = Vec::new();
        (&mut entry).take(limit as u64).read_to_end(&mut content)?;
        if entry.size() > content.len() as u64 {
            output.truncated.insert(relative.clone());
        }
        if content.contains(&0) {
            output.omitted += 1;
            continue;
        }
        // A prefix may end inside a UTF-8 sequence; retain only complete characters.
        if let Err(error) = std::str::from_utf8(&content) {
            if error.error_len().is_none() && output.truncated.contains(&relative) {
                content.truncate(error.valid_up_to());
            } else {
                output.omitted += 1;
                continue;
            }
        }
        retained += content.len();
        ensure!(
            !output.files.contains_key(&relative),
            "duplicate archive entry: {relative}"
        );
        output.files.insert(relative, content);
    }
    Ok(output)
}

fn classify(path: &str) -> Option<&'static str> {
    let lower = path.to_ascii_lowercase();
    let file = lower.rsplit('/').next()?;
    if file == ".cargo_vcs_info.json" {
        return Some("release_metadata");
    }
    if file == "cargo.toml" || file == "cargo.toml.orig" {
        return Some("manifest");
    }
    if file.starts_with("readme") {
        return Some("readme");
    }
    if [
        "changelog",
        "changes",
        "releases",
        "migration",
        "migrating",
        "upgrade",
    ]
    .iter()
    .any(|prefix| file.starts_with(prefix))
    {
        return Some("changelog");
    }
    let extension = lower.rsplit('.').next().unwrap_or("");
    if !matches!(extension, "md" | "mdx" | "rst" | "txt" | "rs") {
        return None;
    }
    if lower
        .split('/')
        .any(|p| matches!(p, "examples" | "example"))
    {
        return Some("example");
    }
    if lower.split('/').any(|p| matches!(p, "tests" | "test")) {
        return Some("test");
    }
    if extension == "rs" {
        if lower.contains("tutorial")
            || lower.contains("cookbook")
            || lower.split('/').any(|part| part == "_derive")
        {
            return Some("documentation");
        }
        if lower.split('/').any(|p| p == "src") {
            return Some("api");
        }
    }
    if matches!(extension, "md" | "mdx" | "rst") {
        return Some("documentation");
    }
    None
}

fn vcs_identity(published: &ArchiveContents) -> Result<(Option<String>, String)> {
    let Some(bytes) = published.files.get(".cargo_vcs_info.json") else {
        return Ok((None, String::new()));
    };
    let value: serde_json::Value =
        serde_json::from_slice(bytes).context("parse published VCS metadata")?;
    let commit = value
        .pointer("/git/sha1")
        .and_then(serde_json::Value::as_str)
        .filter(|sha| sha.len() == 40 && sha.bytes().all(|c| c.is_ascii_hexdigit()))
        .map(str::to_owned);
    let path = value
        .get("path_in_vcs")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");
    if !path.is_empty() {
        ensure!(
            !path.contains('\\')
                && path
                    .split('/')
                    .all(|part| !part.is_empty() && part != "." && part != "..")
                && Path::new(path)
                    .components()
                    .all(|c| matches!(c, Component::Normal(_))),
            "unsafe path_in_vcs in published metadata"
        );
    }
    Ok((commit, path.to_owned()))
}

fn github_repository(url: &str) -> Option<(String, String)> {
    let url = reqwest::Url::parse(url).ok()?;
    if url.host_str()? != "github.com"
        || !matches!(url.scheme(), "https" | "http")
        || !url.username().is_empty()
        || url.password().is_some()
        || url.port().is_some()
    {
        return None;
    }
    let mut parts = url.path().trim_matches('/').split('/');
    let owner = parts.next()?;
    let repo = parts.next()?.trim_end_matches(".git");
    if parts.next().is_some()
        || [owner, repo].iter().any(|part| {
            part.is_empty()
                || !part
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'-' | b'_' | b'.'))
        })
    {
        return None;
    }
    Some((owner.to_owned(), repo.to_owned()))
}

fn candidates_from(
    archive: &ArchiveContents,
    published: bool,
    url: impl Fn(&str) -> String,
) -> Vec<Candidate> {
    archive
        .files
        .iter()
        .filter_map(|(path, bytes)| {
            let text = std::str::from_utf8(bytes).ok()?;
            if text.trim().is_empty() {
                return None;
            }
            Some(Candidate {
                path: path.clone(),
                kind: classify(path)?,
                text: text.to_owned(),
                url: url(path),
                published,
                truncated: archive.truncated.contains(path),
            })
        })
        .collect()
}

fn relevant_repository_path(path: &str, package_path: &str) -> bool {
    if !package_path.is_empty() && path.starts_with(&format!("{package_path}/")) {
        return true;
    }
    // Monorepos commonly keep examples and changelogs at their root.
    if !path.contains('/') {
        return true;
    }
    matches!(
        path.split('/').next().unwrap_or(""),
        "examples" | "docs" | "doc" | "book" | "guide" | "guides"
    ) || (package_path.is_empty()
        && matches!(path.split('/').next().unwrap_or(""), "src" | "tests"))
}

fn importance(candidate: &Candidate) -> (u8, usize, String) {
    let file = candidate
        .path
        .rsplit('/')
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    let rank = match candidate.kind {
        "manifest" if candidate.published && candidate.path == "Cargo.toml" => 0,
        "release_metadata" if candidate.published => 1,
        "readme" if !candidate.path.contains('/') => 2,
        "changelog" if candidate.path.matches('/').count() <= 1 => 3,
        "api" if file == "lib.rs" && candidate.published => 4,
        "documentation" if candidate.path.contains("tutorial") => 5,
        "documentation"
            if candidate.path.contains("reference")
                || candidate.path.split('/').any(|part| part == "_derive") =>
        {
            6
        }
        _ => 10,
    };
    (
        rank,
        candidate.path.matches('/').count(),
        candidate.path.clone(),
    )
}

fn choose_evidence(mut candidates: Vec<Candidate>, warnings: &mut Vec<String>) -> Vec<Candidate> {
    candidates.sort_by_key(|c| {
        (
            !c.published,
            c.kind == "example" && !c.path.ends_with(".rs"),
            importance(c),
        )
    });
    let mut hashes = BTreeSet::new();
    candidates.retain(|candidate| hashes.insert(sha256(candidate.text.as_bytes())));
    let eligible = candidates.len();
    let mut primary = Vec::new();
    let mut groups: BTreeMap<&str, VecDeque<Candidate>> = BTreeMap::new();
    for candidate in candidates {
        if importance(&candidate).0 < 10 {
            primary.push(candidate);
        } else {
            groups
                .entry(candidate.kind)
                .or_default()
                .push_back(candidate);
        }
    }
    primary.sort_by_key(|c| {
        (
            importance(c).0,
            !c.published,
            importance(c).1,
            c.path.clone(),
        )
    });
    let mut ordered = primary;
    // Interleave examples, documentation, API source, and tests so a large
    // examples tree cannot consume the entire collection budget.
    while groups.values().any(|group| !group.is_empty()) {
        for kind in [
            "example",
            "documentation",
            "api",
            "test",
            "changelog",
            "readme",
            "manifest",
            "release_metadata",
        ] {
            if let Some(candidate) = groups.get_mut(kind).and_then(VecDeque::pop_front) {
                ordered.push(candidate);
            }
        }
    }
    let mut chosen = Vec::new();
    let mut used = 0;
    let mut selected_by_kind: BTreeMap<&str, usize> = BTreeMap::new();
    for mut candidate in ordered {
        if chosen.len() == MAX_EVIDENCE_FILES || used >= MAX_EVIDENCE_BYTES {
            break;
        }
        if matches!(candidate.kind, "manifest" | "readme" | "changelog")
            && selected_by_kind.get(candidate.kind).copied().unwrap_or(0) >= 3
        {
            continue;
        }
        let limit = MAX_FILE_BYTES.min(MAX_EVIDENCE_BYTES - used);
        if candidate.text.len() > limit {
            // Avoid a nearly empty final file; favor complete smaller sources.
            if limit < 1024 {
                continue;
            }
            let mut boundary = limit;
            while !candidate.text.is_char_boundary(boundary) {
                boundary -= 1;
            }
            if let Some(line_boundary) = candidate.text[..boundary].rfind('\n') {
                boundary = line_boundary + 1;
            }
            candidate.text.truncate(boundary);
            candidate.truncated = true;
        }
        if candidate.truncated {
            warnings.push(format!(
                "Evidence is a bounded prefix of {} ({} bytes retained).",
                candidate.url,
                candidate.text.len()
            ));
        }
        used += candidate.text.len();
        *selected_by_kind.entry(candidate.kind).or_default() += 1;
        chosen.push(candidate);
    }
    if eligible > chosen.len() {
        warnings.push(format!("{} distinct eligible source files were omitted by the evidence budget; {} files selected.", eligible - chosen.len(), chosen.len()));
    }
    chosen
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::{Compression, write::GzEncoder};

    fn archive(entries: &[(&str, &[u8], tar::EntryType)]) -> Vec<u8> {
        let encoder = GzEncoder::new(Vec::new(), Compression::default());
        let mut archive = tar::Builder::new(encoder);
        for (path, bytes, entry_type) in entries {
            let mut header = tar::Header::new_gnu();
            header.set_size(bytes.len() as u64);
            header.set_mode(0o644);
            header.set_entry_type(*entry_type);
            header.as_mut_bytes()[..path.len()].copy_from_slice(path.as_bytes());
            header.set_cksum();
            archive.append(&header, *bytes).unwrap();
        }
        archive.into_inner().unwrap().finish().unwrap()
    }

    #[test]
    fn selects_latest_stable_non_yanked_and_exact_prerelease() {
        let rows = [
            ("1.9.0", false),
            ("2.0.0-beta.1", false),
            ("1.10.0", false),
            ("2.0.0", true),
        ];
        let index = rows.into_iter().map(|(version, yanked)| serde_json::json!({"name":"test", "vers":version, "cksum":"0".repeat(64), "yanked":yanked}).to_string()).collect::<Vec<_>>().join("\n");
        assert_eq!(select_release(&index, "test", None).unwrap().vers, "1.10.0");
        assert_eq!(
            select_release(&index, "test", Some("2.0.0-beta.1"))
                .unwrap()
                .vers,
            "2.0.0-beta.1"
        );
        assert!(select_release(&index, "test", Some("2.0.0")).is_err());
        assert!(select_release(&index, "test", Some("^1")).is_err());
    }

    #[test]
    fn verifies_archive_checksum_before_use() {
        assert!(
            verify_checksum(
                b"abc",
                "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
            )
            .is_ok()
        );
        assert!(verify_checksum(b"modified", &sha256(b"abc")).is_err());
        assert!(verify_checksum(b"abc", "invalid").is_err());
    }

    #[test]
    fn rejects_archive_traversal_duplicates_and_wrong_root() {
        for path in [
            "crate/../README.md",
            "crate/./README.md",
            "/crate/README.md",
            "crate\\README.md",
        ] {
            assert!(
                read_archive(
                    &archive(&[(path, b"text", tar::EntryType::Regular)]),
                    Some("crate")
                )
                .is_err(),
                "accepted {path}"
            );
        }
        let duplicate = archive(&[
            ("crate/README.md", b"one", tar::EntryType::Regular),
            ("crate/README.md", b"two", tar::EntryType::Regular),
        ]);
        assert!(read_archive(&duplicate, Some("crate")).is_err());
        assert!(
            read_archive(
                &archive(&[("other/README.md", b"", tar::EntryType::Regular)]),
                Some("crate")
            )
            .is_err()
        );
    }

    #[test]
    fn rejects_link_entries_without_losing_regular_evidence() {
        for kind in [tar::EntryType::Symlink, tar::EntryType::Link] {
            let fixture = archive(&[
                ("crate/README.md", b"", kind),
                (
                    "crate/src/lib.rs",
                    b"//! trusted text",
                    tar::EntryType::Regular,
                ),
            ]);
            let result = read_archive(&fixture, Some("crate")).unwrap();
            assert_eq!(result.special_files, 1);
            assert!(!result.files.contains_key("README.md"));
            assert_eq!(result.files.len(), 1);
        }
    }

    #[test]
    fn accepts_global_pax_metadata_before_snapshot_root() {
        let fixture = archive(&[
            (
                "pax_global_header",
                b"19 comment=fixture\n",
                tar::EntryType::new(b'g'),
            ),
            (
                "snapshot/README.md",
                b"documentation",
                tar::EntryType::Regular,
            ),
        ]);
        let result = read_archive(&fixture, None).unwrap();
        assert_eq!(result.files["README.md"], b"documentation");
    }

    #[test]
    fn bounds_evidence_and_preserves_different_source_categories() {
        let candidates = (0..100)
            .map(|index| Candidate {
                path: format!("source-{index}.rs"),
                kind: ["documentation", "api", "example", "test"][index % 4],
                text: format!("// Source {index}\n{}", "sample\n".repeat(8000)),
                url: format!("https://example.test/source-{index}"),
                published: true,
                truncated: false,
            })
            .collect();
        let mut warnings = Vec::new();
        let selected = choose_evidence(candidates, &mut warnings);
        assert!(selected.len() <= MAX_EVIDENCE_FILES);
        assert!(selected.iter().map(|file| file.text.len()).sum::<usize>() <= MAX_EVIDENCE_BYTES);
        assert!(
            selected
                .iter()
                .all(|file| file.text.len() <= MAX_FILE_BYTES)
        );
        for kind in ["documentation", "api", "example", "test"] {
            assert!(selected.iter().any(|file| file.kind == kind));
        }
        assert!(warnings.iter().any(|warning| warning.contains("omitted")));
    }

    #[test]
    fn keeps_text_metadata_and_ignores_binaries() {
        let fixture = archive(&[
            ("crate/README.md", b"hello", tar::EntryType::Regular),
            ("crate/src/lib.rs", b"//! docs", tar::EntryType::Regular),
            ("crate/image.png", b"binary", tar::EntryType::Regular),
            ("crate/docs/binary.md", b"a\0b", tar::EntryType::Regular),
        ]);
        let collected = read_archive(&fixture, Some("crate")).unwrap();
        assert_eq!(collected.files.len(), 2);
        assert_eq!(collected.files["README.md"], b"hello");
        assert_eq!(collected.omitted, 1);
    }

    #[test]
    fn resolves_monorepo_scope_and_rejects_untrusted_repository_hosts() {
        assert!(relevant_repository_path("axum/src/lib.rs", "axum"));
        assert!(relevant_repository_path(
            "examples/chat/src/main.rs",
            "axum"
        ));
        assert!(!relevant_repository_path("axum-extra/src/lib.rs", "axum"));
        assert!(!relevant_repository_path("unrelated/src/lib.rs", ""));
        assert!(relevant_repository_path("src/lib.rs", ""));
        assert_eq!(
            github_repository("https://github.com/tokio-rs/axum.git"),
            Some(("tokio-rs".into(), "axum".into()))
        );
        assert!(github_repository("https://github.com.evil.test/a/b").is_none());
        assert!(github_repository("https://user@github.com/a/b").is_none());
        assert_eq!(sparse_path("clap"), "cl/ap/clap");
        assert_eq!(sparse_path("syn"), "3/s/syn");
    }

    #[test]
    fn preserves_existing_output() {
        let directory = tempfile::tempdir().unwrap();
        fs::write(directory.path().join("important"), "preserve").unwrap();
        assert!(ensure_empty(directory.path()).is_err());
        assert_eq!(
            fs::read_to_string(directory.path().join("important")).unwrap(),
            "preserve"
        );
    }
}
