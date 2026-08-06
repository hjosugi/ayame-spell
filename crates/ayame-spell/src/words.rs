//! Bulk word workflows: collect, add, and interactive triage.

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::io::IsTerminal;
use std::path::{Path, PathBuf};

use anyhow::Context;
use ayame_spell_core::config::LoadedConfig;
use ayame_spell_core::IssueKind;
use clap::{Subcommand, ValueEnum};
use dialoguer::{FuzzySelect, Select};

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
    /// Search flagged words and choose a dictionary, ignore, fix, or skip
    /// action for each one.
    Triage {
        #[arg(value_name = "PATH")]
        paths: Vec<PathBuf>,
        /// Only include one finding kind.
        #[arg(long, value_enum)]
        kind: Option<TriageKind>,
        /// Only include words flagged at least this many times.
        #[arg(long, default_value_t = 1)]
        min_count: usize,
        /// Review at most this many words after sorting and filtering.
        #[arg(long)]
        limit: Option<usize>,
    },
}

#[derive(Clone, Copy, ValueEnum)]
pub enum TriageKind {
    Typo,
    UnknownWord,
    EnVariant,
    JaVariant,
}

impl TriageKind {
    fn matches(self, kind: IssueKind) -> bool {
        matches!(
            (self, kind),
            (Self::Typo, IssueKind::Typo)
                | (Self::UnknownWord, IssueKind::UnknownWord)
                | (Self::EnVariant, IssueKind::EnVariant)
                | (Self::JaVariant, IssueKind::JaVariant)
        )
    }
}

struct Occurrence {
    path: PathBuf,
    offset: usize,
    len: usize,
}

struct Collected {
    word: String,
    count: usize,
    kind: IssueKind,
    example: String,
    suggestions: Vec<String>,
    occurrences: Vec<Occurrence>,
}

struct CollectedScan {
    loaded: LoadedConfig,
    list: Vec<Collected>,
    originals: BTreeMap<PathBuf, String>,
}

fn collect_words(paths: &[PathBuf]) -> anyhow::Result<CollectedScan> {
    let cwd = std::env::current_dir()?;
    let (loaded, checker) =
        check::load_context(paths.first().map_or(cwd.as_path(), |p| p.as_path()))?;
    let (reports, _stats) = check::scan(&loaded, &checker, paths, None, false, false, None)?;

    let mut map: BTreeMap<String, Collected> = BTreeMap::new();
    let mut originals = BTreeMap::new();
    for report in reports {
        originals.insert(report.path.clone(), report.original_text);
        for item in report.items {
            let issue = item.issue;
            if !matches!(
                issue.kind,
                IssueKind::Typo
                    | IssueKind::UnknownWord
                    | IssueKind::EnVariant
                    | IssueKind::JaVariant
            ) {
                continue;
            }
            let key = if issue.word.is_ascii() {
                issue.word.to_ascii_lowercase()
            } else {
                issue.word.clone()
            };
            let example = format!(
                "{}:{}: {}",
                report.path.display(),
                issue.line,
                context_snippet(&item.line_text)
            );
            let suggestions = issue.suggestions;
            let occurrence = Occurrence {
                path: report.path.clone(),
                offset: issue.offset,
                len: issue.len,
            };
            map.entry(key.clone())
                .and_modify(|c| {
                    c.count += 1;
                    for suggestion in &suggestions {
                        if !c.suggestions.contains(suggestion) {
                            c.suggestions.push(suggestion.clone());
                        }
                    }
                    c.occurrences.push(Occurrence {
                        path: occurrence.path.clone(),
                        offset: occurrence.offset,
                        len: occurrence.len,
                    });
                })
                .or_insert(Collected {
                    word: key,
                    count: 1,
                    kind: issue.kind,
                    example,
                    suggestions,
                    occurrences: vec![occurrence],
                });
        }
    }
    let mut list: Vec<Collected> = map.into_values().collect();
    list.sort_by(|a, b| b.count.cmp(&a.count).then(a.word.cmp(&b.word)));
    Ok(CollectedScan {
        loaded,
        list,
        originals,
    })
}

