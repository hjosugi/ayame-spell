---
title: cSpellから移行
description: cSpellの単語、無視語、path、既知dictionary名をimportします。
---

## Preview

fileを明示するか、`cspell.json`、`.cspell.json`、
`cspell.config.yaml`、`cspell.config.yml`を自動検出します。

```sh
ayame-spell import cspell cspell.json --dry-run
```

previewにはmerge後のTOMLと`ayame-words.txt`の結果を両方表示します。

## 書き込み

```sh
ayame-spell import cspell
git diff -- ayame-spell.toml ayame-words.txt
```

既存のconfig配列と単語を保持し、重複を除きます。`[words].project`に独自の
単語fileが設定済みなら、そのpathを保持してimport単語を書き込みます。

## 対応関係

| cSpell | ayame-spell |
| --- | --- |
| `words` | sortした`ayame-words.txt`の各行 |
| `ignoreWords` | `[words].ignore` |
| `ignorePaths` | `[files].exclude` |
| 既知の`dictionaries` | `[words].dictionaries`の`registry:name` |

TypeScript/Node、Python、Rust、Go、Java/Kotlin、.NET、C++、
Docker/Kubernetes、cloud provider、Terraform、data science、finance、webの
一般的な名前をregistry packへ対応付けます。

## 未変換の設定

未知dictionary名と対応しないtop-level keyを`not translated`の下へすべて
表示します。cSpellのaffix dictionary、regex方針、language設定は確認が必要です。
名前だけをcopyして動作も保持したように見せることはありません。
