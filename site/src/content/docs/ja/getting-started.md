---
title: はじめに
description: ayame-spell をインストールし、最初のチェックとプロジェクト設定を行います。
---

## Rust なしでインストール

shell installer は GitHub Release から環境に合う archive を選び、
`SHA256SUMS` と照合して `~/.local/bin` へ binary を配置します。

```sh
curl --proto '=https' --tlsv1.2 -LsSf \
  https://raw.githubusercontent.com/hjosugi/ayame-spell/main/install.sh | sh
```

Windows PowerShell では次を実行します。

```powershell
irm https://raw.githubusercontent.com/hjosugi/ayame-spell/main/install.ps1 | iex
```

release ごとに version 固定の Homebrew / Scoop manifest も添付します。

```sh
brew install --formula \
  https://github.com/ayame-editor/ayame-spell/releases/latest/download/ayame-spell.rb
```

```powershell
scoop install https://github.com/ayame-editor/ayame-spell/releases/latest/download/ayame-spell.json
```

Node project では registry account なしで、release 添付の checksum 検証済み
native wrapper を利用できます。

```sh
npx https://github.com/ayame-editor/ayame-spell/releases/download/v0.5.0/ayame-spell-npm-v0.5.0.tgz check .
```

同じ wrapper を npm に公開後は、短い `npx ayame-spell check .` も利用できます。

CI 用 container は次のとおりです。

```sh
docker run --rm -v "$PWD:/work" -w /work \
  ghcr.io/hjosugi/ayame-spell:0.5.0 check .
```

各 release には AUR maintainer と直接の `makepkg -si` 用に、x86_64/aarch64
対応 `ayame-spell-bin` の `PKGBUILD` と `.SRCINFO` を添付します。再現可能な
manifest generator は
[配布用 source](https://github.com/ayame-editor/ayame-spell/tree/main/packaging)
にあります。

## Rust でインストール

Rust 1.91 以降を使って crates.io からインストールします。

```sh
cargo install ayame-spell
ayame-spell --version
```

また、[GitHub Releases](https://github.com/ayame-editor/ayame-spell/releases/latest)
から環境に合うアーカイブを取得し、`ayame-spell`（Windows は
`ayame-spell.exe`）を `PATH` の通った場所へ置きます。アーカイブの
`completions/` にはシェル補完も入っています。

## 最初のチェック

プロジェクトのディレクトリで実行します。

```sh
ayame-spell
```

サブコマンドとパスを省略すると、カレントディレクトリを修正表モードで確認します。
`.gitignore` を尊重し、一般的な自動生成ロックファイルを除外します。既知の
スペルミスと、有効になっている日本語表記の問題を報告します。

対象パスを指定する場合と、安全な修正を適用する場合は次のとおりです。

```sh
ayame-spell check README.md docs/
ayame-spell fix README.md docs/
```

`fix` が適用するのは、候補が一つの修正と機械的な文字幅変換だけです。未知語や
候補が複数ある修正は、人が確認できるよう残します。

## 設定ファイルを作る

```sh
ayame-spell init
ayame-spell config
```

`init` は `ayame-spell.toml` を作成します。`config` はユーザー設定と
プロジェクト設定をマージし、既定値を適用した最終設定を表示します。

静かな基準として使うなら、生成される修正表モードの設定だけで十分です。辞書に
ない単語も検出する場合は、次を実行します。

```sh
ayame-spell dict add en-base
```

続いて次の設定を追加します。

```toml
[check]
mode = "dictionary"
```

調整方法は[モード](./modes/)と
[設定リファレンス](./reference/configuration/)を参照してください。

## プロジェクト単語を追加する

```sh
ayame-spell words add ProjectName APIName
ayame-spell words collect
ayame-spell words triage
```

プロジェクト単語は既定で `ayame-words.txt` に入ります。このファイルを
コミットすると、CLI とすべてのエディターで同じ語彙を使えます。

## 次に読むページ

- [エディター設定](./editors/)
- [CI 設定](./ci/)
- [すべての指摘コード](./reference/rules/)
- [日本語チェック](./japanese/)
