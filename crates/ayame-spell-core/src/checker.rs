//! The orchestrating checker: applies configuration, runs the tokenizer,
//! correction tables, wordlist dictionaries, and Japanese checks over a
//! text, and returns [`Issue`]s.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use globset::{GlobBuilder, GlobSet, GlobSetBuilder};

use crate::config::{EnglishLocale, LoadedConfig, Mode, SyntaxProfile};
use crate::corrections::{Corrections, Verdict};
use crate::dictionary::WordSets;
use crate::issue::{Issue, IssueKind};
use crate::japanese::{self, JapaneseChecker, KatakanaOcc};
use crate::tokenizer::{self, TokenizerOptions};

/// Inline directives recognized anywhere in a file/line.
const DIRECTIVE_PREFIX: &[u8] = b"ayame-spell:";

const PROSE_EXTENSIONS: [&str; 10] = [
    "md", "markdown", "txt", "text", "rst", "adoc", "asciidoc", "tex", "org", "typ",
];

struct CompiledOverride {
    globs: GlobSet,
    mode: Option<Mode>,
    profile: Option<SyntaxProfile>,
    japanese: Option<bool>,
}

pub struct Checker {
    mode: Mode,
    profile: SyntaxProfile,
    ja_enabled: bool,
    corrections: Corrections,
    locale_variants: HashMap<String, String>,
    words: WordSets,
    /// Words never flagged by any check: config ignore + project words +
    /// global words, lowercased.
    allow: HashSet<String>,
    ja: Option<JapaneseChecker>,
    overrides: Vec<CompiledOverride>,
    tok: TokenizerOptions,
}

impl Checker {
    /// Build a checker from a loaded configuration. Missing dictionary
    /// files are reported as warnings, not errors, so a half-set-up
    /// project still checks what it can.
    pub fn new(loaded: &LoadedConfig) -> (Self, Vec<String>) {
        let cfg = &loaded.config;
        let mut warnings = Vec::new();

        let mut corrections = Corrections::new(cfg.corrections.builtin);
        for (typo, fixes) in &cfg.corrections.words {
            corrections.insert(typo, fixes);
        }
        for reference in &cfg.corrections.extra {
            match loaded.resolve_ref(reference) {
                Ok(path) if path.is_file() => {
                    if let Err(e) = corrections.load_tsv(&path) {
                        warnings.push(e.to_string());
                    }
                }
                Ok(path) => warnings.push(missing_ref(reference, &path)),
                Err(e) => warnings.push(e.to_string()),
            }
        }

        let dictionary_possible = cfg.check.mode == Mode::Dictionary
            || cfg
                .overrides
                .iter()
                .any(|o| o.mode == Some(Mode::Dictionary));
        let mut words = if dictionary_possible {
            WordSets::with_code_terms()
        } else {
            WordSets::default()
        };
        if dictionary_possible {
            for reference in &cfg.words.dictionaries {
                match loaded.resolve_ref(reference) {
                    Ok(path) if path.is_file() => {
                        if let Err(e) = words.add_wordlist_file(&path) {
                            warnings.push(e.to_string());
                        }
                    }
                    Ok(path) => warnings.push(missing_ref(reference, &path)),
                    Err(e) => warnings.push(e.to_string()),
                }
            }
        }

        let mut allow: HashSet<String> = cfg.words.ignore.iter().cloned().collect();
        for path in [
            Some(loaded.project_words_path()),
            crate::global_words_path(),
        ]
        .into_iter()
        .flatten()
        {
            if let Ok(text) = std::fs::read_to_string(&path) {
                for line in text.lines() {
                    let w = line.trim();
                    if !w.is_empty() && !w.starts_with('#') {
                        allow.insert(w.to_lowercase());
                    }
                }
            }
        }

        let ja = cfg.japanese.enabled.then(|| {
            let mut ja = JapaneseChecker::new(
                cfg.japanese.katakana_style.into(),
                crate::japanese::JapaneseOptions {
                    flag_fullwidth_alnum: cfg.japanese.flag_fullwidth_alnum,
                    flag_halfwidth_kana: cfg.japanese.flag_halfwidth_kana,
                    flag_compatibility: cfg.japanese.flag_compatibility,
                    kanji_consistency: cfg.japanese.kanji_consistency,
                    number_consistency: cfg.japanese.number_consistency,
                    punctuation_consistency: cfg.japanese.punctuation_consistency,
                    fullwidth_space: cfg.japanese.fullwidth_space.into(),
                },
            );
            for (variant, preferred) in &cfg.japanese.variants {
                ja.add_variant(variant, preferred);
            }
            for reference in &cfg.japanese.variant_files {
                match loaded.resolve_ref(reference) {
                    Ok(path) if path.is_file() => match std::fs::read_to_string(&path) {
                        Ok(text) => {
                            if let Err(e) = ja.load_variant_rules(&text) {
                                warnings.push(format!("in {}: {e}", path.display()));
                            }
                        }
                        Err(e) => warnings.push(format!("{}: {e}", path.display())),
                    },
                    Ok(path) => warnings.push(missing_ref(reference, &path)),
                    Err(e) => warnings.push(e.to_string()),
                }
            }
            ja
        });

        let mut overrides = Vec::new();
        for o in &cfg.overrides {
            let mut builder = GlobSetBuilder::new();
            let mut ok = true;
            for pattern in &o.paths {
                match GlobBuilder::new(pattern).literal_separator(false).build() {
                    Ok(g) => {
                        builder.add(g);
                    }
                    Err(e) => {
                        warnings.push(format!("invalid override glob `{pattern}`: {e}"));
                        ok = false;
                    }
                }
            }
            if ok {
                if let Ok(globs) = builder.build() {
                    overrides.push(CompiledOverride {
                        globs,
                        mode: o.mode,
                        profile: o.profile,
                        japanese: o.japanese,
                    });
                }
            }
        }

        let checker = Self {
            mode: cfg.check.mode,
            profile: cfg.check.profile,
            ja_enabled: cfg.japanese.enabled,
            corrections,
            locale_variants: english_variants(cfg.check.locale),
            words,
            allow,
            ja,
            overrides,
            tok: TokenizerOptions {
                min_word_len: cfg.check.min_word_len,
                max_token_len: cfg.check.max_token_len,
            },
        };
        (checker, warnings)
    }