fn context_snippet(line: &str) -> String {
    const MAX_CHARS: usize = 96;
    let trimmed = line.trim();
    let mut chars = trimmed.chars();
    let snippet: String = chars.by_ref().take(MAX_CHARS).collect();
    if chars.next().is_some() {
        format!("{snippet}…")
    } else {
        snippet
    }
}

pub fn run(cmd: WordsCmd) -> anyhow::Result<i32> {
    match cmd {
        WordsCmd::Collect {
            paths,
            min_count,
            plain,
            json,
        } => {
            let CollectedScan { list, .. } = collect_words(&paths)?;
            cache_completion_words(&list)?;
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
        WordsCmd::Triage {
            paths,
            kind,
            min_count,
            limit,
        } => triage(&paths, kind, min_count, limit),
    }
}

fn cache_completion_words(words: &[Collected]) -> anyhow::Result<()> {
    let Some(path) = ayame_spell_core::completion_words_cache_path() else {
        return Ok(());
    };
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let words: Vec<&str> = words
        .iter()
        .map(|collected| collected.word.as_str())
        .collect();
    std::fs::write(path, serde_json::to_vec(&words)?)?;
    Ok(())
}

/// Return cached `words collect` candidates without scanning the project.
pub fn completion_words(prefix: &str) -> anyhow::Result<Vec<String>> {
    let Some(path) = ayame_spell_core::completion_words_cache_path() else {
        return Ok(Vec::new());
    };
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.into()),
    };
    let prefix = prefix.to_lowercase();
    let mut words: Vec<String> = serde_json::from_slice::<Vec<String>>(&bytes)?
        .into_iter()
        .filter(|word| word.to_lowercase().starts_with(&prefix))
        .collect();
    words.sort();
    Ok(words)
}

fn triage(
    paths: &[PathBuf],
    kind: Option<TriageKind>,
    min_count: usize,
    limit: Option<usize>,
) -> anyhow::Result<i32> {
    let CollectedScan {
        loaded,
        mut list,
        originals,
    } = collect_words(paths)?;
    list.retain(|item| {
        item.count >= min_count && kind.is_none_or(|filter| filter.matches(item.kind))
    });
    if let Some(limit) = limit {
        list.truncate(limit);
    }
    if list.is_empty() {
        println!("nothing to triage — no flagged words match the filters");
        return Ok(0);
    }

    anyhow::ensure!(
        std::io::stdin().is_terminal() && std::io::stderr().is_terminal(),
        "words triage needs an interactive terminal; use `words collect` for non-interactive output"
    );

    let total = list.len();
    let mut project_words = Vec::new();
    let mut global_words = Vec::new();
    let mut ignored_words = Vec::new();
    let mut fixes = Vec::new();
    let mut skipped = 0usize;

    while !list.is_empty() {
        let mut labels: Vec<String> = list
            .iter()
            .map(|item| {
                format!(
                    "{}  ({}×, {})  {}",
                    item.word,
                    item.count,
                    item.kind.code(),
                    item.example
                )
            })
            .collect();
        labels.push("Finish and apply the decisions above".to_string());
        let selected = FuzzySelect::new()
            .with_prompt(format!(
                "{} of {total} word(s) remain — type to search, arrows to page",
                list.len()
            ))
            .items(&labels)
            .default(0)
            .max_length(20)
            .report(false)
            .interact()
            .context("words triage could not read the interactive terminal")?;
        if selected == list.len() {
            skipped += list.len();
            break;
        }

        let item = list.remove(selected);
        let mut actions = vec![
            TriageAction::Project,
            TriageAction::Global,
            TriageAction::Ignore,
        ];
        if !item.suggestions.is_empty() {
            actions.push(TriageAction::Fix);
        }
        actions.push(TriageAction::Skip);
        let action_labels: Vec<&str> = actions.iter().map(|action| action.label()).collect();
        let action = actions[Select::new()
            .with_prompt(format!(
                "{} ({} occurrence(s), {})",
                item.word, item.count, item.example
            ))
            .items(&action_labels)
            .default(0)
            .report(false)
            .interact()
            .context("words triage could not read the selected action")?];

        match action {
            TriageAction::Project => project_words.push(item.word),
            TriageAction::Global => global_words.push(item.word),
            TriageAction::Ignore => ignored_words.push(item.word),
            TriageAction::Fix => {
                let suggestions = item.suggestions;
                let choice = if suggestions.len() == 1 {
                    0
                } else {
                    Select::new()
                        .with_prompt(format!("Replace {} with", item.word))
                        .items(&suggestions)
                        .default(0)
                        .report(false)
                        .interact()
                        .context("words triage could not read the selected replacement")?
                };
                fixes.push(FixDecision {
                    replacement: suggestions[choice].clone(),
                    occurrences: item.occurrences,
                });
            }
            TriageAction::Skip => skipped += 1,
        }
    }

    let mut changed = BTreeSet::new();
    let fixed = apply_triage_fixes(&fixes, &originals, &mut changed)?;
    if !project_words.is_empty() {
        let path = loaded.project_words_path();
        append_words(&path, &project_words)?;
        changed.insert(path);
    }
    if !global_words.is_empty() {
        let path = ayame_spell_core::global_words_path()
            .context("cannot determine the global config directory")?;
        append_words(&path, &global_words)?;
        changed.insert(path);
    }
    if !ignored_words.is_empty() {
        changed.insert(add_to_string_array(
            &loaded,
            "words",
            "ignore",
            &ignored_words,
        )?);
    }

    println!(
        "triage summary: {} project, {} global, {} ignored, {fixed} occurrence(s) fixed, {skipped} skipped",
        project_words.len(),
        global_words.len(),
        ignored_words.len()
    );
    if changed.is_empty() {
        println!("changed files: none");
    } else {
        println!("changed files:");
        for path in changed {
            println!("  {}", path.display());
        }
    }
    Ok(0)
}

