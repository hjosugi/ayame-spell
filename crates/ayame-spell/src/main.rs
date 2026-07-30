mod check;
mod dict;
mod lsp;
mod words;

use std::{
    collections::HashSet,
    io::{IsTerminal, Write},
    path::{Path, PathBuf},
};

use anyhow::Context;
use clap::{Args, CommandFactory, Parser, Subcommand, ValueEnum};
use dialoguer::{Confirm, Input, Select};

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

    #[command(flatten)]
    scan: ScanArgs,

    /// Apply safe fixes in place (shorthand for `fix`).
    #[arg(short, long)]
    write: bool,

    /// Output format.
    #[arg(long, value_enum, default_value_t = Format::Human)]
    format: Format,
}

#[derive(Args, Clone)]
struct ScanArgs {
    /// Files or directories to check (`-` reads standard input).
    #[arg(value_name = "PATH")]
    paths: Vec<PathBuf>,

    /// Load exactly this configuration file.
    #[arg(long, value_name = "PATH", conflicts_with = "no_config")]
    config: Option<PathBuf>,

    /// Ignore project and global configuration files.
    #[arg(long)]
    no_config: bool,

    /// Override `[check].mode`.
    #[arg(long, value_enum)]
    mode: Option<ModeArg>,

    /// Exclude an additional glob (repeatable).
    #[arg(long, value_name = "GLOB")]
    exclude: Vec<String>,

    /// Do not honour `.gitignore`, `.ignore`, or Git exclude files.
    #[arg(long)]
    no_ignore: bool,

    /// Include hidden files and directories.
    #[arg(long)]
    hidden: bool,

    /// Colour policy for human output.
    #[arg(long, value_enum, default_value_t = ColorChoice::Auto)]
    color: ColorChoice,

    /// Print findings only, without summaries.
    #[arg(short, long, conflicts_with = "verbose")]
    quiet: bool,

    /// Report configuration sources, skipped files, and elapsed time.
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Display name used for standard input (also selects overrides).
    #[arg(long, value_name = "PATH")]
    stdin_filename: Option<PathBuf>,

    /// Skip files larger than this many bytes (overrides
    /// `[files].max-file-size`).
    #[arg(long, value_name = "BYTES")]
    max_file_size: Option<u64>,

    /// Worker threads (overrides the detected CPU count).
    #[arg(long, short = 'j')]
    threads: Option<usize>,
}

#[derive(Clone, Copy, ValueEnum)]
enum ModeArg {
    Corrections,
    Dictionary,
    Off,
}