    /// Add words to the in-memory allow set (mirrors what an editor "add
    /// word" action just wrote to disk, without a full reload).
    pub fn allow_words<I: IntoIterator<Item = S>, S: AsRef<str>>(&mut self, words: I) {
        for w in words {
            self.allow.insert(w.as_ref().trim().to_lowercase());
        }
    }

    /// Effective settings for a file (path relative to the project root).
    fn effective(&self, path: Option<&Path>) -> (Mode, SyntaxProfile, bool, bool) {
        let mut mode = self.mode;
        let mut profile = self.profile;
        let mut ja_on = self.ja_enabled;
        if let Some(p) = path {
            for o in &self.overrides {
                if o.globs.is_match(p) {
                    if let Some(m) = o.mode {
                        mode = m;
                    }
                    if let Some(value) = o.profile {
                        profile = value;
                    }
                    if let Some(j) = o.japanese {
                        ja_on = j;
                    }
                }
            }
        }
        let is_prose = path
            .and_then(Path::extension)
            .and_then(|e| e.to_str())
            .is_some_and(|e| {
                let e = e.to_ascii_lowercase();
                PROSE_EXTENSIONS.contains(&e.as_str())
            });
        (mode, profile, ja_on && self.ja.is_some(), is_prose)
    }

    /// Check a text. `path` (relative to the project root) selects
    /// per-glob overrides and the prose/code distinction.
    pub fn check(&self, text: &str, path: Option<&Path>) -> Vec<Issue> {
        let (mode, profile, ja_on, is_prose) = self.effective(path);
        if mode == Mode::Off && !ja_on {
            return Vec::new();
        }
        if memchr::memmem::find(text.as_bytes(), b"ayame-spell:ignore-file").is_some() {
            return Vec::new();
        }
        let checked_text = crate::syntax::mask(text, path, profile);

        let mut issues = Vec::new();
        let mut occs: Vec<KatakanaOcc> = Vec::new();
        let want_consistency = ja_on
            && self
                .ja
                .as_ref()
                .is_some_and(JapaneseChecker::wants_consistency);

        let mut offset = 0usize;
        let mut line_no = 0u32;
        let mut skip_next = false;
        for raw_line in checked_text.split_inclusive('\n') {
            line_no += 1;
            let line_offset = offset;
            offset += raw_line.len();
            let line = raw_line.trim_end_matches(['\n', '\r']);

            if skip_next {
                skip_next = false;
                continue;
            }
            if let Some(pos) = memchr::memmem::find(line.as_bytes(), DIRECTIVE_PREFIX) {
                let rest = &line[pos..];
                if rest.starts_with("ayame-spell:ignore-next-line") {
                    skip_next = true;
                    continue;
                }
                if rest.starts_with("ayame-spell:ignore-line") {
                    continue;
                }
            }

            if mode != Mode::Off {
                self.check_english(line, line_no, line_offset, mode, &mut issues);
            }
            if ja_on {
                if let Some(ja) = &self.ja {
                    let occs_ref = if want_consistency {
                        Some(&mut occs)
                    } else {
                        None
                    };
                    ja.check_line(line, line_no, line_offset, is_prose, &mut issues, occs_ref);
                }
            }
        }

        if want_consistency {
            issues.extend(japanese::consistency_issues(&occs));
        }
        if ja_on {
            if let Some(ja) = &self.ja {
                issues.extend(ja.document_issues(&checked_text));
            }
        }
        // The allow list also silences Japanese findings (exact form).
        issues.retain(|issue| !self.allow.contains(&issue.word.to_lowercase()));
        issues.sort_by_key(|i| i.offset);
        issues
    }

