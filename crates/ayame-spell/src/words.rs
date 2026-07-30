//! Bulk word workflows: collect, add, and interactive triage.

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::{Path, PathBuf};

use anyhow::Context;
use ayame_spell_core::config::LoadedConfig;
use ayame_spell_core::IssueKind;
use clap::Subcommand;
use dialoguer::{Confirm, MultiSelect};

use crate::check;

#[derive(Subcommand)]
pub enum WordsCmd {
    /// Collect flagged words across files, ranked by frequency.
    Collect {
        #[arg(value_name = "PATH")]
        paths: Vec<PathBuf>,
        /// Only include words flagged at least this many times.
        #[arg(long, default_value_t = 1)]
        min_count: usize,
        /// Print bare words only (ready to append to a word file).
        #[arg(long)]
        plain: bool,
        #[arg(long)]
        json: bool,
    },
    /// Add words to the project (default) or global word file.
    Add {
        #[arg(required = true)]
        words: Vec<String>,
        #[arg(long)]
        global: bool,
    },
    /// Interactive bulk triage of flagged words: multi-select what goes to
    /// the project dictionary, the global dictionary, or the ignore list.
    Triage {
        #[arg(value_name = "PATH")]
        paths: Vec<PathBuf>,
    },
}

struct Collected {
    word: String,
    count: usize,
    kind: IssueKind,
    example: String,
}

fn collect_words(paths: &[PathBuf]) -> anyhow::Result<(LoadedConfig, Vec<Collected>)> {
    let cwd = std::env::current_dir()?;
    let (loaded, checker) =
        check::load_context(paths.first().map_or(cwd.as_path(), |p| p.as_path()))?;
    let (reports, _stats) = check::scan(&loaded, &checker, paths, None, false, false)?;

    let mut map: BTreeMap<String, Collected> = BTreeMap::new();
    for report in reports {
        for item in report.items {
            let issue = item.issue;
            if !matches!(
                issue.kind,
                IssueKind::Typo | IssueKind::UnknownWord | IssueKind::JaVariant
            ) {
                continue;
            }
            let key = if issue.word.is_ascii() {
                issue.word.to_ascii_lowercase()
            } else {
                issue.word.clone()
            };
            let example = format!("{}:{}", report.path.display(), issue.line);
            map.entry(key.clone())
                .and_modify(|c| c.count += 1)
                .or_insert(Collected {
                    word: key,
                    count: 1,
                    kind: issue.kind,
                    example,
                });
        }
    }
    let mut list: Vec<Collected> = map.into_values().collect();
    list.sort_by(|a, b| b.count.cmp(&a.count).then(a.word.cmp(&b.word)));
    Ok((loaded, list))
}

pub fn run(cmd: WordsCmd) -> anyhow::Result<i32> {
    match cmd {
        WordsCmd::Collect {
            paths,
            min_count,
            plain,
            json,
        } => {
            let (_loaded, list) = collect_words(&paths)?;
            for c in list.iter().filter(|c| c.count >= min_count) {
                if json {
                    println!(
                        "{}",
                        serde_json::json!({
                            "word": c.word,
                            "count": c.count,
                            "kind": c.kind.code(),
                            "example": c.example,
                        })
                    );
                } else if plain {
                    println!("{}", c.word);
                } else {
                    println!("{:6}  {:14}  {}", c.count, c.kind.code(), c.word);
                }
            }
            Ok(0)
        }
        WordsCmd::Add { words, global } => {
            let cwd = std::env::current_dir()?;
            let loaded = ayame_spell_core::config::discover(&cwd)?;
            let path = if global {
                ayame_spell_core::global_words_path()
                    .context("cannot determine the global config directory")?
            } else {
                loaded.project_words_path()
            };
            let added = append_words(&path, &words)?;
            println!("added {added} word(s) to {}", path.display());
            Ok(0)
        }
        WordsCmd::Triage { paths } => triage(&paths),
    }
}

