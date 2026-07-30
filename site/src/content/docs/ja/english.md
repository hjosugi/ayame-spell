---
title: 英語locale方針
description: 複合語とacronymを低ノイズで扱いながら、en-USまたはen-GB表記を統一します。
---

既定では一般的な両地域のスペルを許可します。

```toml
[check]
locale = "any"
```

リポジトリの表記方針がある場合は地域を選びます。

```toml
[check]
locale = "en-US" # colour → color
```

または次のとおりです。

```toml
[check]
locale = "en-GB" # color → colour
```

## Localeルールの報告内容

`en-variant`は`typo`とは別です。両方とも実在する語ですが、片方が設定方針と
異なります。組み込み表はcolor/colour、behavior/behaviour、
organization/organisation、analyze/analyseなど、技術文書で一般的なペアを
対象にします。

`"any"`ではlocale指摘を出しません。製品固有の表記はproject wordやinline
directiveで許可でき、方針全体を無効にする必要はありません。

## Tokenの動作

`don't`などの短縮形は1トークンのままです。`state-of-the-art`のような
hyphen複合語は要素ごとに確認します。所有格はbase wordを使い、`APIs`、`IDs`
の複数形acronymと全大文字acronymを未知語にしません。

CamelCaseとPascalCaseは、有効な構文profileがその領域を確認するときだけ
大文字境界で分割します。たとえば`"all"`では`NmaeService`の`Nmae`が対象に
なり、`"source"`ではコメント・文字列以外の識別子をmaskします。

