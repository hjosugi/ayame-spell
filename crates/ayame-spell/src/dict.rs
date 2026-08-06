//! Shared-dictionary registry client.
//!
//! Dictionaries are published as plain text files behind a JSON index on
//! GitHub Pages. `dict add` downloads versioned bytes into the platform cache
//! (sha256-verified), wires the dictionary into the project config, and writes
//! `ayame-spell.lock` so a whole team resolves identical bytes.

use std::collections::{BTreeSet, HashSet};
use std::io::{IsTerminal, Read};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use anyhow::Context;
use clap::{Subcommand, ValueEnum};
use dialoguer::MultiSelect;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use ayame_spell_core::registry_lock::{split_reference, LockedDictionary, RegistryLock, LOCK_FILE};

use crate::words::{add_to_string_array, remove_from_string_array, replace_registry_reference};

const DEFAULT_REGISTRY: &str = "https://hjosugi.github.io/ayame-spell/registry/index.json";
const INDEX_TTL: Duration = Duration::from_secs(24 * 60 * 60);

#[derive(Subcommand)]
pub enum DictCmd {
    /// List available dictionaries and their install status.
    List {
        /// Emit one JSON array for scripting.
        #[arg(long)]
        json: bool,
        /// Filter by language.
        #[arg(long, value_enum)]
        lang: Option<Language>,
        /// Filter by dictionary kind.
        #[arg(long, value_enum)]
        kind: Option<DictKind>,
    },
    /// Download dictionaries and enable them in the project config.
    Add {
        names: Vec<String>,
        /// Download to the cache only; leave the project config untouched.
        #[arg(long)]
        cache_only: bool,
        /// Filter the interactive picker by language.
        #[arg(long, value_enum)]
        lang: Option<Language>,
        /// Filter the interactive picker by dictionary kind.
        #[arg(long, value_enum)]
        kind: Option<DictKind>,
    },
    /// Search registry names and descriptions.
    Search {
        query: String,
        #[arg(long, value_enum)]
        lang: Option<Language>,
        #[arg(long, value_enum)]
        kind: Option<DictKind>,
        #[arg(long)]
        json: bool,
    },
    /// Show metadata and project status for one dictionary.
    Info {
        name: String,
        #[arg(long)]
        json: bool,
    },
    /// Delete a cached dictionary and disable it in the project config.
    Remove { name: String },
    /// Compare cached dictionaries with the registry and update unlocked ones.
    Update {
        /// Exit with status 1 when an update is available; write nothing.
        #[arg(long)]
        check: bool,
    },
    /// Copy a registry dictionary into the project and rewrite its config
    /// reference for offline use.
    Vendor {
        name: String,
        /// Project-relative destination directory.
        #[arg(long, default_value = "dict")]
        dir: PathBuf,
    },
}

#[derive(Clone, Copy, ValueEnum)]
pub enum Language {
    En,
    Ja,
}

impl Language {
    fn as_str(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Ja => "ja",
        }
    }
}

#[derive(Clone, Copy, ValueEnum)]
pub enum DictKind {
    Wordlist,
    Corrections,
    Variants,
}

impl DictKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Wordlist => "wordlist",
            Self::Corrections => "corrections",
            Self::Variants => "variants",
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
struct Index {
    #[allow(dead_code)]
    version: u32,
    dictionaries: Vec<Entry>,
}

#[derive(Clone, Deserialize, Serialize)]
struct Entry {
    name: String,
    #[serde(default = "default_dictionary_version")]
    version: String,
    language: String,
    kind: String,
    description: String,
    #[serde(default = "default_provenance")]
    provenance: String,
    file: String,
    sha256: String,
    entries: usize,
    #[serde(default)]
    versions: Vec<Release>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    license: Option<String>,
}

#[derive(Clone, Deserialize, Serialize)]
struct Release {
    version: String,
    file: String,
    sha256: String,
    entries: usize,
}

fn default_dictionary_version() -> String {
    "1.0.0".to_string()
}

fn default_provenance() -> String {
    "not specified".to_string()
}

fn registry_url() -> String {
    std::env::var("AYAME_SPELL_REGISTRY").unwrap_or_else(|_| DEFAULT_REGISTRY.to_string())
}

