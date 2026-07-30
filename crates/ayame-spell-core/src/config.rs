//! Configuration: `ayame-spell.toml` at the project root, merged over the
//! per-user global config (`~/.config/ayame-spell/config.toml`).
//!
//! Dictionary references understand three forms:
//! - `registry:name` — a dictionary installed from the ayame-spell registry
//!   (`ayame-spell dict add name`), resolved through `ayame-spell.lock`;
//! - `registry:name@version` — an explicit immutable registry release;
//! - a path relative to the project root;
//! - an absolute path.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::japanese::{KatakanaStyle, SpacePolicy};

pub const PROJECT_FILE_NAMES: [&str; 2] = ["ayame-spell.toml", ".ayame-spell.toml"];
pub const CONFIG_SCHEMA: &str = include_str!("../schema/ayame-spell.json");

/// Globs excluded in every project on top of `.gitignore`: machine-written
/// files whose "words" are package names and minified identifiers.
pub const DEFAULT_EXCLUDES: [&str; 7] = [
    "*.lock",
    "*.sum",
    "package-lock.json",
    "pnpm-lock.yaml",
    "yarn.lock",
    "*.min.js",
    "*.min.css",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    /// Flag only known misspellings (near-zero false positives).
    #[default]
    Corrections,
    /// Corrections plus unknown-word detection against wordlists.
    Dictionary,
    Off,
}

