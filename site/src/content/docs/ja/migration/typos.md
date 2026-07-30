---
title: typosから移行
description: _typos.tomlのextend-wordsとextend-excludeをimportします。
---

## Preview

```sh
ayame-spell import typos _typos.toml --dry-run
```

pathを省略すると、現在のprojectにある`_typos.toml`を使います。

## 書き込み

```sh
ayame-spell import typos
ayame-spell config --validate
```

既存configへmergeし、無関係なsectionを置換しません。

## 対応関係

| typos | ayame-spell |
| --- | --- |
| `[default.extend-words]` | `[corrections.words]` |
| `[files].extend-exclude` | `[files].exclude` |

同じ語へのmappingは許可項目として残ります。置換mappingは元と置換先の表記を
そのまま保ちます。

## 未変換の設定

ほかのtop-level tableはreportします。種類別tokenization、regex識別子置換、
対応先のない設定は確認するまでtypos側に残し、黙って捨てません。