fn fetch_index() -> anyhow::Result<Index> {
    if let Some(index) = read_cached_index(true)? {
        return Ok(index);
    }
    let url = registry_url();
    let fetched = match ureq::get(&url).call() {
        Ok(mut response) => response.body_mut().read_to_string()?,
        Err(error) => {
            if let Some(index) = read_cached_index(false)? {
                eprintln!("warning: cannot refresh registry index ({error}); using cached copy");
                return Ok(index);
            }
            return Err(error).with_context(|| format!("cannot fetch registry index {url}"));
        }
    };
    let index: Index = serde_json::from_str(&fetched)
        .with_context(|| format!("invalid registry index at {url}"))?;
    if let Some(path) = ayame_spell_core::registry_index_cache_path() {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, fetched)?;
    }
    Ok(index)
}

fn read_cached_index(require_fresh: bool) -> anyhow::Result<Option<Index>> {
    let Some(path) = ayame_spell_core::registry_index_cache_path() else {
        return Ok(None);
    };
    if require_fresh {
        let fresh = std::fs::metadata(&path)
            .and_then(|metadata| metadata.modified())
            .ok()
            .and_then(|modified| SystemTime::now().duration_since(modified).ok())
            .is_some_and(|age| age <= INDEX_TTL);
        if !fresh {
            return Ok(None);
        }
    }
    match std::fs::read_to_string(path) {
        Ok(contents) => Ok(Some(
            serde_json::from_str(&contents).context("invalid cached registry index")?,
        )),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

/// Return cached registry candidates without performing network I/O.
pub fn completion_names(prefix: &str, installed_only: bool) -> anyhow::Result<Vec<String>> {
    let prefix = prefix.to_lowercase();
    let installed = installed_names();
    let mut names: Vec<String> = read_cached_index(false)?
        .into_iter()
        .flat_map(|index| index.dictionaries)
        .filter(|entry| !installed_only || installed.contains(&entry.name))
        .map(|entry| entry.name)
        .filter(|name| name.to_lowercase().starts_with(&prefix))
        .collect();
    names.sort();
    Ok(names)
}

fn source_url(file: &str) -> String {
    let base = registry_url();
    let base = base
        .rsplit_once('/')
        .map(|(base, _)| base)
        .unwrap_or(base.as_str());
    format!("{base}/{file}")
}

fn fetch_bytes(url: &str) -> anyhow::Result<Vec<u8>> {
    let mut resp = ureq::get(url)
        .call()
        .with_context(|| format!("cannot fetch {url}"))?;
    let mut buf = Vec::new();
    resp.body_mut()
        .as_reader()
        .take(64 * 1024 * 1024)
        .read_to_end(&mut buf)?;
    Ok(buf)
}

fn cache_path(name: &str) -> anyhow::Result<PathBuf> {
    ayame_spell_core::registry_cache_path(name).context("cannot determine the cache directory")
}

struct SelectedRelease<'a> {
    version: &'a str,
    file: &'a str,
    sha256: &'a str,
    entries: usize,
}

fn select_release<'a>(
    entry: &'a Entry,
    requested_version: Option<&str>,
) -> anyhow::Result<SelectedRelease<'a>> {
    let version = requested_version.unwrap_or(&entry.version);
    if version == entry.version {
        return Ok(SelectedRelease {
            version: &entry.version,
            file: &entry.file,
            sha256: &entry.sha256,
            entries: entry.entries,
        });
    }
    let release = entry
        .versions
        .iter()
        .find(|release| release.version == version)
        .with_context(|| {
            format!(
                "dictionary `{}` has no version `{version}` (current: {})",
                entry.name, entry.version
            )
        })?;
    Ok(SelectedRelease {
        version: &release.version,
        file: &release.file,
        sha256: &release.sha256,
        entries: release.entries,
    })
}

fn download(entry: &Entry, release: &SelectedRelease<'_>) -> anyhow::Result<PathBuf> {
    let url = source_url(release.file);
    let bytes = fetch_bytes(&url)?;
    let digest = hex(&Sha256::digest(&bytes));
    anyhow::ensure!(
        digest == release.sha256.to_lowercase(),
        "checksum mismatch for {} (expected {}, got {digest})",
        entry.name,
        release.sha256
    );
    let path = cache_path(&format!("{}@{}", entry.name, release.version))?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, &bytes)?;
    Ok(path)
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn installed_names() -> HashSet<String> {
    ayame_spell_core::registry_cache_dir()
        .and_then(|directory| std::fs::read_dir(directory).ok())
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| {
            entry
                .path()
                .file_stem()
                .and_then(|stem| stem.to_str())
                .map(|stem| split_reference(stem).0.to_string())
        })
        .collect()
}

