---
title: ほかのツールから移行
description: dry-runと明示的な未変換reportを使い、cSpell、typos、prhの資産をimportします。
---

## 自動移行

ayame-spellは表現できる設定をimportし、対応できない項目を黙って捨てずすべて
表示します。

```sh
ayame-spell import cspell --dry-run
ayame-spell import typos --dry-run
ayame-spell import prh rules.yml --dry-run
```

ツール別ガイド:

- [cSpell](./migration/cspell/)
- [typos](./migration/typos/)
- [prh](./migration/prh/)

`--dry-run`を外すと既存の`ayame-spell.toml`へmergeします。cSpellの単語は
sortして`ayame-words.txt`へ、prh ruleはプロジェクト内のTOML rule fileへ
生成します。

## 移行結果を検証

```sh
ayame-spell config --validate
ayame-spell words collect
ayame-spell check . --format brief
```

同じcommitに両ツールを実行して差を分類し、本当に必要なproject語彙だけを
残します。import reportを、判断が残った設定のchecklistとして使います。

## textlintに残す範囲

文法、文構造、文脈依存の用語など、決定的なスペル・表記以外のruleはtextlintに
残します。許可語は`ayame-words.txt`、固定誤字は`[corrections.words]`、
日本語表記はvariant file、無視pathは`[files].exclude`へ移します。