fn triage(paths: &[PathBuf]) -> anyhow::Result<i32> {
    let (loaded, list) = collect_words(paths)?;
    if list.is_empty() {
        println!("nothing to triage — no flagged words");
        return Ok(0);
    }

    let labels: Vec<String> = list
        .iter()
        .map(|c| {
            format!(
                "{}  ({}×, {}, e.g. {})",
                c.word,
                c.count,
                c.kind.code(),
                c.example
            )
        })
        .collect();

    let selected = MultiSelect::new()
        .with_prompt(format!(
            "{} word(s) flagged — select words to ADD TO THE PROJECT dictionary (Space: toggle, Enter: confirm)",
            list.len()
        ))
        .items(&labels)
        .report(false)
        .interact()
        .context("triage needs an interactive terminal")?;

    let selected: HashSet<usize> = selected.into_iter().collect();
    let project_words: Vec<String> = list
        .iter()
        .enumerate()
        .filter(|(i, _)| selected.contains(i))
        .map(|(_, c)| c.word.clone())
        .collect();
    if !project_words.is_empty() {
        let path = loaded.project_words_path();
        let n = append_words(&path, &project_words)?;
        println!("added {n} word(s) to {}", path.display());
    }

    let rest: Vec<&Collected> = list
        .iter()
        .enumerate()
        .filter(|(i, _)| !selected.contains(i))
        .map(|(_, c)| c)
        .collect();
    if rest.is_empty() {
        return Ok(0);
    }

    let labels: Vec<String> = rest
        .iter()
        .map(|c| format!("{}  ({}×, {})", c.word, c.count, c.kind.code()))
        .collect();
    let selected = MultiSelect::new()
        .with_prompt("Select words to ADD TO THE GLOBAL dictionary")
        .items(&labels)
        .report(false)
        .interact()?;
    let selected: HashSet<usize> = selected.into_iter().collect();
    let global_words: Vec<String> = rest
        .iter()
        .enumerate()
        .filter(|(i, _)| selected.contains(i))
        .map(|(_, c)| c.word.clone())
        .collect();
    if !global_words.is_empty() {
        let path = ayame_spell_core::global_words_path()
            .context("cannot determine the global config directory")?;
        let n = append_words(&path, &global_words)?;
        println!("added {n} word(s) to {}", path.display());
    }

    let rest: Vec<&&Collected> = rest
        .iter()
        .enumerate()
        .filter(|(i, _)| !selected.contains(i))
        .map(|(_, c)| c)
        .collect();
    if rest.is_empty() {
        return Ok(0);
    }

    let ignore = Confirm::new()
        .with_prompt(format!(
            "Add the remaining {} word(s) to [words].ignore in ayame-spell.toml?",
            rest.len()
        ))
        .default(false)
        .interact()?;
    if ignore {
        let words: Vec<String> = rest.iter().map(|c| c.word.clone()).collect();
        let path = add_to_string_array(&loaded, "words", "ignore", &words)?;
        println!(
            "added {} word(s) to [words].ignore in {}",
            words.len(),
            path.display()
        );
    }
    Ok(0)
}

/// Append words to a word file (one per line, sorted, deduplicated;
/// leading comment lines are preserved). Returns how many were new.
pub fn append_words(path: &Path, words: &[String]) -> anyhow::Result<usize> {
    let existing = std::fs::read_to_string(path).unwrap_or_default();
    let mut header: Vec<&str> = Vec::new();
    let mut set: BTreeSet<String> = BTreeSet::new();
    let mut in_header = true;
    for line in existing.lines() {
        let trimmed = line.trim();
        if in_header && (trimmed.starts_with('#') || trimmed.is_empty()) {
            header.push(line);
            continue;
        }
        in_header = false;
        if !trimmed.is_empty() && !trimmed.starts_with('#') {
            set.insert(trimmed.to_string());
        }
    }
    let before = set.len();
    for w in words {
        let w = w.trim();
        if !w.is_empty() {
            set.insert(w.to_string());
        }
    }
    let added = set.len() - before;

    let mut out = String::new();
    if header.is_empty() && existing.is_empty() {
        out.push_str("# ayame-spell word file — one word per line.\n");
    } else {
        for h in &header {
            out.push_str(h);
            out.push('\n');
        }
    }
    for w in &set {
        out.push_str(w);
        out.push('\n');
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, out).with_context(|| format!("cannot write {}", path.display()))?;
    Ok(added)
}

/// Add values to a `[table].key` string array in the project config,
/// preserving formatting; creates the file/table/array as needed.
/// Returns the path of the config file written.
pub fn add_to_string_array(
    loaded: &LoadedConfig,
    table: &str,
    key: &str,
    values: &[String],
) -> anyhow::Result<PathBuf> {
    let path = loaded
        .project_file
        .clone()
        .unwrap_or_else(|| loaded.root.join("ayame-spell.toml"));
    let text = std::fs::read_to_string(&path).unwrap_or_default();
    let mut doc: toml_edit::DocumentMut = text
        .parse()
        .with_context(|| format!("cannot parse {}", path.display()))?;

    let table_item = doc
        .entry(table)
        .or_insert(toml_edit::Item::Table(toml_edit::Table::new()));
    let tbl = table_item
        .as_table_mut()
        .with_context(|| format!("[{table}] in {} is not a table", path.display()))?;
    let arr_item = tbl
        .entry(key)
        .or_insert(toml_edit::value(toml_edit::Array::new()));
    let arr = arr_item
        .as_array_mut()
        .with_context(|| format!("{table}.{key} in {} is not an array", path.display()))?;

    let existing: HashSet<String> = arr
        .iter()
        .filter_map(|v| v.as_str().map(str::to_string))
        .collect();
    for v in values {
        if !existing.contains(v) {
            arr.push(v.as_str());
        }
    }
    std::fs::write(&path, doc.to_string())
        .with_context(|| format!("cannot write {}", path.display()))?;
    Ok(path)
}

/// Remove a value from a `[table].key` string array in the project config.
pub fn remove_from_string_array(
    loaded: &LoadedConfig,
    table: &str,
    key: &str,
    value: &str,
) -> anyhow::Result<bool> {
    let Some(path) = loaded.project_file.clone() else {
        return Ok(false);
    };
    let text = std::fs::read_to_string(&path)?;
    let mut doc: toml_edit::DocumentMut = text.parse()?;
    let Some(arr) = doc
        .get_mut(table)
        .and_then(|t| t.get_mut(key))
        .and_then(toml_edit::Item::as_array_mut)
    else {
        return Ok(false);
    };
    let before = arr.len();
    arr.retain(|v| v.as_str() != Some(value));
    let removed = arr.len() != before;
    if removed {
        std::fs::write(&path, doc.to_string())?;
    }
    Ok(removed)
}
