//! Repository automation:
//! - `cargo xtask registry` regenerates the dictionary registry index.
//! - `cargo xtask completions` regenerates checked-in shell completions.
//! - `cargo xtask cli-docs` regenerates the EN/JA CLI reference from Clap.
//! - `cargo xtask rules-docs` regenerates the EN/JA rules reference.
//! - `cargo xtask man` regenerates the checked-in manual page from Clap.

use std::collections::{BTreeSet, HashSet};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::Context;
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
    version: String,
    language: String,
    kind: String,
    description: String,
    provenance: String,
    file: String,
    license: String,
    #[serde(default, rename = "release")]
    releases: Vec<SourceRelease>,
}

#[derive(Deserialize)]
struct SourceRelease {
    version: String,
    file: String,
}

#[derive(Serialize)]
struct Index {
    version: u32,
    dictionaries: Vec<IndexDict>,
}

#[derive(Serialize)]
struct IndexDict {
    name: String,
    version: String,
    language: String,
    kind: String,
    description: String,
    provenance: String,
    file: String,
    sha256: String,
    entries: usize,
    versions: Vec<IndexRelease>,
    license: String,
}

#[derive(Serialize)]
struct IndexRelease {
    version: String,
    file: String,
    sha256: String,
    entries: usize,
}

fn main() -> anyhow::Result<()> {
    let task = std::env::args().nth(1).unwrap_or_default();
    match task.as_str() {
        "registry" => registry(),
        "completions" => completions(),
        "cli-docs" => cli_docs(),
        "rules-docs" => rules_docs(),
        "man" => man_page(),
        "config-schema" => config_schema(),
        _ => {
            eprintln!(
                "usage: cargo xtask <registry|completions|cli-docs|rules-docs|man|config-schema>"
            );
            std::process::exit(2);
        }
    }
}

fn config_schema() -> anyhow::Result<()> {
    let path = repo_root()
        .join("site")
        .join("public")
        .join("schema")
        .join("v1")
        .join("ayame-spell.json");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, ayame_spell_core::config::CONFIG_SCHEMA)?;
    println!("wrote {}", path.display());
    Ok(())
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

    let en_base_file = source
        .dictionaries
        .iter()
        .find(|dictionary| dictionary.name == "en-base")
        .map(|dictionary| registry_dir.join(&dictionary.file))
        .context("registry must contain en-base")?;
    let en_base = wordlist_entries(&en_base_file)?;
    let mut dictionaries = Vec::new();
    let mut names = HashSet::new();
    for d in source.dictionaries {
        anyhow::ensure!(
            names.insert(d.name.clone()),
            "duplicate dictionary name `{}`",
            d.name
        );
        anyhow::ensure!(!d.version.trim().is_empty(), "{} has no version", d.name);
        anyhow::ensure!(
            valid_semver(&d.version),
            "{} has invalid semantic version `{}`",
            d.name,
            d.version
        );
        anyhow::ensure!(
            !d.provenance.trim().is_empty(),
            "{} has no provenance",
            d.name
        );
        anyhow::ensure!(!d.license.trim().is_empty(), "{} has no license", d.name);
        let path = registry_dir.join(&d.file);
        let (sha256, entries) = registry_file_metadata(&path, &d.kind)?;
        lint_registry_file(&d.name, &d.kind, &path, &en_base)?;
        let mut versions = vec![IndexRelease {
            version: d.version.clone(),
            file: d.file.clone(),
            sha256: sha256.clone(),
            entries,
        }];
        let mut version_names = HashSet::from([d.version.clone()]);
        for release in d.releases {
            anyhow::ensure!(
                version_names.insert(release.version.clone()),
                "duplicate version {}@{}",
                d.name,
                release.version
            );
            anyhow::ensure!(
                valid_semver(&release.version),
                "{} has invalid semantic version `{}`",
                d.name,
                release.version
            );
            let release_path = registry_dir.join(&release.file);
            let (release_sha256, release_entries) = registry_file_metadata(&release_path, &d.kind)?;
            lint_registry_file(&d.name, &d.kind, &release_path, &en_base)?;
            versions.push(IndexRelease {
                version: release.version,
                file: release.file,
                sha256: release_sha256,
                entries: release_entries,
            });
        }
        versions.sort_by(|left, right| right.version.cmp(&left.version));
        println!(
            "{:24} {:9} {:>6} entries  sha256 {}",
            d.name,
            d.version,
            entries,
            &sha256[..12]
        );
        dictionaries.push(IndexDict {
            name: d.name,
            version: d.version,
            language: d.language,
            kind: d.kind,
            description: d.description,
            provenance: d.provenance,
            file: d.file,
            sha256,
            entries,
            versions,
            license: d.license,
        });
    }

    let index = Index {
        version: 2,
        dictionaries,
    };
    let out = registry_dir.join("index.json");
    std::fs::write(&out, serde_json::to_string_pretty(&index)? + "\n")?;
    println!("wrote {}", out.display());
    Ok(())
}

