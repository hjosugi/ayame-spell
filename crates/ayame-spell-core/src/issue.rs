use serde::{Deserialize, Serialize};

/// A single finding in a checked text.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Issue {
    /// 1-based line number.
    pub line: u32,
    /// Byte offset of the flagged text within its line.
    pub col: usize,
    /// Absolute byte offset of the flagged text in the checked text.
    pub offset: usize,
    /// Byte length of the flagged text.
    pub len: usize,
    /// The flagged text itself.
    pub word: String,
    pub kind: IssueKind,
    /// Replacement candidates, best first.
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum IssueKind {
    /// A known misspelling from a corrections table.
    Typo,
    /// A word not present in any active dictionary (dictionary mode only).
    UnknownWord,
    /// A spelling conflicts with the configured English locale.
    EnVariant,
    /// A Japanese katakana notation variant (表記ゆれ).
    JaVariant,
    /// Fullwidth alphanumerics (１２３ＡＢＣ).
    FullwidthAlnum,
    /// Halfwidth katakana (ｶﾀｶﾅ).
    HalfwidthKana,
    /// Fullwidth space (U+3000).
    FullwidthSpace,
    /// A Japanese compatibility character has a standard NFKC form.
    JaCompatibility,
    /// A number/unit spelling is inconsistent within the document.
    JaNumberStyle,
    /// Japanese punctuation style is inconsistent within the document.
    JaPunctuation,
}

/// Localized, stable metadata for one issue code.
#[derive(Debug, Clone, Copy)]
pub struct RuleInfo {
    pub title: &'static str,
    pub summary: &'static str,
    pub explanation: &'static str,
    pub config_key: &'static str,
    pub example: &'static str,
    pub silence: &'static str,
}

impl IssueKind {
    /// Every issue kind emitted by the checker.
    pub const ALL: [Self; 10] = [
        Self::Typo,
        Self::UnknownWord,
        Self::EnVariant,
        Self::JaVariant,
        Self::FullwidthAlnum,
        Self::HalfwidthKana,
        Self::FullwidthSpace,
        Self::JaCompatibility,
        Self::JaNumberStyle,
        Self::JaPunctuation,
    ];

