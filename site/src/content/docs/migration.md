---
title: Migrating from other tools
description: Import cSpell, typos, and prh assets with a dry-run and explicit loss report.
---

## Automated migration

ayame-spell imports the settings it can represent and prints every unsupported
item instead of silently dropping it:

```sh
ayame-spell import cspell --dry-run
ayame-spell import typos --dry-run
ayame-spell import prh rules.yml --dry-run
```

Use the tool-specific guides:

- [cSpell](./migration/cspell/)
- [typos](./migration/typos/)
- [prh](./migration/prh/)

Without `--dry-run`, the commands merge into the existing
`ayame-spell.toml`. cSpell words are sorted into `ayame-words.txt`; prh rules
are generated as a project-local TOML rule file.

## Validate the migration

```sh
ayame-spell config --validate
ayame-spell words collect
ayame-spell check . --format brief
```

Run both tools over the same commit, classify differences, and retain only
genuine project vocabulary. The import report is the checklist for settings
that still need a decision.

## textlint responsibilities

Keep textlint for grammar, sentence structure, terminology context, and other
rules outside deterministic spelling or notation. Move allowed terms to
`ayame-words.txt`, fixed typos to `[corrections.words]`, Japanese notation to
variant files, and ignored paths to `[files].exclude`.