fn registry_file_metadata(path: &Path, kind: &str) -> anyhow::Result<(String, usize)> {
    let bytes =
        std::fs::read(path).map_err(|error| anyhow::anyhow!("{}: {error}", path.display()))?;
    let sha256 = hex(&Sha256::digest(&bytes));
    let text =
        String::from_utf8(bytes).with_context(|| format!("{} is not UTF-8", path.display()))?;
    let entries = match kind {
        "wordlist" | "corrections" => text
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .count(),
        "variants" => toml::from_str::<toml::Value>(&text)?
            .get("variants")
            .and_then(toml::Value::as_table)
            .map_or(0, |variants| variants.len()),
        other => anyhow::bail!("unknown dictionary kind `{other}`"),
    };
    Ok((sha256, entries))
}

fn wordlist_entries(path: &Path) -> anyhow::Result<BTreeSet<String>> {
    let text = std::fs::read_to_string(path)?;
    Ok(text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_ascii_lowercase)
        .collect())
}

fn valid_semver(version: &str) -> bool {
    let (without_build, build) = version
        .split_once('+')
        .map_or((version, None), |(version, build)| (version, Some(build)));
    if build.is_some_and(|build| !valid_identifiers(build, false)) {
        return false;
    }
    let (core, prerelease) = without_build
        .split_once('-')
        .map_or((without_build, None), |(core, prerelease)| {
            (core, Some(prerelease))
        });
    if prerelease.is_some_and(|prerelease| !valid_identifiers(prerelease, true)) {
        return false;
    }
    let parts: Vec<&str> = core.split('.').collect();
    parts.len() == 3
        && parts.iter().all(|part| {
            !part.is_empty()
                && part.bytes().all(|byte| byte.is_ascii_digit())
                && (part == &"0" || !part.starts_with('0'))
        })
}

fn valid_identifiers(value: &str, reject_numeric_leading_zero: bool) -> bool {
    !value.is_empty()
        && value.split('.').all(|identifier| {
            !identifier.is_empty()
                && identifier
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
                && (!reject_numeric_leading_zero
                    || !identifier.bytes().all(|byte| byte.is_ascii_digit())
                    || identifier == "0"
                    || !identifier.starts_with('0'))
        })
}

fn lint_registry_file(
    name: &str,
    kind: &str,
    path: &Path,
    en_base: &BTreeSet<String>,
) -> anyhow::Result<()> {
    match kind {
        "wordlist" => lint_wordlist(name, path, en_base),
        "corrections" => lint_corrections(path),
        "variants" => lint_variants(path),
        other => anyhow::bail!("{name} has unknown dictionary kind `{other}`"),
    }
}

fn lint_wordlist(name: &str, path: &Path, en_base: &BTreeSet<String>) -> anyhow::Result<()> {
    let text = std::fs::read_to_string(path)?;
    let entries: Vec<String> = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_ascii_lowercase)
        .collect();
    let mut sorted = entries.clone();
    sorted.sort();
    sorted.dedup();
    anyhow::ensure!(
        entries == sorted,
        "{} must be sorted and deduplicated",
        path.display()
    );
    if name != "en-base" {
        let duplicates: Vec<&str> = entries
            .iter()
            .filter(|entry| en_base.contains(*entry))
            .map(String::as_str)
            .take(10)
            .collect();
        anyhow::ensure!(
            duplicates.is_empty(),
            "{} repeats en-base entries: {}",
            path.display(),
            duplicates.join(", ")
        );
    }
    Ok(())
}

