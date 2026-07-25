//! `cargo xtask registry` — regenerate `site/registry/index.json` from
//! `site/registry/registry.toml` plus the dictionary files' actual
//! contents (sha256, entry counts).

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Deserialize)]
struct Source {
    #[serde(rename = "dictionary")]
    dictionaries: Vec<SourceDict>,
}

#[derive(Deserialize)]
struct SourceDict {
    name: String,
    language: String,
    kind: String,
    description: String,
    file: String,
    #[serde(default)]
    license: Option<String>,
}

#[derive(Serialize)]
struct Index {
    version: u32,
    dictionaries: Vec<IndexDict>,
}

#[derive(Serialize)]
struct IndexDict {
    name: String,
    language: String,
    kind: String,
    description: String,
    file: String,
    sha256: String,
    entries: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    license: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let task = std::env::args().nth(1).unwrap_or_default();
    match task.as_str() {
        "registry" => registry(),
        _ => {
            eprintln!("usage: cargo xtask registry");
            std::process::exit(2);
        }
    }
}

fn repo_root() -> PathBuf {
    // CARGO_MANIFEST_DIR = <repo>/crates/xtask
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("workspace root")
        .to_path_buf()
}

fn registry() -> anyhow::Result<()> {
    let registry_dir = repo_root().join("site").join("registry");
    let source: Source = toml::from_str(&std::fs::read_to_string(
        registry_dir.join("registry.toml"),
    )?)?;

    let mut dictionaries = Vec::new();
    for d in source.dictionaries {
        let path = registry_dir.join(&d.file);
        let bytes = std::fs::read(&path).map_err(|e| anyhow::anyhow!("{}: {e}", path.display()))?;
        let sha256 = hex(&Sha256::digest(&bytes));
        let entries = String::from_utf8_lossy(&bytes)
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .count();
        println!(
            "{:24} {:>6} entries  sha256 {}",
            d.name,
            entries,
            &sha256[..12]
        );
        dictionaries.push(IndexDict {
            name: d.name,
            language: d.language,
            kind: d.kind,
            description: d.description,
            file: d.file,
            sha256,
            entries,
            license: d.license,
        });
    }

    let index = Index {
        version: 1,
        dictionaries,
    };
    let out = registry_dir.join("index.json");
    std::fs::write(&out, serde_json::to_string_pretty(&index)? + "\n")?;
    println!("wrote {}", out.display());
    Ok(())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
