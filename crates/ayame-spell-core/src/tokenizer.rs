//! Identifier-aware word extraction.
//!
//! The tokenizer is where false positives go to die: URLs, e-mail addresses,
//! hex literals, hash/base64-like tokens, and escape sequences are excluded
//! before any dictionary is consulted, and identifiers are split on
//! `camelCase` / `snake_case` boundaries.

use memchr::memmem;

#[derive(Debug, Clone)]
pub struct TokenizerOptions {
    /// Words shorter than this are never flagged.
    pub min_word_len: usize,
    /// Tokens longer than this that contain digits are treated as
    /// hashes/identifiers and skipped whole.
    pub max_token_len: usize,
}

impl Default for TokenizerOptions {
    fn default() -> Self {
        Self {
            min_word_len: 3,
            max_token_len: 40,
        }
    }
}

/// A candidate word found in a line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Word<'t> {
    pub text: &'t str,
    /// Byte offset of the word within the line.
    pub start: usize,
}

/// Extract checkable English words from one line.
pub fn words_in_line<'t>(line: &'t str, opts: &TokenizerOptions) -> Vec<Word<'t>> {
    let bytes = line.as_bytes();
    let skips = skip_spans(line);
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if !bytes[i].is_ascii_alphabetic() {
            i += 1;
            continue;
        }
        // Run of ASCII letters and apostrophes (contractions: "doesn't").
        let start = i;
        while i < bytes.len() && (bytes[i].is_ascii_alphabetic() || bytes[i] == b'\'') {
            i += 1;
        }
        let mut end = i;
        while end > start && bytes[end - 1] == b'\'' {
            end -= 1;
        }
        if overlaps(&skips, start, end) {
            continue;
        }
        // A word glued to a backslash is usually an escape sequence: \nfoo.
        if start > 0 && bytes[start - 1] == b'\\' {
            continue;
        }
        let (t0, t1) = token_bounds(bytes, start, end);
        if is_junk_token(&line[t0..t1], opts.max_token_len) {
            i = t1;
            continue;
        }
        let mut lexical_end = end;
        let run = &line[start..end];
        if is_plural_acronym(run) {
            continue;
        }
        if run.len() > 2
            && (run.ends_with("'s") || run.ends_with("'S"))
            && run[..run.len() - 2]
                .bytes()
                .any(|byte| byte.is_ascii_alphabetic())
        {
            lexical_end -= 2;
        }
        for (off, sub) in split_case(&line[start..lexical_end]) {
            if sub.len() >= opts.min_word_len {
                out.push(Word {
                    text: sub,
                    start: start + off,
                });
            }
        }
    }
    out
}

fn is_plural_acronym(run: &str) -> bool {
    run.len() >= 3
        && run.ends_with('s')
        && run[..run.len() - 1]
            .bytes()
            .all(|byte| byte.is_ascii_uppercase())
}

/// Byte spans of the line that must not be spell checked (URLs, e-mails).
fn skip_spans(line: &str) -> Vec<(usize, usize)> {
    let bytes = line.as_bytes();
    let mut spans: Vec<(usize, usize)> = Vec::new();
    for pos in memmem::find_iter(bytes, b"://") {
        spans.push(token_bounds(bytes, pos, (pos + 3).min(bytes.len())));
    }
    for pos in memchr::memchr_iter(b'@', bytes) {
        let (t0, t1) = token_bounds(bytes, pos, pos + 1);
        let local_ok = bytes[t0..pos].iter().any(u8::is_ascii_alphanumeric);
        let domain_ok = pos + 1 < t1 && bytes[pos + 1..t1].contains(&b'.');
        if local_ok && domain_ok {
            spans.push((t0, t1));
        }
    }
    spans.sort_unstable();
    spans
}

fn overlaps(spans: &[(usize, usize)], start: usize, end: usize) -> bool {
    spans.iter().any(|&(a, b)| start < b && end > a)
}

fn is_token_delim(b: u8) -> bool {
    b.is_ascii_whitespace()
        || matches!(
            b,
            b'"' | b'`' | b'(' | b')' | b'[' | b']' | b'{' | b'}' | b'<' | b'>' | b',' | b';'
        )
        || b >= 128
}

/// Expand `[start, end)` to the surrounding whitespace/bracket-delimited
/// token. Non-ASCII bytes end the token, so the result is always a valid
/// `str` slice boundary.
fn token_bounds(bytes: &[u8], start: usize, end: usize) -> (usize, usize) {
    let mut t0 = start;
    while t0 > 0 && !is_token_delim(bytes[t0 - 1]) {
        t0 -= 1;
    }
    let mut t1 = end;
    while t1 < bytes.len() && !is_token_delim(bytes[t1]) {
        t1 += 1;
    }
    (t0, t1)
}

/// Heuristics for tokens that are data, not prose: hex literals, hashes,
/// base64 blobs, UUIDs.
fn is_junk_token(token: &str, max_token_len: usize) -> bool {
    let bytes = token.as_bytes();
    let digits = bytes.iter().filter(|b| b.is_ascii_digit()).count();
    if token.len() > max_token_len && digits > 0 {
        return true;
    }
    if let Some(rest) = token
        .strip_prefix("0x")
        .or_else(|| token.strip_prefix("0X"))
    {
        if !rest.is_empty()
            && rest
                .bytes()
                .all(|b| b.is_ascii_hexdigit() || b == b'_' || b == b'u' || b == b'i')
        {
            return true;
        }
    }
    let alnum = bytes.iter().filter(|b| b.is_ascii_alphanumeric()).count();
    // Long hex runs: SHAs, UUIDs (dashes don't count as alnum).
    if alnum >= 12
        && digits > 0
        && bytes
            .iter()
            .filter(|b| b.is_ascii_alphanumeric())
            .all(u8::is_ascii_hexdigit)
    {
        return true;
    }
    // Base64/hash-like: long, digit-mixed, with frequent case/digit flips.
    if alnum >= 16 && digits >= 3 && transitions(bytes) >= 4 {
        return true;
    }
    false
}