/// Effective configuration with all defaults applied.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    pub check: CheckConfig,
    pub files: FilesConfig,
    pub words: WordsConfig,
    pub corrections: CorrectionsConfig,
    pub japanese: JapaneseConfig,
    pub overrides: Vec<OverrideConfig>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct CheckConfig {
    pub mode: Mode,
    /// English regional spelling policy.
    pub locale: EnglishLocale,
    /// How markup and source syntax is filtered before checking.
    pub profile: SyntaxProfile,
    /// Words shorter than this are never flagged.
    pub min_word_len: usize,
    /// Longer digit-containing tokens are treated as hashes and skipped.
    pub max_token_len: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
pub enum EnglishLocale {
    #[default]
    #[serde(rename = "any")]
    Any,
    #[serde(rename = "en-US")]
    EnUs,
    #[serde(rename = "en-GB")]
    EnGb,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SyntaxProfile {
    /// Select prose or source filtering from the file extension.
    Auto,
    /// Check prose while masking Markdown markup and code.
    Prose,
    /// Check only comments and string literals in source files.
    Source,
    /// Check every token, preserving the pre-1.0 behavior.
    #[default]
    All,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct FilesConfig {
    /// Glob patterns excluded in addition to `.gitignore`.
    pub exclude: Vec<String>,
    pub include_hidden: bool,
    /// Files larger than this many bytes are skipped (0 = no limit).
    pub max_file_size: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct WordsConfig {
    /// Project word file, relative to the project root. Created on demand
    /// by `ayame-spell words add` and editor quick fixes.
    pub project: String,
    /// Words never flagged, in any mode.
    pub ignore: Vec<String>,
    /// Wordlists for dictionary mode (`registry:name`,
    /// `registry:name@version`, or paths).
    pub dictionaries: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct CorrectionsConfig {
    /// Use the built-in English corrections table (typos-dict).
    pub builtin: bool,
    /// Extra correction tables: TSV files (`typo<TAB>fix[,fix]`) or
    /// `registry:name` / `registry:name@version` references.
    pub extra: Vec<String>,
    /// Inline corrections; a fix equal to its typo whitelists the word.
    pub words: BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct JapaneseConfig {
    pub enabled: bool,
    pub katakana_style: KatakanaStyleConfig,
    /// Inline variant rules: `"変種" = "正規形"`.
    pub variants: BTreeMap<String, String>,
    /// Variant rule files (TOML `[variants]` tables) or registry references.
    pub variant_files: Vec<String>,
    pub flag_fullwidth_alnum: bool,
    pub flag_halfwidth_kana: bool,
    /// Flag compatibility characters such as ㎏ with an NFKC suggestion.
    pub flag_compatibility: bool,
    /// Flag minority kanji/okurigana forms when a document mixes known pairs.
    pub kanji_consistency: bool,
    /// Flag minority Arabic/kanji number forms for the same value and unit.
    pub number_consistency: bool,
    /// Flag minority Japanese/fullwidth punctuation styles.
    pub punctuation_consistency: bool,
    pub fullwidth_space: SpacePolicyConfig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum KatakanaStyleConfig {
    #[default]
    Consistency,
    Long,
    Short,
    Off,
}

impl From<KatakanaStyleConfig> for KatakanaStyle {
    fn from(v: KatakanaStyleConfig) -> Self {
        match v {
            KatakanaStyleConfig::Consistency => KatakanaStyle::Consistency,
            KatakanaStyleConfig::Long => KatakanaStyle::Long,
            KatakanaStyleConfig::Short => KatakanaStyle::Short,
            KatakanaStyleConfig::Off => KatakanaStyle::Off,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SpacePolicyConfig {
    #[default]
    Code,
    Always,
    Never,
}

impl From<SpacePolicyConfig> for SpacePolicy {
    fn from(v: SpacePolicyConfig) -> Self {
        match v {
            SpacePolicyConfig::Code => SpacePolicy::Code,
            SpacePolicyConfig::Always => SpacePolicy::Always,
            SpacePolicyConfig::Never => SpacePolicy::Never,
        }
    }
}

/// Per-glob overrides; later entries win.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct OverrideConfig {
    pub paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<Mode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile: Option<SyntaxProfile>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub japanese: Option<bool>,
}

// ---------------------------------------------------------------------------
// Raw (partial) config as read from files, before merging and defaults.
// ---------------------------------------------------------------------------

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub struct RawConfig {
    check: RawCheck,
    files: RawFiles,
    words: RawWords,
    corrections: RawCorrections,
    japanese: RawJapanese,
    overrides: Vec<OverrideConfig>,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
struct RawCheck {
    mode: Option<Mode>,
    locale: Option<EnglishLocale>,
    profile: Option<SyntaxProfile>,
    min_word_len: Option<usize>,
    max_token_len: Option<usize>,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
struct RawFiles {
    exclude: Vec<String>,
    include_hidden: Option<bool>,
    max_file_size: Option<u64>,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
struct RawWords {
    project: Option<String>,
    ignore: Vec<String>,
    dictionaries: Vec<String>,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
struct RawCorrections {
    builtin: Option<bool>,
    extra: Vec<String>,
    words: BTreeMap<String, OneOrMany>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum OneOrMany {
    One(String),
    Many(Vec<String>),
}

impl OneOrMany {
    fn into_vec(self) -> Vec<String> {
        match self {
            OneOrMany::One(s) => vec![s],
            OneOrMany::Many(v) => v,
        }
    }
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
struct RawJapanese {
    enabled: Option<bool>,
    katakana_style: Option<KatakanaStyleConfig>,
    variants: BTreeMap<String, String>,
    variant_files: Vec<String>,
    flag_fullwidth_alnum: Option<bool>,
    flag_halfwidth_kana: Option<bool>,
    flag_compatibility: Option<bool>,
    kanji_consistency: Option<bool>,
    number_consistency: Option<bool>,
    punctuation_consistency: Option<bool>,
    fullwidth_space: Option<SpacePolicyConfig>,
}

impl RawConfig {
    pub fn parse(text: &str) -> anyhow::Result<Self> {
        Ok(toml::from_str(text)?)
    }

    /// Overlay `other` (higher priority) onto `self`. Scalars are replaced,
    /// lists and maps are extended.
    pub fn merge(mut self, other: RawConfig) -> Self {
        self.check.mode = other.check.mode.or(self.check.mode);
        self.check.locale = other.check.locale.or(self.check.locale);
        self.check.profile = other.check.profile.or(self.check.profile);
        self.check.min_word_len = other.check.min_word_len.or(self.check.min_word_len);
        self.check.max_token_len = other.check.max_token_len.or(self.check.max_token_len);

        self.files.exclude.extend(other.files.exclude);
        self.files.include_hidden = other.files.include_hidden.or(self.files.include_hidden);
        self.files.max_file_size = other.files.max_file_size.or(self.files.max_file_size);

        self.words.project = other.words.project.or(self.words.project);
        self.words.ignore.extend(other.words.ignore);
        self.words.dictionaries.extend(other.words.dictionaries);

        self.corrections.builtin = other.corrections.builtin.or(self.corrections.builtin);
        self.corrections.extra.extend(other.corrections.extra);
        self.corrections.words.extend(other.corrections.words);

        self.japanese.enabled = other.japanese.enabled.or(self.japanese.enabled);
        self.japanese.katakana_style = other
            .japanese
            .katakana_style
            .or(self.japanese.katakana_style);
        self.japanese.variants.extend(other.japanese.variants);
        self.japanese
            .variant_files
            .extend(other.japanese.variant_files);
        self.japanese.flag_fullwidth_alnum = other
            .japanese
            .flag_fullwidth_alnum
            .or(self.japanese.flag_fullwidth_alnum);
        self.japanese.flag_halfwidth_kana = other
            .japanese
            .flag_halfwidth_kana
            .or(self.japanese.flag_halfwidth_kana);
        self.japanese.flag_compatibility = other
            .japanese
            .flag_compatibility
            .or(self.japanese.flag_compatibility);
        self.japanese.kanji_consistency = other
            .japanese
            .kanji_consistency
            .or(self.japanese.kanji_consistency);
        self.japanese.number_consistency = other
            .japanese
            .number_consistency
            .or(self.japanese.number_consistency);
        self.japanese.punctuation_consistency = other
            .japanese
            .punctuation_consistency
            .or(self.japanese.punctuation_consistency);
        self.japanese.fullwidth_space = other
            .japanese
            .fullwidth_space
            .or(self.japanese.fullwidth_space);

        self.overrides.extend(other.overrides);
        self
    }

    pub fn finalize(self) -> Config {
        let mut ignore = self.words.ignore;
        for w in &mut ignore {
            *w = w.to_lowercase();
        }
        Config {
            check: CheckConfig {
                mode: self.check.mode.unwrap_or_default(),
                locale: self.check.locale.unwrap_or_default(),
                profile: self.check.profile.unwrap_or_default(),
                min_word_len: self.check.min_word_len.unwrap_or(3),
                max_token_len: self.check.max_token_len.unwrap_or(40),
            },
            files: FilesConfig {
                exclude: DEFAULT_EXCLUDES
                    .iter()
                    .map(ToString::to_string)
                    .chain(self.files.exclude)
                    .collect(),
                include_hidden: self.files.include_hidden.unwrap_or(false),
                max_file_size: self.files.max_file_size.unwrap_or(0),
            },
            words: WordsConfig {
                project: self
                    .words
                    .project
                    .unwrap_or_else(|| "ayame-words.txt".to_string()),
                ignore,
                dictionaries: self.words.dictionaries,
            },
            corrections: CorrectionsConfig {
                builtin: self.corrections.builtin.unwrap_or(true),
                extra: self.corrections.extra,
                words: self
                    .corrections
                    .words
                    .into_iter()
                    .map(|(k, v)| (k, v.into_vec()))
                    .collect(),
            },
            japanese: JapaneseConfig {
                enabled: self.japanese.enabled.unwrap_or(true),
                katakana_style: self.japanese.katakana_style.unwrap_or_default(),
                variants: self.japanese.variants,
                variant_files: self.japanese.variant_files,
                flag_fullwidth_alnum: self.japanese.flag_fullwidth_alnum.unwrap_or(true),
                flag_halfwidth_kana: self.japanese.flag_halfwidth_kana.unwrap_or(true),
                flag_compatibility: self.japanese.flag_compatibility.unwrap_or(true),
                kanji_consistency: self.japanese.kanji_consistency.unwrap_or(true),
                number_consistency: self.japanese.number_consistency.unwrap_or(true),
                punctuation_consistency: self.japanese.punctuation_consistency.unwrap_or(true),
                fullwidth_space: self.japanese.fullwidth_space.unwrap_or_default(),
            },
            overrides: self.overrides,
        }
    }
}

/// Validate one configuration file without discovery or merging.
///
/// This reports unknown keys before deserializing values so the error can
/// include a nearby valid key.
pub fn validate_config(text: &str) -> anyhow::Result<()> {
    let value: toml::Value = toml::from_str(text)?;
    validate_keys(&value, "")?;
    RawConfig::parse(text)?;
    Ok(())
}

fn validate_keys(value: &toml::Value, path: &str) -> anyhow::Result<()> {
    let Some(table) = value.as_table() else {
        return Ok(());
    };
    let Some(allowed) = allowed_keys(path) else {
        return Ok(());
    };
    for (key, child) in table {
        if !allowed.contains(&key.as_str()) {
            let suggestion = allowed
                .iter()
                .min_by_key(|candidate| edit_distance(key, candidate))
                .filter(|candidate| edit_distance(key, candidate) <= suggestion_limit(key))
                .map(|candidate| join_key(path, candidate));
            let unknown = join_key(path, key);
            if let Some(suggestion) = suggestion {
                anyhow::bail!("unknown config key `{unknown}`; did you mean `{suggestion}`?");
            }
            anyhow::bail!("unknown config key `{unknown}`");
        }
        let child_path = join_key(path, key);
        if child_path == "overrides" {
            if let Some(items) = child.as_array() {
                for item in items {
                    validate_keys(item, "overrides")?;
                }
            }
        } else {
            validate_keys(child, &child_path)?;
        }
    }
    Ok(())
}

fn allowed_keys(path: &str) -> Option<&'static [&'static str]> {
    match path {
        "" => Some(&[
            "check",
            "files",
            "words",
            "corrections",
            "japanese",
            "overrides",
        ]),
        "check" => Some(&["mode", "locale", "profile", "min-word-len", "max-token-len"]),
        "files" => Some(&["exclude", "include-hidden", "max-file-size"]),
        "words" => Some(&["project", "ignore", "dictionaries"]),
        "corrections" => Some(&["builtin", "extra", "words"]),
        "japanese" => Some(&[
            "enabled",
            "katakana-style",
            "variants",
            "variant-files",
            "flag-fullwidth-alnum",
            "flag-halfwidth-kana",
            "flag-compatibility",
            "kanji-consistency",
            "number-consistency",
            "punctuation-consistency",
            "fullwidth-space",
        ]),
        "overrides" => Some(&["paths", "mode", "profile", "japanese"]),
        // User-defined correction and variant keys are intentionally open.
        "corrections.words" | "japanese.variants" => None,
        _ => Some(&[]),
    }
}

fn join_key(path: &str, key: &str) -> String {
    if path.is_empty() {
        key.to_string()
    } else {
        format!("{path}.{key}")
    }
}

fn suggestion_limit(key: &str) -> usize {
    3.max(key.chars().count() / 3)
}

fn edit_distance(left: &str, right: &str) -> usize {
    let right: Vec<char> = right.chars().collect();
    let mut previous: Vec<usize> = (0..=right.len()).collect();
    for (row, left_char) in left.chars().enumerate() {
        let mut current = vec![row + 1];
        for (column, right_char) in right.iter().enumerate() {
            current.push(
                (previous[column + 1] + 1)
                    .min(current[column] + 1)
                    .min(previous[column] + usize::from(left_char != *right_char)),
            );
        }
        previous = current;
    }
    previous[right.len()]
}

/// A discovered and merged configuration.
#[derive(Debug, Clone)]
pub struct LoadedConfig {
    pub config: Config,
    /// Project root: the directory holding the config file, else the
    /// nearest `.git` ancestor, else the start directory.
    pub root: PathBuf,
    pub project_file: Option<PathBuf>,
    pub global_file: Option<PathBuf>,
}

impl LoadedConfig {
    /// Resolve a dictionary/corrections reference to a path.
    pub fn resolve_ref(&self, reference: &str) -> anyhow::Result<PathBuf> {
        if let Some(reference) = reference.strip_prefix("registry:") {
            let resolved = crate::registry_lock::resolve(&self.root, reference)?;
            if let Some(expected) = resolved.sha256.as_deref() {
                if resolved.path.is_file() {
                    crate::registry_lock::verify(&resolved.path, expected)?;
                }
            }
            Ok(resolved.path)
        } else {
            let p = Path::new(reference);
            Ok(if p.is_absolute() {
                p.to_path_buf()
            } else {
                self.root.join(p)
            })
        }
    }

    /// Absolute path of the project word file.
    pub fn project_words_path(&self) -> PathBuf {
        let p = Path::new(&self.config.words.project);
        if p.is_absolute() {
            p.to_path_buf()
        } else {
            self.root.join(p)
        }
    }
}

/// Walk up from `start` to find the project config, merge it over the
/// global config, and apply defaults.
pub fn discover(start: &Path) -> anyhow::Result<LoadedConfig> {
    let start = if start.is_file() {
        start.parent().unwrap_or(start).to_path_buf()
    } else {
        start.to_path_buf()
    };
    let start = start.canonicalize().unwrap_or_else(|_| start.to_path_buf());

    let mut project_file = None;
    let mut git_root = None;
    for dir in start.ancestors() {
        for name in PROJECT_FILE_NAMES {
            let candidate = dir.join(name);
            if candidate.is_file() {
                project_file = Some(candidate);
                break;
            }
        }
        if project_file.is_some() {
            break;
        }
        if git_root.is_none() && dir.join(".git").exists() {
            git_root = Some(dir.to_path_buf());
        }
    }

    let root = project_file
        .as_deref()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .or(git_root)
        .unwrap_or(start);

    let global_file = crate::global_config_dir()
        .map(|d| d.join("config.toml"))
        .filter(|p| p.is_file());

    let mut raw = RawConfig::default();
    if let Some(path) = &global_file {
        let text = std::fs::read_to_string(path)?;
        let parsed =
            RawConfig::parse(&text).map_err(|e| anyhow::anyhow!("in {}: {e}", path.display()))?;
        raw = raw.merge(parsed);
    }
    if let Some(path) = &project_file {
        let text = std::fs::read_to_string(path)?;
        let parsed =
            RawConfig::parse(&text).map_err(|e| anyhow::anyhow!("in {}: {e}", path.display()))?;
        raw = raw.merge(parsed);
    }

    Ok(LoadedConfig {
        config: raw.finalize(),
        root,
        project_file,
        global_file,
    })
}

/// Load one explicit configuration file without merging project or global
/// configuration. This is used by the CLI's reproducible `--config` mode.
pub fn load_explicit(path: &Path) -> anyhow::Result<LoadedConfig> {
    let path = path
        .canonicalize()
        .map_err(|e| anyhow::anyhow!("cannot open {}: {e}", path.display()))?;
    anyhow::ensure!(path.is_file(), "config is not a file: {}", path.display());
    let text = std::fs::read_to_string(&path)?;
    let raw = RawConfig::parse(&text).map_err(|e| anyhow::anyhow!("in {}: {e}", path.display()))?;
    let root = path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    Ok(LoadedConfig {
        config: raw.finalize(),
        root,
        project_file: Some(path),
        global_file: None,
    })
}

/// Load configuration according to explicit CLI selection.
///
/// `explicit` loads exactly that file. `no_config` ignores both project and
/// global files. Otherwise this is equivalent to [`discover`].
pub fn discover_selected(
    start: &Path,
    explicit: Option<&Path>,
    no_config: bool,
) -> anyhow::Result<LoadedConfig> {
    if let Some(path) = explicit {
        anyhow::ensure!(
            !no_config,
            "--config and --no-config cannot be used together"
        );
        load_explicit(path)
    } else if no_config {
        let root = if start.is_file() {
            start.parent().unwrap_or(start)
        } else {
            start
        };
        Ok(defaults(
            &root.canonicalize().unwrap_or_else(|_| root.to_path_buf()),
        ))
    } else {
        discover(start)
    }
}

/// Defaults with no config files at all, rooted at `root`.
pub fn defaults(root: &Path) -> LoadedConfig {
    LoadedConfig {
        config: RawConfig::default().finalize(),
        root: root.to_path_buf(),
        project_file: None,
        global_file: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_sensible() {
        let c = RawConfig::default().finalize();
        assert_eq!(c.check.mode, Mode::Corrections);
        assert_eq!(c.check.locale, EnglishLocale::Any);
        assert_eq!(c.check.profile, SyntaxProfile::All);
        assert_eq!(c.check.min_word_len, 3);
        assert!(c.corrections.builtin);
        assert!(c.japanese.enabled);
        assert_eq!(c.words.project, "ayame-words.txt");
    }

    #[test]
    fn parse_and_merge() {
        let global = RawConfig::parse(
            r#"
            [words]
            ignore = ["globalword"]
            "#,
        )
        .unwrap();
        let project = RawConfig::parse(
            r#"
            [check]
            mode = "dictionary"
            locale = "en-GB"
            profile = "auto"

            [words]
            ignore = ["projectword"]
            dictionaries = ["registry:en-base"]

            [corrections.words]
            teh = "the"
            neet = "neet"

            [japanese]
            katakana-style = "long"

            [[overrides]]
            paths = ["docs/**"]
            mode = "corrections"
            profile = "prose"
            "#,
        )
        .unwrap();
        let c = global.merge(project).finalize();
        assert_eq!(c.check.mode, Mode::Dictionary);
        assert_eq!(c.check.locale, EnglishLocale::EnGb);
        assert_eq!(c.check.profile, SyntaxProfile::Auto);
        assert_eq!(c.words.ignore, ["globalword", "projectword"]);
        assert_eq!(c.corrections.words["teh"], vec!["the"]);
        assert_eq!(c.japanese.katakana_style, KatakanaStyleConfig::Long);
        assert_eq!(c.overrides.len(), 1);
    }

    #[test]
    fn unknown_keys_are_rejected() {
        assert!(RawConfig::parse("[check]\ntypo-key = 1\n").is_err());
    }

    #[test]
    fn validation_suggests_the_nearest_full_key() {
        let error = validate_config("[japanese]\nkatakana-stle = \"long\"\n")
            .unwrap_err()
            .to_string();
        assert!(error.contains("unknown config key `japanese.katakana-stle`"));
        assert!(error.contains("did you mean `japanese.katakana-style`?"));
    }

    #[test]
    fn published_schema_tracks_every_effective_config_key() {
        let schema: serde_json::Value = serde_json::from_str(CONFIG_SCHEMA).unwrap();
        let config = serde_json::to_value(RawConfig::default().finalize()).unwrap();
        assert_schema_keys(&schema, &config);

        let override_value = serde_json::to_value(OverrideConfig {
            paths: vec!["docs/**".to_string()],
            mode: Some(Mode::Dictionary),
            profile: Some(SyntaxProfile::Prose),
            japanese: Some(false),
        })
        .unwrap();
        assert_eq!(
            sorted_keys(&schema["properties"]["overrides"]["items"]["properties"]),
            sorted_keys(&override_value)
        );
    }

    fn assert_schema_keys(schema: &serde_json::Value, config: &serde_json::Value) {
        assert_eq!(
            sorted_keys(&schema["properties"]),
            sorted_keys(config),
            "top-level schema keys must match Config"
        );
        for section in ["check", "files", "words", "corrections", "japanese"] {
            assert_eq!(
                sorted_keys(&schema["properties"][section]["properties"]),
                sorted_keys(&config[section]),
                "schema keys must match Config.{section}"
            );
        }
    }

    fn sorted_keys(value: &serde_json::Value) -> Vec<&str> {
        let mut keys: Vec<&str> = value
            .as_object()
            .expect("JSON object")
            .keys()
            .map(String::as_str)
            .collect();
        keys.sort_unstable();
        keys
    }

    #[test]
    fn explicit_config_does_not_merge_global_or_project_files() {
        let temp = tempfile::tempdir().unwrap();
        let explicit = temp.path().join("custom.toml");
        std::fs::write(&explicit, "[check]\nmode = \"off\"\n").unwrap();

        let loaded = load_explicit(&explicit).unwrap();

        assert_eq!(loaded.config.check.mode, Mode::Off);
        assert_eq!(loaded.project_file.as_deref(), Some(explicit.as_path()));
        assert!(loaded.global_file.is_none());
    }
}
