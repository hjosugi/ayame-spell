---
title: prhから移行
description: 対応するliteralとregex ruleをproject variant fileへ変換します。
---

## Preview

```sh
ayame-spell import prh rules.yml --dry-run
```

config参照と生成TOMLを表示し、どちらのfileにも書き込みません。

## 書き込み

```sh
ayame-spell import prh rules.yml
git diff -- ayame-spell.toml dict/imported-prh.toml
```

別のproject-local fileは`--output path/to/rules.toml`で指定します。

## 対応subset

各ruleには文字列`expected`と`pattern`または`patterns`が必要です。literal
patternはescapeします。`/expression/`と`/expression/i`をRust regexへ変換し、
置換の`$1`などのcapture参照も利用できます。

生成fileの形式:

```toml
[[rules]]
pattern = "(?i)Web ?サイト"
replace = "ウェブサイト"
```

## 未変換のrule

expectedがないrule、文字列でないpattern、Rustが対応しないregex機能をrule番号
付きで一覧表示します。少なくとも1件を変換できるまで書き込みません。文構造や
文脈依存ruleはprhまたはtextlintに残します。

