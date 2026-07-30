---
title: Japanese writing guide
description: Configure katakana consistency, preferred variants, fullwidth and halfwidth checks, and prose-sensitive spaces.
---

Japanese checks run independently from the English checking mode. A project can
set `[check].mode = "off"` and still enforce Japanese notation, or disable
Japanese only for selected paths.

## Low-noise default: consistency

```toml
[japanese]
enabled = true
katakana-style = "consistency"
```

Consistency mode does not declare `サーバ` or `サーバー` correct by itself.
A document using only one form is clean. If both known forms occur, ayame-spell
reports the minority form and suggests the majority form.

This keeps existing house styles usable while catching accidental mixing.
Consistency is document-local: another file may consistently choose the other
form.

## Long and short styles

Choose a direction when a style guide requires it:

```toml
[japanese]
katakana-style = "long"  # サーバ → サーバー
```

or:

```toml
[japanese]
katakana-style = "short" # サーバー → サーバ
```

`"off"` disables the built-in long-vowel pair policy. Explicit variant rules
still run unless `[japanese].enabled` is `false`.

## Why consistency is the default

Older Japanese technical-writing practice often applied a mechanical rule that
omitted a word-final long-vowel mark for terms of three morae or more. In the
2019 revision of JIS Z 8301, the table stating that omission principle was
removed; the standard instead points primarily to the Cabinet Notification
guidance for loanword notation. The
[Japan Standards Association's revision briefing](https://webdesk.jsa.or.jp/pdf/dev/md_4632.pdf)
lists this deletion.

Real projects still contain both established styles, and product names or
domain terminology may intentionally differ. ayame-spell therefore detects
inconsistency by default rather than treating either long or short spelling as
universally wrong.

## Explicit variants

Use inline mappings for project-specific preferred forms:

```toml
[japanese.variants]
"インタフェース" = "インターフェース"
"ソフトウエア" = "ソフトウェア"
```

For a reusable rule set:

```toml
[japanese]
variant-files = ["dict/product-variants.toml"]
```

```toml
# dict/product-variants.toml
[variants]
"ウエブサイト" = "ウェブサイト"
```

Registry dictionaries provide larger sets:

```sh
ayame-spell dict add ja-tech-variants
```

`ja-tech-variants` is a compact, opinionated modern tech-writing set.
`ja-variants` contains broader SudachiDict-derived notation pairs; review its
preferred forms against your style before enabling it.

Variant files may also contain a prh-compatible regular-expression subset:

```toml
[[rules]]
pattern = "Web ?サイト"
replace = "ウェブサイト"
```

Rust regular expressions and `$1`-style replacement captures are supported.
Use `ayame-spell import prh rules.yml` to translate supported rules and list
every rule that cannot be represented.

## Kanji and okurigana consistency

```toml
[japanese]
kanji-consistency = true
```

The default reports only mixing within one document for a conservative set of
pairs such as `子供`/`子ども`, `行なう`/`行う`, and
`取扱い`/`取り扱い`. A document that consistently uses either form is clean.
Enable `registry:ja-kanji-variants` when a style guide requires a specific
preferred direction; that dictionary is off until explicitly added.

This follows the same low-noise policy as katakana consistency: house styles
remain valid, while accidental variation is visible.

## Numbers, units, and compatibility characters

```toml
[japanese]
number-consistency = true
flag-compatibility = true
```

Equivalent forms such as `1,000円` and `一〇〇〇円` are compared within a
document, and only the minority style is reported. Compatibility units and
symbols such as `㎏` and `㎡` receive their standard NFKC suggestions (`kg`,
`m2`). Fullwidth ASCII digits remain covered by `flag-fullwidth-alnum`.

These defaults improve searching, copying, and machine processing without
declaring Arabic or kanji numerals universally preferable.

## Punctuation consistency

```toml
[japanese]
punctuation-consistency = true
```

When a document mixes `、。` and `，．`, ayame-spell reports the minority
marks and suggests the majority style. A document consistently following
either a normal Japanese prose style or a technical fullwidth-comma/full-stop
style is clean.

## Fullwidth alphanumerics

```toml
[japanese]
flag-fullwidth-alnum = true
```

Runs such as `ＡＢＣ１２３` are reported as `fullwidth-alnum` with the safe
replacement `ABC123`.

## Halfwidth katakana

```toml
[japanese]
flag-halfwidth-kana = true
```

`ﾃﾞｰﾀ` is reported as `halfwidth-kana` with `データ` as its safe replacement.
Voiced and semi-voiced marks are combined during conversion.

## Fullwidth spaces

```toml
[japanese]
fullwidth-space = "code"
```

Policies:

- `"code"` (default): report U+3000 in source files, but allow it in recognized
  prose formats.
- `"always"`: report it in every file.
- `"never"`: do not report it.

Use `"always"` when Markdown and other prose must also avoid fullwidth
indentation.

## Disable Japanese by path

```toml
[[overrides]]
paths = ["vendor/**", "tests/fixtures/**"]
japanese = false
```

Later matching overrides win. Inline directives can suppress exceptional lines
or files.
