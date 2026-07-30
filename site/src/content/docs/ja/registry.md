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
ayame-spell dict add en-base@1.0.0
ayame-spell dict add --cache-only rust
ayame-spell dict update
ayame-spell dict update --check
ayame-spell dict vendor en-base
ayame-spell dict remove python
```

- `list` はインデックスを取得し、キャッシュ済みの項目に `*` を付ける。
- `add` は辞書を取得し、種類に合う設定配列へ追加する。
- `--cache-only` はプロジェクト設定を変更せずに取得する。
- `update` は `up to date` を報告するか、pinしていない項目をversion間で更新する。
  `--check` は書き込まず、更新があれば終了コード `1` を返す。
- `vendor` は検証済みバイト列を `dict/` 以下へ複製し、プロジェクト設定を相対
  パスへ書き換える。
- `remove` はキャッシュファイルとプロジェクト設定の参照を削除する。

## 辞書の種類

| 種類 | ファイル形式 | 追加先 |
| --- | --- | --- |
| `wordlist` | UTF-8 テキスト、1 行 1 語、`#` コメント | `[words].dictionaries` |
| `corrections` | `誤字<TAB>修正[,修正]` | `[corrections].extra` |
| `variants` | TOML の `[variants]` マップ | `[japanese].variant-files` |

レジストリ参照は `registry:name` または明示的な
`registry:name@version` 形式です。通常の `dict add` は、解決したversion、
変更しない配信ファイル、SHA-256を `ayame-spell.lock` に記録します。この
lockfileをコミットすると、レジストリに新versionがあっても別環境で同じバイト列を
取得・検証できます。

公開済みversionはインデックスの `versions` 配列と配信ファイルに残し、書き換え
ません。明示的な `@version` pin は `dict update` でも進みません。checker は
ロック済みキャッシュを読み込む前に SHA-256 を検証します。

## オフライン・完全再現で使う

次の1コマンドで辞書をプロジェクトへ置き、参照を書き換えます。

```sh
ayame-spell dict vendor en-base
```

生成される設定は相対パスを使います。

```toml
[words]
dictionaries = ["dict/en-base.txt", "dict/team.txt"]
```

ネットワークアクセスを避け、プロジェクトと一緒に正確な内容をレビューできます。

## 非公開レジストリ

一つの HTTP(S) ベース URL に `index.json` とファイルを配置し、次を設定します。

```sh
export AYAME_SPELL_REGISTRY=https://docs.example.com/spelling/index.json
ayame-spell dict --registry https://docs.example.com/spelling/index.json list
```

インデックスのスキーマは次のとおりです。

```json
{
  "version": 2,
  "dictionaries": [
    {
      "name": "team",
      "version": "1.0.0",
      "language": "en",
      "kind": "wordlist",
      "description": "社名と製品用語",
      "provenance": "Example Corp が保守",
      "file": "dicts/team.txt",
      "sha256": "...",
      "entries": 120,
      "versions": [
        {
          "version": "1.0.0",
          "file": "dicts/team.txt",
          "sha256": "...",
          "entries": 120
        }
      ],
      "license": "Proprietary"
    }
  ]
}
```

## 辞書を追加する

ファイル形式、サイズ制限、versionの不変性、由来、ライセンス、pull requestの
チェックリストは
[辞書コントリビューションガイド](https://github.com/hjosugi/ayame-spell/blob/main/CONTRIBUTING-dictionaries.ja.md)
を参照します。ayame-spell のチェックアウトで次を行います。

1. UTF-8 データを `site/registry/dicts/` に追加する。
2. `site/registry/registry.toml` にversion付き `[[dictionary]]` を追加する。
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
含めません。生成処理は重複と、`en-base` に既にある言語別wordlist項目を拒否
します。
