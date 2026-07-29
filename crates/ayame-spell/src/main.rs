mod check;
mod dict;
mod lsp;
mod words;

use std::{io::Write, path::PathBuf};

use clap::{CommandFactory, Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(
    name = "ayame-spell",
    version,
    about = "Fast, low-noise spell checker for code and prose — English & Japanese",
    long_about = "Fast, low-noise spell checker for code and prose — English & Japanese.\n\
                  Running with no subcommand checks the given paths (default: current directory)."
)]
struct Cli {
    #[command(subcommand)]
    cmd: Option<Cmd>,

    /// Files or directories to check.
    #[arg(value_name = "PATH")]
    paths: Vec<PathBuf>,

    /// Apply safe fixes in place (shorthand for `fix`).
    #[arg(short, long)]
    write: bool,

    /// Output format.
    #[arg(long, value_enum, default_value_t = Format::Human)]
    format: Format,

    /// Worker threads (default: number of CPUs).
    #[arg(long, short = 'j')]
    threads: Option<usize>,
}

#[derive(Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Format {
    Human,
    Brief,
    Json,
}

#[derive(Subcommand)]
enum Cmd {
    /// Check files and report issues (the default).
    Check {
        #[arg(value_name = "PATH")]
        paths: Vec<PathBuf>,
        /// Apply safe fixes in place.
        #[arg(short, long)]
        write: bool,
        #[arg(long, value_enum, default_value_t = Format::Human)]
        format: Format,
        #[arg(long, short = 'j')]
        threads: Option<usize>,
    },
    /// Apply all safe fixes in place (single-candidate corrections and
    /// mechanical notation conversions).
    Fix {
        #[arg(value_name = "PATH")]
        paths: Vec<PathBuf>,
        #[arg(long, short = 'j')]
        threads: Option<usize>,
    },
    /// Word management: bulk collection, triage, and dictionary additions.
    Words {
        #[command(subcommand)]
        cmd: words::WordsCmd,
    },
    /// Shared dictionaries from the ayame-spell registry.
    Dict {
        #[command(subcommand)]
        cmd: dict::DictCmd,
    },
    /// Write a starter ayame-spell.toml in the current directory.
    Init {
        /// Overwrite an existing config file.
        #[arg(long)]
        force: bool,
    },
    /// Print the effective merged configuration.
    Config,
    /// Generate a shell completion script on standard output.
    Completions {
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
    /// Run the LSP server (used by editor integrations).
    Lsp {
        /// Use standard input/output transport. Accepted for client
        /// compatibility; stdio is always the transport.
        #[arg(long)]
        stdio: bool,
    },
}

fn main() {
    // Die quietly on a closed pipe (`ayame-spell check | head`) instead of
    // panicking mid-report.
    #[cfg(unix)]
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }
    let cli = Cli::parse();
    let result = match cli.cmd {
        None => check::run(cli.paths, cli.write, cli.format, cli.threads),
        Some(Cmd::Check {
            paths,
            write,
            format,
            threads,
        }) => check::run(paths, write, format, threads),
        Some(Cmd::Fix { paths, threads }) => check::run(paths, true, Format::Human, threads),
        Some(Cmd::Words { cmd }) => words::run(cmd),
        Some(Cmd::Dict { cmd }) => dict::run(cmd),
        Some(Cmd::Init { force }) => init(force),
        Some(Cmd::Config) => print_config(),
        Some(Cmd::Completions { shell }) => print_completions(shell),
        Some(Cmd::Lsp { stdio: _ }) => lsp::run(),
    };
    match result {
        Ok(code) => std::process::exit(code),
        Err(e) => {
            eprintln!("error: {e:#}");
            std::process::exit(2);
        }
    }
}

fn print_completions(shell: clap_complete::Shell) -> anyhow::Result<i32> {
    std::io::stdout().write_all(&completion_script(shell)?)?;
    Ok(0)
}

fn completion_script(shell: clap_complete::Shell) -> anyhow::Result<Vec<u8>> {
    let mut command = Cli::command();
    let name = command.get_name().to_owned();
    let mut output = Vec::new();
    clap_complete::generate(shell, &mut command, name, &mut output);

    if shell == clap_complete::Shell::PowerShell {
        let script = String::from_utf8(output)?;
        output = add_powershell_value_completions(script)?.into_bytes();
    } else if shell == clap_complete::Shell::Elvish {
        let script = String::from_utf8(output)?;
        output = add_elvish_value_completions(script)?.into_bytes();
    }

    Ok(output)
}