fn transitions(bytes: &[u8]) -> usize {
    bytes
        .windows(2)
        .filter(|w| {
            let (a, b) = (w[0], w[1]);
            (a.is_ascii_lowercase() && b.is_ascii_uppercase())
                || (a.is_ascii_alphabetic() && b.is_ascii_digit())
                || (a.is_ascii_digit() && b.is_ascii_alphabetic())
        })
        .count()
}

/// Split an identifier run on case boundaries:
/// `getUserName` → `get`, `User`, `Name`; `HTTPServer` → `HTTP`, `Server`.
/// Returns `(byte offset within the run, subword)` pairs.
pub fn split_case(run: &str) -> Vec<(usize, &str)> {
    let bytes = run.as_bytes();
    let mut parts = Vec::new();
    let mut start = 0;
    for i in 1..bytes.len() {
        let prev = bytes[i - 1];
        let cur = bytes[i];
        let next = bytes.get(i + 1).copied();
        let boundary = (prev.is_ascii_lowercase() && cur.is_ascii_uppercase())
            || (prev.is_ascii_uppercase()
                && cur.is_ascii_uppercase()
                && next.is_some_and(|n| n.is_ascii_lowercase()));
        if boundary {
            parts.push((start, &run[start..i]));
            start = i;
        }
    }
    parts.push((start, &run[start..]));
    parts
}

/// Re-apply the case pattern of `pattern` to `suggestion`
/// (suggestions in correction tables are lowercase).
pub fn match_case(pattern: &str, suggestion: &str) -> String {
    let letters: Vec<u8> = pattern.bytes().filter(u8::is_ascii_alphabetic).collect();
    if letters.iter().all(u8::is_ascii_lowercase) {
        return suggestion.to_string();
    }
    if letters.len() > 1 && letters.iter().all(u8::is_ascii_uppercase) {
        return suggestion.to_ascii_uppercase();
    }
    let first_upper = letters.first().is_some_and(u8::is_ascii_uppercase);
    let rest_lower = letters.iter().skip(1).all(u8::is_ascii_lowercase);
    if first_upper && rest_lower {
        let mut chars = suggestion.chars();
        return match chars.next() {
            Some(f) => f.to_ascii_uppercase().to_string() + chars.as_str(),
            None => String::new(),
        };
    }
    suggestion.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(line: &str) -> Vec<&str> {
        words_in_line(line, &TokenizerOptions::default())
            .into_iter()
            .map(|w| w.text)
            .collect()
    }

    #[test]
    fn plain_words() {
        assert_eq!(
            words("the quick brown fox"),
            ["the", "quick", "brown", "fox"]
        );
    }

    #[test]
    fn short_words_skipped() {
        assert_eq!(words("a an it the"), ["the"]);
    }

    #[test]
    fn camel_case_split() {
        assert_eq!(words("getUserName"), ["get", "User", "Name"]);
        assert_eq!(words("HTTPServerError"), ["HTTP", "Server", "Error"]);
        assert_eq!(words("parse_html_doc"), ["parse", "html", "doc"]);
    }

    #[test]
    fn urls_skipped() {
        assert_eq!(
            words("see https://exmaple.com/foo?bar=baz for info"),
            ["see", "for", "info"]
        );
    }

    #[test]
    fn emails_skipped() {
        assert_eq!(words("mail foo.bar@exmaple.com now"), ["mail", "now"]);
    }

    #[test]
    fn hex_and_hashes_skipped() {
        assert_eq!(words("id 0xDEADBEEF okay"), ["okay"]);
        assert_eq!(
            words("sha 550e8400e29b41d4a716446655440000 okay"),
            ["sha", "okay"]
        );
        assert_eq!(
            words("token eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9 end"),
            ["token", "end"]
        );
    }

    #[test]
    fn escape_sequences_skipped() {
        assert_eq!(words(r"print \nfoo and \tbar"), ["print", "and"]);
    }

    #[test]
    fn contractions_kept_whole() {
        assert_eq!(words("doesn't matter"), ["doesn't", "matter"]);
        assert_eq!(words("'quoted' words"), ["quoted", "words"]);
        assert_eq!(words("developer's APIs and IDs"), ["developer", "and"]);
        assert_eq!(
            words("state-of-the-art tools"),
            ["state", "the", "art", "tools"]
        );
    }

    #[test]
    fn digits_break_words() {
        assert_eq!(words("sha256sum utf8only"), ["sha", "sum", "utf", "only"]);
    }

    #[test]
    fn case_matching() {
        assert_eq!(match_case("teh", "the"), "the");
        assert_eq!(match_case("Teh", "the"), "The");
        assert_eq!(match_case("TEH", "the"), "THE");
        assert_eq!(match_case("tEh", "the"), "the");
    }

    #[test]
    fn non_ascii_is_ignored() {
        assert_eq!(words("日本語のtextです"), ["text"]);
    }
}
