---
title: 構文を考慮したチェック
description: 識別子のノイズを避け、Markdownの文章とソースのコメント・文字列を確認します。
---

`[check].profile` はスペルルールへ渡すバイトを制御します。mask後も元の UTF-8
バイト長、行番号、offsetを保つため、診断と修正位置は変更前ソースと一致します。

## Profile

| Profile | 動作 |
| --- | --- |
| `"all"` | 全トークンを確認。互換性のための既定値。 |
| `"auto"` | Markdown/MDXは文章、認識済みプログラミング言語はソース、それ以外は`all`。 |
| `"prose"` | Markdownのfence・inline code・link targetを除外し、文章とfront matterの値を確認。 |
| `"source"` | ソースの識別子と演算子を除外し、コメントと文字列リテラルを確認。 |

新しい `ayame-spell init` 設定は `"auto"` を選びます。既存プロジェクトは
明示変更するまで `"all"` のままです。

```toml
[check]
profile = "auto"

[[overrides]]
paths = ["docs/generated/**"]
profile = "all"
```

## Markdownの動作

fenced code、backtickのinline code、Markdown linkのtarget部分をmaskします。
link labelは文章として残します。YAML front matterではkeyとdelimiterをmaskし、
`title`や`description`などの値を確認します。

複数行のfenceを追跡し、文書を暗黙に切り捨てません。閉じていないfenceは、残りを
文章だと推測せずfence領域としてmaskします。

## ソースの動作

source profileは行コメント、block comment、引用文字列、template文字列、
Python形式のtriple quoteを認識します。tree-sitter文法ではなく、処理範囲を
限定した字句heuristicです。

この選択により、言語ごとの起動costを増やさず、編集中の不完全なファイルも
予測可能に扱えます。特殊なliteral delimiterには保守的になることがあるため、
生成DSLは`"all"`、自動判定しない拡張子は`"source"`をpath overrideで指定します。

## 複合語と大文字小文字

hyphen区切りの複合語は各要素を確認します。短縮形は1トークンのまま、所有格の
`'s`はlookup前に除き、`APIs`や`IDs`の複数形acronymは許可します。全大文字の
acronymは未知語にしません。mixed-case識別子は大文字境界で分割するため、
`all` profileでは`NmaeService`の`Nmae`と`Service`を確認します。

