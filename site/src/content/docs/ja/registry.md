---
title: 辞書レジストリ
description: ayame-spell の共有辞書を導入、確認、キャッシュ、ミラー、追加します。
---

import RegistryTable from "../../../components/RegistryTable.astro";

レジストリは JSON インデックスを通じて単語リストと表記ルールを公開します。
ayame-spell はダウンロードした各ファイルを、インデックスの SHA-256 値と照合
します。

<RegistryTable locale="ja" />

## コマンド

```sh
ayame-spell dict list
ayame-spell dict add en-base python
ayame-spell dict add --cache-only rust
ayame-spell dict update
ayame-spell dict remove python
```

- `list` はインデックスを取得し、キャッシュ済みの項目に `*` を付ける。
- `add` は辞書を取得し、種類に合う設定配列へ追加する。
- `--cache-only` はプロジェクト設定を変更せずに取得する。
- `update` は現在キャッシュしている全項目を再取得する。
- `remove` はキャッシュファイルとプロジェクト設定の参照を削除する。

## 辞書の種類

| 種類 | ファイル形式 | 追加先 |
| --- | --- | --- |
| `wordlist` | UTF-8 テキスト、1 行 1 語、`#` コメント | `[words].dictionaries` |
| `corrections` | `誤字<TAB>修正[,修正]` | `[corrections].extra` |
| `variants` | TOML の `[variants]` マップ | `[japanese].variant-files` |

レジストリ参照は `registry:name` 形式です。現在のレジストリ形式は、取得時に
インデックスの SHA-256 で内容を固定します。設定内の辞書バージョン指定はまだ
対応していません。将来のレジストリ更新後もビルドを完全に同じにする必要がある
場合は、ファイルをプロジェクト内へ置いてください。

## オフライン・完全再現で使う

辞書を取得してリポジトリへコピーし、レジストリ参照を相対パスへ置き換えます。

```toml
[words]
dictionaries = ["dict/en-base.txt", "dict/team.txt"]
```

ネットワークアクセスを避け、プロジェクトと一緒に正確な内容をレビューできます。

## 非公開レジストリ

一つの HTTP(S) ベース URL に `index.json` とファイルを配置し、次を設定します。

```sh
export AYAME_SPELL_REGISTRY=https://docs.example.com/spelling/index.json
```

インデックスのスキーマは次のとおりです。

```json
{
  "version": 1,
  "dictionaries": [
    {
      "name": "team",
      "language": "en",
      "kind": "wordlist",
      "description": "社名と製品用語",
      "file": "dicts/team.txt",
      "sha256": "...",
      "entries": 120,
      "license": "Proprietary"
    }
  ]
}
```

## 辞書を追加する

ayame-spell のチェックアウトで次を行います。

1. UTF-8 データを `site/registry/dicts/` に追加する。
2. `site/registry/registry.toml` に `[[dictionary]]` を追加する。
3. 出典とライセンスをファイルヘッダーに記載し、必要なら `NOTICE.md` も更新する。
4. インデックスを生成する。

   ```sh
   cargo xtask registry
   ```

5. リポジトリのチェックを実行する。

   ```sh
   cargo test --workspace
   cargo run -p ayame-spell -- check .
   ```

項目はソートし、一つの技術領域または表記方針に絞り、秘密情報や非公開識別子を
含めないでください。一つのコードベースの全識別子ではなく、複数プロジェクトで
役立つ用語を単語リストにします。