impl From<ModeArg> for ayame_spell_core::config::Mode {
    fn from(value: ModeArg) -> Self {
        match value {
            ModeArg::Corrections => Self::Corrections,
            ModeArg::Dictionary => Self::Dictionary,
            ModeArg::Off => Self::Off,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ColorChoice {
    Auto,
    Always,
    Never,
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
        #[command(flatten)]
        scan: ScanArgs,
        /// Apply safe fixes in place.
        #[arg(short, long)]
        write: bool,
        #[arg(long, value_enum, default_value_t = Format::Human)]
        format: Format,
    },
    /// Apply all safe fixes in place (single-candidate corrections and
    /// mechanical notation conversions).
    Fix {
        #[command(flatten)]
        scan: ScanArgs,
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
        /// Run the guided setup wizard.
        #[arg(long, conflicts_with = "yes")]
        interactive: bool,
        /// Use the non-interactive starter configuration.
        #[arg(long)]
        yes: bool,
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
        None => run_scan(cli.scan, cli.write, cli.format),
        Some(Cmd::Check {
            scan,
            write,
            format,
        }) => run_scan(scan, write, format),
        Some(Cmd::Fix { scan }) => run_scan(scan, true, Format::Human),
        Some(Cmd::Words { cmd }) => words::run(cmd),
        Some(Cmd::Dict { cmd }) => dict::run(cmd),
        Some(Cmd::Init {
            force,
            interactive,
            yes,
        }) => init(force, interactive, yes),
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

fn run_scan(scan: ScanArgs, fix: bool, format: Format) -> anyhow::Result<i32> {
    check::run(check::RunOptions {
        paths: scan.paths,
        fix,
        format,
        threads: scan.threads,
        config: scan.config,
        no_config: scan.no_config,
        mode: scan.mode.map(Into::into),
        exclude: scan.exclude,
        no_ignore: scan.no_ignore,
        hidden: scan.hidden,
        color: scan.color,
        quiet: scan.quiet,
        verbose: scan.verbose,
        stdin_filename: scan.stdin_filename,
        max_file_size: scan.max_file_size,
    })
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

struct InitAnswers {
    mode: &'static str,
    japanese_enabled: bool,
    katakana_style: &'static str,
    dictionaries: Vec<String>,
    project_words: String,
    excludes: Vec<String>,
}

fn init(force: bool, interactive: bool, yes: bool) -> anyhow::Result<i32> {
    let path = std::env::current_dir()?.join("ayame-spell.toml");
    if path.exists() && !force {
        anyhow::bail!(
            "{} already exists (use --force to overwrite)",
            path.display()
        );
    }

    let auto_interactive =
        !force && !yes && std::io::stdin().is_terminal() && std::io::stdout().is_terminal();
    if interactive || auto_interactive {
        return init_interactive(&path);
    }

    std::fs::write(&path, INIT_TEMPLATE)
        .with_context(|| format!("cannot write {}", path.display()))?;
    println!("wrote {}", path.display());
    Ok(0)
}

fn init_interactive(path: &Path) -> anyhow::Result<i32> {
    let mode_index = Select::new()
        .with_prompt("Checking mode")
        .items(&[
            "corrections (low-noise known typos)",
            "dictionary (all unknown words)",
        ])
        .default(0)
        .interact()
        .context("interactive setup needs a terminal")?;
    let mode = if mode_index == 0 {
        "corrections"
    } else {
        "dictionary"
    };

    let japanese_enabled = Confirm::new()
        .with_prompt("Enable Japanese notation checks?")
        .default(true)
        .interact()?;
    let katakana_style = if japanese_enabled {
        let styles = ["consistency", "long", "short", "off"];
        styles[Select::new()
            .with_prompt("Katakana long-vowel policy")
            .items(&styles)
            .default(0)
            .interact()?]
    } else {
        "off"
    };

    let root = path.parent().unwrap_or_else(|| Path::new("."));
    let detected = detected_dictionaries(root);
    let dictionaries = dict::interactive_selection(&detected)?;
    let project_words: String = Input::new()
        .with_prompt("Project word file")
        .default("ayame-words.txt".to_string())
        .interact_text()?;
    let excludes_text: String = Input::new()
        .with_prompt("Additional exclude globs (comma-separated, blank for none)")
        .allow_empty(true)
        .interact_text()?;
    let excludes = excludes_text
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect();
    let answers = InitAnswers {
        mode,
        japanese_enabled,
        katakana_style,
        dictionaries,
        project_words,
        excludes,
    };
    let rendered = render_init_config(&answers)?;

    println!("\nConfiguration preview:\n\n{rendered}");
    if !Confirm::new()
        .with_prompt("Write this configuration?")
        .default(true)
        .interact()?
    {
        println!("cancelled; no files written");
        return Ok(0);
    }

    dict::install_names(&answers.dictionaries, true)?;
    std::fs::write(path, rendered).with_context(|| format!("cannot write {}", path.display()))?;
    println!("wrote {}", path.display());

    if Confirm::new()
        .with_prompt(format!("Create {} now?", answers.project_words))
        .default(true)
        .interact()?
    {
        let word_path = {
            let candidate = Path::new(&answers.project_words);
            if candidate.is_absolute() {
                candidate.to_path_buf()
            } else {
                root.join(candidate)
            }
        };
        words::append_words(&word_path, &[])?;
        println!("wrote {}", word_path.display());
    }

    println!(
        "\nCI snippet:\n\n  - name: Check spelling\n    run: |\n      cargo install ayame-spell --version {}\n      ayame-spell check .",
        env!("CARGO_PKG_VERSION")
    );
    Ok(0)
}

fn detected_dictionaries(root: &Path) -> Vec<String> {
    let mut names = HashSet::new();
    if root.join("Cargo.toml").is_file() {
        names.insert("rust");
    }
    if root.join("package.json").is_file() {
        names.insert("web");
    }
    if ["pyproject.toml", "requirements.txt", "setup.py"]
        .iter()
        .any(|name| root.join(name).is_file())
    {
        names.insert("python");
    }
    let mut names: Vec<String> = names.into_iter().map(str::to_string).collect();
    names.sort();
    names
}

fn render_init_config(answers: &InitAnswers) -> anyhow::Result<String> {
    let mut doc: toml_edit::DocumentMut = INIT_TEMPLATE.parse()?;
    let dictionary_refs: Vec<String> = answers
        .dictionaries
        .iter()
        .map(|name| format!("registry:{name}"))
        .collect();
    doc["check"]["mode"] = toml_edit::value(answers.mode);
    doc["words"]["project"] = toml_edit::value(&answers.project_words);
    doc["words"]["dictionaries"] = toml_edit::value(string_array(&dictionary_refs));
    doc["japanese"]["enabled"] = toml_edit::value(answers.japanese_enabled);
    doc["japanese"]["katakana-style"] = toml_edit::value(answers.katakana_style);
    if !answers.excludes.is_empty() {
        let mut files = toml_edit::Table::new();
        files["exclude"] = toml_edit::value(string_array(&answers.excludes));
        doc["files"] = toml_edit::Item::Table(files);
    }
    Ok(doc.to_string())
}

fn string_array(values: &[String]) -> toml_edit::Array {
    let mut array = toml_edit::Array::new();
    for value in values {
        array.push(value.as_str());
    }
    array
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

    #[test]
    fn init_detects_project_dictionaries_and_renders_them() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::write(
            temp.path().join("Cargo.toml"),
            "[package]\nname = \"demo\"\n",
        )
        .unwrap();
        std::fs::write(temp.path().join("package.json"), "{}\n").unwrap();
        assert_eq!(detected_dictionaries(temp.path()), ["rust", "web"]);

        let rendered = render_init_config(&InitAnswers {
            mode: "dictionary",
            japanese_enabled: false,
            katakana_style: "off",
            dictionaries: vec!["rust".to_string(), "web".to_string()],
            project_words: "project-words.txt".to_string(),
            excludes: vec!["generated/**".to_string()],
        })
        .unwrap();
        assert!(rendered.contains("mode = \"dictionary\""));
        assert!(rendered.contains("dictionaries = [\"registry:rust\", \"registry:web\"]"));
        assert!(rendered.contains("project = \"project-words.txt\""));
        assert!(rendered.contains("exclude = [\"generated/**\"]"));
    }
}