#[derive(Clone, Copy)]
enum TriageAction {
    Project,
    Global,
    Ignore,
    Fix,
    Skip,
}

impl TriageAction {
    fn label(self) -> &'static str {
        match self {
            Self::Project => "Add to the project dictionary",
            Self::Global => "Add to the global dictionary",
            Self::Ignore => "Add to [words].ignore",
            Self::Fix => "Fix every occurrence",
            Self::Skip => "Skip",
        }
    }
}

struct FixDecision {
    replacement: String,
    occurrences: Vec<Occurrence>,
}

fn apply_triage_fixes(
    decisions: &[FixDecision],
    originals: &BTreeMap<PathBuf, String>,
    changed: &mut BTreeSet<PathBuf>,
) -> anyhow::Result<usize> {
    let mut by_path: BTreeMap<&Path, Vec<(usize, usize, &str)>> = BTreeMap::new();
    for decision in decisions {
        for occurrence in &decision.occurrences {
            by_path.entry(&occurrence.path).or_default().push((
                occurrence.offset,
                occurrence.len,
                &decision.replacement,
            ));
        }
    }

    let mut outputs = BTreeMap::new();
    let mut fixed = 0;
    for (path, mut edits) in by_path {
        let original = originals
            .get(path)
            .with_context(|| format!("missing scanned content for {}", path.display()))?;
        let current = std::fs::read_to_string(path)
            .with_context(|| format!("cannot read {}", path.display()))?;
        anyhow::ensure!(
            &current == original,
            "{} changed on disk after scanning; refusing to overwrite it",
            path.display()
        );
        edits.sort_by_key(|edit| std::cmp::Reverse(edit.0));
        let mut output = current;
        let mut previous_start = output.len();
        for (offset, len, replacement) in edits {
            let end = offset.saturating_add(len);
            anyhow::ensure!(
                offset <= end
                    && end <= previous_start
                    && output.is_char_boundary(offset)
                    && output.is_char_boundary(end),
                "invalid or overlapping fix range at byte {offset} in {}",
                path.display()
            );
            output.replace_range(offset..end, replacement);
            previous_start = offset;
            fixed += 1;
        }
        outputs.insert(path.to_path_buf(), output);
    }

    for (path, output) in outputs {
        std::fs::write(&path, output)
            .with_context(|| format!("cannot write {}", path.display()))?;
        changed.insert(path);
    }
    Ok(fixed)
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

/// Add or replace one entry in `[corrections.words]`.
pub fn set_correction(
    loaded: &LoadedConfig,
    word: &str,
    replacement: &str,
) -> anyhow::Result<PathBuf> {
    let path = loaded
        .project_file
        .clone()
        .unwrap_or_else(|| loaded.root.join("ayame-spell.toml"));
    let text = std::fs::read_to_string(&path).unwrap_or_default();
    let mut doc: toml_edit::DocumentMut = text
        .parse()
        .with_context(|| format!("cannot parse {}", path.display()))?;
    let corrections = doc
        .entry("corrections")
        .or_insert(toml_edit::Item::Table(toml_edit::Table::new()))
        .as_table_mut()
        .with_context(|| format!("[corrections] in {} is not a table", path.display()))?;
    let words = corrections
        .entry("words")
        .or_insert(toml_edit::Item::Table(toml_edit::Table::new()))
        .as_table_mut()
        .with_context(|| format!("[corrections.words] in {} is not a table", path.display()))?;
    words.insert(word, toml_edit::value(replacement));
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

/// Replace all registry references for `name` in a configuration array with
/// one project-local path, preserving unrelated values.
pub fn replace_registry_reference(
    loaded: &LoadedConfig,
    table: &str,
    key: &str,
    name: &str,
    replacement: &str,
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
    let table = table_item
        .as_table_mut()
        .with_context(|| format!("[{table}] in {} is not a table", path.display()))?;
    let item = table
        .entry(key)
        .or_insert(toml_edit::value(toml_edit::Array::new()));
    let array = item
        .as_array_mut()
        .with_context(|| format!("{key} in {} is not an array", path.display()))?;
    let mut rewritten = toml_edit::Array::new();
    let mut replaced = false;
    for value in array.iter() {
        let value = value
            .as_str()
            .with_context(|| format!("{key} in {} must contain strings", path.display()))?;
        let matches = value.strip_prefix("registry:").is_some_and(|reference| {
            ayame_spell_core::registry_lock::split_reference(reference).0 == name
        });
        if matches {
            if !replaced {
                rewritten.push(replacement);
                replaced = true;
            }
        } else {
            rewritten.push(value);
        }
    }
    if !replaced {
        rewritten.push(replacement);
    }
    *array = rewritten;
    std::fs::write(&path, doc.to_string())
        .with_context(|| format!("cannot write {}", path.display()))?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_snippets_are_unicode_safe_and_bounded() {
        let line = format!("  {} tail", "語".repeat(100));
        let snippet = context_snippet(&line);
        assert!(snippet.ends_with('…'));
        assert_eq!(snippet.chars().count(), 97);
    }

    #[test]
    fn locale_variants_are_available_to_word_triage() {
        assert!(TriageKind::EnVariant.matches(IssueKind::EnVariant));
        assert!(!TriageKind::EnVariant.matches(IssueKind::Typo));
    }

    #[test]
    fn triage_fixes_all_occurrences_after_verifying_the_snapshot() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("input.md");
        let original = "teh and teh\n".to_string();
        std::fs::write(&path, &original).unwrap();
        let mut originals = BTreeMap::new();
        originals.insert(path.clone(), original);
        let decisions = [FixDecision {
            replacement: "the".to_string(),
            occurrences: vec![
                Occurrence {
                    path: path.clone(),
                    offset: 0,
                    len: 3,
                },
                Occurrence {
                    path: path.clone(),
                    offset: 8,
                    len: 3,
                },
            ],
        }];
        let mut changed = BTreeSet::new();
        assert_eq!(
            apply_triage_fixes(&decisions, &originals, &mut changed).unwrap(),
            2
        );
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "the and the\n");
        assert_eq!(changed, BTreeSet::from([path]));
    }
}
