---
title: Configuration reference
description: Every ayame-spell configuration key, default, merge rule, and override precedence rule.
---

All keys are optional. Unknown keys are errors, which catches misspelled
settings instead of silently ignoring them.

## Discovery and precedence

For a checked path, ayame-spell walks upward and uses the first
`ayame-spell.toml` or `.ayame-spell.toml` it finds. If neither exists, the
nearest Git root becomes the project root; otherwise the starting directory is
the root.

The effective configuration is built in this order:

1. Built-in defaults.
2. Global `ayame-spell/config.toml` in the platform config directory.
3. The discovered project configuration.
4. Matching `[[overrides]]`, in file order.

For global and project files, scalar values in the project replace global
values. Lists are appended and maps are extended, with project map entries
replacing equal global keys. `[[overrides]]` entries are also appended.

Run `ayame-spell config` to print the effective merged configuration and the
files from which it was loaded.

The versioned JSON Schema is published at
[`schema/v1/ayame-spell.json`](https://hjosugi.github.io/ayame-spell/schema/v1/ayame-spell.json).
Print the same schema offline with `ayame-spell config --schema`, or validate a
discovered or explicit file with:

```sh
ayame-spell config --validate
ayame-spell config --validate config/strict.toml
```

Unknown keys are rejected with a nearby-key suggestion. Editors with TOML
schema comments can opt in without waiting for catalog discovery:

```toml
#:schema https://hjosugi.github.io/ayame-spell/schema/v1/ayame-spell.json
```

For isolated automation and portable installations, these environment
variables replace the corresponding platform locations:

| Variable | Meaning |
| --- | --- |
| `AYAME_SPELL_CONFIG_DIR` | Directory containing global `config.toml` and `words.txt`. |
| `AYAME_SPELL_CACHE_DIR` | Application cache directory; registry files are stored under `dicts/`. |
| `AYAME_SPELL_REGISTRY` | URL of the dictionary registry `index.json`. |

## `[check]`

| Key | Type | Default | Meaning |
| --- | --- | --- | --- |
| `mode` | `"corrections"` \| `"dictionary"` \| `"off"` | `"corrections"` | English checking mode. Japanese checks are independent. |
| `min-word-len` | non-negative integer | `3` | ASCII subwords shorter than this byte length are skipped. Dictionary-mode unknown words also have a minimum length of four. |
| `max-token-len` | non-negative integer | `40` | Longer digit-bearing tokens are treated as hashes or generated identifiers and skipped. |

```toml
[check]
mode = "dictionary"
min-word-len = 3
max-token-len = 40
```

## `[files]`

| Key | Type | Default | Meaning |
| --- | --- | --- | --- |
| `exclude` | array of glob strings | see below | Additional excludes, relative to the project root. |
| `include-hidden` | boolean | `false` | Include hidden files and directories. `.git` itself is always excluded. |
| `max-file-size` | integer bytes | `0` | Skip larger files. `0` means unlimited; skipped files are counted. |

The built-in excludes are always present and user excludes are appended:

```text
*.lock
*.sum
package-lock.json
pnpm-lock.yaml
yarn.lock
*.min.js
*.min.css
```

`.gitignore` is also respected.

```toml
[files]
exclude = ["vendor/**", "snapshots/**"]
include-hidden = false
max-file-size = 10485760
```

## `[words]`

| Key | Type | Default | Meaning |
| --- | --- | --- | --- |
| `project` | path string | `"ayame-words.txt"` | Team word file. Relative paths resolve from the project root. |
| `ignore` | array of strings | `[]` | Words never reported in any English mode. Matching is case-insensitive. Exact forms also silence Japanese variant findings. |
| `dictionaries` | array of references | `[]` | Wordlists used by dictionary mode. |

References can be an absolute path, a path relative to the project root, or
`registry:name`. Registry files must first be cached with `dict add`.

```toml
[words]
project = "config/accepted-words.txt"
ignore = ["exmaple"]
dictionaries = ["registry:en-base", "dict/team.txt"]
```

The per-user global words file is loaded in addition to the project file. Use
`ayame-spell words add --global WORD` to update it.

## `[corrections]`

| Key | Type | Default | Meaning |
| --- | --- | --- | --- |
| `builtin` | boolean | `true` | Enable the bundled `typos-dict` correction table. |
| `extra` | array of references | `[]` | Extra TSV correction tables or `registry:name` references. |

Each non-comment TSV line has a typo, a tab, and comma-separated replacements:

```text
recieve	receive
fo	foo,of
```

### `[corrections.words]`

An inline map from typo to one replacement or an array of replacements. A
replacement identical to the typo is an allow-list entry.

```toml
[corrections.words]
teh = "the"
fo = ["of", "go"]
neet = "neet"
```

Matching is case-insensitive and replacements preserve the source case where
possible.

## `[japanese]`

| Key | Type | Default | Meaning |
| --- | --- | --- | --- |
| `enabled` | boolean | `true` | Enable all configured Japanese checks. |
| `katakana-style` | `"consistency"` \| `"long"` \| `"short"` \| `"off"` | `"consistency"` | Katakana long-vowel policy. |
| `variant-files` | array of references | `[]` | TOML variant-rule files or registry dictionaries. |
| `flag-fullwidth-alnum` | boolean | `true` | Report fullwidth ASCII letters and digits. |
| `flag-halfwidth-kana` | boolean | `true` | Report halfwidth katakana. |
| `fullwidth-space` | `"code"` \| `"always"` \| `"never"` | `"code"` | Where U+3000 is reported. |

`"code"` reports fullwidth spaces outside recognized prose extensions.
`"consistency"` reports only the minority form when a document mixes a known
short/long katakana pair. `"long"` and `"short"` enforce a direction, while
`"off"` disables the built-in pair policy. Custom variants still apply while
Japanese checks are enabled.

```toml
[japanese]
enabled = true
katakana-style = "consistency"
variant-files = ["registry:ja-tech-variants", "dict/product-variants.toml"]
flag-fullwidth-alnum = true
flag-halfwidth-kana = true
fullwidth-space = "code"
```

### `[japanese.variants]`

An inline map from a variant to its preferred form:

```toml
[japanese.variants]
"インタフェース" = "インターフェース"
```

A variant file contains the same map under `[variants]` (a top-level map is
also accepted):

```toml
[variants]
"ソフトウエア" = "ソフトウェア"
```

## `[[overrides]]`

| Key | Type | Required | Meaning |
| --- | --- | --- | --- |
| `paths` | array of glob strings | yes | Paths matched relative to the project root. |
| `mode` | checking mode | no | Replace `[check].mode` for matching files. |
| `japanese` | boolean | no | Enable or disable Japanese checks for matching files. |

Every matching entry is applied in file order. Later entries win independently
for each property:

```toml
[[overrides]]
paths = ["docs/**"]
mode = "dictionary"

[[overrides]]
paths = ["docs/generated/**"]
mode = "off"
japanese = false
```

For `docs/generated/api.md`, the second entry wins for both properties. For
other files in `docs/`, dictionary mode applies and the global Japanese setting
is unchanged.

Overrides do not replace wordlists, correction tables, file-walk settings, or
individual Japanese rule settings.

## Complete example

```toml
[check]
mode = "corrections"
min-word-len = 3
max-token-len = 40

[files]
exclude = ["vendor/**"]
include-hidden = false
max-file-size = 0

[words]
project = "ayame-words.txt"
ignore = ["exmaple"]
dictionaries = ["registry:en-base"]

[corrections]
builtin = true
extra = ["dict/fixes.tsv"]

[corrections.words]
teh = "the"

[japanese]
enabled = true
katakana-style = "consistency"
variant-files = ["registry:ja-tech-variants"]
flag-fullwidth-alnum = true
flag-halfwidth-kana = true
fullwidth-space = "code"

[japanese.variants]
"インタフェース" = "インターフェース"

[[overrides]]
paths = ["docs/**"]
mode = "dictionary"
```