fn matches_filters(entry: &Entry, lang: Option<Language>, kind: Option<DictKind>) -> bool {
    lang.map_or(true, |value| entry.language == value.as_str())
        && kind.map_or(true, |value| entry.kind == value.as_str())
}

#[derive(Serialize)]
struct ListedEntry<'a> {
    #[serde(flatten)]
    entry: &'a Entry,
    installed: bool,
}

fn print_entries(entries: &[&Entry], installed: &HashSet<String>, json: bool) {
    if json {
        let records: Vec<ListedEntry<'_>> = entries
            .iter()
            .map(|entry| ListedEntry {
                entry,
                installed: installed.contains(&entry.name),
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&records).unwrap());
        return;
    }
    for entry in entries {
        let mark = if installed.contains(&entry.name) {
            "*"
        } else {
            " "
        };
        println!(
            "{mark} {:20} {:9} {:3} {:12} {:>7}  {}",
            entry.name, entry.version, entry.language, entry.kind, entry.entries, entry.description
        );
    }
}

fn interactive_selection_filtered(
    default_names: &[String],
    lang: Option<Language>,
    kind: Option<DictKind>,
) -> anyhow::Result<Vec<String>> {
    anyhow::ensure!(
        std::io::stdin().is_terminal() && std::io::stderr().is_terminal(),
        "`dict add` without names needs an interactive terminal; use `ayame-spell dict list` and pass names explicitly"
    );
    let index = fetch_index()?;
    let installed = installed_names();
    let defaults: HashSet<&str> = default_names.iter().map(String::as_str).collect();
    let entries: Vec<&Entry> = index
        .dictionaries
        .iter()
        .filter(|entry| matches_filters(entry, lang, kind))
        .collect();
    anyhow::ensure!(
        !entries.is_empty(),
        "no registry dictionaries match the selected filters"
    );
    let labels: Vec<String> = entries
        .iter()
        .map(|entry| {
            let mark = if installed.contains(&entry.name) {
                "installed"
            } else {
                "available"
            };
            format!(
                "{:20} {:3} {:12} {:>7}  {} ({mark})",
                entry.name, entry.language, entry.kind, entry.entries, entry.description
            )
        })
        .collect();
    let preselected: Vec<bool> = entries
        .iter()
        .map(|entry| installed.contains(&entry.name) || defaults.contains(entry.name.as_str()))
        .collect();
    let selected = MultiSelect::new()
        .with_prompt("Select dictionaries to install and enable")
        .items(&labels)
        .defaults(&preselected)
        .report(false)
        .interact()
        .context("dictionary picker needs an interactive terminal")?;
    Ok(selected
        .into_iter()
        .map(|index| entries[index].name.clone())
        .collect())
}

/// Select registry dictionaries interactively, preselecting detected project
/// dictionaries and already-installed entries.
pub fn interactive_selection(default_names: &[String]) -> anyhow::Result<Vec<String>> {
    interactive_selection_filtered(default_names, None, None)
}

/// Download and optionally enable named registry dictionaries.
pub fn install_names(names: &[String], cache_only: bool) -> anyhow::Result<()> {
    if names.is_empty() {
        return Ok(());
    }
    let index = fetch_index()?;
    let cwd = std::env::current_dir()?;
    let loaded = ayame_spell_core::config::discover(&cwd)?;
    let mut lock = RegistryLock::load(&loaded.root)?;
    let mut lock_changed = false;
    for requested in names {
        let (name, explicit_version) = split_reference(requested);
        let entry = index
            .dictionaries
            .iter()
            .find(|entry| entry.name == name)
            .with_context(|| {
                format!("`{name}` is not in the registry (see `ayame-spell dict list`)")
            })?;
        let locked_version = lock.get(name).map(|dictionary| dictionary.version.as_str());
        let release = select_release(entry, explicit_version.or(locked_version))?;
        let path = download(entry, &release)?;
        println!(
            "installed {}@{} ({} entries) -> {}",
            name,
            release.version,
            release.entries,
            path.display()
        );
        if !cache_only {
            lock.upsert(LockedDictionary {
                name: name.to_string(),
                version: release.version.to_string(),
                sha256: release.sha256.to_string(),
                file: release.file.to_string(),
            });
            lock_changed = true;
            if let Some((table, key)) = config_slot(&entry.kind) {
                let reference = explicit_version.map_or_else(
                    || format!("registry:{name}"),
                    |version| format!("registry:{name}@{version}"),
                );
                let config = add_to_string_array(&loaded, table, key, &[reference])?;
                println!("enabled in {} ([{table}].{key})", config.display());
            } else {
                eprintln!(
                    "warning: unknown dictionary kind `{}`; enable it manually",
                    entry.kind
                );
            }
        }
    }
    if lock_changed {
        let path = lock.save(&loaded.root)?;
        println!("locked dictionary bytes in {}", path.display());
    }
    Ok(())
}

