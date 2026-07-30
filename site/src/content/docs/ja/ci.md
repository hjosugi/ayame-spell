---
title: CI 設定
description: GitHub Actions、GitLab CI、一般的なシェルベースのビルドで ayame-spell を実行します。
---

ayame-spell は指摘が残っていると終了コード `1` を返すため、通常の CI ステップ
だけでビルドを失敗させられます。`ayame-spell.toml`、`ayame-words.txt`、
設定から参照するローカル辞書をコミットしてください。

## GitHub Actions

```yaml
name: spelling

on:
  pull_request:
  push:
    branches: [main]

jobs:
  ayame-spell:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo install --locked ayame-spell
      - run: ayame-spell check . --format brief
```

ビルド時間を短縮する場合は、コンパイルの代わりにバージョンを固定したリリース
アーカイブを取得します。同じリリースの `SHA256SUMS` と照合してください。

## GitLab CI

```yaml
spell:
  image: rust:1.80
  cache:
    paths:
      - .cargo/
  before_script:
    - cargo install --locked ayame-spell
  script:
    - ayame-spell check . --format brief
```

## 注釈用の JSON Lines

```sh
ayame-spell check . --format json > ayame-spell.jsonl
```

出力の各行は独立した JSON オブジェクトです。`type` が `issue` のレコードを
抽出し、`path`、`line`、`column`、`message`、`kind` をネイティブ注釈へ
変換します。最後の `summary` レコードには走査の集計が入ります。

```sh
jq -c 'select(.type == "issue")' ayame-spell.jsonl
```

人向け形式は解析せず、コンパイラー風のログには `brief`、自動処理には `json` を
使ってください。

## CI でレジストリ辞書を使う

レジストリ参照はローカルキャッシュから解決するため、チェック前に導入します。

```sh
ayame-spell dict add --cache-only en-base python
ayame-spell check .
```

完全再現またはオフラインの CI では辞書をリポジトリ内に置き、
`[words].dictionaries` から相対パスで参照します。

## ドキュメントの鮮度を確認する

このリポジトリでは CLI リファレンスを Clap から生成し、差分がないことを
確認します。

```sh
cargo xtask cli-docs
git diff --exit-code -- site/src/content/docs/reference/cli.md \
  site/src/content/docs/ja/reference/cli.md
```
