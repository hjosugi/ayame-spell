---
title: Migrating from typos
description: Import extend-words and extend-exclude from _typos.toml.
---

## Preview

```sh
ayame-spell import typos _typos.toml --dry-run
```

When the path is omitted, `_typos.toml` in the current project is used.

## Write

```sh
ayame-spell import typos
ayame-spell config --validate
```

The importer merges with existing config and does not replace unrelated
sections.

## Mappings

| typos | ayame-spell |
| --- | --- |
| `[default.extend-words]` | `[corrections.words]` |
| `[files].extend-exclude` | `[files].exclude` |

A self-mapping remains an allow entry. Replacement mappings keep their exact
source and target spelling.

## Untranslated settings

Other top-level tables are reported. Type-specific tokenization, regex
identifier remapping, and settings with no equivalent stay in typos until
reviewed; no setting is silently discarded.

