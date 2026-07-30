---
title: 設定リファレンス
description: ayame-spell の全設定キー、既定値、マージ規則、上書き優先順位。
---

すべてのキーは省略できます。未知のキーはエラーになるため、設定名の書き間違いを
黙って無視しません。

## 検出と優先順位

チェック対象から上位ディレクトリへ進み、最初に見つかった
`ayame-spell.toml` または `.ayame-spell.toml` を使います。どちらもない場合は
最寄りの Git ルート、それもなければ開始ディレクトリがプロジェクトルートです。

最終設定は次の順序で作ります。

1. 組み込みの既定値。
2. OS の設定ディレクトリにある `ayame-spell/config.toml`。
3. 検出したプロジェクト設定。
4. 一致する `[[overrides]]` を記載順に適用。

ユーザー設定とプロジェクト設定では、プロジェクト側のスカラー値がユーザー側を
置き換えます。リストは連結し、マップは拡張します。同じマップキーはプロジェクト
側が優先です。`[[overrides]]` も連結します。

`ayame-spell config` を実行すると、読み込んだファイルと既定値を反映した最終
設定を表示できます。

バージョン固定の JSON Schema は
[`schema/v1/ayame-spell.json`](https://hjosugi.github.io/ayame-spell/schema/v1/ayame-spell.json)
で公開します。同じスキーマは `ayame-spell config --schema` でオフライン出力
でき、検出した設定または指定ファイルを次のように検証できます。

```sh
ayame-spell config --validate
ayame-spell config --validate config/strict.toml
```

未知のキーは近い正規キーの候補付きでエラーになります。TOML の schema コメントを
扱えるエディターでは、カタログによる自動検出を待たず明示できます。

```toml
#:schema https://hjosugi.github.io/ayame-spell/schema/v1/ayame-spell.json
```

分離した自動処理やポータブル環境では、次の環境変数で OS の既定場所を
置き換えられます。

| 環境変数 | 意味 |
| --- | --- |
| `AYAME_SPELL_CONFIG_DIR` | ユーザー全体の `config.toml` と `words.txt` を置くディレクトリ。 |
| `AYAME_SPELL_CACHE_DIR` | アプリ用キャッシュディレクトリ。レジストリ辞書は `dicts/` 以下へ保存。 |
| `AYAME_SPELL_REGISTRY` | 辞書レジストリの `index.json` URL。 |

## `[check]`

| キー | 型 | 既定値 | 意味 |
| --- | --- | --- | --- |
| `mode` | `"corrections"` \| `"dictionary"` \| `"off"` | `"corrections"` | 英単語のチェックモード。日本語チェックは独立。 |
| `min-word-len` | 0 以上の整数 | `3` | このバイト長より短い ASCII 部分語を除外。辞書モードの未知語には別途 4 文字の下限もある。 |
| `max-token-len` | 0 以上の整数 | `40` | これより長く数字を含むトークンを、ハッシュや生成識別子として除外。 |

```toml
[check]
mode = "dictionary"
min-word-len = 3
max-token-len = 40
```

## `[files]`

| キー | 型 | 既定値 | 意味 |
| --- | --- | --- | --- |
| `exclude` | glob 文字列の配列 | 下記 | プロジェクトルート基準の追加除外。 |
| `include-hidden` | 真偽値 | `false` | 隠しファイルと隠しディレクトリを含める。`.git` 自体は常に除外。 |
| `max-file-size` | バイト数の整数 | `0` | これより大きいファイルを除外。`0` は無制限で、除外件数は報告する。 |

次の組み込み除外は常に存在し、ユーザーの指定は後ろへ追加されます。

```text
*.lock
*.sum
package-lock.json
pnpm-lock.yaml
yarn.lock
*.min.js
*.min.css
```

`.gitignore` も尊重します。

```toml
[files]
exclude = ["vendor/**", "snapshots/**"]
include-hidden = false
max-file-size = 10485760
```

## `[words]`

| キー | 型 | 既定値 | 意味 |
| --- | --- | --- | --- |
| `project` | パス文字列 | `"ayame-words.txt"` | チーム用単語ファイル。相対パスはプロジェクトルート基準。 |
| `ignore` | 文字列の配列 | `[]` | 全英語モードで報告しない単語。大文字小文字を区別しない。完全一致する日本語表記ゆれも抑止。 |
| `dictionaries` | 参照の配列 | `[]` | 辞書モードで使う単語リスト。 |

参照には絶対パス、プロジェクトルートからの相対パス、
`registry:name` / `registry:name@version` を指定できます。レジストリファイルは
先に `dict add` でキャッシュし、pinなし参照を再現するには生成された
`ayame-spell.lock` をコミットします。

```toml
[words]
project = "config/accepted-words.txt"
ignore = ["exmaple"]
dictionaries = ["registry:en-base", "dict/team.txt"]
```

ユーザー全体の単語ファイルもプロジェクト単語に加えて読み込みます。更新には
`ayame-spell words add --global WORD` を使います。

## `[corrections]`

| キー | 型 | 既定値 | 意味 |
| --- | --- | --- | --- |
| `builtin` | 真偽値 | `true` | 同梱の `typos-dict` 修正表を有効化。 |
| `extra` | 参照の配列 | `[]` | 追加 TSV 修正表または `registry:name`。 |

TSV のコメント以外の各行には、誤字、タブ、カンマ区切りの修正候補を書きます。

```text
recieve	receive
fo	foo,of
```

### `[corrections.words]`

誤字から一つの修正文字列、または修正文字列配列へのインラインマップです。誤字と
同じ修正値を指定すると許可リストになります。

```toml
[corrections.words]
teh = "the"
fo = ["of", "go"]
neet = "neet"
```

照合は大文字小文字を区別せず、可能な場合は元の大文字小文字パターンを保って
置換します。

## `[japanese]`

| キー | 型 | 既定値 | 意味 |
| --- | --- | --- | --- |
| `enabled` | 真偽値 | `true` | 設定済みの日本語チェック全体を有効化。 |
| `katakana-style` | `"consistency"` \| `"long"` \| `"short"` \| `"off"` | `"consistency"` | カタカナ長音の方針。 |
| `variant-files` | 参照の配列 | `[]` | TOML 表記ゆれルールまたはレジストリ辞書。 |
| `flag-fullwidth-alnum` | 真偽値 | `true` | 全角 ASCII 英字・数字を報告。 |
| `flag-halfwidth-kana` | 真偽値 | `true` | 半角カタカナを報告。 |
| `fullwidth-space` | `"code"` \| `"always"` \| `"never"` | `"code"` | U+3000 を報告する場所。 |

`"code"` は文章として認識する拡張子以外で全角スペースを報告します。
`"consistency"` は同じ文書内で既知の長音あり・なしが混在した場合だけ少数側を
報告します。`"long"` と `"short"` は方向を強制し、`"off"` は組み込みペアの
方針を無効にします。日本語チェックが有効なら、独自の表記ゆれルールは引き続き
適用します。

```toml
[japanese]
enabled = true
katakana-style = "consistency"
variant-files = ["registry:ja-tech-variants", "dict/product-variants.toml"]
flag-fullwidth-alnum = true
flag-halfwidth-kana = true
fullwidth-space = "code"
```

### `[japanese.variants]`

表記ゆれから推奨表記へのインラインマップです。

```toml
[japanese.variants]
"インタフェース" = "インターフェース"
```

表記ゆれファイルには同じマップを `[variants]` の下へ書きます。トップレベルの
マップも読み込めます。

```toml
[variants]
"ソフトウエア" = "ソフトウェア"
```

## `[[overrides]]`

| キー | 型 | 必須 | 意味 |
| --- | --- | --- | --- |
| `paths` | glob 文字列の配列 | 必須 | プロジェクトルートからの相対パスに照合。 |
| `mode` | チェックモード | 任意 | 一致したファイルの `[check].mode` を置換。 |
| `japanese` | 真偽値 | 任意 | 一致したファイルの日本語チェックを有効化または無効化。 |

一致する全項目を記載順に適用します。各プロパティで後の項目が優先です。

```toml
[[overrides]]
paths = ["docs/**"]
mode = "dictionary"

[[overrides]]
paths = ["docs/generated/**"]
mode = "off"
japanese = false
```

`docs/generated/api.md` では、両方のプロパティで二つ目の項目が優先です。
`docs/` 内のほかのファイルは辞書モードになり、日本語の全体設定は変わりません。

上書きでは単語リスト、修正表、ファイル走査設定、個別の日本語ルール設定を変更
できません。

## 完全な例

```toml
[check]
mode = "corrections"
min-word-len = 3
max-token-len = 40

[files]
exclude = ["vendor/**"]
include-hidden = false
max-file-size = 0

[words]
project = "ayame-words.txt"
ignore = ["exmaple"]
dictionaries = ["registry:en-base"]

[corrections]
builtin = true
extra = ["dict/fixes.tsv"]

[corrections.words]
teh = "the"

[japanese]
enabled = true
katakana-style = "consistency"
variant-files = ["registry:ja-tech-variants"]
flag-fullwidth-alnum = true
flag-halfwidth-kana = true
fullwidth-space = "code"

[japanese.variants]
"インタフェース" = "インターフェース"

[[overrides]]
paths = ["docs/**"]
mode = "dictionary"
```
