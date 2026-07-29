---
title: ほかのツールから移行
description: cSpell、typos、textlint、prh の語彙とルールを ayame-spell へ移します。
---

自動化された `ayame-spell import` コマンドは計画中で、v0.3 ではまだ利用
できません。次の対応関係により、内容を確認しながら手動移行できます。
ayame-spell の結果が安定するまで旧ツールも CI に残し、その後に重複チェックを
外してください。

## cSpell から

代表的な対応関係です。

| cSpell | ayame-spell |
| --- | --- |
| `words` | `ayame-words.txt` の各行 |
| `ignoreWords` | `[words].ignore` |
| `ignorePaths` | `[files].exclude` |
| `dictionaries` / `dictionaryDefinitions` | `[words].dictionaries` のレジストリ参照またはローカルパス |
| 言語・ファイル別上書き | `paths` と `mode` を持つ `[[overrides]]` |

`words` を 1 行 1 語で出力し、ソートと重複除去をしてから辞書モードを始めます。

```toml
[check]
mode = "dictionary"

[words]
project = "ayame-words.txt"
dictionaries = ["registry:en-base"]
```

cSpell の辞書には ayame-spell が読まない形式や接辞データが含まれる場合が
あります。UTF-8 の 1 行 1 語ファイルへ変換してください。正規表現、複合語方針、
ロケール別の大文字小文字、言語別辞書には直接の対応先がないため、名前だけを
コピーせず効果を確認します。

## typos から

ayame-spell の修正表モードが最も近い既定値です。

```toml
[check]
mode = "corrections"
```

| typos の設定 | ayame-spell |
| --- | --- |
| 除外ファイル | `[files].exclude` |
| 許可する識別子 | `ayame-words.txt` または `[words].ignore` |
| 識別子の置換 | `[corrections.words]` |
| ファイル別チェック | `[[overrides]]` |

```toml
[corrections.words]
teh = "the"
Productname = "ProductName"
intentional = "intentional"
```

自分自身への置換は許可項目です。正規表現による識別子変換と種類別
トークン化には直接の対応先がありません。

## textlint から

ayame-spell が実装しない文法・スタイルルールには textlint を残します。
スペルと決定的な表記ルールだけを移します。

- 許可語 → `ayame-words.txt` または `[words].ignore`
- 固定の誤字置換 → `[corrections.words]`
- 日本語の表記ペア → `[japanese.variants]` または表記ゆれファイル
- 除外パス → `[files].exclude`

文構造、句読点の数、文脈依存の用語、正規表現を解析するルールは textlint の
担当として残します。

## prh から

単純な prh ルールが次の場合、

```yaml
- expected: WebSocket
  patterns:
    - web socket
    - websocket
```

ASCII トークンのインライン修正へ変換できます。

```toml
[corrections.words]
websocket = "WebSocket"
```

日本語なら表記ゆれへ変換します。

```toml
[japanese.variants]
"ソフトウエア" = "ソフトウェア"
```

ルールが多い場合は再利用できる TOML ファイルにします。

```toml
[variants]
"インタフェース" = "インターフェース"
"ウエブ" = "ウェブ"
```

prh のキャプチャーグループ、正規表現、単語境界、複数トークンの書き換えには
直接の対応先がありません。そのルールは prh に残してください。

## 移行結果を確認する

```sh
ayame-spell config
ayame-spell words collect
ayame-spell check . --format brief
```

同じコミットに両方のツールを実行して差を分類し、本当に必要なプロジェクト語彙
だけを追加します。古い無視項目を確認せず一括移行すると、将来の誤りまで隠すため
避けてください。
