mod check;
mod dict;
mod lsp;
mod words;

use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

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
    /// Run the LSP server (used by editor integrations).
    Lsp,
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
        Some(Cmd::Lsp) => lsp::run(),
    };
    match result {
        Ok(code) => std::process::exit(code),
        Err(e) => {
            eprintln!("error: {e:#}");
            std::process::exit(2);
        }
    }
}

const INIT_TEMPLATE: &str = r#"# ayame-spell configuration
# Reference: https://github.com/hjosugi/ayame-spell#configuration

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
