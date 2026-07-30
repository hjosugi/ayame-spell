//! Shared-dictionary registry client.
//!
//! Dictionaries are published as plain text files behind a JSON index on
//! GitHub Pages. `dict add` downloads into `~/.cache/ayame-spell/dicts/`
//! (sha256-verified) and wires the dictionary into the project config, so
//! a whole team gets it by committing one config line.

use std::collections::HashSet;
use std::io::{IsTerminal, Read};
use std::path::PathBuf;

use anyhow::Context;
use clap::{Subcommand, ValueEnum};
use dialoguer::MultiSelect;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::words::{add_to_string_array, remove_from_string_array};

const DEFAULT_REGISTRY: &str = "https://hjosugi.github.io/ayame-spell/registry/index.json";

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
    /// Re-download every cached dictionary from the registry.
    Update,
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

#[derive(Deserialize)]
struct Index {
    #[allow(dead_code)]
    version: u32,
    dictionaries: Vec<Entry>,
}

#[derive(Clone, Deserialize, Serialize)]
struct Entry {
    name: String,
    language: String,
    kind: String,
    description: String,
    file: String,
    sha256: String,
    entries: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    license: Option<String>,
}

fn registry_url() -> String {
    std::env::var("AYAME_SPELL_REGISTRY").unwrap_or_else(|_| DEFAULT_REGISTRY.to_string())
}

fn fetch_index() -> anyhow::Result<Index> {
    let url = registry_url();
    let body = ureq::get(&url)
        .call()
        .with_context(|| format!("cannot fetch registry index {url}"))?
        .into_string()?;
    serde_json::from_str(&body).with_context(|| format!("invalid registry index at {url}"))
}

fn source_url(entry: &Entry) -> String {
    let base = registry_url();
    let base = base
        .rsplit_once('/')
        .map(|(base, _)| base)
        .unwrap_or(base.as_str());
    format!("{base}/{}", entry.file)
}

fn fetch_bytes(url: &str) -> anyhow::Result<Vec<u8>> {
    let resp = ureq::get(url)
        .call()
        .with_context(|| format!("cannot fetch {url}"))?;
    let mut buf = Vec::new();
    resp.into_reader()
        .take(64 * 1024 * 1024)
        .read_to_end(&mut buf)?;
    Ok(buf)
}

fn cache_path(name: &str) -> anyhow::Result<PathBuf> {
    ayame_spell_core::registry_cache_path(name).context("cannot determine the cache directory")
}

fn download(entry: &Entry) -> anyhow::Result<PathBuf> {
    let url = source_url(entry);
    let bytes = fetch_bytes(&url)?;
    let digest = hex(&Sha256::digest(&bytes));
    anyhow::ensure!(
        digest == entry.sha256.to_lowercase(),
        "checksum mismatch for {} (expected {}, got {digest})",
        entry.name,
        entry.sha256
    );
    let path = cache_path(&entry.name)?;
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
                .map(str::to_string)
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
            "{mark} {:20} {:3} {:12} {:>7}  {}",
            entry.name, entry.language, entry.kind, entry.entries, entry.description
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
    for name in names {
        let entry = index
            .dictionaries
            .iter()
            .find(|entry| &entry.name == name)
            .with_context(|| {
                format!("`{name}` is not in the registry (see `ayame-spell dict list`)")
            })?;
        let path = download(entry)?;
        println!(
            "installed {} ({} entries) -> {}",
            name,
            entry.entries,
            path.display()
        );
        if !cache_only {
            if let Some((table, key)) = config_slot(&entry.kind) {
                let reference = format!("registry:{name}");
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
    Ok(())
}

fn is_enabled(loaded: &ayame_spell_core::LoadedConfig, entry: &Entry) -> bool {
    let reference = format!("registry:{}", entry.name);
    match entry.kind.as_str() {
        "wordlist" => loaded.config.words.dictionaries.contains(&reference),
        "corrections" => loaded.config.corrections.extra.contains(&reference),
        "variants" => loaded.config.japanese.variant_files.contains(&reference),
        _ => false,
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
            let entry = index
                .dictionaries
                .iter()
                .find(|entry| entry.name == name)
                .with_context(|| {
                    format!("`{name}` is not in the registry (see `ayame-spell dict list`)")
                })?;
            let cache = cache_path(&name)?;
            let loaded = ayame_spell_core::config::discover(&std::env::current_dir()?)?;
            let enabled = is_enabled(&loaded, entry);
            let installed = cache.is_file();
            let source = source_url(entry);
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "name": entry.name,
                        "language": entry.language,
                        "kind": entry.kind,
                        "description": entry.description,
                        "license": entry.license,
                        "entries": entry.entries,
                        "sha256": entry.sha256,
                        "cache_path": cache,
                        "installed": installed,
                        "enabled": enabled,
                        "source_url": source,
                    }))?
                );
            } else {
                println!("name:        {}", entry.name);
                println!("language:    {}", entry.language);
                println!("kind:        {}", entry.kind);
                println!("description: {}", entry.description);
                println!(
                    "license:     {}",
                    entry.license.as_deref().unwrap_or("not specified")
                );
                println!("entries:      {}", entry.entries);
                println!("sha256:       {}", entry.sha256);
                println!("cache:        {}", cache.display());
                println!("installed:    {installed}");
                println!("enabled:      {enabled}");
                println!("source:       {source}");
            }
            Ok(0)
        }
        DictCmd::Remove { name } => {
            let path = cache_path(&name)?;
            let mut did_something = false;
            if path.exists() {
                std::fs::remove_file(&path)?;
                println!("removed {}", path.display());
                did_something = true;
            }
            let cwd = std::env::current_dir()?;
            let loaded = ayame_spell_core::config::discover(&cwd)?;
            let reference = format!("registry:{name}");
            for (table, key) in [
                ("words", "dictionaries"),
                ("corrections", "extra"),
                ("japanese", "variant-files"),
            ] {
                if remove_from_string_array(&loaded, table, key, &reference)? {
                    println!("disabled in project config ([{table}].{key})");
                    did_something = true;
                }
            }
            if !did_something {
                println!("`{name}` was not installed");
            }
            Ok(0)
        }
        DictCmd::Update => {
            let index = fetch_index()?;
            let dir = ayame_spell_core::registry_cache_dir()
                .context("cannot determine the cache directory")?;
            let mut updated = 0;
            for entry in &index.dictionaries {
                if dir.join(format!("{}.txt", entry.name)).exists() {
                    download(entry)?;
                    println!("updated {}", entry.name);
                    updated += 1;
                }
            }
            if updated == 0 {
                println!("no cached dictionaries to update");
            }
            Ok(0)
        }
    }
}
