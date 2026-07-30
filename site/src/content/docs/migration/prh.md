---
title: Migrating from prh
description: Convert supported literal and regex rules into a project variant file.
---

## Preview

```sh
ayame-spell import prh rules.yml --dry-run
```

The preview shows the config reference and generated TOML without writing
either file.

## Write

```sh
ayame-spell import prh rules.yml
git diff -- ayame-spell.toml dict/imported-prh.toml
```

Use `--output path/to/rules.toml` to choose another project-local file.

## Supported subset

Each rule needs a string `expected` plus `pattern` or `patterns`. Literal
patterns are escaped. `/expression/` and `/expression/i` become Rust regexes;
capture references such as `$1` remain available in the replacement.

Generated files use:

```toml
[[rules]]
pattern = "(?i)Web ?サイト"
replace = "ウェブサイト"
```

## Untranslated rules

Missing expected values, non-string patterns, and regex features unsupported by
Rust are listed with their rule number. At least one rule must translate before
the command writes. Sentence-structure and context-sensitive rules should
remain in prh or textlint.

