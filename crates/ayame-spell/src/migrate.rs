//! Migration from cSpell, typos, and prh configuration.

use std::collections::{BTreeSet, HashSet};
use std::path::{Path, PathBuf};

use anyhow::Context;
use clap::Subcommand;
use regex::Regex;
use serde::{Deserialize, Serialize};
use toml_edit::{Array, DocumentMut, Item, Table};

#[derive(Subcommand)]
pub enum ImportCmd {
    /// Import cSpell words, ignores, paths, and known dictionaries.
    Cspell {
        /// cspell.json, .cspell.json, or cspell.config.yaml.
        path: Option<PathBuf>,
        /// Print the merged output without writing.
        #[arg(long)]
        dry_run: bool,
    },
    /// Import _typos.toml extend-words and extend-exclude.
    Typos {
        /// typos configuration (default: _typos.toml).
        path: Option<PathBuf>,
        /// Print the merged output without writing.
        #[arg(long)]
        dry_run: bool,
    },
    /// Import a supported subset of prh YAML rules.
    Prh {
        /// prh YAML rule file.
        path: PathBuf,
        /// Project-relative TOML rule file to generate.
        #[arg(long, default_value = "dict/imported-prh.toml")]
        output: PathBuf,
        /// Print the merged config and rule file without writing.
        #[arg(long)]
        dry_run: bool,
    },
}

pub fn run(command: ImportCmd) -> anyhow::Result<i32> {
    match command {
        ImportCmd::Cspell { path, dry_run } => import_cspell(path.as_deref(), dry_run),
        ImportCmd::Typos { path, dry_run } => import_typos(path.as_deref(), dry_run),
        ImportCmd::Prh {
            path,
            output,
            dry_run,
        } => import_prh(&path, &output, dry_run),
    }
}

fn import_cspell(path: Option<&Path>, dry_run: bool) -> anyhow::Result<i32> {
    let cwd = std::env::current_dir()?;
    let path = path
        .map(Path::to_path_buf)
        .or_else(|| {
            [
                "cspell.json",
                ".cspell.json",
                "cspell.config.yaml",
                "cspell.config.yml",
            ]
            .into_iter()
            .map(|name| cwd.join(name))
            .find(|path| path.is_file())
        })
        .context("no cSpell config found; pass cspell.json or cspell.config.yaml")?;
    let text = std::fs::read_to_string(&path)
        .with_context(|| format!("cannot read {}", path.display()))?;
    let value: serde_json::Value = if matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("yaml" | "yml")
    ) {
        let yaml: serde_yaml::Value = serde_yaml::from_str(&text)
            .with_context(|| format!("cannot parse {}", path.display()))?;
        serde_json::to_value(yaml)?
    } else {
        serde_json::from_str(&text).with_context(|| format!("cannot parse {}", path.display()))?
    };
    let config: CspellConfig = serde_json::from_value(value.clone())?;
    let mut untranslated = unsupported_json_keys(
        &value,
        &[
            "$schema",
            "version",
            "words",
            "ignoreWords",
            "ignorePaths",
            "dictionaries",
        ],
    );

    let loaded = ayame_spell_core::config::discover(&cwd)?;
    let (config_path, mut document) = project_document(&loaded)?;
    extend_array(&mut document, "files", "exclude", config.ignore_paths);
    extend_array(&mut document, "words", "ignore", config.ignore_words);
    let mut registry = Vec::new();
    for dictionary in config.dictionaries {
        match cspell_dictionary(&dictionary) {
            Some(name) => registry.push(format!("registry:{name}")),
            None => untranslated.push(format!(
                "cSpell dictionary `{dictionary}` has no ayame-spell registry mapping"
            )),
        }
    }
    extend_array(&mut document, "words", "dictionaries", registry);
    set_default_string(&mut document, "words", "project", "ayame-words.txt");

    let words_path = loaded.project_words_path();
    let mut words = existing_words(&words_path)?;
    words.extend(
        config
            .words
            .into_iter()
            .map(|word| word.trim().to_string())
            .filter(|word| !word.is_empty()),
    );
    finish_import(
        &config_path,
        &document,
        Some((&words_path, render_words(&words))),
        None,
        dry_run,
        &untranslated,
    )
}

#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct CspellConfig {
    words: Vec<String>,
    ignore_words: Vec<String>,
    ignore_paths: Vec<String>,
    dictionaries: Vec<String>,
}

