---
title: 終了コードと出力形式
description: ayame-spell の終了状態、人向け、簡潔、JSON Lines 出力を連携に利用します。
---

## 終了コード

| コード | 意味 |
| --- | --- |
| `0` | コマンドが成功し、未修正の指摘がない。 |
| `1` | チェックは成功したが、一つ以上の指摘が残っている。 |
| `2` | 使用法、設定、ファイル、ネットワークなどの処理エラー。 |

`fix` では、安全に適用できた指摘だけなら終了コード `1` になりません。修正後に
指摘が残っている場合だけ `1` を返します。

## 人向け形式

`--format human` が既定です。

```text
docs/guide.md:4:3: recieve → receive [typo]
```

標準出力が端末なら、対象語と候補に色を付けます。概要は標準エラー出力へ書きます。

```text
1 issue(s) in 1 file(s) — 12 file(s) checked
```

概要には修正済み件数、除外したバイナリ、`max-file-size` により除外した
ファイル数も含まれます。

## 簡潔形式

`--format brief` はコンパイラー風で色のないレコードを出力します。

```text
docs/guide.md:4:3: recieve -> receive
```

`path:line:column` を認識する CI ログに使います。

## JSON Lines 形式

`--format json` は指摘ごとに一つの JSON オブジェクトを書き、概要は出しません。

```json
{"path":"docs/guide.md","line":4,"column":3,"offset":42,"length":7,"word":"recieve","kind":"typo","suggestions":["receive"],"message":"`recieve` should be `receive`"}
```

| フィールド | 型 | 意味 |
| --- | --- | --- |
| `path` | 文字列 | ファイル走査が報告するチェック対象パス。 |
| `line` | 整数 | 1 始まりの行。 |
| `column` | 整数 | 1 始まりの文字位置。 |
| `offset` | 整数 | ファイルテキスト内の 0 始まりバイト位置。 |
| `length` | 整数 | 指摘部分のバイト長。 |
| `word` | 文字列 | 元のテキスト。 |
| `kind` | 文字列 | 安定した[指摘コード](./rules/)。 |
| `suggestions` | 文字列配列 | 順位付きの置換候補。 |
| `message` | 文字列 | 人が読める説明。 |

ストリームを一行ずつ読み込んでください。出力全体は JSON 配列ではありません。

## 単語収集の出力

`words collect` には個別の `--plain` と `--json` があります。

```sh
ayame-spell words collect --plain
ayame-spell words collect --json
```

JSON Lines の各オブジェクトは `word`、`count`、`kind`、`example` を含みます。
