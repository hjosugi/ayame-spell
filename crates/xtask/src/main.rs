//! Repository automation:
//! - `cargo xtask registry` regenerates the dictionary registry index.
//! - `cargo xtask completions` regenerates checked-in shell completions.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;

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
        "completions" => completions(),
        _ => {
            eprintln!("usage: cargo xtask <registry|completions>");
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

fn completions() -> anyhow::Result<()> {
    let output_dir = repo_root().join("contrib").join("completions");
    std::fs::create_dir_all(&output_dir)?;

    for (shell, file_name) in [
        ("bash", "ayame-spell.bash"),
        ("zsh", "_ayame-spell"),
        ("fish", "ayame-spell.fish"),
        ("powershell", "_ayame-spell.ps1"),
        ("elvish", "ayame-spell.elv"),
    ] {
        let output = Command::new(cargo())
            .current_dir(repo_root())
            .args([
                "run",
                "--quiet",
                "-p",
                "ayame-spell",
                "--",
                "completions",
                shell,
            ])
            .output()?;

        if !output.status.success() {
            anyhow::bail!(
                "failed to generate {shell} completions:\n{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        let path = output_dir.join(file_name);
        std::fs::write(&path, output.stdout)?;
        println!("wrote {}", path.display());
    }

    Ok(())
}

fn cargo() -> OsString {
    std::env::var_os("CARGO").unwrap_or_else(|| OsString::from("cargo"))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