fn is_enabled(loaded: &ayame_spell_core::LoadedConfig, entry: &Entry) -> bool {
    let references = match entry.kind.as_str() {
        "wordlist" => &loaded.config.words.dictionaries,
        "corrections" => &loaded.config.corrections.extra,
        "variants" => &loaded.config.japanese.variant_files,
        _ => return false,
    };
    references.iter().any(|reference| {
        reference
            .strip_prefix("registry:")
            .is_some_and(|reference| split_reference(reference).0 == entry.name)
    })
}

fn configured_registry_references<'a>(
    loaded: &'a ayame_spell_core::LoadedConfig,
    kind: &str,
) -> &'a [String] {
    match kind {
        "wordlist" => &loaded.config.words.dictionaries,
        "corrections" => &loaded.config.corrections.extra,
        "variants" => &loaded.config.japanese.variant_files,
        _ => &[],
    }
}

/// Which config array a dictionary kind belongs to.
fn config_slot(kind: &str) -> Option<(&'static str, &'static str)> {
    match kind {
        "wordlist" => Some(("words", "dictionaries")),
        "corrections" => Some(("corrections", "extra")),
        "variants" => Some(("japanese", "variant-files")),
        _ => None,
    }
}

pub fn run(cmd: DictCmd) -> anyhow::Result<i32> {
    match cmd {
        DictCmd::List { json, lang, kind } => {
            let index = fetch_index()?;
            let installed = installed_names();
            let entries: Vec<&Entry> = index
                .dictionaries
                .iter()
                .filter(|entry| matches_filters(entry, lang, kind))
                .collect();
            print_entries(&entries, &installed, json);
            if !json {
                eprintln!("\n* = installed; add with `ayame-spell dict add <name>`");
            }
            Ok(0)
        }
        DictCmd::Add {
            mut names,
            cache_only,
            lang,
            kind,
        } => {
            if names.is_empty() {
                names = interactive_selection_filtered(&[], lang, kind)?;
            }
            install_names(&names, cache_only)?;
            Ok(0)
        }
        DictCmd::Search {
            query,
            lang,
            kind,
            json,
        } => {
            let index = fetch_index()?;
            let query = query.to_lowercase();
            let installed = installed_names();
            let entries: Vec<&Entry> = index
                .dictionaries
                .iter()
                .filter(|entry| matches_filters(entry, lang, kind))
                .filter(|entry| {
                    entry.name.to_lowercase().contains(&query)
                        || entry.description.to_lowercase().contains(&query)
                })
                .collect();
            print_entries(&entries, &installed, json);
            Ok(if entries.is_empty() { 1 } else { 0 })
        }
        DictCmd::Info { name, json } => {
            let index = fetch_index()?;
            let (base_name, requested_version) = split_reference(&name);
            let entry = index
                .dictionaries
                .iter()
                .find(|entry| entry.name == base_name)
                .with_context(|| {
                    format!("`{name}` is not in the registry (see `ayame-spell dict list`)")
                })?;
            let loaded = ayame_spell_core::config::discover(&std::env::current_dir()?)?;
            let lock = RegistryLock::load(&loaded.root)?;
            let locked_version = lock
                .get(base_name)
                .map(|dictionary| dictionary.version.as_str());
            let release = select_release(entry, requested_version.or(locked_version))?;
            let cache = cache_path(&format!("{base_name}@{}", release.version))?;
            let enabled = is_enabled(&loaded, entry);
            let installed = cache.is_file();
            let source = source_url(release.file);
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "name": entry.name,
                        "version": release.version,
                        "available_versions": entry.versions.iter().map(|release| release.version.as_str()).collect::<Vec<_>>(),
                        "language": entry.language,
                        "kind": entry.kind,
                        "description": entry.description,
                        "provenance": entry.provenance,
                        "license": entry.license,
                        "entries": release.entries,
                        "sha256": release.sha256,
                        "cache_path": cache,
                        "installed": installed,
                        "enabled": enabled,
                        "source_url": source,
                    }))?
                );
            } else {
                println!("name:        {}", entry.name);
                println!("version:     {}", release.version);
                println!("language:    {}", entry.language);
                println!("kind:        {}", entry.kind);
                println!("description: {}", entry.description);
                println!("provenance:  {}", entry.provenance);
                println!(
                    "license:     {}",
                    entry.license.as_deref().unwrap_or("not specified")
                );
                println!("entries:      {}", release.entries);
                println!("sha256:       {}", release.sha256);
                println!("cache:        {}", cache.display());
                println!("installed:    {installed}");
                println!("enabled:      {enabled}");
                println!("source:       {source}");
            }
            Ok(0)
        }
        DictCmd::Remove { name } => {
            let (base_name, _) = split_reference(&name);
            let mut did_something = false;
            if let Some(directory) = ayame_spell_core::registry_cache_dir() {
                if let Ok(entries) = std::fs::read_dir(directory) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        let matches = path
                            .file_stem()
                            .and_then(|stem| stem.to_str())
                            .is_some_and(|stem| split_reference(stem).0 == base_name);
                        if matches {
                            std::fs::remove_file(&path)?;
                            println!("removed {}", path.display());
                            did_something = true;
                        }
                    }
                }
            }
            let cwd = std::env::current_dir()?;
            let loaded = ayame_spell_core::config::discover(&cwd)?;
            for (table, key) in [
                ("words", "dictionaries"),
                ("corrections", "extra"),
                ("japanese", "variant-files"),
            ] {
                let references = match (table, key) {
                    ("words", "dictionaries") => &loaded.config.words.dictionaries,
                    ("corrections", "extra") => &loaded.config.corrections.extra,
                    ("japanese", "variant-files") => &loaded.config.japanese.variant_files,
                    _ => unreachable!(),
                };
                let matching: Vec<String> = references
                    .iter()
                    .filter(|reference| {
                        reference
                            .strip_prefix("registry:")
                            .is_some_and(|reference| split_reference(reference).0 == base_name)
                    })
                    .cloned()
                    .collect();
                for reference in matching {
                    if remove_from_string_array(&loaded, table, key, &reference)? {
                        println!("disabled in project config ([{table}].{key})");
                        did_something = true;
                    }
                }
            }
            let mut lock = RegistryLock::load(&loaded.root)?;
            if lock.remove(base_name) {
                lock.save(&loaded.root)?;
                println!("removed {base_name} from {LOCK_FILE}");
                did_something = true;
            }
            if !did_something {
                println!("`{name}` was not installed");
            }
            Ok(0)
        }
        DictCmd::Update { check } => update_dictionaries(check),
        DictCmd::Vendor { name, dir } => vendor(&name, &dir),
    }
}

