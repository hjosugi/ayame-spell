//! Known-misspelling correction tables (typos-style checking).
//!
//! The built-in English table comes from the `typos-dict` crate
//! (MIT OR Apache-2.0, ~95k entries). Users can extend it with inline
//! config entries and TSV files, and can whitelist words by mapping them
//! to themselves.

use std::collections::HashMap;
use std::path::Path;

use crate::tokenizer::match_case;

/// Result of looking a word up in the correction tables.
#[derive(Debug, PartialEq, Eq)]
pub enum Verdict {
    /// The word is explicitly whitelisted (entry maps to itself).
    Allowed,
    /// The word is a known misspelling; case-adjusted fixes, best first.
    Typo(Vec<String>),
}

enum Entry {
    Allow,
    Fix(Vec<String>),
}

/// Words the upstream corrections data flags but that are everyday
/// identifiers in real code (serde's `ser`, the flate2 crate, the
/// misspelled-by-standard HTTP `Referer` header).
const BUILTIN_ALLOW: [&str; 3] = ["ser", "flate", "referer"];

#[derive(Default)]
pub struct Corrections {
    custom: HashMap<String, Entry>,
    builtin: bool,
}

impl Corrections {
    pub fn new(builtin: bool) -> Self {
        Self {
            custom: HashMap::new(),
            builtin,
        }
    }

    /// Register a custom correction. A fix equal to the typo itself (case
    /// insensitively) whitelists the word instead.
    pub fn insert(&mut self, typo: &str, fixes: &[String]) {
        let key = typo.to_ascii_lowercase();
        let entry = if fixes.len() == 1 && fixes[0].eq_ignore_ascii_case(typo) {
            Entry::Allow
        } else {
            Entry::Fix(fixes.to_vec())
        };
        self.custom.insert(key, entry);
    }

    /// Load a TSV file: `typo<TAB>fix[,fix...]`, `#` comments allowed.
    pub fn load_tsv(&mut self, path: &Path) -> anyhow::Result<usize> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("cannot read corrections file {}: {e}", path.display()))?;
        let mut n = 0;
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((typo, rest)) = line.split_once('\t') else {
                continue;
            };
            let fixes: Vec<String> = rest
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            if !fixes.is_empty() {
                self.insert(typo.trim(), &fixes);
                n += 1;
            }
        }
        Ok(n)
    }

    /// Look up a word. `None` means the tables have no opinion.
    pub fn check(&self, word: &str) -> Option<Verdict> {
        let lower = word.to_ascii_lowercase();
        if let Some(entry) = self.custom.get(&lower) {
            return Some(match entry {
                Entry::Allow => Verdict::Allowed,
                Entry::Fix(fixes) => {
                    Verdict::Typo(fixes.iter().map(|f| match_case(word, f)).collect())
                }
            });
        }
        #[cfg(feature = "builtin-en")]
        if self.builtin {
            if BUILTIN_ALLOW.contains(&lower.as_str()) {
                return Some(Verdict::Allowed);
            }
            if let Some(fixes) = typos_dict::WORD.find(&unicase::UniCase::new(word)) {
                return Some(Verdict::Typo(
                    fixes.iter().map(|f| match_case(word, f)).collect(),
                ));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn custom_beats_builtin() {
        let mut c = Corrections::new(true);
        c.insert("teh", &["tea".to_string()]);
        assert_eq!(c.check("teh"), Some(Verdict::Typo(vec!["tea".into()])));
    }

    #[test]
    fn identity_whitelists() {
        let mut c = Corrections::new(true);
        c.insert("recieve", &["recieve".to_string()]);
        assert_eq!(c.check("recieve"), Some(Verdict::Allowed));
        assert_eq!(c.check("RECIEVE"), Some(Verdict::Allowed));
    }

    #[cfg(feature = "builtin-en")]
    #[test]
    fn builtin_catches_known_typos() {
        let c = Corrections::new(true);
        match c.check("recieve") {
            Some(Verdict::Typo(fixes)) => assert!(fixes.contains(&"receive".to_string())),
            other => panic!("expected typo verdict, got {other:?}"),
        }
        // Case is preserved in the fix.
        match c.check("Recieve") {
            Some(Verdict::Typo(fixes)) => assert!(fixes.contains(&"Receive".to_string())),
            other => panic!("expected typo verdict, got {other:?}"),
        }
        assert_eq!(c.check("receive"), None);
    }

    #[cfg(feature = "builtin-en")]
    #[test]
    fn builtin_can_be_disabled() {
        let c = Corrections::new(false);
        assert_eq!(c.check("recieve"), None);
    }
}