fn lint_corrections(path: &Path) -> anyhow::Result<()> {
    let text = std::fs::read_to_string(path)?;
    let mut typos = BTreeSet::new();
    for (line_number, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (typo, fixes) = line.split_once('\t').with_context(|| {
            format!(
                "{}:{} must be typo<TAB>fix[,fix...]",
                path.display(),
                line_number + 1
            )
        })?;
        anyhow::ensure!(
            !typo.trim().is_empty() && fixes.split(',').any(|fix| !fix.trim().is_empty()),
            "{}:{} has an empty typo or fix",
            path.display(),
            line_number + 1
        );
        anyhow::ensure!(
            typos.insert(typo.trim().to_ascii_lowercase()),
            "{}:{} repeats correction `{}`",
            path.display(),
            line_number + 1,
            typo.trim()
        );
    }
    anyhow::ensure!(
        !typos.is_empty(),
        "{} has no correction entries",
        path.display()
    );
    Ok(())
}

fn lint_variants(path: &Path) -> anyhow::Result<()> {
    let text = std::fs::read_to_string(path)?;
    let value: toml::Value =
        toml::from_str(&text).with_context(|| format!("cannot parse {}", path.display()))?;
    let variants = value
        .get("variants")
        .and_then(toml::Value::as_table)
        .context("variant dictionary must contain a [variants] table")?;
    anyhow::ensure!(
        !variants.is_empty(),
        "{} has no variant entries",
        path.display()
    );
    for (variant, preferred) in variants {
        let preferred = preferred.as_str().with_context(|| {
            format!(
                "{} variant `{variant}` must map to a string",
                path.display()
            )
        })?;
        anyhow::ensure!(
            !variant.trim().is_empty() && !preferred.trim().is_empty() && variant != preferred,
            "{} has invalid variant mapping `{variant}` = `{preferred}`",
            path.display()
        );
    }
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

fn cli_docs() -> anyhow::Result<()> {
    let mut sections = Vec::new();
    collect_cli_help(&[], &mut sections)?;

    let docs = repo_root()
        .join("site")
        .join("src")
        .join("content")
        .join("docs");
    let targets = [
        (
            docs.join("reference").join("cli.md"),
            "CLI reference",
            "Every ayame-spell command and flag, generated from Clap.",
            "This page is generated from the command-line parser. Do not edit it by hand.",
            EN_CLI_PREAMBLE,
        ),
        (
            docs.join("ja").join("reference").join("cli.md"),
            "CLI リファレンス",
            "Clap から生成した ayame-spell の全コマンドと全フラグ。",
            "このページはコマンドラインパーサーから生成しています。直接編集しないでください。",
            JA_CLI_PREAMBLE,
        ),
    ];

    for (path, title, description, notice, preamble) in targets {
        let mut markdown = format!(
            "---\ntitle: {title}\ndescription: {description}\n---\n\n\
             <!-- Generated by cargo xtask cli-docs. -->\n\n{notice}\n\n{preamble}\n"
        );
        for (command, help) in &sections {
            markdown.push_str("\n## `");
            markdown.push_str(command);
            markdown.push_str("`\n\n```text\n$ ");
            markdown.push_str(command);
            markdown.push_str(" --help\n");
            markdown.push_str(help.trim_end());
            markdown.push_str("\n```\n");
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, markdown)?;
        println!("wrote {}", path.display());
    }
    Ok(())
}

fn rules_docs() -> anyhow::Result<()> {
    let docs = repo_root()
        .join("site")
        .join("src")
        .join("content")
        .join("docs");
    for (path, japanese) in [
        (docs.join("reference").join("rules.md"), false),
        (docs.join("ja").join("reference").join("rules.md"), true),
    ] {
        let mut output = if japanese {
            "---\ntitle: ルール一覧\ndescription: ayame-spell の全指摘コードと修正動作。\n---\n\n\
             <!-- Generated by cargo xtask rules-docs. -->\n\n\
             指摘コードは、人向け出力、JSON Lines、GitHub 注釈、SARIF、単語収集、\n\
             LSP 診断で共通して使う安定した機械可読識別子です。\n\n\
             | コード | 説明 | 主な設定 |\n| --- | --- | --- |\n"
                .to_string()
        } else {
            "---\ntitle: Rules reference\ndescription: Every stable ayame-spell issue code and its fix behavior.\n---\n\n\
             <!-- Generated by cargo xtask rules-docs. -->\n\n\
             Issue codes are stable machine-readable identifiers shared by human output,\n\
             JSON Lines, GitHub annotations, SARIF, word collection, and LSP diagnostics.\n\n\
             | Code | Description | Primary configuration |\n| --- | --- | --- |\n"
                .to_string()
        };

        for kind in ayame_spell_core::IssueKind::ALL {
            let info = kind.info(japanese);
            output.push_str(&format!(
                "| [`{0}`](#{0}) | {1} | `{2}` |\n",
                kind.code(),
                markdown_cell(info.summary),
                markdown_cell(info.config_key)
            ));
        }
        for kind in ayame_spell_core::IssueKind::ALL {
            let info = kind.info(japanese);
            output.push_str(&format!(
                "\n## `{}`\n\n{}\n\n{}\n\n- **{}:** `{}`\n- **{}:** `{}`\n- **{}:** {}\n",
                kind.code(),
                info.summary,
                info.explanation,
                if japanese { "設定" } else { "Configuration" },
                info.config_key,
                if japanese { "例" } else { "Example" },
                info.example,
                if japanese {
                    "無視する方法"
                } else {
                    "How to silence"
                },
                info.silence,
            ));
        }
        output.push_str(if japanese {
            "\n## トークンの除外処理\n\n\
             英語ルールの前に、トークナイザーは camelCase と PascalCase を分割し、\
             URL、メールアドレス、16進数、UUID 風、base64 風、長く数字を含む\
             トークン、エスケープ風の単語を除外して `min-word-len` を適用します。\
             これは低ノイズ設計の一部で、個別の指摘コードはありません。\n"
        } else {
            "\n## Token filtering\n\n\
             Before English rules run, the tokenizer splits camelCase and PascalCase, skips\
             URLs, email addresses, hexadecimal, UUID-like, base64-like, long digit-bearing,\
             and escape-like tokens, then applies `min-word-len`. This filtering is part of\
             the low-noise design and does not have a separate issue code.\n"
        });
        std::fs::write(&path, output)?;
        println!("wrote {}", path.display());
    }
    Ok(())
}

fn markdown_cell(value: &str) -> String {
    value.replace('|', "\\|").replace('\n', " ")
}

fn man_page() -> anyhow::Result<()> {
    let mut sections = Vec::new();
    collect_cli_help(&[], &mut sections)?;

    let mut man = format!(
        ".TH AYAME-SPELL 1 \"\" \"ayame-spell {}\" \"User Commands\"\n\
         .SH NAME\n\
         ayame-spell \\- fast, low-noise spell checker for code and prose\n\
         .SH SYNOPSIS\n\
         .B ayame-spell\n\
         .RI [ OPTIONS ]\\ [ PATH ...]\n\
         .br\n\
         .B ayame-spell\n\
         .I COMMAND\n\
         .RI [ ARGS ]\n\
         .SH DESCRIPTION\n\
         ayame-spell checks English and Japanese text in source trees and prose.\n\
         With no command it checks the supplied paths, or the current directory.\n\
         .SH COMMAND HELP\n",
        env!("CARGO_PKG_VERSION")
    );

    for (command, help) in sections {
        man.push_str(".SS ");
        man.push_str(&roff_literal(&command));
        man.push('\n');
        man.push_str(".nf\n");
        man.push_str(&roff_literal(help.trim_end()));
        man.push_str("\n.fi\n");
    }

    man.push_str(
        ".SH FILES\n\
         .TP\n\
         .I ayame-spell.toml\n\
         Project configuration.\n\
         .TP\n\
         .I ayame-words.txt\n\
         Project word list used by the default configuration.\n\
         .SH EXIT STATUS\n\
         0 means no findings, 1 means findings were reported, and 2 means an error occurred.\n\
         .SH DOCUMENTATION\n\
         https://ayame-editor.github.io/ayame-spell/\n\
         .SH AUTHORS\n\
         The ayame-spell contributors.\n",
    );

    let path = repo_root()
        .join("contrib")
        .join("man")
        .join("ayame-spell.1");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, man)?;
    println!("wrote {}", path.display());
    Ok(())
}

