use serde::Serialize;

/// A single finding in a checked text.
#[derive(Debug, Clone, Serialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum IssueKind {
    /// A known misspelling from a corrections table.
    Typo,
    /// A word not present in any active dictionary (dictionary mode only).
    UnknownWord,
    /// A Japanese katakana notation variant (表記ゆれ).
    JaVariant,
    /// Fullwidth alphanumerics (１２３ＡＢＣ).
    FullwidthAlnum,
    /// Halfwidth katakana (ｶﾀｶﾅ).
    HalfwidthKana,
    /// Fullwidth space (U+3000).
    FullwidthSpace,
}

impl IssueKind {
    /// Stable machine-readable code, used as the LSP diagnostic code and in
    /// JSON output.
    pub fn code(self) -> &'static str {
        match self {
            IssueKind::Typo => "typo",
            IssueKind::UnknownWord => "unknown-word",
            IssueKind::JaVariant => "ja-variant",
            IssueKind::FullwidthAlnum => "fullwidth-alnum",
            IssueKind::HalfwidthKana => "halfwidth-kana",
            IssueKind::FullwidthSpace => "fullwidth-space",
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
