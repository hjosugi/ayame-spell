# ayame-spell へのコントリビューション

[English](CONTRIBUTING.md)

ayame-spell をより高速で静かに、使いやすくするための協力を歓迎します。
大きな挙動変更や形式変更では、実装前に設計を合意できるよう、先に issue を
作成してください。

## 開発環境

stable Rust と Node.js 24 をインストールします。リポジトリを clone した後、
次を実行してください。

```sh
cargo build --workspace
cargo test --workspace
npm ci --prefix site
npm run check --prefix site
```

VS Code 拡張には独立した Node.js workspace があります。

```sh
npm ci --prefix editors/vscode
npm run check --prefix editors/vscode
```

## pull request 前の確認

CI と同じ主要チェックを実行します。

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
cargo deny check
npm run check --prefix site
```

workspace の MSRV はルート `Cargo.toml` の `rust-version`（現在は Rust 1.91）
です。CI は stable Rust の lint・test job に加えて、このバージョンで workspace
をコンパイルします。

生成物に影響する変更では、再生成した結果もコミットしてください。

```sh
cargo xtask registry
cargo xtask completions
cargo xtask cli-docs
```

CI はレジストリ索引、シェル補完、生成された EN/JA CLI リファレンスの差分を
拒否します。

## 文書と翻訳

英語と日本語の文書は、次の明示的なペアとして管理します。

- `README.md` と `README.ja.md`
- `DESIGN.md` と `DESIGN.ja.md`
- `CONTRIBUTING.md` と `CONTRIBUTING.ja.md`
- `site/src/content/docs/` 以下の各ページと、対応する `ja/` ページ

同じ pull request でペアの両方を更新し、見出しレベルの並びも一致させて
ください。提出前に `npm run check:i18n --prefix site` を実行します。CI は
ページの存在、見出し構造、ランディングページのアンカー、設定・ルールの網羅性、
変更された各ペアの両方に変更があることを確認します。

整形や生成出力で共有する URL の修正など、実際に言語へ依存しない変更では、
pull request の説明に `i18n-skip: <理由>` を記載できます。理由は必須で、
例外を使ったことは CI 出力にも表示されます。翻訳を後回しにする目的では
使わないでください。

CLI リファレンスは Clap から生成します。コマンドパーサーまたは
`crates/xtask/src/main.rs` の EN/JA 前文を変更し、`cargo xtask cli-docs` を
実行してください。生成後の CLI ページは直接編集しません。

## 辞書のコントリビューション

レジストリのソースは `site/registry/registry.toml` と
`site/registry/dicts/` にあります。辞書の出典とライセンスを記録し、項目を
ソートして重複を除きます。version、由来、ライセンス、サイズ、pull request の
完全なチェックリストは
[`CONTRIBUTING-dictionaries.ja.md`](CONTRIBUTING-dictionaries.ja.md)を参照します。
その後、次を実行します。

```sh
cargo xtask registry
git diff -- site/registry/index.json
```

プロジェクトに同梱・配信できるのは MIT OR Apache-2.0 と両立するライセンスの
データだけです。帰属表示が必要な場合は `NOTICE.md` も更新してください。

## コミットと pull request

コミットは一つの目的に絞り、`Add dictionary search` のような命令形の要約を
使います。挙動変更にはテストを追加し、利用者に見えるトレードオフを pull
request で説明してください。`Closes #123` は受け入れ条件をすべて満たす場合
だけ記載します。

生成されたビルドディレクトリ、エディター個人設定、認証情報、ダウンロードした
レジストリキャッシュはコミットしません。すべてのコントリビューションは、
リポジトリの MIT OR Apache-2.0 ライセンス条件で提出されます。

参加時は[行動規範](CODE_OF_CONDUCT.md)に従ってください。セキュリティ脆弱性は
[SECURITY.md](SECURITY.md)に従って非公開で報告し、未公開の脆弱性を public
issue に投稿しないでください。
