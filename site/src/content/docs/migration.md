---
title: Migrating from other tools
description: Move cSpell, typos, textlint, and prh vocabulary and rules into ayame-spell.
---

Automated `ayame-spell import` commands are planned but not available in
v0.3. The mappings below provide a reviewable manual migration. Keep the old
tool in CI until ayame-spell's result is stable, then remove duplicated checks.

## From cSpell

Typical mappings:

| cSpell | ayame-spell |
| --- | --- |
| `words` | Lines in `ayame-words.txt` |
| `ignoreWords` | `[words].ignore` |
| `ignorePaths` | `[files].exclude` |
| `dictionaries` / `dictionaryDefinitions` | `[words].dictionaries` with registry or local paths |
| language/file overrides | `[[overrides]]` with `paths` and `mode` |

Export `words` as one word per line, sort and deduplicate them, then start
dictionary mode:

```toml
[check]
mode = "dictionary"

[words]
project = "ayame-words.txt"
dictionaries = ["registry:en-base"]
```

cSpell dictionaries may use formats or affix data that ayame-spell does not
read. Convert them to a UTF-8 one-word-per-line file. Regular expressions,
compound-word policies, locale-specific casing, and per-language dictionaries
do not have direct equivalents; review their effect instead of copying names.

## From typos

ayame-spell corrections mode is the closest default:

```toml
[check]
mode = "corrections"
```

Mappings:

| typos setting | ayame-spell |
| --- | --- |
| excluded files | `[files].exclude` |
| accepted identifiers | `ayame-words.txt` or `[words].ignore` |
| identifier replacements | `[corrections.words]` |
| file-specific checking | `[[overrides]]` |

```toml
[corrections.words]
teh = "the"
Productname = "ProductName"
intentional = "intentional"
```

A self-mapping is an allow entry. Regex-based identifier remapping and
type-specific tokenization do not map directly.

## From textlint

Keep textlint for grammar and stylistic rules that ayame-spell does not
implement. Move only spelling and deterministic notation rules:

- Allowed terms → `ayame-words.txt` or `[words].ignore`.
- Fixed typo replacements → `[corrections.words]`.
- Japanese notation pairs → `[japanese.variants]` or a variant file.
- Ignored paths → `[files].exclude`.

Rules that analyze sentence structure, punctuation counts, terminology context,
or regular expressions remain textlint responsibilities.

## From prh

A simple prh rule:

```yaml
- expected: WebSocket
  patterns:
    - web socket
    - websocket
```

can become inline corrections for ASCII tokens:

```toml
[corrections.words]
websocket = "WebSocket"
```

or Japanese variants:

```toml
[japanese.variants]
"ソフトウエア" = "ソフトウェア"
```

For many Japanese rules, create a reusable TOML file:

```toml
[variants]
"インタフェース" = "インターフェース"
"ウエブ" = "ウェブ"
```

prh's capture groups, regular expressions, word boundaries, and multi-token
rewrites do not have direct equivalents. Keep those rules in prh.

## Validate the migration

```sh
ayame-spell config
ayame-spell words collect
ayame-spell check . --format brief
```

Run both tools over the same commit, classify differences, and add only genuine
project vocabulary. Avoid bulk-importing every old ignore entry without review;
stale exceptions hide future mistakes.
