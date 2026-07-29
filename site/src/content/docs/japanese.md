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