    fn check_english(
        &self,
        line: &str,
        line_no: u32,
        line_offset: usize,
        mode: Mode,
        issues: &mut Vec<Issue>,
    ) {
        for w in tokenizer::words_in_line(line, &self.tok) {
            let lower = w.text.to_ascii_lowercase();
            if self.allow.contains(&lower) {
                continue;
            }
            match self.corrections.check(w.text) {
                Some(Verdict::Allowed) => continue,
                Some(Verdict::Typo(suggestions)) => {
                    issues.push(Issue {
                        line: line_no,
                        col: w.start,
                        offset: line_offset + w.start,
                        len: w.text.len(),
                        word: w.text.to_string(),
                        kind: IssueKind::Typo,
                        suggestions,
                    });
                    continue;
                }
                None => {}
            }
            if let Some(preferred) = self.locale_variants.get(&lower) {
                issues.push(Issue {
                    line: line_no,
                    col: w.start,
                    offset: line_offset + w.start,
                    len: w.text.len(),
                    word: w.text.to_string(),
                    kind: IssueKind::EnVariant,
                    suggestions: vec![tokenizer::match_case(w.text, preferred)],
                });
                continue;
            }
            if mode == Mode::Dictionary
                && w.text.len() >= 4
                && !w.text.bytes().all(|b| b.is_ascii_uppercase())
                && !self.words.contains(&lower)
            {
                let suggestions = self
                    .words
                    .suggest(&lower, 4)
                    .into_iter()
                    .map(|s| tokenizer::match_case(w.text, &s))
                    .collect();
                issues.push(Issue {
                    line: line_no,
                    col: w.start,
                    offset: line_offset + w.start,
                    len: w.text.len(),
                    word: w.text.to_string(),
                    kind: IssueKind::UnknownWord,
                    suggestions,
                });
            }
        }
    }
}

fn english_variants(locale: EnglishLocale) -> HashMap<String, String> {
    let mut variants = HashMap::new();
    if locale == EnglishLocale::Any {
        return variants;
    }
    for line in include_str!("../data/en-variants.tsv").lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((us, gb)) = line.split_once('\t') {
            let (variant, preferred) = match locale {
                EnglishLocale::Any => continue,
                EnglishLocale::EnUs => (gb, us),
                EnglishLocale::EnGb => (us, gb),
            };
            variants.insert(variant.to_string(), preferred.to_string());
        }
    }
    variants
}

