---
title: Migrating from cSpell
description: Import cSpell words, ignores, paths, and known dictionary names.
---

## Preview

Pass a file explicitly or let ayame-spell discover `cspell.json`,
`.cspell.json`, `cspell.config.yaml`, or `cspell.config.yml`:

```sh
ayame-spell import cspell cspell.json --dry-run
```

The preview prints both the merged TOML and resulting `ayame-words.txt`.

## Write

```sh
ayame-spell import cspell
git diff -- ayame-spell.toml ayame-words.txt
```

Existing config arrays and words are preserved and deduplicated. If
`[words].project` already names a custom word file, the importer keeps that
path and writes imported words there.

## Mappings

| cSpell | ayame-spell |
| --- | --- |
| `words` | Sorted lines in `ayame-words.txt` |
| `ignoreWords` | `[words].ignore` |
| `ignorePaths` | `[files].exclude` |
| known `dictionaries` | `registry:name` in `[words].dictionaries` |

Common names for TypeScript/Node, Python, Rust, Go, Java/Kotlin, .NET, C++,
Docker/Kubernetes, cloud providers, Terraform, data science, finance, and web
terms map to registry packs.

## Untranslated settings

Unknown dictionary names and every unsupported top-level key are printed under
`not translated`. cSpell affix dictionaries, regex policies, and language
settings require review; the importer never pretends that copying a dictionary
name preserved its behavior.