    /// Stable machine-readable code, used as the LSP diagnostic code and in
    /// JSON output.
    pub fn code(self) -> &'static str {
        match self {
            IssueKind::Typo => "typo",
            IssueKind::UnknownWord => "unknown-word",
            IssueKind::EnVariant => "en-variant",
            IssueKind::JaVariant => "ja-variant",
            IssueKind::FullwidthAlnum => "fullwidth-alnum",
            IssueKind::HalfwidthKana => "halfwidth-kana",
            IssueKind::FullwidthSpace => "fullwidth-space",
            IssueKind::JaCompatibility => "ja-compatibility",
            IssueKind::JaNumberStyle => "ja-number-style",
            IssueKind::JaPunctuation => "ja-punctuation",
        }
    }

    /// Resolve a stable issue code.
    pub fn from_code(code: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.code() == code)
    }

    /// Rule metadata in English or Japanese.
    pub fn info(self, japanese: bool) -> RuleInfo {
        if japanese {
            self.info_ja()
        } else {
            self.info_en()
        }
    }

    fn info_en(self) -> RuleInfo {
        match self {
            Self::Typo => RuleInfo {
                title: "Known misspelling",
                summary: "A token matches a built-in, configured, or inline correction.",
                explanation: "The correction table recognized the token as a known misspelling. One candidate is safe to apply automatically; several candidates require a choice.",
                config_key: "[corrections].builtin, [corrections].extra, [corrections.words]",
                example: "teh → the",
                silence: "Add the word to [words].ignore, add an identity correction, or use an inline ayame-spell ignore directive.",
            },
            Self::UnknownWord => RuleInfo {
                title: "Unknown dictionary word",
                summary: "Dictionary mode found a word in no active word list.",
                explanation: "This rule only runs in dictionary mode. Suggestions are edit-distance matches and are never applied without review.",
                config_key: "[check].mode, [words].project, [words].dictionaries, [words].ignore",
                example: "recieve → receive",
                silence: "Add the intended word with `words add`, include a dictionary, or add it to [words].ignore.",
            },
            Self::EnVariant => RuleInfo {
                title: "English locale variant",
                summary: "A spelling conflicts with the configured en-US or en-GB policy.",
                explanation: "The token is valid English, but its regional spelling differs from [check].locale. The default any policy accepts both forms.",
                config_key: "[check].locale",
                example: "colour → color (en-US)",
                silence: "Set locale to any, choose the matching regional policy, add the word to [words].ignore, or use an inline ignore directive.",
            },
            Self::JaVariant => RuleInfo {
                title: "Japanese notation variant",
                summary: "Katakana spelling is inconsistent with the selected or majority style.",
                explanation: "The checker found both forms of a known pair, a form that conflicts with the configured long-vowel style, or a configured custom variant.",
                config_key: "[japanese].katakana-style, [japanese.variants], [japanese].variant-files",
                example: "サーバ → サーバー",
                silence: "Use consistency mode, choose long or short style, remove the custom variant, or use an inline ignore directive.",
            },
            Self::FullwidthAlnum => RuleInfo {
                title: "Fullwidth alphanumeric",
                summary: "Fullwidth ASCII letters or digits can be converted mechanically.",
                explanation: "Characters in the fullwidth ASCII letter and digit ranges were found. The suggested halfwidth form is a safe fix.",
                config_key: "[japanese].enabled",
                example: "ＡＢＣ１２３ → ABC123",
                silence: "Disable Japanese checks for the applicable file or use an inline ignore directive.",
            },
            Self::HalfwidthKana => RuleInfo {
                title: "Halfwidth katakana",
                summary: "Halfwidth katakana can be normalized to fullwidth katakana.",
                explanation: "A halfwidth katakana run was found. Dakuten and handakuten are combined in the suggested fullwidth form.",
                config_key: "[japanese].enabled",
                example: "ｶﾀｶﾅ → カタカナ",
                silence: "Disable Japanese checks for the applicable file or use an inline ignore directive.",
            },
            Self::FullwidthSpace => RuleInfo {
                title: "Fullwidth space",
                summary: "A U+3000 space appears where the configured policy forbids it.",
                explanation: "The fullwidth-space policy applies to prose by default and suggests a normal ASCII space as a safe mechanical fix.",
                config_key: "[japanese].fullwidth-space",
                example: "前　後 → 前 後",
                silence: "Set the fullwidth-space policy to off or restrict it with a file override.",
            },
            Self::JaCompatibility => RuleInfo {
                title: "Japanese compatibility character",
                summary: "A compatibility unit or symbol has a standard NFKC form.",
                explanation: "Compatibility characters such as ㎏ and ㎡ can be represented with ordinary letters and symbols, improving search and interoperability.",
                config_key: "[japanese].flag-compatibility",
                example: "㎏ → kg",
                silence: "Disable flag-compatibility or use an inline ignore directive.",
            },
            Self::JaNumberStyle => RuleInfo {
                title: "Japanese number-style inconsistency",
                summary: "The same number and unit use both Arabic and kanji digits.",
                explanation: "The checker compares equivalent number/unit forms within one document and reports only the minority style.",
                config_key: "[japanese].number-consistency",
                example: "1,000円 / 一〇〇〇円 → 1,000円",
                silence: "Disable number-consistency or make the document use one style.",
            },
            Self::JaPunctuation => RuleInfo {
                title: "Japanese punctuation inconsistency",
                summary: "The document mixes Japanese and fullwidth-comma/full-stop styles.",
                explanation: "The checker reports the minority forms when 、/。 and ，/． are mixed in one document.",
                config_key: "[japanese].punctuation-consistency",
                example: "説明，続き。 → 説明、続き。",
                silence: "Disable punctuation-consistency or make the document use one punctuation style.",
            },
        }
    }

    fn info_ja(self) -> RuleInfo {
        match self {
            Self::Typo => RuleInfo {
                title: "既知のスペルミス",
                summary: "組み込み・設定・インラインの修正表にトークンが一致しました。",
                explanation: "修正表が既知のスペルミスとして検出しました。候補が一つなら安全に自動修正でき、複数なら選択が必要です。",
                config_key: "[corrections].builtin, [corrections].extra, [corrections.words]",
                example: "teh → the",
                silence: "[words].ignore、同一語への修正、またはインラインの ayame-spell ignore 指示へ追加します。",
            },
            Self::UnknownWord => RuleInfo {
                title: "辞書にない単語",
                summary: "辞書モードで、有効な単語リストのどれにもない語を検出しました。",
                explanation: "辞書モードだけで動作します。候補は編集距離から求め、確認なしには適用しません。",
                config_key: "[check].mode, [words].project, [words].dictionaries, [words].ignore",
                example: "recieve → receive",
                silence: "`words add` で正しい語を追加するか、辞書または [words].ignore へ追加します。",
            },
            Self::EnVariant => RuleInfo {
                title: "英語の地域別スペル",
                summary: "設定した en-US / en-GB 方針と異なるスペルです。",
                explanation: "有効な英単語ですが、[check].locale で選んだ地域表記と異なります。既定の any は両方を許可します。",
                config_key: "[check].locale",
                example: "colour → color (en-US)",
                silence: "locale を any または対象地域へ変更するか、[words].ignore かインライン指示で無視します。",
            },
            Self::JaVariant => RuleInfo {
                title: "日本語の表記ゆれ",
                summary: "カタカナ表記が指定スタイルまたは文書内の多数側と一致しません。",
                explanation: "既知ペアの両表記、設定した長音スタイルと異なる形、または独自の表記ゆれを検出しました。",
                config_key: "[japanese].katakana-style, [japanese.variants], [japanese].variant-files",
                example: "サーバ → サーバー",
                silence: "consistency・long・short の方針を選ぶか、独自ルールを外すか、インラインで無視します。",
            },
            Self::FullwidthAlnum => RuleInfo {
                title: "全角英数字",
                summary: "全角 ASCII 英字・数字を機械的に半角へ変換できます。",
                explanation: "全角 ASCII 英字・数字の範囲にある文字を検出しました。半角への変換は安全な修正です。",
                config_key: "[japanese].enabled",
                example: "ＡＢＣ１２３ → ABC123",
                silence: "対象ファイルの日本語チェックを無効にするか、インラインで無視します。",
            },
            Self::HalfwidthKana => RuleInfo {
                title: "半角カタカナ",
                summary: "半角カタカナを全角カタカナへ正規化できます。",
                explanation: "半角カタカナの連続を検出しました。候補では濁点・半濁点も全角の形へ合成します。",
                config_key: "[japanese].enabled",
                example: "ｶﾀｶﾅ → カタカナ",
                silence: "対象ファイルの日本語チェックを無効にするか、インラインで無視します。",
            },
            Self::FullwidthSpace => RuleInfo {
                title: "全角スペース",
                summary: "設定した方針の対象位置に U+3000 の空白があります。",
                explanation: "既定では文章ファイルに全角スペース方針を適用し、安全な半角スペースを候補にします。",
                config_key: "[japanese].fullwidth-space",
                example: "前　後 → 前 後",
                silence: "fullwidth-space 方針を off にするか、ファイル上書き設定で対象を限定します。",
            },
            Self::JaCompatibility => RuleInfo {
                title: "日本語の互換文字",
                summary: "互換単位・記号を標準的な NFKC 形へ変換できます。",
                explanation: "㎏ や ㎡ のような互換文字を通常の文字と記号で表すと、検索や相互運用が安定します。",
                config_key: "[japanese].flag-compatibility",
                example: "㎏ → kg",
                silence: "flag-compatibility を無効にするか、インラインで無視します。",
            },
            Self::JaNumberStyle => RuleInfo {
                title: "数字表記の不統一",
                summary: "同じ数値と単位に算用数字と漢数字が混在しています。",
                explanation: "一文書内で同値の数値・単位表記を比較し、少数側だけを報告します。",
                config_key: "[japanese].number-consistency",
                example: "1,000円 / 一〇〇〇円 → 1,000円",
                silence: "number-consistency を無効にするか、文書内の表記を統一します。",
            },
            Self::JaPunctuation => RuleInfo {
                title: "句読点の不統一",
                summary: "和文句読点と全角カンマ・ピリオドが混在しています。",
                explanation: "一文書内で 、/。 と ，/． が混在するとき、少数側だけを報告します。",
                config_key: "[japanese].punctuation-consistency",
                example: "説明，続き。 → 説明、続き。",
                silence: "punctuation-consistency を無効にするか、文書内の句読点を統一します。",
            },
        }
    }
}