fn update_dictionaries(check: bool) -> anyhow::Result<i32> {
    let index = fetch_index()?;
    let loaded = ayame_spell_core::config::discover(&std::env::current_dir()?)?;
    let mut lock = RegistryLock::load(&loaded.root)?;
    let mut names: BTreeSet<String> = installed_names().into_iter().collect();
    names.extend(
        lock.dictionaries
            .iter()
            .map(|dictionary| dictionary.name.clone()),
    );
    if names.is_empty() {
        println!("no cached dictionaries to update");
        return Ok(0);
    }

    let mut updates = 0usize;
    let mut lock_changed = false;
    for name in names {
        let Some(entry) = index.dictionaries.iter().find(|entry| entry.name == name) else {
            eprintln!("warning: `{name}` is no longer present in the registry");
            continue;
        };
        let pinned_version = configured_registry_references(&loaded, &entry.kind)
            .iter()
            .filter_map(|reference| reference.strip_prefix("registry:"))
            .find_map(|reference| {
                let (configured_name, version) = split_reference(reference);
                (configured_name == name).then_some(version).flatten()
            });
        if let Some(version) = pinned_version {
            let release = select_release(entry, Some(version))?;
            println!("pinned {name}@{} ({})", release.version, release.sha256);
            continue;
        }

        let current = select_release(entry, None)?;
        let locked = lock.get(&name).cloned();
        let path = cache_path(&format!("{name}@{}", current.version))?;
        let up_to_date = locked.as_ref().is_some_and(|dictionary| {
            dictionary.version == current.version
                && dictionary.sha256.eq_ignore_ascii_case(current.sha256)
                && path.is_file()
                && ayame_spell_core::registry_lock::verify(&path, current.sha256).is_ok()
        });
        if up_to_date {
            println!("up to date {name}@{} ({})", current.version, current.sha256);
            continue;
        }

        updates += 1;
        let old = locked
            .as_ref()
            .map_or("unlocked", |dictionary| dictionary.version.as_str());
        if check {
            println!("update available {name}: {old} -> {}", current.version);
            continue;
        }
        download(entry, &current)?;
        lock.upsert(LockedDictionary {
            name: name.clone(),
            version: current.version.to_string(),
            sha256: current.sha256.to_string(),
            file: current.file.to_string(),
        });
        lock_changed = true;
        println!("updated {name}: {old} -> {}", current.version);
    }

    if lock_changed {
        lock.save(&loaded.root)?;
    }
    Ok(if check && updates > 0 { 1 } else { 0 })
}

