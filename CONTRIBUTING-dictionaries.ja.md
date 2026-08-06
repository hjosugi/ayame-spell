# 辞書へのコントリビューション

辞書の変更はレビュー可能で、再現でき、法的に安全である必要があります。
レジストリの原本は `site/registry/registry.toml`、生成メタデータは
`site/registry/index.json` です。

## ファイル形式

- `wordlist`: UTF-8 テキストで 1 行 1 小文字語。空行と `#` で始まる行は無視。
- `corrections`: UTF-8 TSV の `誤字<TAB>修正[,修正]`。
- `variants`: UTF-8 TOML の文字列間 `[variants]` テーブル。

事前相談がない場合、一つの目的に絞った辞書を 10,000 項目未満にします。
wordlist は整列・重複排除し、`en-base` の項目を繰り返してはいけません。
`cargo xtask registry` がこれを検査します。

## バージョン、由来、ライセンス

各 `[[dictionary]]` には次が必須です。

- 一意な名前とセマンティック `version`
- 言語、種類、説明、変更しないファイルパス
- 具体的な `provenance`（由来）
- MIT OR Apache-2.0 の本プロジェクトから再配布可能なライセンス

公開済みの名前/versionが指すバイト列を変更しないでください。新しいファイルと
versionを追加し、古いファイルを `[[dictionary.release]]` で残すことで pin を
維持します。再配布条件を確認せず、Webサイト、規格、ベンダー、パッケージ索引
からデータを複製してはいけません。帰属表示が必要なら `NOTICE.md` も更新します。

## Pull request チェックリスト

- [ ] UTF-8で、一目的に絞り、整列・重複排除している。
- [ ] `registry.toml` にversion、由来、ライセンスがある。
- [ ] 公開済みversionのファイルを残し、変更していない。
- [ ] 必要な帰属表示を `NOTICE.md` に記載した。
- [ ] `cargo xtask registry` が成功し、`index.json` を再生成した。
- [ ] 動作が変わる場合、代表的な正常／エラー fixture を更新した。
- [ ] build 直後の CLI で `contrib/quality/check_quality.py` が成功した。
- [ ] `cargo test --workspace` が成功した。
- [ ] 日英のレジストリ文書が同期している。

実行コマンド:

```sh
cargo xtask registry
git diff --check
cargo test --workspace
cargo build -p ayame-spell --locked
python3 contrib/quality/check_quality.py --binary target/debug/ayame-spell
```
