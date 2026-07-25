//! Shared-dictionary registry client.
//!
//! Dictionaries are published as plain text files behind a JSON index on
//! GitHub Pages. `dict add` downloads into `~/.cache/ayame-spell/dicts/`
//! (sha256-verified) and wires the dictionary into the project config, so
//! a whole team gets it by committing one config line.

use std::collections::HashSet;
use std::io::Read;
use std::path::PathBuf;

use anyhow::Context;
use clap::Subcommand;
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::words::{add_to_string_array, remove_from_string_array};

const DEFAULT_REGISTRY: &str = "https://hjosugi.github.io/ayame-spell/registry/index.json";

#[derive(Subcommand)]
pub enum DictCmd {
    /// List available dictionaries and their install status.
    List,
    /// Download dictionaries and enable them in the project config.
    Add {
        #[arg(required = true)]
        names: Vec<String>,
        /// Download to the cache only; leave the project config untouched.
        #[arg(long)]
        cache_only: bool,
    },
    /// Delete a cached dictionary and disable it in the project config.
    Remove { name: String },
    /// Re-download every cached dictionary from the registry.
    Update,
}

#[derive(Deserialize)]
struct Index {
    #[allow(dead_code)]
    version: u32,
    dictionaries: Vec<Entry>,
}

#[derive(Deserialize)]
struct Entry {
    name: String,
    language: String,
    kind: String,
    description: String,
    file: String,
    sha256: String,
    entries: usize,
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
    let base = registry_url();
    let base = base
        .rsplit_once('/')
        .map(|(b, _)| b)
        .unwrap_or(base.as_str());
    let url = format!("{base}/{}", entry.file);
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
        DictCmd::List => {
            let index = fetch_index()?;
            let installed: HashSet<String> = ayame_spell_core::registry_cache_dir()
                .and_then(|d| std::fs::read_dir(d).ok())
                .into_iter()
                .flatten()
                .flatten()
                .filter_map(|e| {
                    e.path()
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .map(str::to_string)
                })
                .collect();
            for d in &index.dictionaries {
                let mark = if installed.contains(&d.name) {
                    "*"
                } else {
                    " "
                };
                println!(
                    "{mark} {:20} {:3} {:12} {:>7}  {}",
                    d.name, d.language, d.kind, d.entries, d.description
                );
            }
            eprintln!("\n* = installed; add with `ayame-spell dict add <name>`");
            Ok(0)
        }
        DictCmd::Add { names, cache_only } => {
            let index = fetch_index()?;
            let cwd = std::env::current_dir()?;
            let loaded = ayame_spell_core::config::discover(&cwd)?;
            for name in &names {
                let entry = index
                    .dictionaries
                    .iter()
                    .find(|d| &d.name == name)
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
                        let cfg = add_to_string_array(&loaded, table, key, &[reference])?;
                        println!("enabled in {} ([{table}].{key})", cfg.display());
                    } else {
                        eprintln!(
                            "warning: unknown dictionary kind `{}`; enable it manually",
                            entry.kind
                        );
                    }
                }
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