fn vendor(name: &str, directory: &Path) -> anyhow::Result<i32> {
    let index = fetch_index()?;
    let (base_name, requested_version) = split_reference(name);
    let entry = index
        .dictionaries
        .iter()
        .find(|entry| entry.name == base_name)
        .with_context(|| {
            format!("`{base_name}` is not in the registry (see `ayame-spell dict list`)")
        })?;
    let loaded = ayame_spell_core::config::discover(&std::env::current_dir()?)?;
    let mut lock = RegistryLock::load(&loaded.root)?;
    let locked_version = lock
        .get(base_name)
        .map(|dictionary| dictionary.version.as_str());
    let release = select_release(entry, requested_version.or(locked_version))?;
    let cache = cache_path(&format!("{base_name}@{}", release.version))?;
    if !cache.is_file() || ayame_spell_core::registry_lock::verify(&cache, release.sha256).is_err()
    {
        download(entry, &release)?;
    }

    let destination_directory = if directory.is_absolute() {
        directory.to_path_buf()
    } else {
        loaded.root.join(directory)
    };
    std::fs::create_dir_all(&destination_directory)?;
    let extension = match entry.kind.as_str() {
        "wordlist" => "txt",
        "corrections" => "tsv",
        "variants" => "toml",
        other => anyhow::bail!("cannot vendor unknown dictionary kind `{other}`"),
    };
    let destination = destination_directory.join(format!("{base_name}.{extension}"));
    std::fs::copy(&cache, &destination).with_context(|| {
        format!(
            "cannot copy {} to {}",
            cache.display(),
            destination.display()
        )
    })?;

    let reference = destination
        .strip_prefix(&loaded.root)
        .unwrap_or(&destination)
        .to_string_lossy()
        .replace('\\', "/");
    let (table, key) = config_slot(&entry.kind)
        .with_context(|| format!("unknown dictionary kind `{}`", entry.kind))?;
    let config = replace_registry_reference(&loaded, table, key, base_name, &reference)?;
    if lock.remove(base_name) {
        lock.save(&loaded.root)?;
    }
    println!(
        "vendored {base_name}@{} -> {}",
        release.version,
        destination.display()
    );
    println!(
        "rewrote [{table}].{key} in {} to `{reference}`",
        config.display()
    );
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn historical_releases_remain_selectable() {
        let entry = Entry {
            name: "fixture".to_string(),
            version: "2.0.0".to_string(),
            language: "en".to_string(),
            kind: "wordlist".to_string(),
            description: "fixture".to_string(),
            provenance: "test".to_string(),
            file: "dicts/fixture-2.txt".to_string(),
            sha256: "22".repeat(32),
            entries: 2,
            versions: vec![
                Release {
                    version: "2.0.0".to_string(),
                    file: "dicts/fixture-2.txt".to_string(),
                    sha256: "22".repeat(32),
                    entries: 2,
                },
                Release {
                    version: "1.0.0".to_string(),
                    file: "dicts/fixture-1.txt".to_string(),
                    sha256: "11".repeat(32),
                    entries: 1,
                },
            ],
            license: Some("MIT".to_string()),
        };

        let current = select_release(&entry, None).unwrap();
        assert_eq!(current.version, "2.0.0");
        let historical = select_release(&entry, Some("1.0.0")).unwrap();
        assert_eq!(historical.file, "dicts/fixture-1.txt");
        assert_eq!(historical.entries, 1);
        assert!(select_release(&entry, Some("0.1.0")).is_err());
    }
}
