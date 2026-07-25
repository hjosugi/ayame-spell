//! Parallel file walking, checking, reporting, and in-place fixing.

use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use anyhow::Context;
use ayame_spell_core::config::LoadedConfig;
use ayame_spell_core::{Checker, Issue};
use ignore::overrides::OverrideBuilder;
use ignore::{WalkBuilder, WalkState};
use serde::Serialize;

use crate::Format;

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
}

#[derive(Default)]
pub struct Stats {
    pub checked: usize,
    pub skipped_binary: usize,
    pub skipped_large: usize,
}

/// Load configuration and build a checker, printing warnings to stderr.
pub fn load_context(start: &Path) -> anyhow::Result<(LoadedConfig, Checker)> {
    let loaded = ayame_spell_core::config::discover(start)?;
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
    builder.filter_entry(|e| e.file_name() != ".git");

    let (tx, rx) = crossbeam_channel::unbounded::<FileReport>();
    let checked = AtomicUsize::new(0);
    let skipped_binary = AtomicUsize::new(0);
    let skipped_large = AtomicUsize::new(0);
    let root = loaded.root.clone();
    let max_size = cfg.files.max_file_size;

    builder.build_parallel().run(|| {
        let tx = tx.clone();
        let root = root.clone();
        let checked = &checked;
        let skipped_binary = &skipped_binary;
        let skipped_large = &skipped_large;
        Box::new(move |entry| {
            let Ok(entry) = entry else {
                return WalkState::Continue;
            };
            if !entry.file_type().is_some_and(|t| t.is_file()) {
                return WalkState::Continue;
            }
            let path = entry.path();
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
            let text = String::from_utf8_lossy(&bytes);
            let rel = path
                .canonicalize()
                .ok()
                .and_then(|c| c.strip_prefix(&root).map(Path::to_path_buf).ok())
                .unwrap_or_else(|| path.to_path_buf());
            let issues = checker.check(&text, Some(&rel));
            checked.fetch_add(1, Ordering::Relaxed);
            if issues.is_empty() {
                return WalkState::Continue;
            }

            let mut fixed = 0;
            let remaining: Vec<Issue> = if fix {
                let (new_text, n, remaining) = apply_fixes(&text, &issues);
                if n > 0 {
                    if let Err(e) = write_in_place(path, &new_text) {
                        eprintln!("error: cannot write {}: {e}", path.display());
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
            });
            WalkState::Continue
        })
    });
    drop(tx);

    let mut reports: Vec<FileReport> = rx.into_iter().collect();
    reports.sort_by(|a, b| a.path.cmp(&b.path));
    Ok((
        reports,
        Stats {
            checked: checked.into_inner(),
            skipped_binary: skipped_binary.into_inner(),
            skipped_large: skipped_large.into_inner(),
        },
    ))
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
    path: &'a Path,
    line: u32,
    column: usize,
    offset: usize,
    length: usize,
    word: &'a str,
    kind: &'a str,
    suggestions: &'a [String],
    message: String,
}

pub fn run(
    paths: Vec<PathBuf>,
    fix: bool,
    format: Format,
    threads: Option<usize>,
) -> anyhow::Result<i32> {
    let cwd = std::env::current_dir()?;
    let (loaded, checker) = load_context(paths.first().map_or(cwd.as_path(), |p| p.as_path()))?;
    let (reports, stats) = scan(&loaded, &checker, &paths, threads, fix)?;

    let color = std::io::stdout().is_terminal() && format == Format::Human;
    let mut issue_count = 0usize;
    let mut fixed_count = 0usize;
    let mut files_with_issues = 0usize;

    for report in &reports {
        fixed_count += report.fixed;
        if !report.items.is_empty() {
            files_with_issues += 1;
        }
        for item in &report.items {
            issue_count += 1;
            print_item(&report.path, item, format, color);
        }
        if report.fixed > 0 && format != Format::Json {
            eprintln!(
                "fixed {} issue(s) in {}",
                report.fixed,
                report.path.display()
            );
        }
    }

    if format != Format::Json {
        let mut summary = format!(
            "{} issue(s) in {} file(s) — {} file(s) checked",
            issue_count, files_with_issues, stats.checked
        );
        if fix {
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
        eprintln!("{summary}");
    }

    Ok(if issue_count > 0 { 1 } else { 0 })
}

fn print_item(path: &Path, item: &Item, format: Format, color: bool) {
    let issue = &item.issue;
    let column = item
        .line_text
        .get(..issue.col)
        .map_or(issue.col, |s| s.chars().count())
        + 1;
    match format {
        Format::Json => {
            let j = JsonIssue {
                path,
                line: issue.line,
                column,
                offset: issue.offset,
                length: issue.len,
                word: &issue.word,
                kind: issue.kind.code(),
                suggestions: &issue.suggestions,
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
            println!(
                "{}:{}:{}: {red}{}{reset}{suggestion} {dim}[{}]{reset}",
                path.display(),
                issue.line,
                column,
                issue.word,
                issue.kind.code(),
            );
        }
    }
}
