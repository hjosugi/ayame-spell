---
title: Rules reference
description: Every stable ayame-spell issue code and its fix behavior.
---

Issue codes are stable machine-readable identifiers used in human output, JSON
Lines, word collection, and LSP diagnostics.

| Code | Trigger | Suggestion | Safe auto-fix |
| --- | --- | --- | --- |
| `typo` | A token matches a built-in, extra, or inline correction. | Configured replacement candidates. | Yes, only with exactly one candidate. |
| `unknown-word` | Dictionary mode finds a word in no active wordlist. | Up to four edit-distance matches. | Never. |
| `ja-variant` | A katakana style pair is inconsistent or a custom variant matches. | Majority or configured preferred form. | Yes when a preferred form exists. |
| `fullwidth-alnum` | A run contains fullwidth ASCII letters or digits. | Halfwidth ASCII conversion. | Yes. |
| `halfwidth-kana` | A run contains halfwidth katakana. | Fullwidth katakana conversion, including voiced marks. | Yes. |
| `fullwidth-space` | U+3000 appears where the configured policy flags it. | A normal ASCII space. | Yes. |

## `typo`

Enabled by `[corrections].builtin`, `[corrections].extra`, and
`[corrections.words]`. Matching is case-insensitive and the replacement tries
to preserve lower, title, or upper case:

```text
teh → the
Teh → The
TEH → THE
```

If a correction has several candidates, `fix` leaves it unchanged.

## `unknown-word`

Only active in dictionary mode. A word is accepted when it appears in the
project file, global file, configured wordlists, ignore list, or correction
allow list. All-uppercase words and words shorter than four bytes are not
reported as unknown.

Use `words collect`, `words add`, or `words triage` to maintain the vocabulary.

## `ja-variant`

This code covers:

- Built-in long-vowel style enforcement.
- Document-level consistency of known short/long pairs.
- `[japanese.variants]` entries.
- Rules loaded through `[japanese].variant-files`.

In consistency mode, a document containing only one form is clean. When both
forms occur, the minority form is reported; ties use a deterministic preferred
form from the built-in pair data.

## Width rules

`fullwidth-alnum` converts characters in the fullwidth ASCII letter/digit
ranges. `halfwidth-kana` converts halfwidth katakana runs and combines dakuten
or handakuten. `fullwidth-space` is controlled by
`[japanese].fullwidth-space`.

All three conversions are mechanical and safe for `fix`, subject to the
configured scope.

## Token filtering

Before English rules run, the tokenizer:

- Splits `camelCase` and `PascalCase`, including acronym boundaries.
- Treats underscores and punctuation as boundaries.
- Skips URLs and email addresses.
- Skips hexadecimal, UUID-like, base64-like, and long digit-bearing tokens.
- Skips backslash-prefixed escape-like words.
- Applies `min-word-len`.

This filtering is part of the low-noise design and does not have separate issue
codes.
