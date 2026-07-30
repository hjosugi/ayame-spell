---
title: 終了コードと出力形式
description: ayame-spell の終了状態、人向け、簡潔、JSON Lines、GitHub 注釈、SARIF 出力を連携に利用します。
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

`--format json` は指摘ごとに一つの JSON オブジェクトを書き、最後に一つの
概要レコードを書きます。

```json
{"version":1,"type":"issue","path":"docs/guide.md","line":4,"column":3,"offset":42,"length":7,"word":"recieve","kind":"typo","suggestions":["receive"],"fix":"receive","message":"`recieve` should be `receive`"}
{"version":1,"type":"summary","issues":1,"files_with_issues":1,"files_checked":12,"fixed":0,"skipped_binary":0,"skipped_large":0}
```

すべてのレコードに数値の `version` と、種類を示す `type` があります。利用側は
未対応の version を拒否し、未知のフィールドを無視してください。version 1 の
既存フィールドと意味は変更しません。フィールドと `kind` の追加だけを行えます。
削除、改名、型変更、意味変更では version 2 に上げます。

機械可読スキーマは
[`schema/v1/ayame-spell-output.json`](https://hjosugi.github.io/ayame-spell/schema/v1/ayame-spell-output.json)
で公開します。

指摘レコードのフィールド:

| フィールド | 型 | 意味 |
| --- | --- | --- |
| `version` | 整数 | JSON Lines 契約の version。現在は `1`。 |
| `type` | 文字列 | 指摘では常に `"issue"`。 |
| `path` | 文字列 | ファイル走査が報告するチェック対象パス。 |
| `line` | 整数 | 1 始まりの行。 |
| `column` | 整数 | 1 始まりの文字位置。 |
| `offset` | 整数 | ファイルテキスト内の 0 始まりバイト位置。 |
| `length` | 整数 | 指摘部分のバイト長。 |
| `word` | 文字列 | 元のテキスト。 |
| `kind` | 文字列 | 安定した[指摘コード](./rules/)。 |
| `suggestions` | 文字列配列 | 順位付きの置換候補。 |
| `fix` | 文字列または null | 非対話で安全な置換。確認が必要なら `null`。 |
| `message` | 文字列 | 人が読める説明。 |

概要レコードのフィールド:

| フィールド | 型 | 意味 |
| --- | --- | --- |
| `version` | 整数 | JSON Lines 契約の version。現在は `1`。 |
| `type` | 文字列 | 常に `"summary"`。 |
| `issues` | 整数 | 任意の修正後に残った指摘数。 |
| `files_with_issues` | 整数 | 指摘が残ったファイル数。 |
| `files_checked` | 整数 | チェックしたテキストファイル数。 |
| `fixed` | 整数 | この実行で安全に修正した指摘数。 |
| `skipped_binary` | 整数 | バイナリと判断して除外したファイル数。 |
| `skipped_large` | 整数 | `max-file-size` により除外したファイル数。 |

指摘がなくても概要を出すため、コマンドが実行されなかった空ストリームと区別
できます。一行ずつ読み込んでください。出力全体は JSON 配列ではありません。

## GitHub 注釈形式

`--format github` は指摘ごとに Workflow command を1件出力します。

```text
::warning file=docs/guide.md,line=4,col=3,title=ayame-spell [typo]::`recieve` should be `receive`
```

GitHub は Pull Request の正確な行へ注釈として表示します。
`GITHUB_ACTIONS=true` で `--format` を省略すると自動的にこの形式を選び、
明示した形式は常に優先されます。

## SARIF 2.1.0 形式

`--format sarif` は全安定[ルール](./rules/)のメタデータと各指摘の result を
含む、1個の SARIF 2.1.0 JSON 文書を出力します。

```sh
ayame-spell check . --format sarif > ayame-spell.sarif
```

`github/codeql-action/upload-sarif` でアップロードできます。行・文字列の桁は
1始まりで、result properties には元の語と順序付き候補を含みます。

## 単語収集の出力

`words collect` には個別の `--plain` と `--json` があります。

```sh
ayame-spell words collect --plain
ayame-spell words collect --json
```

JSON Lines の各オブジェクトは `word`、`count`、`kind`、`example` を含みます。
