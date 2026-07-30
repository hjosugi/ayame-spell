---
title: Syntax-aware checking
description: Check Markdown prose and source comments or strings without identifier noise.
---

`[check].profile` controls which bytes reach the spelling rules. Masking keeps
the original UTF-8 byte length, line numbers, and offsets, so diagnostics and
fixes still point into the unmodified source.

## Profiles

| Profile | Behavior |
| --- | --- |
| `"all"` | Check every token. This is the compatibility default. |
| `"auto"` | Use prose filtering for Markdown/MDX, source filtering for recognized programming languages, and `all` elsewhere. |
| `"prose"` | In Markdown, skip fenced and inline code plus link targets; check prose and front-matter values. |
| `"source"` | Check comments and string literals while masking source identifiers and operators. |

New `ayame-spell init` configurations select `"auto"`. Existing projects keep
`"all"` until they opt in.

```toml
[check]
profile = "auto"

[[overrides]]
paths = ["docs/generated/**"]
profile = "all"
```

## Markdown behavior

Fenced code, inline backtick code, and the target portion of Markdown links are
masked. Link labels remain prose. In YAML front matter, keys and delimiters are
masked while values such as `title` and `description` are checked.

The parser tracks fences across lines and never truncates the document. An
unclosed fence masks the remaining fenced region rather than guessing that
code is prose.

## Source behavior

The source profile recognizes line comments, block comments, quoted strings,
template strings, and Python-style triple-quoted strings. It is intentionally
a bounded lexical heuristic, not a tree-sitter grammar.

This choice keeps startup and incomplete-file handling predictable across
languages. It can treat an unusual literal delimiter conservatively; use a
path override with `"all"` for generated DSLs or `"source"` for an extension
that is not selected automatically.

## Compound and case rules

Hyphenated compounds are checked component by component.
Contractions remain one token, possessive `'s` is removed before lookup, and
plural acronyms such as `APIs` and `IDs` are accepted. ALL-CAPS acronyms are
not unknown words. Mixed-case identifiers are split at case boundaries, so
`NmaeService` checks `Nmae` and `Service` when the `all` profile is active.