fn cspell_dictionary(name: &str) -> Option<&'static str> {
    match name.to_ascii_lowercase().as_str() {
        "aws" => Some("aws"),
        "cpp" | "c-cpp" => Some("cpp"),
        "companies" | "softwareterms" | "web" => Some("web"),
        "datascience" | "data-science" => Some("data-science"),
        "docker" | "kubernetes" => Some("docker-k8s"),
        "dotnet" | "csharp" => Some("dotnet"),
        "finance" => Some("finance"),
        "gcp" => Some("gcp"),
        "go" | "golang" => Some("go"),
        "java" | "kotlin" => Some("java-kotlin"),
        "node" | "npm" | "typescript" => Some("typescript-node"),
        "python" => Some("python"),
        "rust" => Some("rust"),
        "terraform" => Some("terraform"),
        _ => None,
    }
}

fn import_typos(path: Option<&Path>, dry_run: bool) -> anyhow::Result<i32> {
    let cwd = std::env::current_dir()?;
    let path = path
        .map(Path::to_path_buf)
        .unwrap_or_else(|| cwd.join("_typos.toml"));
    let text = std::fs::read_to_string(&path)
        .with_context(|| format!("cannot read {}", path.display()))?;
    let value: toml::Value =
        toml::from_str(&text).with_context(|| format!("cannot parse {}", path.display()))?;
    let loaded = ayame_spell_core::config::discover(&cwd)?;
    let (config_path, mut document) = project_document(&loaded)?;
    let mut untranslated = Vec::new();

    if let Some(words) = value
        .get("default")
        .and_then(|default| default.get("extend-words"))
        .and_then(toml::Value::as_table)
    {
        for (word, replacement) in words {
            if let Some(replacement) = replacement.as_str() {
                set_correction(&mut document, word, replacement);
            } else {
                untranslated.push(format!("default.extend-words.{word} is not a string"));
            }
        }
    }
    if let Some(exclude) = value
        .get("files")
        .and_then(|files| files.get("extend-exclude"))
    {
        let values = match exclude {
            toml::Value::String(value) => vec![value.clone()],
            toml::Value::Array(values) => values
                .iter()
                .filter_map(|value| {
                    value.as_str().map(str::to_string).or_else(|| {
                        untranslated
                            .push("files.extend-exclude contains a non-string entry".to_string());
                        None
                    })
                })
                .collect(),
            _ => {
                untranslated.push("files.extend-exclude is not a string or array".to_string());
                Vec::new()
            }
        };
        extend_array(&mut document, "files", "exclude", values);
    }
    if let Some(table) = value.as_table() {
        for key in table
            .keys()
            .filter(|key| !matches!(key.as_str(), "default" | "files"))
        {
            untranslated.push(format!("top-level typos key `{key}` is not translated"));
        }
    }
    report_unsupported_toml_keys(&value, "default", &["extend-words"], &mut untranslated);
    report_unsupported_toml_keys(&value, "files", &["extend-exclude"], &mut untranslated);
    finish_import(&config_path, &document, None, None, dry_run, &untranslated)
}

#[derive(Serialize)]
struct PrhOutput {
    rules: Vec<PrhOutputRule>,
}

#[derive(Serialize)]
struct PrhOutputRule {
    pattern: String,
    replace: String,
}