fn missing_ref(reference: &str, path: &Path) -> String {
    if let Some(name) = reference.strip_prefix("registry:") {
        format!(
            "dictionary `{reference}` is not installed (expected {}); run `ayame-spell dict add {name}`",
            path.display()
        )
    } else {
        format!("dictionary file not found: {}", path.display())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{defaults, RawConfig};
    use proptest::prelude::*;

    fn checker_with(toml: &str) -> Checker {
        let mut loaded = defaults(Path::new("/nonexistent-root"));
        loaded.config = RawConfig::parse(toml).unwrap().finalize();
        Checker::new(&loaded).0
    }

    #[cfg(feature = "builtin-en")]
    #[test]
    fn corrections_mode_end_to_end() {
        let c = checker_with("");
        let issues = c.check("We recieve data,\nthen send it back.\n", None);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].word, "recieve");
        assert_eq!(issues[0].kind, IssueKind::Typo);
        assert_eq!(issues[0].line, 1);
        assert_eq!(issues[0].col, 3);
        assert!(issues[0].suggestions.contains(&"receive".to_string()));
    }

    #[cfg(feature = "builtin-en")]
    #[test]
    fn ignore_list_and_inline_whitelist() {
        let c = checker_with(
            r#"
            [words]
            ignore = ["recieve"]
            "#,
        );
        assert!(c.check("recieve", None).is_empty());
        let c = checker_with("[corrections.words]\nrecieve = \"recieve\"\n");
        assert!(c.check("recieve", None).is_empty());
    }

    #[test]
    fn directives() {
        let c = checker_with("");
        assert!(c
            .check(
                "recieve everything # ayame-spell:ignore-file\nrecieve\n",
                None
            )
            .is_empty());
        assert!(c
            .check("# ayame-spell:ignore-next-line\nrecieve\n", None)
            .is_empty());
        assert!(c
            .check("recieve # ayame-spell:ignore-line\n", None)
            .is_empty());
    }

    #[test]
    fn dictionary_mode_flags_unknown_words() {
        let mut c = checker_with("[check]\nmode = \"dictionary\"\n");
        c.words.add_wordlist_str("hello\nworld\nreceive\n");
        let issues = c.check("hello zzqqy\n", None);
        assert!(issues.iter().any(|i| i.kind == IssueKind::UnknownWord));
        // Known words and ALL-CAPS acronyms pass.
        assert!(c.check("hello world RECV\n", None).is_empty());
    }

    #[test]
    fn english_locale_flags_only_the_opposite_variant() {
        let us = checker_with("[check]\nlocale = \"en-US\"\n");
        let issues = us.check("color and colour\n", None);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].kind, IssueKind::EnVariant);
        assert_eq!(issues[0].word, "colour");
        assert_eq!(issues[0].suggestions, ["color"]);

        let gb = checker_with("[check]\nlocale = \"en-GB\"\n");
        let issues = gb.check("color and colour\n", None);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].word, "color");
        assert_eq!(issues[0].suggestions, ["colour"]);
        assert!(checker_with("")
            .check("color and colour\n", None)
            .is_empty());
    }

    #[test]
    fn syntax_profiles_keep_offsets_while_reducing_source_noise() {
        let checker = checker_with("[check]\nprofile = \"auto\"\n");
        let source = "let recieve = 1; // recieve comment\n";
        let issues = checker.check(source, Some(Path::new("src/main.rs")));
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].offset, source.rfind("recieve").unwrap());

        let markdown = "Prose recieve. `code recieve`\n";
        let issues = checker.check(markdown, Some(Path::new("guide.md")));
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].offset, markdown.find("recieve").unwrap());
    }

    proptest! {
        #[test]
        fn every_reported_span_is_a_utf8_boundary(text in any::<String>()) {
            let checker = checker_with("");
            for issue in checker.check(&text, None) {
                let end = issue.offset + issue.len;
                prop_assert!(end <= text.len());
                prop_assert!(text.is_char_boundary(issue.offset));
                prop_assert!(text.is_char_boundary(end));
                prop_assert_eq!(&text[issue.offset..end], issue.word.as_str());
            }
        }
    }

    #[test]
    fn japanese_consistency_end_to_end() {
        let c = checker_with("");
        let text = "サーバーを起動する。\nサーバーを停止する。\n古いサーバを削除する。\n";
        let issues = c.check(text, None);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].word, "サーバ");
        assert_eq!(issues[0].suggestions, ["サーバー"]);
        assert_eq!(issues[0].line, 3);
    }

    #[test]
    fn overrides_switch_mode_by_glob() {
        let c = checker_with(
            r#"
            [check]
            mode = "corrections"

            [[overrides]]
            paths = ["docs/**"]
            mode = "off"
            "#,
        );
        let text = "recieve\n";
        assert!(!c.check(text, Some(Path::new("src/main.rs"))).is_empty());
        assert!(c.check(text, Some(Path::new("docs/guide.md"))).is_empty());
    }

    #[test]
    fn prose_extension_detection_for_fullwidth_space() {
        let c = checker_with("");
        let text = "　全角スペースで字下げ。\n";
        assert!(c.check(text, Some(Path::new("novel.md"))).is_empty());
        assert!(!c.check(text, Some(Path::new("main.rs"))).is_empty());
    }
}
