//! Core engine for [ayame-spell](https://github.com/hjosugi/ayame-spell):
//! a fast, low-noise spell checker for code and prose, for English and
//! Japanese.
//!
//! The engine has two English checking modes:
//!
//! - **corrections** (default): only known misspellings are flagged, using a
//!   curated typo → fix table. Near-zero false positives; safe for CI.
//! - **dictionary**: additionally flags words missing from configured
//!   wordlists (project words, global words, registry dictionaries).
//!
//! Japanese checks are independent of the mode: katakana notation variants
//! (表記ゆれ), fullwidth alphanumerics, halfwidth katakana, and fullwidth
//! spaces.

pub mod checker;
pub mod config;
pub mod corrections;
pub mod dictionary;
pub mod issue;
pub mod japanese;
pub mod tokenizer;

pub use checker::Checker;
pub use config::{Config, LoadedConfig, Mode};
pub use issue::{Issue, IssueKind};

use std::path::PathBuf;

/// Directory where registry dictionaries are cached
/// (`~/.cache/ayame-spell/dicts`).
pub fn registry_cache_dir() -> Option<PathBuf> {
    dirs::cache_dir().map(|d| d.join("ayame-spell").join("dicts"))
}

/// Path of the cached registry dictionary `name`, if the cache directory is
/// known. The file may or may not exist yet.
pub fn registry_cache_path(name: &str) -> Option<PathBuf> {
    registry_cache_dir().map(|d| d.join(format!("{name}.txt")))
}

/// Directory for global (per-user) configuration
/// (`~/.config/ayame-spell`).
pub fn global_config_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("ayame-spell"))
}

/// Path of the global word list (`~/.config/ayame-spell/words.txt`).
pub fn global_words_path() -> Option<PathBuf> {
    global_config_dir().map(|d| d.join("words.txt"))
}
