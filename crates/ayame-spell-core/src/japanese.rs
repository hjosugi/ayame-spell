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

/// Curated pairs of trailing-long-vowel variants: `short<TAB>long`.
const KATAKANA_PAIRS: &str = include_str!("../data/katakana-pairs.tsv");

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
    pub style: KatakanaStyle,
    pub flag_fullwidth_alnum: bool,
    pub flag_halfwidth_kana: bool,
    pub fullwidth_space: SpacePolicy,
}

impl JapaneseChecker {
    pub fn new(
        style: KatakanaStyle,
        flag_fullwidth_alnum: bool,
        flag_halfwidth_kana: bool,
        fullwidth_space: SpacePolicy,
    ) -> Self {
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
            style,
            flag_fullwidth_alnum,
            flag_halfwidth_kana,
            fullwidth_space,
        }
    }

    /// Custom variant rule; applies in every style.
    pub fn add_variant(&mut self, variant: &str, preferred: &str) {
        self.variants
            .insert(variant.to_string(), preferred.to_string());
    }

    /// Load variant rules from TOML text: a `[variants]` table of
    /// `"変種" = "正規形"` entries (also accepted at the top level).
    pub fn load_variant_rules(&mut self, text: &str) -> anyhow::Result<usize> {
        let value: toml::Value = text.parse()?;
        let table = value
            .get("variants")
            .and_then(toml::Value::as_table)
            .or_else(|| value.as_table())
            .cloned()
            .unwrap_or_default();
        let mut n = 0;
        for (variant, preferred) in table {
            if let Some(p) = preferred.as_str() {
                self.add_variant(&variant, p);
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
        if line.is_ascii() {
            return;
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
            }
        }
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
        JapaneseChecker::new(style, true, true, SpacePolicy::Code)
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
}