impl Issue {
    /// The replacement that is safe to apply non-interactively, if any.
    ///
    /// Typos with exactly one candidate and mechanical notation conversions
    /// are safe; unknown words never are, and typos with several candidates
    /// need a human choice.
    pub fn safe_fix(&self) -> Option<&str> {
        match self.kind {
            IssueKind::UnknownWord => None,
            IssueKind::Typo => (self.suggestions.len() == 1).then(|| self.suggestions[0].as_str()),
            _ => self.suggestions.first().map(String::as_str),
        }
    }

    /// Human-readable one-line description.
    pub fn message(&self) -> String {
        match self.kind {
            IssueKind::Typo => match self.suggestions.as_slice() {
                [one] => format!("`{}` should be `{}`", self.word, one),
                many => format!("`{}` should be one of: {}", self.word, join(many)),
            },
            IssueKind::UnknownWord => format!("`{}` is not a known word", self.word),
            IssueKind::EnVariant => format!(
                "regional spelling `{}` → `{}`",
                self.word,
                self.suggestions.first().map(String::as_str).unwrap_or("?")
            ),
            IssueKind::JaVariant => format!(
                "`{}` は `{}` に統一 (表記ゆれ)",
                self.word,
                self.suggestions.first().map(String::as_str).unwrap_or("?")
            ),
            IssueKind::FullwidthAlnum => format!(
                "fullwidth alphanumerics `{}` → `{}`",
                self.word,
                self.suggestions.first().map(String::as_str).unwrap_or("?")
            ),
            IssueKind::HalfwidthKana => format!(
                "halfwidth katakana `{}` → `{}`",
                self.word,
                self.suggestions.first().map(String::as_str).unwrap_or("?")
            ),
            IssueKind::FullwidthSpace => "fullwidth space (U+3000)".to_string(),
            IssueKind::JaCompatibility => format!(
                "compatibility character `{}` → `{}`",
                self.word,
                self.suggestions.first().map(String::as_str).unwrap_or("?")
            ),
            IssueKind::JaNumberStyle => format!(
                "`{}` は `{}` に統一 (数字表記)",
                self.word,
                self.suggestions.first().map(String::as_str).unwrap_or("?")
            ),
            IssueKind::JaPunctuation => format!(
                "`{}` は `{}` に統一 (句読点)",
                self.word,
                self.suggestions.first().map(String::as_str).unwrap_or("?")
            ),
        }
    }
}

fn join(suggestions: &[String]) -> String {
    suggestions
        .iter()
        .map(|s| format!("`{s}`"))
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::IssueKind;
    use std::collections::HashSet;

    #[test]
    fn every_issue_code_has_unique_bilingual_rule_metadata() {
        let mut codes = HashSet::new();
        for kind in IssueKind::ALL {
            assert!(codes.insert(kind.code()));
            assert_eq!(IssueKind::from_code(kind.code()), Some(kind));
            for japanese in [false, true] {
                let info = kind.info(japanese);
                assert!(!info.title.is_empty());
                assert!(!info.summary.is_empty());
                assert!(!info.explanation.is_empty());
                assert!(!info.config_key.is_empty());
                assert!(!info.example.is_empty());
                assert!(!info.silence.is_empty());
            }
        }
    }
}