fn import_prh(path: &Path, output: &Path, dry_run: bool) -> anyhow::Result<i32> {
    let text =
        std::fs::read_to_string(path).with_context(|| format!("cannot read {}", path.display()))?;
    let value: serde_yaml::Value =
        serde_yaml::from_str(&text).with_context(|| format!("cannot parse {}", path.display()))?;
    let rules = value
        .get("rules")
        .and_then(serde_yaml::Value::as_sequence)
        .context("prh file has no `rules` sequence")?;
    let mut translated = Vec::new();
    let mut untranslated = Vec::new();
    if let Some(mapping) = value.as_mapping() {
        for key in mapping
            .keys()
            .filter_map(serde_yaml::Value::as_str)
            .filter(|key| !matches!(*key, "version" | "rules"))
        {
            untranslated.push(format!("top-level prh key `{key}` is not translated"));
        }
    }
    for (index, rule) in rules.iter().enumerate() {
        if let Some(mapping) = rule.as_mapping() {
            for key in mapping
                .keys()
                .filter_map(serde_yaml::Value::as_str)
                .filter(|key| !matches!(*key, "expected" | "pattern" | "patterns"))
            {
                untranslated.push(format!("rule {} key `{key}` is not translated", index + 1));
            }
        }
        let expected = rule.get("expected").and_then(serde_yaml::Value::as_str);
        let patterns = rule.get("patterns").or_else(|| rule.get("pattern"));
        let Some(expected) = expected else {
            untranslated.push(format!("rule {} has no string `expected`", index + 1));
            continue;
        };
        let patterns: Vec<&str> = match patterns {
            Some(serde_yaml::Value::String(pattern)) => vec![pattern],
            Some(serde_yaml::Value::Sequence(patterns)) => {
                let mut strings = Vec::new();
                for (pattern_index, pattern) in patterns.iter().enumerate() {
                    if let Some(pattern) = pattern.as_str() {
                        strings.push(pattern);
                    } else {
                        untranslated.push(format!(
                            "rule {} pattern {} is not a string",
                            index + 1,
                            pattern_index + 1
                        ));
                    }
                }
                strings
            }
            _ => {
                untranslated.push(format!("rule {} has no string pattern", index + 1));
                continue;
            }
        };
        for pattern in patterns {
            let pattern = match normalize_prh_pattern(pattern) {
                Ok(pattern) => pattern,
                Err(error) => {
                    untranslated.push(format!(
                        "rule {} pattern `{pattern}` is unsupported: {error}",
                        index + 1
                    ));
                    continue;
                }
            };
            match Regex::new(&pattern) {
                Ok(_) => translated.push(PrhOutputRule {
                    pattern,
                    replace: expected.to_string(),
                }),
                Err(error) => untranslated.push(format!(
                    "rule {} pattern `{}` is unsupported: {error}",
                    index + 1,
                    pattern
                )),
            }
        }
    }
    anyhow::ensure!(!translated.is_empty(), "no prh rules could be translated");

    let cwd = std::env::current_dir()?;
    let loaded = ayame_spell_core::config::discover(&cwd)?;
    let (config_path, mut document) = project_document(&loaded)?;
    let output_path = if output.is_absolute() {
        output.to_path_buf()
    } else {
        loaded.root.join(output)
    };
    let reference = output_path
        .strip_prefix(&loaded.root)
        .unwrap_or(&output_path)
        .to_string_lossy()
        .replace('\\', "/");
    extend_array(&mut document, "japanese", "variant-files", [reference]);
    set_bool(&mut document, "japanese", "enabled", true);
    let rules_text = toml::to_string_pretty(&PrhOutput { rules: translated })?;
    finish_import(
        &config_path,
        &document,
        None,
        Some((&output_path, rules_text)),
        dry_run,
        &untranslated,
    )
}

fn normalize_prh_pattern(pattern: &str) -> anyhow::Result<String> {
    if pattern.starts_with('/') {
        if let Some(end) = pattern.rfind('/') {
            if end > 0 {
                let expression = &pattern[1..end];
                let flags = &pattern[end + 1..];
                if !flags.chars().all(|flag| flag == 'i') {
                    anyhow::bail!("only the `i` regex flag is supported");
                }
                return Ok(if flags.contains('i') {
                    format!("(?i){expression}")
                } else {
                    expression.to_string()
                });
            }
        }
    }
    Ok(regex::escape(pattern))
}

fn report_unsupported_toml_keys(
    value: &toml::Value,
    table_name: &str,
    supported: &[&str],
    untranslated: &mut Vec<String>,
) {
    if let Some(table) = value.get(table_name).and_then(toml::Value::as_table) {
        for key in table
            .keys()
            .filter(|key| !supported.contains(&key.as_str()))
        {
            untranslated.push(format!("typos key `{table_name}.{key}` is not translated"));
        }
    }
}

fn project_document(
    loaded: &ayame_spell_core::LoadedConfig,
) -> anyhow::Result<(PathBuf, DocumentMut)> {
    let path = loaded
        .project_file
        .clone()
        .unwrap_or_else(|| loaded.root.join("ayame-spell.toml"));
    let text = std::fs::read_to_string(&path).unwrap_or_default();
    let document = text
        .parse()
        .with_context(|| format!("cannot parse {}", path.display()))?;
    Ok((path, document))
}

fn table_mut<'a>(document: &'a mut DocumentMut, name: &str) -> anyhow::Result<&'a mut Table> {
    document
        .entry(name)
        .or_insert(Item::Table(Table::new()))
        .as_table_mut()
        .with_context(|| format!("[{name}] is not a table"))
}

