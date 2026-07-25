//! Wordlist dictionaries for unknown-word ("dictionary mode") checking.
//!
//! Large wordlists are held in FSTs (fast lookup, tiny memory, and free
//! Levenshtein-automaton suggestions); small mutable lists (project words,
//! global words) live in a hash set so they can be updated without an FST
//! rebuild.

use std::collections::{BTreeSet, HashSet};
use std::path::Path;

use fst::automaton::Levenshtein;
use fst::{IntoStreamer, Streamer};

/// Programming vocabulary that would otherwise drown users in false
/// positives on their first run (only consulted in dictionary mode).
const CODE_TERMS: &str = include_str!("../data/code-terms.txt");

#[derive(Default)]
pub struct WordSets {
    sets: Vec<fst::Set<Vec<u8>>>,
    exact: HashSet<String>,
}

impl WordSets {
    /// Build with the embedded programming-terms list.
    pub fn with_code_terms() -> Self {
        let mut w = Self::default();
        w.add_wordlist_str(CODE_TERMS);
        w
    }

    /// Add a wordlist from text: one word per line, `#` comments.
    pub fn add_wordlist_str(&mut self, text: &str) {
        let words: BTreeSet<String> = text
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .map(str::to_lowercase)
            .collect();
        if let Ok(set) = fst::Set::from_iter(words) {
            self.sets.push(set);
        }
    }

    /// Add a wordlist file.
    pub fn add_wordlist_file(&mut self, path: &Path) -> anyhow::Result<()> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("cannot read wordlist {}: {e}", path.display()))?;
        self.add_wordlist_str(&text);
        Ok(())
    }

    /// Add individual words to the mutable set (project/global words).
    pub fn add_words<I: IntoIterator<Item = S>, S: AsRef<str>>(&mut self, words: I) {
        for w in words {
            let w = w.as_ref().trim();
            if !w.is_empty() && !w.starts_with('#') {
                self.exact.insert(w.to_lowercase());
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.sets.is_empty() && self.exact.is_empty()
    }

    /// Case-insensitive membership; `word` must already be lowercase.
    pub fn contains(&self, word_lower: &str) -> bool {
        self.exact.contains(word_lower) || self.sets.iter().any(|s| s.contains(word_lower))
    }

    /// Spelling suggestions for an unknown word, best first.
    pub fn suggest(&self, word_lower: &str, max: usize) -> Vec<String> {
        if word_lower.len() > 24 {
            return Vec::new();
        }
        let dist = if word_lower.len() <= 6 { 1 } else { 2 };
        let mut found: Vec<String> = Vec::new();
        if let Ok(lev) = Levenshtein::new(word_lower, dist) {
            for set in &self.sets {
                let mut stream = set.search(&lev).into_stream();
                while let Some(key) = stream.next() {
                    if let Ok(s) = std::str::from_utf8(key) {
                        found.push(s.to_string());
                    }
                }
            }
        }
        for w in &self.exact {
            if edit_distance(word_lower, w) <= dist as usize {
                found.push(w.clone());
            }
        }
        found.sort();
        found.dedup();
        found.sort_by_key(|c| {
            (
                edit_distance(word_lower, c),
                c.len().abs_diff(word_lower.len()),
            )
        });
        found.truncate(max);
        found
    }
}

/// Plain Levenshtein distance over bytes (words here are ASCII).
pub fn edit_distance(a: &str, b: &str) -> usize {
    let a = a.as_bytes();
    let b = b.as_bytes();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut cur = vec![0; b.len() + 1];
    for (i, &ca) in a.iter().enumerate() {
        cur[0] = i + 1;
        for (j, &cb) in b.iter().enumerate() {
            let cost = usize::from(ca != cb);
            cur[j + 1] = (prev[j] + cost).min(prev[j + 1] + 1).min(cur[j] + 1);
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contains_and_suggest() {
        let mut w = WordSets::default();
        w.add_wordlist_str("apple\nbanana\ncherry\n# comment\n");
        w.add_words(["Durian"]);
        assert!(w.contains("apple"));
        assert!(w.contains("durian"));
        assert!(!w.contains("aple"));
        assert_eq!(w.suggest("aple", 3), vec!["apple".to_string()]);
    }

    #[test]
    fn code_terms_present() {
        let w = WordSets::with_code_terms();
        assert!(w.contains("stdin"));
        assert!(w.contains("grpc"));
        assert!(w.contains("tokio"));
    }

    #[test]
    fn distance() {
        assert_eq!(edit_distance("kitten", "sitting"), 3);
        assert_eq!(edit_distance("", "abc"), 3);
        assert_eq!(edit_distance("abc", "abc"), 0);
    }
}
