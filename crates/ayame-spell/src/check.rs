//! Parallel file walking, checking, reporting, and in-place fixing.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::io::{IsTerminal, Read};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Instant, SystemTime};

use anyhow::Context;
use ayame_spell_core::config::LoadedConfig;
use ayame_spell_core::{Checker, Issue, Mode};
use dialoguer::Select;
use ignore::overrides::OverrideBuilder;
use ignore::{WalkBuilder, WalkState};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{ColorChoice, Format};

const JSON_OUTPUT_VERSION: u8 = 1;

/// One issue with enough context to display it.
pub struct Item {
    pub issue: Issue,
    pub line_text: String,
}

pub struct FileReport {
    /// Path relative to the invocation directory (for display).
    pub path: PathBuf,
    pub items: Vec<Item>,
    pub fixed: usize,
    pub original_text: String,
    pub modified: Option<SystemTime>,
}

#[derive(Default)]
pub struct Stats {
    pub checked: usize,
    pub cached: usize,
    pub skipped_binary: usize,
    pub skipped_large: usize,
}

pub struct RunOptions {
    pub paths: Vec<PathBuf>,
    pub fix: FixMode,
    pub baseline: BaselineMode,
    pub format: Format,
    pub threads: Option<usize>,
    pub config: Option<PathBuf>,
    pub no_config: bool,
    pub mode: Option<Mode>,
    pub exclude: Vec<String>,
    pub no_ignore: bool,
    pub hidden: bool,
    pub color: ColorChoice,
    pub quiet: bool,
    pub verbose: u8,
    pub stdin_filename: Option<PathBuf>,
    pub max_file_size: Option<u64>,
    pub no_cache: bool,
    pub cache_dir: Option<PathBuf>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FixMode {
    None,
    Apply,
    DryRun,
    Interactive,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BaselineMode {
    Apply,
    Ignore,
    Write,
    Prune,
}

const BASELINE_VERSION: u8 = 1;
const BASELINE_FILE: &str = "ayame-spell-baseline.json";

#[derive(Deserialize, Serialize)]
struct Baseline {
    version: u8,
    entries: Vec<BaselineEntry>,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct BaselineEntry {
    fingerprint: String,
    path: String,
    code: String,
    word: String,
    context_hash: String,
    count: usize,
}

const SCAN_CACHE_VERSION: u8 = 1;

#[derive(Clone)]
pub struct ScanCache {
    directory: PathBuf,
    config_sha256: String,
}

#[derive(Deserialize, Serialize)]
struct ScanCacheEntry {
    version: u8,
    binary_version: String,
    path: String,
    size: u64,
    modified_ns: u64,
    content_sha256: String,
    config_sha256: String,
    issues: Vec<Issue>,
}

impl ScanCache {
    fn new(directory: PathBuf, loaded: &LoadedConfig) -> anyhow::Result<Self> {
        std::fs::create_dir_all(&directory)
            .with_context(|| format!("cannot create scan cache {}", directory.display()))?;
        Ok(Self {
            directory,
            config_sha256: config_fingerprint(loaded)?,
        })
    }

    fn entry_path(&self, path: &Path) -> PathBuf {
        let mut digest = Sha256::new();
        digest.update(path.to_string_lossy().as_bytes());
        self.directory
            .join(format!("{}.json", hex_digest(digest.finalize())))
    }

    fn load(
        &self,
        path: &Path,
        size: u64,
        modified: Option<SystemTime>,
        content_sha256: &str,
    ) -> Option<Vec<Issue>> {
        let entry: ScanCacheEntry =
            serde_json::from_slice(&std::fs::read(self.entry_path(path)).ok()?).ok()?;
        (entry.version == SCAN_CACHE_VERSION
            && entry.binary_version == env!("CARGO_PKG_VERSION")
            && entry.path == path.to_string_lossy()
            && entry.size == size
            && entry.modified_ns == system_time_ns(modified)
            && entry.content_sha256 == content_sha256
            && entry.config_sha256 == self.config_sha256)
            .then_some(entry.issues)
    }

    fn save(
        &self,
        path: &Path,
        size: u64,
        modified: Option<SystemTime>,
        content_sha256: String,
        issues: &[Issue],
    ) {
        let entry = ScanCacheEntry {
            version: SCAN_CACHE_VERSION,
            binary_version: env!("CARGO_PKG_VERSION").to_string(),
            path: path.to_string_lossy().into_owned(),
            size,
            modified_ns: system_time_ns(modified),
            content_sha256,
            config_sha256: self.config_sha256.clone(),
            issues: issues.to_vec(),
        };
        if let Ok(bytes) = serde_json::to_vec(&entry) {
            let _ = std::fs::write(self.entry_path(path), bytes);
        }
    }
}

fn system_time_ns(time: Option<SystemTime>) -> u64 {
    time.and_then(|time| time.duration_since(SystemTime::UNIX_EPOCH).ok())
        .map_or(0, |duration| {
            duration.as_nanos().min(u128::from(u64::MAX)) as u64
        })
}

fn config_fingerprint(loaded: &LoadedConfig) -> anyhow::Result<String> {
    let mut digest = Sha256::new();
    digest.update(env!("CARGO_PKG_VERSION").as_bytes());
    digest.update(serde_json::to_vec(&loaded.config)?);
    let mut paths = Vec::new();
    paths.extend(loaded.project_file.clone());
    paths.extend(loaded.global_file.clone());
    paths.push(loaded.project_words_path());
    paths.extend(ayame_spell_core::global_words_path());
    paths.push(loaded.root.join(ayame_spell_core::registry_lock::LOCK_FILE));
    for reference in loaded
        .config
        .words
        .dictionaries
        .iter()
        .chain(&loaded.config.corrections.extra)
        .chain(&loaded.config.japanese.variant_files)
    {
        if let Ok(path) = loaded.resolve_ref(reference) {
            paths.push(path);
        }
    }
    paths.sort();
    paths.dedup();
    for path in paths {
        digest.update(path.to_string_lossy().as_bytes());
        if let Ok(bytes) = std::fs::read(path) {
            digest.update(Sha256::digest(bytes));
        }
    }
    Ok(hex_digest(digest.finalize()))
}

/// Load configuration and build a checker, printing warnings to stderr.
pub fn load_context(start: &Path) -> anyhow::Result<(LoadedConfig, Checker)> {
    let loaded = ayame_spell_core::config::discover(start)?;
    build_checker(loaded)
}

fn build_checker(loaded: LoadedConfig) -> anyhow::Result<(LoadedConfig, Checker)> {
    let (checker, warnings) = Checker::new(&loaded);
    for w in warnings {
        eprintln!("warning: {w}");
    }
    Ok((loaded, checker))
}

/// Walk `paths` in parallel, check every text file, and optionally apply
/// safe fixes in place.
pub fn scan(
    loaded: &LoadedConfig,
    checker: &Checker,
    paths: &[PathBuf],
    threads: Option<usize>,
    fix: bool,
    no_ignore: bool,
    cache: Option<&ScanCache>,
) -> anyhow::Result<(Vec<FileReport>, Stats)> {
    let paths: Vec<PathBuf> = if paths.is_empty() {
        vec![std::env::current_dir()?]
    } else {
        paths.to_vec()
    };
    for p in &paths {
        anyhow::ensure!(p.exists(), "path does not exist: {}", p.display());
    }

    let mut builder = WalkBuilder::new(&paths[0]);
    for p in &paths[1..] {
        builder.add(p);
    }
    let cfg = &loaded.config;
    builder
        .hidden(!cfg.files.include_hidden)
        .ignore(!no_ignore)
        .git_ignore(!no_ignore)
        .git_global(!no_ignore)
        .git_exclude(!no_ignore)
        .follow_links(false)
        .threads(
            threads.unwrap_or_else(|| std::thread::available_parallelism().map_or(4, usize::from)),
        );
    if !cfg.files.exclude.is_empty() {
        let mut ob = OverrideBuilder::new(&loaded.root);
        for pattern in &cfg.files.exclude {
            ob.add(&format!("!{pattern}"))
                .with_context(|| format!("invalid exclude glob `{pattern}`"))?;
        }
        builder.overrides(ob.build()?);
    }
    let cache_directory = cache.map(|cache| {
        cache
            .directory
            .canonicalize()
            .unwrap_or_else(|_| cache.directory.clone())
    });
    builder.filter_entry(move |entry| {
        if entry.file_name() == ".git" || entry.file_name() == BASELINE_FILE {
            return false;
        }
        cache_directory.as_ref().map_or(true, |directory| {
            !entry
                .file_type()
                .is_some_and(|file_type| file_type.is_dir())
                || entry
                    .path()
                    .canonicalize()
                    .map_or(entry.path() != directory, |path| path != *directory)
        })
    });

    let (tx, rx) = crossbeam_channel::unbounded::<FileReport>();
    let (error_tx, error_rx) = crossbeam_channel::unbounded::<String>();
    let checked = AtomicUsize::new(0);
    let skipped_binary = AtomicUsize::new(0);
    let skipped_large = AtomicUsize::new(0);
    let cached = AtomicUsize::new(0);
    let root = loaded.root.clone();
    let max_size = cfg.files.max_file_size;

    builder.build_parallel().run(|| {
        let tx = tx.clone();
        let error_tx = error_tx.clone();
        let root = root.clone();
        let checked = &checked;
        let skipped_binary = &skipped_binary;
        let skipped_large = &skipped_large;
        let cached = &cached;
        let cache = cache.cloned();
        Box::new(move |entry| {
            let Ok(entry) = entry else {
                return WalkState::Continue;
            };
            if !entry.file_type().is_some_and(|t| t.is_file()) {
                return WalkState::Continue;
            }
            let path = entry.path();
            let metadata = entry.metadata().ok();
            let modified = metadata.as_ref().and_then(|meta| meta.modified().ok());
            let size = metadata.as_ref().map_or(0, |metadata| metadata.len());
            if max_size > 0 {
                if let Ok(meta) = entry.metadata() {
                    if meta.len() > max_size {
                        skipped_large.fetch_add(1, Ordering::Relaxed);
                        return WalkState::Continue;
                    }
                }
            }
            let Ok(bytes) = std::fs::read(path) else {
                return WalkState::Continue;
            };
            if bytes[..bytes.len().min(8192)].contains(&0) {
                skipped_binary.fetch_add(1, Ordering::Relaxed);
                return WalkState::Continue;
            }
            let text = String::from_utf8_lossy(&bytes).into_owned();
            let content_sha256 = hex_digest(Sha256::digest(&bytes));
            let rel = path
                .canonicalize()
                .ok()
                .and_then(|c| c.strip_prefix(&root).map(Path::to_path_buf).ok())
                .unwrap_or_else(|| path.to_path_buf());
            let issues = cache
                .as_ref()
                .and_then(|cache| {
                    cache
                        .load(path, size, modified, &content_sha256)
                        .inspect(|_| {
                            cached.fetch_add(1, Ordering::Relaxed);
                        })
                })
                .unwrap_or_else(|| {
                    let issues = checker.check(&text, Some(&rel));
                    if let Some(cache) = &cache {
                        cache.save(path, size, modified, content_sha256, &issues);
                    }
                    issues
                });
            checked.fetch_add(1, Ordering::Relaxed);
            if issues.is_empty() {
                return WalkState::Continue;
            }

            let mut fixed = 0;
            let remaining: Vec<Issue> = if fix {
                let (new_text, n, remaining) = apply_fixes(&text, &issues);
                if n > 0 {
                    if !file_is_unchanged(path, &bytes, modified) {
                        let _ = error_tx.send(format!(
                            "{} changed on disk after scanning; refusing to overwrite it",
                            path.display()
                        ));
                        return WalkState::Continue;
                    }
                    if let Err(e) = write_in_place(path, &new_text) {
                        let _ = error_tx.send(format!("cannot write {}: {e}", path.display()));
                        return WalkState::Continue;
                    }
                }
                fixed = n;
                remaining
            } else {
                issues
            };

            let lines: Vec<&str> = text.split('\n').collect();
            let items = remaining
                .into_iter()
                .map(|issue| {
                    let line_text = lines
                        .get(issue.line as usize - 1)
                        .map(|l| l.trim_end_matches('\r').to_string())
                        .unwrap_or_default();
                    Item { issue, line_text }
                })
                .collect();
            let _ = tx.send(FileReport {
                path: path.to_path_buf(),
                items,
                fixed,
                original_text: text,
                modified,
            });
            WalkState::Continue
        })
    });
    drop(tx);
    drop(error_tx);

    if let Some(error) = error_rx.into_iter().next() {
        anyhow::bail!("{error}");
    }

    let mut reports: Vec<FileReport> = rx.into_iter().collect();
    reports.sort_by(|a, b| a.path.cmp(&b.path));
    Ok((
        reports,
        Stats {
            checked: checked.into_inner(),
            cached: cached.into_inner(),
            skipped_binary: skipped_binary.into_inner(),
            skipped_large: skipped_large.into_inner(),
        },
    ))
}

fn scan_stdin(
    checker: &Checker,
    display_path: PathBuf,
) -> anyhow::Result<(Vec<FileReport>, Stats)> {
    let mut bytes = Vec::new();
    std::io::stdin().read_to_end(&mut bytes)?;
    if bytes[..bytes.len().min(8192)].contains(&0) {
        return Ok((
            Vec::new(),
            Stats {
                skipped_binary: 1,
                ..Stats::default()
            },
        ));
    }
    let text = String::from_utf8_lossy(&bytes);
    let issues = checker.check(&text, Some(&display_path));
    let lines: Vec<&str> = text.split('\n').collect();
    let items: Vec<Item> = issues
        .into_iter()
        .map(|issue| {
            let line_text = lines
                .get(issue.line as usize - 1)
                .map(|line| line.trim_end_matches('\r').to_string())
                .unwrap_or_default();
            Item { issue, line_text }
        })
        .collect();
    let reports = if items.is_empty() {
        Vec::new()
    } else {
        vec![FileReport {
            path: display_path,
            items,
            fixed: 0,
            original_text: text.into_owned(),
            modified: None,
        }]
    };
    Ok((
        reports,
        Stats {
            checked: 1,
            ..Stats::default()
        },
    ))
}

fn file_is_unchanged(path: &Path, original: &[u8], modified: Option<SystemTime>) -> bool {
    let current_modified = std::fs::metadata(path)
        .ok()
        .and_then(|metadata| metadata.modified().ok());
    current_modified == modified && std::fs::read(path).is_ok_and(|bytes| bytes == original)
}

fn baseline_path(loaded: &LoadedConfig) -> PathBuf {
    loaded.root.join(BASELINE_FILE)
}

fn baseline_report_path(root: &Path, report: &FileReport) -> String {
    report
        .path
        .strip_prefix(root)
        .unwrap_or(&report.path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn baseline_entry(path: &str, item: &Item) -> BaselineEntry {
    let mut context = Sha256::new();
    context.update(item.line_text.trim().as_bytes());
    let context_hash = hex_digest(context.finalize());
    let mut fingerprint = Sha256::new();
    fingerprint.update(path.as_bytes());
    fingerprint.update([0]);
    fingerprint.update(item.issue.kind.code().as_bytes());
    fingerprint.update([0]);
    fingerprint.update(item.issue.word.to_lowercase().as_bytes());
    fingerprint.update([0]);
    fingerprint.update(context_hash.as_bytes());
    BaselineEntry {
        fingerprint: hex_digest(fingerprint.finalize()),
        path: path.to_string(),
        code: item.issue.kind.code().to_string(),
        word: item.issue.word.clone(),
        context_hash,
        count: 1,
    }
}

fn hex_digest(bytes: impl AsRef<[u8]>) -> String {
    bytes
        .as_ref()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn baseline_from_reports(root: &Path, reports: &[FileReport]) -> Baseline {
    let mut entries: BTreeMap<String, BaselineEntry> = BTreeMap::new();
    for report in reports {
        let path = baseline_report_path(root, report);
        for item in &report.items {
            let entry = baseline_entry(&path, item);
            entries
                .entry(entry.fingerprint.clone())
                .and_modify(|existing| existing.count += 1)
                .or_insert(entry);
        }
    }
    Baseline {
        version: BASELINE_VERSION,
        entries: entries.into_values().collect(),
    }
}

fn read_baseline(path: &Path) -> anyhow::Result<Option<Baseline>> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    let baseline: Baseline = serde_json::from_str(&text)
        .with_context(|| format!("cannot parse baseline {}", path.display()))?;
    anyhow::ensure!(
        baseline.version == BASELINE_VERSION,
        "unsupported baseline version {} in {}",
        baseline.version,
        path.display()
    );
    Ok(Some(baseline))
}

fn write_baseline(path: &Path, baseline: &Baseline) -> anyhow::Result<()> {
    let mut output = serde_json::to_string_pretty(baseline)?;
    output.push('\n');
    std::fs::write(path, output)
        .with_context(|| format!("cannot write baseline {}", path.display()))
}

fn suppress_baseline(root: &Path, reports: &mut [FileReport], baseline: &Baseline) -> usize {
    let mut remaining: HashMap<&str, usize> = baseline
        .entries
        .iter()
        .map(|entry| (entry.fingerprint.as_str(), entry.count))
        .collect();
    let mut suppressed = 0;
    for report in reports {
        let path = baseline_report_path(root, report);
        report.items.retain(|item| {
            let entry = baseline_entry(&path, item);
            let Some(count) = remaining.get_mut(entry.fingerprint.as_str()) else {
                return true;
            };
            if *count == 0 {
                return true;
            }
            *count -= 1;
            suppressed += 1;
            false
        });
    }
    suppressed
}

fn prune_baseline(existing: Baseline, current: &Baseline) -> (Baseline, usize) {
    let current_counts: HashMap<&str, usize> = current
        .entries
        .iter()
        .map(|entry| (entry.fingerprint.as_str(), entry.count))
        .collect();
    let before: usize = existing.entries.iter().map(|entry| entry.count).sum();
    let mut entries = Vec::new();
    for mut entry in existing.entries {
        let count = current_counts
            .get(entry.fingerprint.as_str())
            .copied()
            .unwrap_or(0)
            .min(entry.count);
        if count > 0 {
            entry.count = count;
            entries.push(entry);
        }
    }
    let after: usize = entries.iter().map(|entry| entry.count).sum();
    (
        Baseline {
            version: BASELINE_VERSION,
            entries,
        },
        before - after,
    )
}

fn apply_report_fixes(reports: &mut [FileReport]) -> anyhow::Result<()> {
    for report in reports {
        let issues: Vec<Issue> = report.items.iter().map(|item| item.issue.clone()).collect();
        let (updated, fixed, remaining) = apply_fixes(&report.original_text, &issues);
        if fixed > 0 {
            anyhow::ensure!(
                file_is_unchanged(
                    &report.path,
                    report.original_text.as_bytes(),
                    report.modified
                ),
                "{} changed on disk after scanning; refusing to overwrite it",
                report.path.display()
            );
            write_in_place(&report.path, &updated)?;
        }
        let lines: Vec<&str> = report.original_text.split('\n').collect();
        report.items = remaining
            .into_iter()
            .map(|issue| {
                let line_text = lines
                    .get(issue.line as usize - 1)
                    .map(|line| line.trim_end_matches('\r').to_string())
                    .unwrap_or_default();
                Item { issue, line_text }
            })
            .collect();
        report.fixed = fixed;
    }
    Ok(())
}

/// Apply all safe fixes; returns the new text, the number of fixes applied,
/// and the issues that still need human attention (positions unadjusted —
/// only for counting/collection, not display against the new text).
fn apply_fixes(text: &str, issues: &[Issue]) -> (String, usize, Vec<Issue>) {
    let mut out = String::with_capacity(text.len());
    let mut last = 0;
    let mut applied = 0;
    let mut remaining = Vec::new();
    for issue in issues {
        match issue.safe_fix() {
            Some(fix) if issue.offset >= last => {
                out.push_str(&text[last..issue.offset]);
                out.push_str(fix);
                last = issue.offset + issue.len;
                applied += 1;
            }
            _ => remaining.push(issue.clone()),
        }
    }
    out.push_str(&text[last..]);
    (out, applied, remaining)
}

fn dry_run(reports: &[FileReport], quiet: bool) -> anyhow::Result<i32> {
    let mut fixed = 0;
    let mut skipped_multiple = 0;
    let mut unfixable = 0;
    for report in reports {
        let issues: Vec<Issue> = report.items.iter().map(|item| item.issue.clone()).collect();
        let (proposed, applied, remaining) = apply_fixes(&report.original_text, &issues);
        fixed += applied;
        skipped_multiple += remaining
            .iter()
            .filter(|issue| issue.suggestions.len() > 1)
            .count();
        unfixable += remaining
            .iter()
            .filter(|issue| issue.suggestions.is_empty())
            .count();
        if proposed != report.original_text {
            print!(
                "{}",
                unified_diff(&report.path, &report.original_text, &proposed)
            );
        }
    }
    if !quiet {
        eprintln!(
            "{fixed} would be fixed, {skipped_multiple} skipped (multiple candidates), {unfixable} unfixable"
        );
    }
    Ok(if fixed + skipped_multiple + unfixable > 0 {
        1
    } else {
        0
    })
}

fn unified_diff(path: &Path, original: &str, proposed: &str) -> String {
    let old_lines = original.lines().count();
    let new_lines = proposed.lines().count();
    let mut output = format!(
        "--- a/{0}\n+++ b/{0}\n@@ -1,{old_lines} +1,{new_lines} @@\n",
        path.display()
    );
    push_diff_lines(&mut output, '-', original);
    push_diff_lines(&mut output, '+', proposed);
    output
}

fn push_diff_lines(output: &mut String, prefix: char, text: &str) {
    for line in text.split_inclusive('\n') {
        output.push(prefix);
        output.push_str(line);
        if !line.ends_with('\n') {
            output.push('\n');
            output.push_str("\\ No newline at end of file\n");
        }
    }
}

fn interactive_fix(loaded: &LoadedConfig, reports: &[FileReport]) -> anyhow::Result<i32> {
    anyhow::ensure!(
        std::io::stdin().is_terminal() && std::io::stderr().is_terminal(),
        "`fix --interactive` needs an interactive terminal"
    );
    let mut apply_all: HashMap<String, String> = HashMap::new();
    let mut fixed = 0;
    let mut skipped = 0;
    let mut skipped_multiple = 0;
    let mut unfixable = 0;
    let mut quit = false;

    for report in reports {
        let mut edits: Vec<(usize, usize, String)> = Vec::new();
        for item in &report.items {
            let issue = &item.issue;
            if let Some(replacement) = apply_all.get(&issue.word) {
                edits.push((issue.offset, issue.len, replacement.clone()));
                fixed += 1;
                continue;
            }

            eprintln!(
                "\n{}:{}:{}: {} [{}]\n  {}",
                report.path.display(),
                issue.line,
                display_column(item),
                issue.word,
                issue.kind.code(),
                item.line_text
            );
            if issue.suggestions.is_empty() {
                unfixable += 1;
                eprintln!("  no replacement candidates");
            }

            let actions = [
                "apply a replacement",
                "skip",
                "apply this replacement to every occurrence of the word",
                "add the word to the project dictionary",
                "quit",
            ];
            let default = if issue.suggestions.is_empty() { 1 } else { 0 };
            match Select::new()
                .with_prompt("Action")
                .items(&actions)
                .default(default)
                .interact()?
            {
                0 => {
                    if let Some(replacement) = select_replacement(issue)? {
                        edits.push((issue.offset, issue.len, replacement));
                        fixed += 1;
                    } else {
                        skipped += 1;
                    }
                }
                1 => {
                    skipped += 1;
                    if issue.suggestions.len() > 1 {
                        skipped_multiple += 1;
                    }
                }
                2 => {
                    if let Some(replacement) = select_replacement(issue)? {
                        apply_all.insert(issue.word.clone(), replacement.clone());
                        edits.push((issue.offset, issue.len, replacement));
                        fixed += 1;
                    } else {
                        skipped += 1;
                    }
                }
                3 => {
                    crate::words::append_words(
                        &loaded.project_words_path(),
                        std::slice::from_ref(&issue.word),
                    )?;
                    eprintln!("added `{}` to the project dictionary", issue.word);
                    skipped += 1;
                }
                _ => {
                    quit = true;
                    break;
                }
            }
        }

        if !edits.is_empty() {
            anyhow::ensure!(
                file_is_unchanged(
                    &report.path,
                    report.original_text.as_bytes(),
                    report.modified
                ),
                "{} changed on disk after scanning; refusing to overwrite it",
                report.path.display()
            );
            let updated = apply_replacements(&report.original_text, &mut edits)?;
            write_in_place(&report.path, &updated)?;
        }
        if quit {
            break;
        }
    }

    eprintln!(
        "{fixed} fixed, {skipped} skipped ({skipped_multiple} multiple candidates), {unfixable} unfixable"
    );
    Ok(if quit || skipped + unfixable > 0 {
        1
    } else {
        0
    })
}

fn select_replacement(issue: &Issue) -> anyhow::Result<Option<String>> {
    match issue.suggestions.as_slice() {
        [] => Ok(None),
        [replacement] => Ok(Some(replacement.clone())),
        candidates => {
            let selected = Select::new()
                .with_prompt(format!("Replacement for `{}`", issue.word))
                .items(candidates)
                .default(0)
                .interact()?;
            Ok(Some(candidates[selected].clone()))
        }
    }
}

fn apply_replacements(
    original: &str,
    edits: &mut [(usize, usize, String)],
) -> anyhow::Result<String> {
    edits.sort_by_key(|edit| std::cmp::Reverse(edit.0));
    let mut output = original.to_string();
    let mut previous_start = original.len();
    for (offset, length, replacement) in edits {
        let end = offset.saturating_add(*length);
        anyhow::ensure!(
            *offset <= end
                && end <= output.len()
                && output.is_char_boundary(*offset)
                && output.is_char_boundary(end)
                && end <= previous_start,
            "invalid or overlapping fix range at byte {offset}"
        );
        output.replace_range(*offset..end, replacement);
        previous_start = *offset;
    }
    Ok(output)
}

/// Write through a sibling temp file + rename, preserving permissions.
fn write_in_place(path: &Path, text: &str) -> anyhow::Result<()> {
    let perms = std::fs::metadata(path)?.permissions();
    let tmp = path.with_file_name(format!(
        ".{}.ayame-tmp",
        path.file_name().and_then(|n| n.to_str()).unwrap_or("file")
    ));
    std::fs::write(&tmp, text)?;
    std::fs::set_permissions(&tmp, perms)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

#[derive(Serialize)]
struct JsonIssue<'a> {
    version: u8,
    #[serde(rename = "type")]
    record_type: &'static str,
    path: &'a Path,
    line: u32,
    column: usize,
    offset: usize,
    length: usize,
    word: &'a str,
    kind: &'a str,
    suggestions: &'a [String],
    fix: Option<&'a str>,
    message: String,
}

#[derive(Serialize)]
struct JsonSummary {
    version: u8,
    #[serde(rename = "type")]
    record_type: &'static str,
    issues: usize,
    files_with_issues: usize,
    files_checked: usize,
    fixed: usize,
    skipped_binary: usize,
    skipped_large: usize,
}

pub fn run(options: RunOptions) -> anyhow::Result<i32> {
    let cwd = std::env::current_dir()?;
    let uses_stdin = options.paths.iter().any(|path| path == Path::new("-"));
    if uses_stdin {
        anyhow::ensure!(
            options.paths.len() == 1,
            "standard input (`-`) cannot be combined with file paths"
        );
        anyhow::ensure!(
            options.fix == FixMode::None,
            "standard input cannot be used with --write or `fix`"
        );
        anyhow::ensure!(
            !matches!(options.baseline, BaselineMode::Write | BaselineMode::Prune),
            "standard input cannot be used with `baseline`"
        );
    } else {
        anyhow::ensure!(
            options.stdin_filename.is_none(),
            "--stdin-filename requires `-` as the input path"
        );
    }

    let start = options
        .paths
        .first()
        .filter(|path| path.as_path() != Path::new("-"))
        .map_or(cwd.as_path(), PathBuf::as_path);
    let mut loaded = ayame_spell_core::config::discover_selected(
        start,
        options.config.as_deref(),
        options.no_config,
    )?;
    if let Some(mode) = options.mode {
        loaded.config.check.mode = mode;
    }
    loaded.config.files.exclude.extend(options.exclude.clone());
    if options.hidden {
        loaded.config.files.include_hidden = true;
    }
    if let Some(max_file_size) = options.max_file_size {
        loaded.config.files.max_file_size = max_file_size;
    }
    let (loaded, checker) = build_checker(loaded)?;
    let cache = scan_cache(&options, &loaded)?;

    if options.verbose > 0 {
        eprintln!("config root: {}", loaded.root.display());
        eprintln!(
            "project config: {}",
            loaded
                .project_file
                .as_deref()
                .map_or_else(|| "none".to_string(), |path| path.display().to_string())
        );
        eprintln!(
            "global config: {}",
            loaded
                .global_file
                .as_deref()
                .map_or_else(|| "none".to_string(), |path| path.display().to_string())
        );
        eprintln!("mode: {:?}", loaded.config.check.mode);
    }

    let started = Instant::now();
    let (mut reports, stats) = if uses_stdin {
        scan_stdin(
            &checker,
            options
                .stdin_filename
                .clone()
                .unwrap_or_else(|| PathBuf::from("<stdin>")),
        )?
    } else {
        scan(
            &loaded,
            &checker,
            &options.paths,
            options.threads,
            false,
            options.no_ignore,
            cache.as_ref(),
        )?
    };

    let baseline_file = baseline_path(&loaded);
    let mut baseline_suppressed = 0;
    match options.baseline {
        BaselineMode::Write => {
            let baseline = baseline_from_reports(&loaded.root, &reports);
            let count: usize = baseline.entries.iter().map(|entry| entry.count).sum();
            write_baseline(&baseline_file, &baseline)?;
            println!("wrote {count} finding(s) to {}", baseline_file.display());
            return Ok(0);
        }
        BaselineMode::Prune => {
            let existing = read_baseline(&baseline_file)?.with_context(|| {
                format!(
                    "baseline does not exist: {}; run `ayame-spell baseline` first",
                    baseline_file.display()
                )
            })?;
            let current = baseline_from_reports(&loaded.root, &reports);
            let (baseline, pruned) = prune_baseline(existing, &current);
            write_baseline(&baseline_file, &baseline)?;
            println!(
                "pruned {pruned} stale finding(s) from {}",
                baseline_file.display()
            );
            return Ok(0);
        }
        BaselineMode::Apply if !uses_stdin => {
            if let Some(baseline) = read_baseline(&baseline_file)? {
                baseline_suppressed = suppress_baseline(&loaded.root, &mut reports, &baseline);
            }
        }
        BaselineMode::Apply | BaselineMode::Ignore => {}
    }

    if options.fix == FixMode::Apply {
        apply_report_fixes(&mut reports)?;
    }

    if options.fix == FixMode::DryRun {
        let code = dry_run(&reports, options.quiet)?;
        if options.verbose > 0 {
            eprintln!("elapsed: {:.3}s", started.elapsed().as_secs_f64());
        }
        return Ok(code);
    }
    if options.fix == FixMode::Interactive {
        let code = interactive_fix(&loaded, &reports)?;
        if options.verbose > 0 {
            eprintln!("elapsed: {:.3}s", started.elapsed().as_secs_f64());
        }
        return Ok(code);
    }

    if options.format == Format::Sarif {
        let issue_count: usize = reports.iter().map(|report| report.items.len()).sum();
        println!("{}", serde_json::to_string_pretty(&sarif_report(&reports))?);
        if options.verbose > 0 {
            eprintln!("elapsed: {:.3}s", started.elapsed().as_secs_f64());
        }
        return Ok(if issue_count > 0 { 1 } else { 0 });
    }

    let color = color_enabled(options.color, options.format);
    let mut issue_count = 0usize;
    let mut fixed_count = 0usize;
    let mut files_with_issues = 0usize;
    let mut hinted = HashSet::new();
    let location_width = reports
        .iter()
        .flat_map(|report| {
            report.items.iter().map(|item| {
                let column = display_column(item);
                format!("{}:{}:{column}", report.path.display(), item.issue.line)
                    .chars()
                    .count()
            })
        })
        .max()
        .unwrap_or(0);

    for report in &reports {
        fixed_count += report.fixed;
        if !report.items.is_empty() {
            files_with_issues += 1;
        }
        for item in &report.items {
            issue_count += 1;
            print_item(&report.path, item, options.format, color, location_width);
            if options.format == Format::Human && !options.quiet && hinted.insert(item.issue.kind) {
                eprintln!(
                    "hint: run `ayame-spell explain {}` for rule details",
                    item.issue.kind.code()
                );
            }
        }
        if report.fixed > 0 && options.format != Format::Json && !options.quiet {
            eprintln!(
                "fixed {} issue(s) in {}",
                report.fixed,
                report.path.display()
            );
        }
    }

    if options.format == Format::Json {
        if !options.quiet {
            let summary = JsonSummary {
                version: JSON_OUTPUT_VERSION,
                record_type: "summary",
                issues: issue_count,
                files_with_issues,
                files_checked: stats.checked,
                fixed: fixed_count,
                skipped_binary: stats.skipped_binary,
                skipped_large: stats.skipped_large,
            };
            println!("{}", serde_json::to_string(&summary).unwrap());
        }
    } else if !options.quiet {
        let mut summary = format!(
            "{} issue(s) in {} file(s) — {} file(s) checked",
            issue_count, files_with_issues, stats.checked
        );
        if options.fix == FixMode::Apply {
            summary.push_str(&format!(", {fixed_count} fixed"));
        }
        if stats.skipped_binary > 0 {
            summary.push_str(&format!(
                ", {} binary file(s) skipped",
                stats.skipped_binary
            ));
        }
        if stats.skipped_large > 0 {
            summary.push_str(&format!(
                ", {} file(s) over max-file-size skipped",
                stats.skipped_large
            ));
        }
        if baseline_suppressed > 0 {
            summary.push_str(&format!(
                ", {baseline_suppressed} baseline finding(s) suppressed"
            ));
        }
        eprintln!("{summary}");
    }

    if options.verbose > 0 {
        eprintln!(
            "elapsed: {:.3}s; cache hits: {}; skipped: {} binary, {} over max-file-size",
            started.elapsed().as_secs_f64(),
            stats.cached,
            stats.skipped_binary,
            stats.skipped_large
        );
        if baseline_suppressed > 0 {
            eprintln!("baseline suppressed: {baseline_suppressed}");
        }
    }

    Ok(if issue_count > 0 { 1 } else { 0 })
}

fn scan_cache(options: &RunOptions, loaded: &LoadedConfig) -> anyhow::Result<Option<ScanCache>> {
    if options.no_cache {
        return Ok(None);
    }
    let in_ci = ["CI", "GITHUB_ACTIONS"]
        .into_iter()
        .any(|variable| std::env::var_os(variable).is_some());
    if in_ci && options.cache_dir.is_none() {
        return Ok(None);
    }
    let directory = options
        .cache_dir
        .clone()
        .or_else(ayame_spell_core::scan_cache_dir);
    directory
        .map(|directory| ScanCache::new(directory, loaded))
        .transpose()
}

fn color_enabled(choice: ColorChoice, format: Format) -> bool {
    if format != Format::Human {
        return false;
    }
    match choice {
        ColorChoice::Always => true,
        ColorChoice::Never => false,
        ColorChoice::Auto => {
            if std::env::var_os("NO_COLOR").is_some() {
                false
            } else if std::env::var_os("CLICOLOR_FORCE")
                .is_some_and(|value| !value.is_empty() && value != "0")
            {
                true
            } else {
                std::io::stdout().is_terminal()
            }
        }
    }
}

fn display_column(item: &Item) -> usize {
    item.line_text
        .get(..item.issue.col)
        .map_or(item.issue.col, |text| text.chars().count())
        + 1
}

fn print_item(path: &Path, item: &Item, format: Format, color: bool, location_width: usize) {
    let issue = &item.issue;
    let column = display_column(item);
    match format {
        Format::Json => {
            let j = JsonIssue {
                version: JSON_OUTPUT_VERSION,
                record_type: "issue",
                path,
                line: issue.line,
                column,
                offset: issue.offset,
                length: issue.len,
                word: &issue.word,
                kind: issue.kind.code(),
                suggestions: &issue.suggestions,
                fix: issue.safe_fix(),
                message: issue.message(),
            };
            println!("{}", serde_json::to_string(&j).unwrap());
        }
        Format::Brief => {
            println!(
                "{}:{}:{}: {} -> {}",
                path.display(),
                issue.line,
                column,
                issue.word,
                issue.suggestions.join(",")
            );
        }
        Format::Github => {
            println!(
                "::warning file={},line={},col={},title={}::{}",
                github_escape(&path.to_string_lossy(), true),
                issue.line,
                column,
                github_escape(&format!("ayame-spell [{}]", issue.kind.code()), true),
                github_escape(&issue.message(), false)
            );
        }
        Format::Human => {
            let (red, green, dim, reset) = if color {
                ("\x1b[31m", "\x1b[32m", "\x1b[2m", "\x1b[0m")
            } else {
                ("", "", "", "")
            };
            let suggestion = if issue.suggestions.is_empty() {
                String::new()
            } else {
                format!(
                    " → {green}{}{reset}",
                    issue.suggestions.join(&format!("{reset}, {green}"))
                )
            };
            let location = format!("{}:{}:{column}", path.display(), issue.line);
            println!(
                "{dim}{location:<location_width$}{reset}: {red}{}{reset}{suggestion} {dim}[{}]{reset}",
                issue.word,
                issue.kind.code(),
            );
        }
        Format::Sarif => unreachable!("SARIF is rendered as one document"),
    }
}

fn github_escape(value: &str, property: bool) -> String {
    let mut escaped = value
        .replace('%', "%25")
        .replace('\r', "%0D")
        .replace('\n', "%0A");
    if property {
        escaped = escaped.replace(':', "%3A").replace(',', "%2C");
    }
    escaped
}

fn sarif_report(reports: &[FileReport]) -> serde_json::Value {
    let rules: Vec<serde_json::Value> = ayame_spell_core::IssueKind::ALL
        .into_iter()
        .map(|kind| {
            let info = kind.info(false);
            serde_json::json!({
                "id": kind.code(),
                "name": info.title,
                "shortDescription": { "text": info.summary },
                "fullDescription": { "text": info.explanation },
                "help": {
                    "text": format!(
                        "{} Configuration: {} How to silence: {} Example: {}",
                        info.explanation, info.config_key, info.silence, info.example
                    )
                },
                "helpUri": format!(
                    "https://hjosugi.github.io/ayame-spell/reference/rules/#{}",
                    kind.code()
                ),
                "properties": {
                    "configKey": info.config_key,
                    "tags": ["spelling", kind.code()]
                }
            })
        })
        .collect();
    let results: Vec<serde_json::Value> = reports
        .iter()
        .flat_map(|report| {
            report.items.iter().map(move |item| {
                let issue = &item.issue;
                let column = display_column(item);
                let uri = report.path.to_string_lossy().replace('\\', "/");
                serde_json::json!({
                    "ruleId": issue.kind.code(),
                    "level": "warning",
                    "message": { "text": issue.message() },
                    "locations": [{
                        "physicalLocation": {
                            "artifactLocation": { "uri": uri },
                            "region": {
                                "startLine": issue.line,
                                "startColumn": column,
                                "endLine": issue.line,
                                "endColumn": column + issue.word.chars().count()
                            }
                        }
                    }],
                    "properties": {
                        "word": issue.word,
                        "suggestions": issue.suggestions
                    }
                })
            })
        })
        .collect();
    serde_json::json!({
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "ayame-spell",
                    "informationUri": "https://hjosugi.github.io/ayame-spell/",
                    "version": env!("CARGO_PKG_VERSION"),
                    "rules": rules
                }
            },
            "results": results
        }]
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ayame_spell_core::IssueKind;
    use serde_json::json;

    #[test]
    fn json_issue_shape_is_stable() {
        let item = Item {
            issue: Issue {
                line: 4,
                col: 2,
                offset: 42,
                len: 7,
                word: "recieve".to_string(),
                kind: IssueKind::Typo,
                suggestions: vec!["receive".to_string()],
            },
            line_text: "  recieve this".to_string(),
        };
        let issue = &item.issue;
        let output = JsonIssue {
            version: JSON_OUTPUT_VERSION,
            record_type: "issue",
            path: Path::new("docs/guide.md"),
            line: issue.line,
            column: 3,
            offset: issue.offset,
            length: issue.len,
            word: &issue.word,
            kind: issue.kind.code(),
            suggestions: &issue.suggestions,
            fix: issue.safe_fix(),
            message: issue.message(),
        };

        assert_eq!(
            serde_json::to_string(&output).unwrap(),
            r#"{"version":1,"type":"issue","path":"docs/guide.md","line":4,"column":3,"offset":42,"length":7,"word":"recieve","kind":"typo","suggestions":["receive"],"fix":"receive","message":"`recieve` should be `receive`"}"#
        );
    }

    #[test]
    fn json_summary_shape_is_stable() {
        let output = JsonSummary {
            version: JSON_OUTPUT_VERSION,
            record_type: "summary",
            issues: 1,
            files_with_issues: 1,
            files_checked: 12,
            fixed: 0,
            skipped_binary: 2,
            skipped_large: 3,
        };

        assert_eq!(
            serde_json::to_string(&output).unwrap(),
            r#"{"version":1,"type":"summary","issues":1,"files_with_issues":1,"files_checked":12,"fixed":0,"skipped_binary":2,"skipped_large":3}"#
        );
    }

    #[test]
    fn json_schema_matches_record_fields_and_issue_codes() {
        let schema: serde_json::Value = serde_json::from_str(include_str!(
            "../../../site/public/schema/v1/ayame-spell-output.json"
        ))
        .unwrap();

        assert_eq!(
            schema["oneOf"][0]["required"],
            json!([
                "version",
                "type",
                "path",
                "line",
                "column",
                "offset",
                "length",
                "word",
                "kind",
                "suggestions",
                "fix",
                "message"
            ])
        );
        assert_eq!(
            schema["oneOf"][0]["properties"]["kind"]["enum"],
            json!([
                "typo",
                "unknown-word",
                "en-variant",
                "ja-variant",
                "fullwidth-alnum",
                "halfwidth-kana",
                "fullwidth-space",
                "ja-compatibility",
                "ja-number-style",
                "ja-punctuation"
            ])
        );
        assert_eq!(
            schema["oneOf"][1]["required"],
            json!([
                "version",
                "type",
                "issues",
                "files_with_issues",
                "files_checked",
                "fixed",
                "skipped_binary",
                "skipped_large"
            ])
        );
    }

    #[test]
    fn replacement_application_is_ordered_and_rejects_overlap() {
        let mut edits = vec![(0, 3, "the".to_string()), (4, 3, "receive".to_string())];
        assert_eq!(
            apply_replacements("teh rcv", &mut edits).unwrap(),
            "the receive"
        );

        let mut overlapping = vec![(0, 3, "a".to_string()), (2, 2, "b".to_string())];
        assert!(apply_replacements("test", &mut overlapping).is_err());
    }

    #[test]
    fn overwrite_guard_detects_same_length_changes() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("input.md");
        std::fs::write(&path, "teh\n").unwrap();
        let modified = std::fs::metadata(&path).unwrap().modified().ok();
        assert!(file_is_unchanged(&path, b"teh\n", modified));

        std::fs::write(&path, "the\n").unwrap();
        assert!(!file_is_unchanged(&path, b"teh\n", modified));
    }
}