fn extend_array(
    document: &mut DocumentMut,
    table: &str,
    key: &str,
    values: impl IntoIterator<Item = String>,
) {
    let table = table_mut(document, table).expect("known config table");
    let item = table.entry(key).or_insert(toml_edit::value(Array::new()));
    let array = item.as_array_mut().expect("known config array");
    let mut existing: HashSet<String> = array
        .iter()
        .filter_map(|value| value.as_str().map(str::to_string))
        .collect();
    for value in values {
        if existing.insert(value.clone()) {
            array.push(value);
        }
    }
}

fn set_default_string(document: &mut DocumentMut, table: &str, key: &str, value: &str) {
    let table = table_mut(document, table).expect("known config table");
    if !table.contains_key(key) {
        table[key] = toml_edit::value(value);
    }
}

fn set_bool(document: &mut DocumentMut, table: &str, key: &str, value: bool) {
    table_mut(document, table).expect("known config table")[key] = toml_edit::value(value);
}

fn set_correction(document: &mut DocumentMut, word: &str, replacement: &str) {
    let corrections = table_mut(document, "corrections").expect("known config table");
    let words = corrections
        .entry("words")
        .or_insert(Item::Table(Table::new()))
        .as_table_mut()
        .expect("[corrections.words] table");
    words[word] = toml_edit::value(replacement);
}

fn existing_words(path: &Path) -> anyhow::Result<BTreeSet<String>> {
    match std::fs::read_to_string(path) {
        Ok(text) => Ok(text
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(str::to_string)
            .collect()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(BTreeSet::new()),
        Err(error) => Err(error.into()),
    }
}

fn render_words(words: &BTreeSet<String>) -> String {
    words.iter().cloned().collect::<Vec<_>>().join("\n") + "\n"
}

fn finish_import(
    config_path: &Path,
    document: &DocumentMut,
    words: Option<(&Path, String)>,
    generated: Option<(&Path, String)>,
    dry_run: bool,
    untranslated: &[String],
) -> anyhow::Result<i32> {
    if dry_run {
        println!("# {}\n{}", config_path.display(), document);
        if let Some((path, text)) = &words {
            println!("\n# {}\n{}", path.display(), text);
        }
        if let Some((path, text)) = &generated {
            println!("\n# {}\n{}", path.display(), text);
        }
    } else {
        std::fs::write(config_path, document.to_string())
            .with_context(|| format!("cannot write {}", config_path.display()))?;
        if let Some((path, text)) = words {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(path, text)?;
        }
        if let Some((path, text)) = generated {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(path, text)?;
        }
        println!("updated {}", config_path.display());
    }
    if untranslated.is_empty() {
        eprintln!("imported all supported settings");
    } else {
        eprintln!("not translated ({}):", untranslated.len());
        for item in untranslated {
            eprintln!("- {item}");
        }
    }
    Ok(0)
}

fn unsupported_json_keys(value: &serde_json::Value, supported: &[&str]) -> Vec<String> {
    value
        .as_object()
        .into_iter()
        .flatten()
        .map(|(key, _)| key)
        .filter(|key| !supported.contains(&key.as_str()))
        .map(|key| format!("top-level cSpell key `{key}` is not translated"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prh_regex_literals_and_flags_are_normalized() {
        assert_eq!(
            normalize_prh_pattern("/Web ?サイト/i").unwrap(),
            "(?i)Web ?サイト"
        );
        assert_eq!(normalize_prh_pattern("C++").unwrap(), "C\\+\\+");
        assert!(normalize_prh_pattern("/word/g").is_err());
    }

    #[test]
    fn dictionary_names_map_to_registry_packs() {
        assert_eq!(cspell_dictionary("typescript"), Some("typescript-node"));
        assert_eq!(cspell_dictionary("golang"), Some("go"));
        assert_eq!(cspell_dictionary("private-team"), None);
    }

    #[test]
    fn cspell_import_preserves_a_custom_project_word_file() {
        let mut document: DocumentMut = "[words]\nproject = \"team.txt\"\n".parse().unwrap();
        set_default_string(&mut document, "words", "project", "ayame-words.txt");
        assert_eq!(document["words"]["project"].as_str(), Some("team.txt"));

        let mut document = DocumentMut::new();
        set_default_string(&mut document, "words", "project", "ayame-words.txt");
        assert_eq!(
            document["words"]["project"].as_str(),
            Some("ayame-words.txt")
        );
    }
}
