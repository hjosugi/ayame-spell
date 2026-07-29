---
title: インラインディレクティブ
description: 1 行、次の 1 行、またはファイル全体の ayame-spell 指摘を抑止します。
---

インラインディレクティブは、ファイル内のどこでも認識する、大文字小文字を区別
した文字列です。英語と日本語の両方の指摘を抑止します。

| ディレクティブ | 動作 |
| --- | --- |
| `ayame-spell:ignore-line` | ディレクティブがある行を除外。 |
| `ayame-spell:ignore-next-line` | ディレクティブ行と直後の 1 行を除外。 |
| `ayame-spell:ignore-file` | ファイル全体を除外。 |

周囲の形式に合うコメント構文で書きます。

```rust
let teh = 1; // ayame-spell:ignore-line

// ayame-spell:ignore-next-line
let recieve = 2;
```

```markdown
<!-- ayame-spell:ignore-file -->
```

## 照合の詳細

- 行内のどこに書いても認識します。
- ディレクティブ自身がある行は常に除外します。
- `ignore-next-line` は空行を含む物理的な 1 行だけに適用します。
- `ignore-file` は行処理より先に、ファイル内のどこからでも検出します。
- `Ayame-spell:ignore-line` のように大文字小文字が違う文字列は一致しません。
- 一単語だけを抑止するディレクティブはありません。プロジェクト単語、ユーザー
  単語、または `[words].ignore` へ追加してください。

繰り返し現れる意図的な表記には設定や辞書を使ってください。ディレクティブは
テスト用データ、誤字の引用、自動生成例に向いています。
