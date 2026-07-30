---
title: CI 設定
description: GitHub Actions、GitLab CI、一般的なシェルベースのビルドで ayame-spell を実行します。
---

ayame-spell は指摘が残っていると終了コード `1` を返すため、通常の CI ステップ
だけでビルドを失敗させられます。`ayame-spell.toml`、`ayame-words.txt`、
設定から参照するローカル辞書をコミットしてください。

## GitHub Actions

このリポジトリには composite Action が含まれます。メジャーリリースと checker
バージョンを固定します。

```yaml
      - uses: actions/checkout@v6
      - uses: hjosugi/ayame-spell@v1
        with:
          version: 1.0.0
```

指定した crates.io バージョンを厳密に導入し、GitHub の注釈を出力します。
`sarif: true` なら代わりに SARIF をアップロードします。この場合、呼び出し側
workflow に `security-events: write` を付与してください。

ソースからビルドする同等の設定は次のとおりです。

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
      - run: ayame-spell check . --format github
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

## CircleCI

```yaml
version: 2.1
jobs:
  spelling:
    docker:
      - image: cimg/rust:1.80
    steps:
      - checkout
      - run: cargo install --locked ayame-spell
      - run: ayame-spell check . --format brief
workflows:
  spelling:
    jobs: [spelling]
```

シェルを使える任意の CI では、次のポータブルな手順を使えます。

```sh
cargo install --locked ayame-spell
ayame-spell check . --format brief
```

## pre-commit

チェック用と手動修正用のフックを公開しています。

```yaml
repos:
  - repo: https://github.com/hjosugi/ayame-spell
    rev: v1.0.0
    hooks:
      - id: ayame-spell
```

ファイルを明示的に書き換える場合は `id: ayame-spell-fix` と
`stages: [manual]` を使い、
`pre-commit run ayame-spell-fix --all-files` を実行します。

## GitHub 注釈と SARIF

```sh
ayame-spell check . --format github
```

GitHub 形式はネイティブな Workflow command を出力し、`GITHUB_ACTIONS=true`
なら自動的に選ばれます。Code scanning には SARIF を生成してアップロードします。

```yaml
      - name: スペルチェック SARIF を生成
        run: ayame-spell check . --format sarif > ayame-spell.sarif
        continue-on-error: true
      - uses: github/codeql-action/upload-sarif@v4
        with:
          sarif_file: ayame-spell.sarif
```

人向け形式は解析せず、コンパイラー風のログには `brief`、自動処理には `json` を
使ってください。

## 既存リポジトリを一括修正せず導入する

最初に内容ベースのベースラインを作成してコミットします。

```sh
ayame-spell baseline .
git add ayame-spell-baseline.json
```

以後の `ayame-spell check .` は既存の指摘を抑制し、新しく増えた指摘だけで失敗
します。フィンガープリントは行番号ではなく、パス、ルール、語、周辺行の内容を
使うため、行を挿入してもベースラインは無効になりません。全件を監査する場合は
`ayame-spell check --no-baseline .` を使います。

既存の指摘を直した後は不要なエントリーを除去し、コミット済みファイルとの差分を
確認します。

```sh
ayame-spell baseline --prune .
git diff --exit-code ayame-spell-baseline.json
```

## CI でレジストリ辞書を使う

レジストリ参照はローカルキャッシュから解決します。`ayame-spell.lock` を
commit し、チェック前に同じ version を復元します。

```sh
ayame-spell dict add --cache-only en-base python
ayame-spell check .
```

完全再現またはオフラインの CI では `ayame-spell dict vendor <name>` を実行し、
コピーしたファイルと書き換え済み設定を commit して相対パスで参照します。

## ドキュメントの鮮度を確認する

このリポジトリでは CLI リファレンスを Clap から生成し、差分がないことを
確認します。

```sh
cargo xtask cli-docs
git diff --exit-code -- site/src/content/docs/reference/cli.md \
  site/src/content/docs/ja/reference/cli.md
```