fn roff_literal(text: &str) -> String {
    text.lines()
        .map(|line| {
            let escaped = line.replace('\\', "\\e");
            if escaped.starts_with('.') || escaped.starts_with('\'') {
                format!("\\&{escaped}")
            } else {
                escaped
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

const EN_CLI_PREAMBLE: &str = r#"## Invocation

With no subcommand, ayame-spell checks the supplied paths or the current
directory. `check` is the explicit equivalent. Global `--write`, `--format`,
and `--threads` options apply to this default invocation; put options after
`check` when using the subcommand.

Exit status and machine-readable fields are documented in
[Exit codes and output formats](./output/).

## Command overview

| Command | Description |
| --- | --- |
| `ayame-spell [PATH]...` | Check paths, or the current directory when omitted. |
| `check [PATH]...` | Run a check explicitly. |
| `fix [PATH]...` | Apply safe fixes, preview a diff, or review findings interactively. |
| `words collect [PATH]...` | Collect findings ranked by frequency. |
| `words add <WORDS>...` | Add project or user words. |
| `words triage [PATH]...` | Review several findings interactively. |
| `baseline [PATH]...` | Record or prune existing findings for incremental adoption. |
| `dict list` | List registry dictionaries and installation state. |
| `dict add <NAMES>...` | Fetch dictionaries and add them to project config. |
| `dict remove <NAME>` | Delete a cache entry and disable it in config. |
| `dict update` | Check or install registry updates for unlocked dictionaries. |
| `dict vendor <NAME>` | Copy a locked registry dictionary into the project. |
| `import cspell [PATH]` | Import cSpell words, ignores, paths, and known dictionaries. |
| `import typos [PATH]` | Import typos corrections and excluded paths. |
| `import prh <PATH>` | Import the supported prh rule subset into a project rule file. |
| `init` | Create a starter `ayame-spell.toml`. |
| `config` | Print merged config after applying defaults. |
| `explain <CODE>` | Explain a rule and how to configure or silence it. |
| `rules` | List every stable issue code. |
| `completions <SHELL>` | Generate shell completion on standard output. |
| `lsp` | Start the language server for editor integrations. |

## Arguments and options

| Argument or option | Applies to | Description |
| --- | --- | --- |
| `[PATH]...` | default, `check`, `fix`, `words collect`, `words triage` | Files or directories. |
| `-w`, `--write` | default, `check` | Apply safe fixes in place. |
| `--dry-run` | `fix` | Print the exact unified diff without writing. |
| `--interactive` | `fix` | Review each finding before writing. |
| `--format <FORMAT>` | default, `check` | `human`, `brief`, `json`, `github`, or `sarif`; defaults to `github` in GitHub Actions and `human` elsewhere. |
| `--list-rules` | root | List every stable issue code. |
| `--lang <LANG>` | `--list-rules`, `rules`, `explain` | `en` or `ja`; defaults from `LANG`. |
| `-j`, `--threads <THREADS>` | default, `check`, `fix` | Worker count; defaults to CPU count. |
| `--min-count <N>` | `words collect` | Emit words seen at least N times; default 1. |
| `--plain` | `words collect` | Emit words only, ready to append. |
| `--json` | `words collect` | JSON Lines with `word`, `count`, `kind`, `example`. |
| `--kind <KIND>` | `words triage` | Filter to `typo`, `unknown-word`, `en-variant`, or `ja-variant`. |
| `--min-count <N>` | `words triage` | Review words seen at least N times. |
| `--limit <N>` | `words triage` | Review at most N words after filtering. |
| `--no-baseline` | default, `check`, `fix` | Report every finding without baseline suppression. |
| `--no-cache` | default, `check`, `fix`, `baseline` | Disable incremental per-file scan results. |
| `--cache-dir <PATH>` | default, `check`, `fix`, `baseline` | Select a scan-cache directory and enable it explicitly in CI. |
| `--prune` | `baseline` | Remove entries whose finding no longer exists. |
| `--schema` | `config` | Print the versioned configuration JSON Schema. |
| `--validate [PATH]` | `config` | Validate a discovered or explicit configuration file. |
| `<WORDS>...` | `words add` | One or more words to add. |
| `--global` | `words add` | Add user words instead of project words. |
| `<NAMES>...` | `dict add` | One or more registry names to fetch. |
| `--cache-only` | `dict add` | Cache without changing project config. |
| `--registry <URL>` | every `dict` command | Override the registry index URL for this invocation. |
| `<NAME>` | `dict remove` | Registry name to remove. |
| `--check` | `dict update` | Exit 1 when an update is available without writing. |
| `<NAME>` | `dict vendor` | Registry name, optionally pinned as `name@version`. |
| `--dir <DIR>` | `dict vendor` | Project-relative destination directory; default `dict`. |
| `[PATH]` | `import cspell`, `import typos` | Source configuration; each command also discovers its conventional filename. |
| `<PATH>` | `import prh` | Source prh YAML rule file. |
| `--output <PATH>` | `import prh` | Project-relative generated TOML; default `dict/imported-prh.toml`. |
| `--dry-run` | every `import` command | Print merged configuration and generated files without writing. |
| `--force` | `init` | Overwrite an existing config file. |
| `<SHELL>` | `completions` | `bash`, `elvish`, `fish`, `powershell`, or `zsh`. |
| `--stdio` | `lsp` | Client compatibility; transport is always stdio. |
| `-h`, `--help` | every command | Print help. |
| `-V`, `--version` | root | Print the version. |

## Shell completions

Generate a completion from the same Clap command tree:

```sh
# bash
mkdir -p ~/.local/share/bash-completion/completions
ayame-spell completions bash \
  > ~/.local/share/bash-completion/completions/ayame-spell

# zsh
mkdir -p ~/.zfunc
ayame-spell completions zsh > ~/.zfunc/_ayame-spell

# fish
mkdir -p ~/.config/fish/completions
ayame-spell completions fish \
  > ~/.config/fish/completions/ayame-spell.fish
```

For PowerShell, add this to `$PROFILE`:

```powershell
ayame-spell completions powershell | Out-String | Invoke-Expression
```

For Elvish, add this to `~/.config/elvish/rc.elv`:

```sh
eval (ayame-spell completions elvish | slurp)
```

Release archives contain pre-generated files in `completions/`.
Registry names and flagged words are completed through a hidden, cache-only
candidate provider rather than `clap_complete::dynamic`: this keeps the Rust
1.80 toolchain and all five generated shells while guaranteeing that Tab never
performs network I/O. `dict list` / `dict add` refresh the registry-index cache,
and `words collect` refreshes the word cache. An empty cache returns no
candidates immediately.

## Generated command help

The sections below are collected recursively from `ayame-spell --help`, so
commands, arguments, defaults, and possible values stay synchronized with
Clap."#;

const JA_CLI_PREAMBLE: &str = r#"## 呼び出し方

サブコマンドを省略した場合、指定パスまたはカレントディレクトリをチェックします。
`check` は同じ処理を明示するサブコマンドです。省略時には全体オプション
`--write`、`--format`、`--threads` を使い、`check` を書く場合はその後ろへ
オプションを置きます。

終了状態と機械可読フィールドは
[終了コードと出力形式](./output/)を参照してください。

## コマンドの概要

| コマンド | 説明 |
| --- | --- |
| `ayame-spell [PATH]...` | パスをチェック。省略時はカレントディレクトリ。 |
| `check [PATH]...` | チェックを明示的に実行。 |
| `fix [PATH]...` | 安全な修正の適用、diff 表示、指摘ごとの対話確認。 |
| `words collect [PATH]...` | 指摘語を頻度順に収集。 |
| `words add <WORDS>...` | プロジェクトまたはユーザー単語を追加。 |
| `words triage [PATH]...` | 複数の指摘語を対話形式で一括整理。 |
| `baseline [PATH]...` | 段階導入用に既存の指摘を記録または整理。 |
| `dict list` | レジストリ辞書と導入状態を一覧表示。 |
| `dict add <NAMES>...` | 辞書を取得してプロジェクト設定へ追加。 |
| `dict remove <NAME>` | キャッシュを削除して設定から無効化。 |
| `dict update` | pin されていない辞書の更新確認または更新。 |
| `dict vendor <NAME>` | lock 済みレジストリ辞書をプロジェクト内へコピー。 |
| `import cspell [PATH]` | cSpell の単語、無視設定、path、既知の辞書を移行。 |
| `import typos [PATH]` | typos の修正語と除外 path を移行。 |
| `import prh <PATH>` | 対応する prh rule をプロジェクト rule file へ移行。 |
| `init` | 初期設定 `ayame-spell.toml` を作成。 |
| `config` | マージ・既定値適用後の最終設定を表示。 |
| `explain <CODE>` | ルールの理由、設定、無視する方法を説明。 |
| `rules` | 安定した全指摘コードを一覧表示。 |
| `completions <SHELL>` | シェル補完を標準出力へ生成。 |
| `lsp` | エディター連携用 LSP サーバーを起動。 |

## 引数とオプション

| 引数・オプション | 対象 | 説明 |
| --- | --- | --- |
| `[PATH]...` | 既定、`check`、`fix`、`words collect`、`words triage` | ファイルまたはディレクトリ。 |
| `-w`, `--write` | 既定、`check` | 安全な修正をその場で適用。 |
| `--dry-run` | `fix` | 書き込まず、適用予定と一致する unified diff を表示。 |
| `--interactive` | `fix` | 書き込み前に指摘を1件ずつ確認。 |
| `--format <FORMAT>` | 既定、`check` | `human`、`brief`、`json`、`github`、`sarif`。GitHub Actions では `github`、その他では `human` が既定。 |
| `--list-rules` | ルート | 安定した全指摘コードを一覧表示。 |
| `--lang <LANG>` | `--list-rules`、`rules`、`explain` | `en` または `ja`。省略時は `LANG` から選択。 |
| `-j`, `--threads <THREADS>` | 既定、`check`、`fix` | ワーカースレッド数。省略時は CPU 数。 |
| `--min-count <N>` | `words collect` | N 回以上現れた語だけを出力。既定は 1。 |
| `--plain` | `words collect` | 追記しやすいよう、語だけを出力。 |
| `--json` | `words collect` | `word`、`count`、`kind`、`example` の JSON Lines。 |
| `--kind <KIND>` | `words triage` | `typo`、`unknown-word`、`en-variant`、`ja-variant` で絞り込み。 |
| `--min-count <N>` | `words triage` | N 回以上現れた語だけを確認。 |
| `--limit <N>` | `words triage` | 絞り込み後の確認件数を最大 N 語に制限。 |
| `--no-baseline` | 既定、`check`、`fix` | ベースラインで抑制せず全指摘を表示。 |
| `--no-cache` | 既定、`check`、`fix`、`baseline` | ファイル単位の差分スキャンキャッシュを無効化。 |
| `--cache-dir <PATH>` | 既定、`check`、`fix`、`baseline` | キャッシュ配置を指定し、CI でも明示的に有効化。 |
| `--prune` | `baseline` | 現在は存在しない指摘のエントリーを除去。 |
| `--schema` | `config` | バージョン固定の設定 JSON Schema を出力。 |
| `--validate [PATH]` | `config` | 検出または指定した設定ファイルを検証。 |
| `<WORDS>...` | `words add` | 追加する一つ以上の単語。 |
| `--global` | `words add` | プロジェクト単語ではなくユーザー単語へ追加。 |
| `<NAMES>...` | `dict add` | 取得する一つ以上のレジストリ名。 |
| `--cache-only` | `dict add` | 設定を変更せずキャッシュだけ作成。 |
| `--registry <URL>` | 全 `dict` コマンド | この実行だけレジストリ索引 URL を差し替え。 |
| `<NAME>` | `dict remove` | 削除するレジストリ名。 |
| `--check` | `dict update` | 書き込まず、更新があれば終了コード 1。 |
| `<NAME>` | `dict vendor` | レジストリ名。`name@version` の pin も指定可能。 |
| `--dir <DIR>` | `dict vendor` | プロジェクト相対の配置先。既定は `dict`。 |
| `[PATH]` | `import cspell`、`import typos` | 移行元設定。省略時は各 tool の標準 file 名を検出。 |
| `<PATH>` | `import prh` | 移行元の prh YAML rule file。 |
| `--output <PATH>` | `import prh` | プロジェクト相対の生成 TOML。既定は `dict/imported-prh.toml`。 |
| `--dry-run` | 全 `import` コマンド | 書き込まず、merge 後の設定と生成 file を表示。 |
| `--force` | `init` | 既存の設定ファイルを上書き。 |
| `<SHELL>` | `completions` | `bash`、`elvish`、`fish`、`powershell`、`zsh`。 |
| `--stdio` | `lsp` | クライアント互換用。通信は常に標準入出力。 |
| `-h`, `--help` | 全コマンド | ヘルプを表示。 |
| `-V`, `--version` | ルート | バージョンを表示。 |

## シェル補完

同じ Clap コマンドツリーから補完を生成します。

```sh
# bash
mkdir -p ~/.local/share/bash-completion/completions
ayame-spell completions bash \
  > ~/.local/share/bash-completion/completions/ayame-spell

# zsh
mkdir -p ~/.zfunc
ayame-spell completions zsh > ~/.zfunc/_ayame-spell

# fish
mkdir -p ~/.config/fish/completions
ayame-spell completions fish \
  > ~/.config/fish/completions/ayame-spell.fish
```

PowerShell は次を `$PROFILE` に追加します。

```powershell
ayame-spell completions powershell | Out-String | Invoke-Expression
```

Elvish は次を `~/.config/elvish/rc.elv` に追加します。

```sh
eval (ayame-spell completions elvish | slurp)
```

リリースアーカイブには生成済みファイルが `completions/` に入っています。
辞書名と指摘語の候補には `clap_complete::dynamic` ではなく、非公開の
キャッシュ専用候補プロバイダーを採用しました。これにより Rust 1.80 と5種類の
生成シェルを維持しつつ、Tab 入力時のネットワークアクセスを禁止できます。
`dict list` / `dict add` が辞書索引キャッシュを、`words collect` が単語
キャッシュを更新します。空のキャッシュでは候補なしとして即座に戻ります。

## 生成されたコマンドヘルプ

以下は `ayame-spell --help` から全階層を再帰的に収集した実際の出力です。
コマンド、引数、既定値、選択肢は常に Clap と同期します。組み込みヘルプの表示言語
は英語ですが、正確な構文は上の日本語表と対応しています。"#;

fn collect_cli_help(
    command_path: &[String],
    sections: &mut Vec<(String, String)>,
) -> anyhow::Result<()> {
    let help = cli_help(command_path)?;
    let command = std::iter::once("ayame-spell")
        .chain(command_path.iter().map(String::as_str))
        .collect::<Vec<_>>()
        .join(" ");
    let children = help_subcommands(&help);
    sections.push((command, help));
    for child in children {
        let mut path = command_path.to_vec();
        path.push(child);
        collect_cli_help(&path, sections)?;
    }
    Ok(())
}

fn cli_help(command_path: &[String]) -> anyhow::Result<String> {
    let output = Command::new(cargo())
        .current_dir(repo_root())
        .args(["run", "--quiet", "-p", "ayame-spell", "--"])
        .args(command_path)
        .arg("--help")
        .output()?;
    if !output.status.success() {
        anyhow::bail!(
            "failed to generate help for `ayame-spell {}`:\n{}",
            command_path.join(" "),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(normalize_help(&String::from_utf8(output.stdout)?))
}

fn normalize_help(help: &str) -> String {
    let mut normalized = help
        .lines()
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n");
    normalized.push('\n');
    normalized
}

fn help_subcommands(help: &str) -> Vec<String> {
    let mut in_commands = false;
    let mut commands = Vec::new();
    for line in help.lines() {
        if line == "Commands:" {
            in_commands = true;
            continue;
        }
        if !in_commands {
            continue;
        }
        if line.is_empty() {
            continue;
        }
        let trimmed = line.trim_start();
        let indent = line.len() - trimmed.len();
        if indent == 0 {
            break;
        }
        if indent == 2 {
            if let Some(name) = trimmed.split_whitespace().next() {
                if name != "help" {
                    commands.push(name.to_string());
                }
            }
        }
    }
    commands
}

fn cargo() -> OsString {
    std::env::var_os("CARGO").unwrap_or_else(|| OsString::from("cargo"))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discovers_subcommands_from_clap_help() {
        let help = "Commands:\n  check  Check files\n  words  Word management\n\nOptions:\n";
        assert_eq!(help_subcommands(help), ["check", "words"]);
    }

    #[test]
    fn generated_help_has_no_trailing_whitespace() {
        assert_eq!(
            normalize_help("Usage: tool  \n  path  \n"),
            "Usage: tool\n  path\n"
        );
    }

    #[test]
    fn roff_literal_escapes_control_lines_and_backslashes() {
        assert_eq!(
            roff_literal(".control\n'control\nC:\\path"),
            "\\&.control\n\\&'control\nC:\\epath"
        );
    }
}
