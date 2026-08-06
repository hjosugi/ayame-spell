//! Japanese notation checks: katakana variants (表記ゆれ), fullwidth
//! alphanumerics, halfwidth katakana, and fullwidth spaces.
//!
//! Katakana handling follows the low-noise philosophy of the rest of the
//! engine. The default style, [`KatakanaStyle::Consistency`], flags nothing
//! on its own — it only reports when one document mixes two spellings of the
//! same word (サーバ vs サーバー), suggesting the majority form. Explicit
//! `long`/`short` styles enforce a direction using a curated pair table
//! (JIS Z 8301:2019 abolished the old "omit ー" rule, so neither direction is
//! forced on users).

use std::collections::HashMap;

use regex::Regex;
use unicode_normalization::UnicodeNormalization;

use crate::issue::{Issue, IssueKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum KatakanaStyle {
    /// Flag only intra-document inconsistencies (default).
    #[default]
    Consistency,
    /// Enforce trailing long vowels: サーバ → サーバー.
    Long,
    /// Enforce omitted trailing long vowels: サーバー → サーバ.
    Short,
    Off,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SpacePolicy {
    /// Flag U+3000 except in prose files (default).
    #[default]
    Code,
    Always,
    Never,
}

#[derive(Debug, Clone, Copy)]
pub struct JapaneseOptions {
    pub flag_fullwidth_alnum: bool,
    pub flag_halfwidth_kana: bool,
    pub flag_compatibility: bool,
    pub kanji_consistency: bool,
    pub number_consistency: bool,
    pub punctuation_consistency: bool,
    pub fullwidth_space: SpacePolicy,
}

impl Default for JapaneseOptions {
    fn default() -> Self {
        Self {
            flag_fullwidth_alnum: true,
            flag_halfwidth_kana: true,
            flag_compatibility: true,
            kanji_consistency: true,
            number_consistency: true,
            punctuation_consistency: true,
            fullwidth_space: SpacePolicy::Code,
        }
    }
}

/// Curated pairs of trailing-long-vowel variants: `short<TAB>long`.
const KATAKANA_PAIRS: &str = include_str!("../data/katakana-pairs.tsv");
const KANJI_PAIRS: [(&str, &str); 8] = [
    ("お問合せ", "お問い合わせ"),
    ("取扱い", "取り扱い"),
    ("子供", "子ども"),
    ("引続き", "引き続き"),
    ("行なう", "行う"),
    ("表わす", "表す"),
    ("読替える", "読み替える"),
    ("問い合せ", "問い合わせ"),
];

struct RegexRule {
    regex: Regex,
    replacement: String,
}

/// One katakana word occurrence, collected for the document-level
/// consistency check.
#[derive(Debug)]
pub struct KatakanaOcc {
    pub form: String,
    pub line: u32,
    pub col: usize,
    pub offset: usize,
}

pub struct JapaneseChecker {
    /// variant → preferred form.
    variants: HashMap<String, String>,
    regex_rules: Vec<RegexRule>,
    pub style: KatakanaStyle,
    pub flag_fullwidth_alnum: bool,
    pub flag_halfwidth_kana: bool,
    pub flag_compatibility: bool,
    pub kanji_consistency: bool,
    pub number_consistency: bool,
    pub punctuation_consistency: bool,
    pub fullwidth_space: SpacePolicy,
}

impl JapaneseChecker {
    pub fn new(style: KatakanaStyle, options: JapaneseOptions) -> Self {
        let mut variants = HashMap::new();
        match style {
            KatakanaStyle::Long => {
                for (short, long) in pairs() {
                    variants.insert(short.to_string(), long.to_string());
                }
            }
            KatakanaStyle::Short => {
                for (short, long) in pairs() {
                    variants.insert(long.to_string(), short.to_string());
                }
            }
            KatakanaStyle::Consistency | KatakanaStyle::Off => {}
        }
        Self {
            variants,
            regex_rules: Vec::new(),
            style,
            flag_fullwidth_alnum: options.flag_fullwidth_alnum,
            flag_halfwidth_kana: options.flag_halfwidth_kana,
            flag_compatibility: options.flag_compatibility,
            kanji_consistency: options.kanji_consistency,
            number_consistency: options.number_consistency,
            punctuation_consistency: options.punctuation_consistency,
            fullwidth_space: options.fullwidth_space,
        }
    }

    /// Custom variant rule; applies in every style.
    pub fn add_variant(&mut self, variant: &str, preferred: &str) {
        self.variants
            .insert(variant.to_string(), preferred.to_string());
    }

    /// Load literal `[variants]` and the supported prh-style
    /// `[[rules]] pattern/replace` subset from TOML.
    pub fn load_variant_rules(&mut self, text: &str) -> anyhow::Result<usize> {
        // `toml::Value`'s `FromStr` parses a single value expression, not a
        // document, so `[[rules]]` fails there. `from_str` parses a document.
        let value: toml::Value = toml::from_str(text)?;
        let table = value
            .get("variants")
            .and_then(toml::Value::as_table)
            .cloned()
            .unwrap_or_default();
        let mut n = 0;
        for (variant, preferred) in table {
            if let Some(p) = preferred.as_str() {
                self.add_variant(&variant, p);
                n += 1;
            }
        }
        if let Some(rules) = value.get("rules").and_then(toml::Value::as_array) {
            for rule in rules {
                let table = rule
                    .as_table()
                    .ok_or_else(|| anyhow::anyhow!("[[rules]] must be a table"))?;
                let pattern = table
                    .get("pattern")
                    .and_then(toml::Value::as_str)
                    .ok_or_else(|| anyhow::anyhow!("[[rules]].pattern must be a string"))?;
                let replacement = table
                    .get("replace")
                    .and_then(toml::Value::as_str)
                    .ok_or_else(|| anyhow::anyhow!("[[rules]].replace must be a string"))?;
                self.regex_rules.push(RegexRule {
                    regex: Regex::new(pattern)
                        .map_err(|error| anyhow::anyhow!("invalid rule `{pattern}`: {error}"))?,
                    replacement: replacement.to_string(),
                });
                n += 1;
            }
        }
        Ok(n)
    }

    pub fn wants_consistency(&self) -> bool {
        self.style == KatakanaStyle::Consistency
    }

    pub fn variant_of(&self, form: &str) -> Option<&String> {
        self.variants.get(form)
    }

    /// Check one line. Katakana occurrences are pushed to `occs` (when
    /// provided) for the later document-level consistency pass.
    pub fn check_line(
        &self,
        line: &str,
        line_no: u32,
        line_offset: usize,
        is_prose: bool,
        issues: &mut Vec<Issue>,
        mut occs: Option<&mut Vec<KatakanaOcc>>,
    ) {
        for rule in &self.regex_rules {
            for matched in rule
                .regex
                .find_iter(line)
                .filter(|matched| !matched.is_empty())
            {
                let word = matched.as_str();
                let replacement = rule.regex.replace(word, rule.replacement.as_str());
                if replacement == word {
                    continue;
                }
                issues.push(Issue {
                    line: line_no,
                    col: matched.start(),
                    offset: line_offset + matched.start(),
                    len: matched.len(),
                    word: word.to_string(),
                    kind: IssueKind::JaVariant,
                    suggestions: vec![replacement.into_owned()],
                });
            }
        }
        let mut iter = line.char_indices().peekable();
        while let Some((i, c)) = iter.next() {
            if is_katakana(c) {
                let start = i;
                let mut end = i + c.len_utf8();
                while let Some(&(j, d)) = iter.peek() {
                    if is_katakana(d) {
                        end = j + d.len_utf8();
                        iter.next();
                    } else {
                        break;
                    }
                }
                let form = &line[start..end];
                if let Some(preferred) = self.variants.get(form) {
                    issues.push(Issue {
                        line: line_no,
                        col: start,
                        offset: line_offset + start,
                        len: form.len(),
                        word: form.to_string(),
                        kind: IssueKind::JaVariant,
                        suggestions: vec![preferred.clone()],
                    });
                } else if let Some(occs) = occs.as_deref_mut() {
                    if form.chars().count() >= 3 {
                        occs.push(KatakanaOcc {
                            form: form.to_string(),
                            line: line_no,
                            col: start,
                            offset: line_offset + start,
                        });
                    }
                }
            } else if self.flag_fullwidth_alnum && fullwidth_alnum(c).is_some() {
                let start = i;
                let mut end = i + c.len_utf8();
                let mut converted = String::new();
                converted.push(fullwidth_alnum(c).unwrap());
                while let Some(&(j, d)) = iter.peek() {
                    if let Some(a) = fullwidth_alnum(d) {
                        converted.push(a);
                        end = j + d.len_utf8();
                        iter.next();
                    } else {
                        break;
                    }
                }
                issues.push(Issue {
                    line: line_no,
                    col: start,
                    offset: line_offset + start,
                    len: end - start,
                    word: line[start..end].to_string(),
                    kind: IssueKind::FullwidthAlnum,
                    suggestions: vec![converted],
                });
            } else if self.flag_halfwidth_kana && is_halfwidth_kana(c) {
                let start = i;
                let mut end = i + c.len_utf8();
                while let Some(&(j, d)) = iter.peek() {
                    if is_halfwidth_kana(d) {
                        end = j + d.len_utf8();
                        iter.next();
                    } else {
                        break;
                    }
                }
                let run = &line[start..end];
                issues.push(Issue {
                    line: line_no,
                    col: start,
                    offset: line_offset + start,
                    len: run.len(),
                    word: run.to_string(),
                    kind: IssueKind::HalfwidthKana,
                    suggestions: vec![halfwidth_to_fullwidth(run)],
                });
            } else if c == '\u{3000}' {
                let flag = match self.fullwidth_space {
                    SpacePolicy::Always => true,
                    SpacePolicy::Never => false,
                    SpacePolicy::Code => !is_prose,
                };
                if flag {
                    issues.push(Issue {
                        line: line_no,
                        col: i,
                        offset: line_offset + i,
                        len: c.len_utf8(),
                        word: c.to_string(),
                        kind: IssueKind::FullwidthSpace,
                        suggestions: vec![" ".to_string()],
                    });
                }
            } else if self.flag_compatibility && is_compatibility_character(c) {
                let normalized: String = c.to_string().nfkc().collect();
                if normalized != c.to_string() {
                    issues.push(Issue {
                        line: line_no,
                        col: i,
                        offset: line_offset + i,
                        len: c.len_utf8(),
                        word: c.to_string(),
                        kind: IssueKind::JaCompatibility,
                        suggestions: vec![normalized],
                    });
                }
            }
        }
    }

    pub fn document_issues(&self, text: &str) -> Vec<Issue> {
        let mut issues = Vec::new();
        if self.kanji_consistency {
            issues.extend(kanji_consistency_issues(text));
        }
        if self.number_consistency {
            issues.extend(number_consistency_issues(text));
        }
        if self.punctuation_consistency {
            issues.extend(punctuation_consistency_issues(text));
        }
        issues
    }
}

/// Document-level katakana consistency: when one document spells the same
/// word both with and without a trailing long vowel, flag the minority form
/// and suggest the majority (ties prefer the modern long form).
pub fn consistency_issues(occs: &[KatakanaOcc]) -> Vec<Issue> {
    let mut groups: HashMap<String, HashMap<&str, Vec<&KatakanaOcc>>> = HashMap::new();
    for occ in occs {
        let key = occ.form.trim_end_matches('ー');
        if key.chars().count() < 2 {
            continue;
        }
        groups
            .entry(key.to_string())
            .or_default()
            .entry(occ.form.as_str())
            .or_default()
            .push(occ);
    }
    let mut issues = Vec::new();
    for forms in groups.into_values() {
        if forms.len() < 2 {
            continue;
        }
        let winner = forms
            .iter()
            .max_by_key(|(form, occs)| (occs.len(), form.ends_with('ー'), form.len()))
            .map(|(form, _)| form.to_string())
            .unwrap_or_default();
        for (form, form_occs) in &forms {
            if **form == winner {
                continue;
            }
            for occ in form_occs {
                issues.push(Issue {
                    line: occ.line,
                    col: occ.col,
                    offset: occ.offset,
                    len: occ.form.len(),
                    word: occ.form.clone(),
                    kind: IssueKind::JaVariant,
                    suggestions: vec![winner.clone()],
                });
            }
        }
    }
    issues
}

fn pairs() -> impl Iterator<Item = (&'static str, &'static str)> {
    KATAKANA_PAIRS.lines().filter_map(|line| {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            return None;
        }
        line.split_once('\t')
    })
}

fn is_katakana(c: char) -> bool {
    matches!(c, '\u{30A1}'..='\u{30FA}' | 'ー' | '\u{30FD}' | '\u{30FE}')
}

fn is_halfwidth_kana(c: char) -> bool {
    matches!(c, '\u{FF61}'..='\u{FF9F}')
}

fn is_compatibility_character(character: char) -> bool {
    matches!(
        character,
        '\u{2100}'..='\u{214f}' | '\u{3300}'..='\u{33ff}'
    )
}

fn kanji_consistency_issues(text: &str) -> Vec<Issue> {
    let mut issues = Vec::new();
    for (variant, preferred) in KANJI_PAIRS {
        let variants: Vec<usize> = text
            .match_indices(variant)
            .map(|(offset, _)| offset)
            .collect();
        let preferreds: Vec<usize> = text
            .match_indices(preferred)
            .map(|(offset, _)| offset)
            .collect();
        if variants.is_empty() || preferreds.is_empty() {
            continue;
        }
        let (minority, form, suggestion) = if variants.len() < preferreds.len() {
            (&variants, variant, preferred)
        } else if preferreds.len() < variants.len() {
            (&preferreds, preferred, variant)
        } else {
            (&variants, variant, preferred)
        };
        for offset in minority {
            issues.push(issue_at(
                text,
                *offset,
                form,
                IssueKind::JaVariant,
                suggestion,
            ));
        }
    }
    issues
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum NumberStyle {
    Arabic,
    Kanji,
}

struct NumberOccurrence {
    offset: usize,
    form: String,
    canonical: String,
    style: NumberStyle,
}

fn number_consistency_issues(text: &str) -> Vec<Issue> {
    let occurrences = number_occurrences(text);
    let mut groups: HashMap<&str, Vec<&NumberOccurrence>> = HashMap::new();
    for occurrence in &occurrences {
        groups
            .entry(occurrence.canonical.as_str())
            .or_default()
            .push(occurrence);
    }
    let mut issues = Vec::new();
    for group in groups.into_values() {
        let arabic = group
            .iter()
            .filter(|occurrence| occurrence.style == NumberStyle::Arabic)
            .count();
        let kanji = group.len() - arabic;
        if arabic == 0 || kanji == 0 {
            continue;
        }
        let preferred_style = if arabic >= kanji {
            NumberStyle::Arabic
        } else {
            NumberStyle::Kanji
        };
        let suggestion = group
            .iter()
            .find(|occurrence| occurrence.style == preferred_style)
            .map(|occurrence| occurrence.form.as_str())
            .unwrap_or_default();
        for occurrence in group
            .iter()
            .filter(|occurrence| occurrence.style != preferred_style)
        {
            issues.push(issue_at(
                text,
                occurrence.offset,
                &occurrence.form,
                IssueKind::JaNumberStyle,
                suggestion,
            ));
        }
    }
    issues
}

fn number_occurrences(text: &str) -> Vec<NumberOccurrence> {
    let mut occurrences = Vec::new();
    let mut indices = text.char_indices().peekable();
    while let Some((start, character)) = indices.next() {
        let style = if character.is_ascii_digit() {
            Some(NumberStyle::Arabic)
        } else if kanji_digit(character).is_some() {
            Some(NumberStyle::Kanji)
        } else {
            None
        };
        let Some(style) = style else {
            continue;
        };
        let mut digits = String::new();
        match style {
            NumberStyle::Arabic => digits.push(character),
            NumberStyle::Kanji => {
                digits.push(kanji_digit(character).expect("number style came from a kanji digit"))
            }
        }
        while let Some(&(_, candidate)) = indices.peek() {
            match style {
                NumberStyle::Arabic if candidate.is_ascii_digit() => {
                    digits.push(candidate);
                    indices.next();
                }
                NumberStyle::Arabic if candidate == ',' => {
                    indices.next();
                }
                NumberStyle::Kanji => {
                    let Some(digit) = kanji_digit(candidate) else {
                        break;
                    };
                    digits.push(digit);
                    indices.next();
                }
                NumberStyle::Arabic => break,
            }
        }
        let Some(&(unit_offset, unit)) = indices.peek() else {
            break;
        };
        if !matches!(
            unit,
            '円' | '人' | '件' | '年' | '月' | '日' | '時' | '分' | '秒'
        ) {
            continue;
        }
        indices.next();
        let end = unit_offset + unit.len_utf8();
        let form = text[start..end].to_string();
        occurrences.push(NumberOccurrence {
            offset: start,
            form,
            canonical: format!("{digits}{unit}"),
            style,
        });
    }
    occurrences
}

fn kanji_digit(character: char) -> Option<char> {
    Some(match character {
        '〇' | '零' => '0',
        '一' => '1',
        '二' => '2',
        '三' => '3',
        '四' => '4',
        '五' => '5',
        '六' => '6',
        '七' => '7',
        '八' => '8',
        '九' => '9',
        _ => return None,
    })
}

fn punctuation_consistency_issues(text: &str) -> Vec<Issue> {
    let japanese = text
        .chars()
        .filter(|character| matches!(character, '、' | '。'))
        .count();
    let fullwidth = text
        .chars()
        .filter(|character| matches!(character, '，' | '．'))
        .count();
    if japanese == 0 || fullwidth == 0 {
        return Vec::new();
    }
    let prefer_japanese = japanese >= fullwidth;
    text.char_indices()
        .filter_map(|(offset, character)| {
            let suggestion = match (prefer_japanese, character) {
                (true, '，') => "、",
                (true, '．') => "。",
                (false, '、') => "，",
                (false, '。') => "．",
                _ => return None,
            };
            Some(issue_at(
                text,
                offset,
                &character.to_string(),
                IssueKind::JaPunctuation,
                suggestion,
            ))
        })
        .collect()
}

fn issue_at(text: &str, offset: usize, word: &str, kind: IssueKind, suggestion: &str) -> Issue {
    let before = &text[..offset];
    let line = before.bytes().filter(|byte| *byte == b'\n').count() as u32 + 1;
    let line_start = before.rfind('\n').map_or(0, |position| position + 1);
    Issue {
        line,
        col: offset - line_start,
        offset,
        len: word.len(),
        word: word.to_string(),
        kind,
        suggestions: vec![suggestion.to_string()],
    }
}

fn fullwidth_alnum(c: char) -> Option<char> {
    match c {
        '\u{FF10}'..='\u{FF19}' | '\u{FF21}'..='\u{FF3A}' | '\u{FF41}'..='\u{FF5A}' => {
            char::from_u32(c as u32 - 0xFEE0)
        }
        _ => None,
    }
}

/// Convert a halfwidth-katakana run to fullwidth, merging ﾞ/ﾟ marks
/// (ｶﾞ → ガ).
pub fn halfwidth_to_fullwidth(run: &str) -> String {
    let mut out = String::with_capacity(run.len());
    let mut chars = run.chars().peekable();
    while let Some(c) = chars.next() {
        if let Some(&next) = chars.peek() {
            if next == '\u{FF9E}' {
                if let Some(v) = voiced(c) {
                    out.push(v);
                    chars.next();
                    continue;
                }
            } else if next == '\u{FF9F}' {
                if let Some(v) = semi_voiced(c) {
                    out.push(v);
                    chars.next();
                    continue;
                }
            }
        }
        out.push(base_kana(c));
    }
    out
}

fn base_kana(c: char) -> char {
    match c {
        '｡' => '。',
        '｢' => '「',
        '｣' => '」',
        '､' => '、',
        '･' => '・',
        'ｦ' => 'ヲ',
        'ｧ' => 'ァ',
        'ｨ' => 'ィ',
        'ｩ' => 'ゥ',
        'ｪ' => 'ェ',
        'ｫ' => 'ォ',
        'ｬ' => 'ャ',
        'ｭ' => 'ュ',
        'ｮ' => 'ョ',
        'ｯ' => 'ッ',
        'ｰ' => 'ー',
        'ｱ' => 'ア',
        'ｲ' => 'イ',
        'ｳ' => 'ウ',
        'ｴ' => 'エ',
        'ｵ' => 'オ',
        'ｶ' => 'カ',
        'ｷ' => 'キ',
        'ｸ' => 'ク',
        'ｹ' => 'ケ',
        'ｺ' => 'コ',
        'ｻ' => 'サ',
        'ｼ' => 'シ',
        'ｽ' => 'ス',
        'ｾ' => 'セ',
        'ｿ' => 'ソ',
        'ﾀ' => 'タ',
        'ﾁ' => 'チ',
        'ﾂ' => 'ツ',
        'ﾃ' => 'テ',
        'ﾄ' => 'ト',
        'ﾅ' => 'ナ',
        'ﾆ' => 'ニ',
        'ﾇ' => 'ヌ',
        'ﾈ' => 'ネ',
        'ﾉ' => 'ノ',
        'ﾊ' => 'ハ',
        'ﾋ' => 'ヒ',
        'ﾌ' => 'フ',
        'ﾍ' => 'ヘ',
        'ﾎ' => 'ホ',
        'ﾏ' => 'マ',
        'ﾐ' => 'ミ',
        'ﾑ' => 'ム',
        'ﾒ' => 'メ',
        'ﾓ' => 'モ',
        'ﾔ' => 'ヤ',
        'ﾕ' => 'ユ',
        'ﾖ' => 'ヨ',
        'ﾗ' => 'ラ',
        'ﾘ' => 'リ',
        'ﾙ' => 'ル',
        'ﾚ' => 'レ',
        'ﾛ' => 'ロ',
        'ﾜ' => 'ワ',
        'ﾝ' => 'ン',
        '\u{FF9E}' => '゛',
        '\u{FF9F}' => '゜',
        other => other,
    }
}

fn voiced(c: char) -> Option<char> {
    Some(match c {
        'ｳ' => 'ヴ',
        'ｶ' => 'ガ',
        'ｷ' => 'ギ',
        'ｸ' => 'グ',
        'ｹ' => 'ゲ',
        'ｺ' => 'ゴ',
        'ｻ' => 'ザ',
        'ｼ' => 'ジ',
        'ｽ' => 'ズ',
        'ｾ' => 'ゼ',
        'ｿ' => 'ゾ',
        'ﾀ' => 'ダ',
        'ﾁ' => 'ヂ',
        'ﾂ' => 'ヅ',
        'ﾃ' => 'デ',
        'ﾄ' => 'ド',
        'ﾊ' => 'バ',
        'ﾋ' => 'ビ',
        'ﾌ' => 'ブ',
        'ﾍ' => 'ベ',
        'ﾎ' => 'ボ',
        _ => return None,
    })
}

fn semi_voiced(c: char) -> Option<char> {
    Some(match c {
        'ﾊ' => 'パ',
        'ﾋ' => 'ピ',
        'ﾌ' => 'プ',
        'ﾍ' => 'ペ',
        'ﾎ' => 'ポ',
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn checker(style: KatakanaStyle) -> JapaneseChecker {
        JapaneseChecker::new(style, JapaneseOptions::default())
    }

    fn check(c: &JapaneseChecker, line: &str) -> Vec<Issue> {
        let mut issues = Vec::new();
        c.check_line(line, 1, 0, false, &mut issues, None);
        issues
    }

    #[test]
    fn long_style_flags_short_forms() {
        let c = checker(KatakanaStyle::Long);
        let issues = check(&c, "サーバの設定");
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].word, "サーバ");
        assert_eq!(issues[0].suggestions, ["サーバー"]);
    }

    #[test]
    fn short_style_flags_long_forms() {
        let c = checker(KatakanaStyle::Short);
        let issues = check(&c, "サーバーの設定");
        assert_eq!(issues[0].suggestions, ["サーバ"]);
    }

    #[test]
    fn consistency_style_flags_nothing_per_line() {
        let c = checker(KatakanaStyle::Consistency);
        assert!(check(&c, "サーバの設定とサーバーの設定").is_empty());
    }

    #[test]
    fn consistency_pass_flags_minority() {
        let c = checker(KatakanaStyle::Consistency);
        let mut issues = Vec::new();
        let mut occs = Vec::new();
        for (i, line) in ["サーバーを起動", "サーバーを停止", "サーバを再起動"]
            .iter()
            .enumerate()
        {
            c.check_line(line, i as u32 + 1, 0, false, &mut issues, Some(&mut occs));
        }
        assert!(issues.is_empty());
        let extra = consistency_issues(&occs);
        assert_eq!(extra.len(), 1);
        assert_eq!(extra[0].word, "サーバ");
        assert_eq!(extra[0].suggestions, ["サーバー"]);
        assert_eq!(extra[0].line, 3);
    }

    #[test]
    fn fullwidth_alnum_converted() {
        let c = checker(KatakanaStyle::Off);
        let issues = check(&c, "バージョン１２３ＡＢＣです");
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].word, "１２３ＡＢＣ");
        assert_eq!(issues[0].suggestions, ["123ABC"]);
        assert_eq!(issues[0].kind, IssueKind::FullwidthAlnum);
    }

    #[test]
    fn halfwidth_kana_converted() {
        let c = checker(KatakanaStyle::Off);
        let issues = check(&c, "ﾃﾞｰﾀﾍﾞｰｽ接続");
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].suggestions, ["データベース"]);
    }

    #[test]
    fn fullwidth_space_policy() {
        let c = checker(KatakanaStyle::Off);
        let mut issues = Vec::new();
        c.check_line("let　x = 1;", 1, 0, false, &mut issues, None);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].kind, IssueKind::FullwidthSpace);
        // Prose files are exempt under the default policy.
        let mut prose_issues = Vec::new();
        c.check_line("　字下げされた段落。", 1, 0, true, &mut prose_issues, None);
        assert!(prose_issues.is_empty());
    }

    #[test]
    fn custom_variant_rules() {
        let mut c = checker(KatakanaStyle::Consistency);
        c.add_variant("インタフェース", "インターフェース");
        let issues = check(&c, "インタフェース仕様");
        assert_eq!(issues[0].suggestions, ["インターフェース"]);
    }

    #[test]
    fn compatibility_units_are_normalized() {
        let c = checker(KatakanaStyle::Off);
        let issues = check(&c, "重量は5㎏、面積は2㎡。");
        assert!(issues
            .iter()
            .any(|issue| issue.word == "㎏" && issue.suggestions == ["kg"]));
        assert!(issues
            .iter()
            .any(|issue| issue.word == "㎡" && issue.suggestions == ["m2"]));
    }

    #[test]
    fn deeper_document_consistency_is_low_noise() {
        let c = checker(KatakanaStyle::Off);
        let text = "子どもは1,000円を使う。子どもを説明、する。\n\
                    子供は一〇〇〇円を使う．\n";
        let issues = c.document_issues(text);
        assert!(issues
            .iter()
            .any(|issue| issue.word == "子供" && issue.kind == IssueKind::JaVariant));
        assert!(issues.iter().any(|issue| {
            issue.word == "一〇〇〇円" && issue.kind == IssueKind::JaNumberStyle
        }));
        assert!(issues
            .iter()
            .any(|issue| issue.word == "．" && issue.kind == IssueKind::JaPunctuation));
        assert!(c.document_issues("子供だけ。1,000円だけ。").is_empty());
    }

    #[test]
    fn number_occurrences_stream_ascii_and_kanji_forms() {
        let text = "費用は1,000円、予備は42件。比較値は一〇〇〇円。";
        let occurrences = number_occurrences(text);
        let forms: Vec<(&str, &str)> = occurrences
            .iter()
            .map(|occurrence| (occurrence.form.as_str(), occurrence.canonical.as_str()))
            .collect();
        assert_eq!(
            forms,
            [
                ("1,000円", "1000円"),
                ("42件", "42件"),
                ("一〇〇〇円", "1000円"),
            ]
        );
        for occurrence in occurrences {
            assert_eq!(
                &text[occurrence.offset..occurrence.offset + occurrence.form.len()],
                occurrence.form
            );
        }
    }

    #[test]
    fn regex_variant_rules_support_a_prh_style_subset() {
        let mut c = checker(KatakanaStyle::Off);
        let loaded = c
            .load_variant_rules(
                r#"
            [[rules]]
            pattern = "Web ?サイト"
            replace = "ウェブサイト"
            "#,
            )
            .unwrap();
        assert_eq!(loaded, 1, "the [[rules]] array must be read");
        let issues = check(&c, "Web サイトを開く");
        assert_eq!(issues[0].suggestions, ["ウェブサイト"]);
    }

    /// A rule file written by `ayame-spell import prh` must load back. This
    /// caught a document-vs-value TOML parsing regression that made every
    /// imported prh rule silently unusable.
    #[test]
    fn variant_rule_files_round_trip_with_a_leading_array_of_tables() {
        let mut c = checker(KatakanaStyle::Off);
        let loaded = c
            .load_variant_rules("[[rules]]\npattern = \"サーバ\"\nreplace = \"サーバー\"\n")
            .unwrap();
        assert_eq!(loaded, 1);
        assert_eq!(check(&c, "サーバを再起動")[0].suggestions, ["サーバー"]);
    }
}
