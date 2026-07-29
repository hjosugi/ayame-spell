//! Repository automation:
//! - `cargo xtask registry` regenerates the dictionary registry index.
//! - `cargo xtask completions` regenerates checked-in shell completions.
//! - `cargo xtask cli-docs` regenerates the EN/JA CLI reference from Clap.

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
        "cli-docs" => cli_docs(),
        _ => {
            eprintln!("usage: cargo xtask <registry|completions|cli-docs>");
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

const EN_CLI_PREAMBLE: &str = r#"## Invocation

With no subcommand, ayame-spell checks the supplied paths or the current
directory. `check` is the explicit equivalent. Global `--write`, `--format`,
and `--threads` options apply to this default invocation; put options after
`check` when using the subcommand.

Exit status and machine-readable fields are documented in
[Exit codes and output formats](./output/).

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

## Generated command help

The sections below are collected recursively from `ayame-spell --help`, so
commands, arguments, defaults, and possible values stay synchronized with
Clap."#;

const JA_CLI_PREAMBLE: &str = r#"## 呼び出し方

サブコマンドを省略すると、指定パスまたはカレントディレクトリをチェックします。
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
| `fix [PATH]...` | 安全な修正をファイルへ適用。 |
| `words collect [PATH]...` | 指摘語を頻度順に収集。 |
| `words add <WORDS>...` | プロジェクトまたはユーザー単語を追加。 |
| `words triage [PATH]...` | 複数の指摘語を対話形式で一括整理。 |
| `dict list` | レジストリ辞書と導入状態を一覧表示。 |
| `dict add <NAMES>...` | 辞書を取得してプロジェクト設定へ追加。 |
| `dict remove <NAME>` | キャッシュを削除して設定から無効化。 |
| `dict update` | キャッシュ済み辞書を再取得。 |
| `init` | 初期設定 `ayame-spell.toml` を作成。 |
| `config` | マージ・既定値適用後の最終設定を表示。 |
| `completions <SHELL>` | シェル補完を標準出力へ生成。 |
| `lsp` | エディター連携用 LSP サーバーを起動。 |

## 引数とオプション

| 引数・オプション | 対象 | 説明 |
| --- | --- | --- |
| `[PATH]...` | 既定、`check`、`fix`、`words collect`、`words triage` | ファイルまたはディレクトリ。 |
| `-w`, `--write` | 既定、`check` | 安全な修正をその場で適用。 |
| `--format <FORMAT>` | 既定、`check` | `human`、`brief`、`json`。既定は `human`。 |
| `-j`, `--threads <THREADS>` | 既定、`check`、`fix` | ワーカースレッド数。省略時は CPU 数。 |
| `--min-count <N>` | `words collect` | N 回以上現れた語だけを出力。既定は 1。 |
| `--plain` | `words collect` | 追記しやすいよう、語だけを出力。 |
| `--json` | `words collect` | `word`、`count`、`kind`、`example` の JSON Lines。 |
| `<WORDS>...` | `words add` | 追加する一つ以上の単語。 |
| `--global` | `words add` | プロジェクト単語ではなくユーザー単語へ追加。 |
| `<NAMES>...` | `dict add` | 取得する一つ以上のレジストリ名。 |
| `--cache-only` | `dict add` | 設定を変更せずキャッシュだけ作成。 |
| `<NAME>` | `dict remove` | 削除するレジストリ名。 |
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
}