fn add_powershell_value_completions(script: String) -> anyhow::Result<String> {
    const ANCHOR: &str = "    $command = @(\n";
    const VALUE_COMPLETIONS: &str = r#"    $lastElement = $commandElements[$commandElements.Count - 1]
    $valueFor = if ($lastElement.Value -in @('--format', 'completions')) {
        $lastElement.Value
    } elseif ($commandElements.Count -ge 3 -and
              $lastElement.Value -eq $wordToComplete) {
        $commandElements[$commandElements.Count - 2].Value
    }

    $values = switch ($valueFor) {
        '--format' { @('human', 'brief', 'json') }
        'completions' { @('bash', 'elvish', 'fish', 'powershell', 'zsh') }
    }
    if ($null -ne $values) {
        $values |
            Where-Object { $_ -like "$wordToComplete*" } |
            ForEach-Object {
                [CompletionResult]::new(
                    $_, $_, [CompletionResultType]::ParameterValue, $_)
            }
        return
    }

"#;

    if !script.contains(ANCHOR) {
        anyhow::bail!("unexpected PowerShell completion template");
    }
    Ok(script.replacen(ANCHOR, &(VALUE_COMPLETIONS.to_owned() + ANCHOR), 1))
}

fn add_elvish_value_completions(script: String) -> anyhow::Result<String> {
    const ANCHOR: &str = "    $completions[$command]\n}\n";
    const VALUE_COMPLETIONS: &str = r#"    if (eq $words[-2] --format) {
        cand human 'Output format'
        cand brief 'Output format'
        cand json 'Output format'
    } elif (eq $words[-2] completions) {
        cand bash 'Shell'
        cand elvish 'Shell'
        cand fish 'Shell'
        cand powershell 'Shell'
        cand zsh 'Shell'
    } else {
        $completions[$command]
    }
}
"#;

    if !script.contains(ANCHOR) {
        anyhow::bail!("unexpected Elvish completion template");
    }
    Ok(script.replacen(ANCHOR, VALUE_COMPLETIONS, 1))
}

const INIT_TEMPLATE: &str = r#"# ayame-spell configuration
# Reference: https://hjosugi.github.io/ayame-spell/reference/configuration/

[check]
# "corrections": flag only known misspellings (near-zero false positives).
# "dictionary":  also flag words missing from the active wordlists.
mode = "corrections"

[words]
# Team-shared word file, committed to git. Editor quick fixes and
# `ayame-spell words add` append here.
project = "ayame-words.txt"
# Words never flagged: ignore = ["exmaple"]
ignore = []
# Wordlists for dictionary mode; install with `ayame-spell dict add <name>`.
dictionaries = []

[japanese]
enabled = true
# "consistency": flag only intra-document 表記ゆれ (default)
# "long":        enforce サーバー style   "short": enforce サーバ style
katakana-style = "consistency"

# Per-glob overrides:
# [[overrides]]
# paths = ["docs/**"]
# mode = "dictionary"
"#;

fn init(force: bool) -> anyhow::Result<i32> {
    let path = std::env::current_dir()?.join("ayame-spell.toml");
    if path.exists() && !force {
        anyhow::bail!(
            "{} already exists (use --force to overwrite)",
            path.display()
        );
    }
    std::fs::write(&path, INIT_TEMPLATE)?;
    println!("wrote {}", path.display());
    Ok(0)
}

fn print_config() -> anyhow::Result<i32> {
    let loaded = ayame_spell_core::config::discover(&std::env::current_dir()?)?;
    println!("# root: {}", loaded.root.display());
    if let Some(p) = &loaded.project_file {
        println!("# project config: {}", p.display());
    }
    if let Some(p) = &loaded.global_file {
        println!("# global config: {}", p.display());
    }
    print!("{}", toml_edit::ser::to_string_pretty(&loaded.config)?);
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_completions_for_every_supported_shell() {
        for shell in clap_complete::Shell::value_variants() {
            let output = completion_script(*shell).unwrap();
            let script = String::from_utf8(output).unwrap();

            for candidate in ["check", "fix", "human", "brief", "json"] {
                assert!(
                    script.contains(candidate),
                    "{shell} completion output should contain {candidate}"
                );
            }
        }
    }

    #[test]
    fn lsp_accepts_conventional_stdio_flag() {
        let cli = Cli::try_parse_from(["ayame-spell", "lsp", "--stdio"]).unwrap();
        assert!(matches!(cli.cmd, Some(Cmd::Lsp { stdio: true })));
    }
}
