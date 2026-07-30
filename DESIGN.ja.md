# ayame-spell 設計メモ

[English](DESIGN.md) · [日本語 README](README.ja.md)

## 市場の空白

typos、cSpell、Harper、codespell、Vale、misspell、typos-lsp、
ltex-plus、textlint、prh、RedPen を対象に 2026 年 7 月に行った競合調査では、
市場が明確に二分されている。

- **修正表方式のツールは CI に強い。** typos は誤検知がほぼなく、
  `-w` による一括修正も安全だが、辞書の共有、エディターからの単語追加、
  未知の誤字検出には対応していない。
- **辞書方式のツールはエディターに強い。** cSpell は 200 以上の辞書
  パッケージを導入でき、クイックフィックスも充実している一方、新規
  プロジェクトでは誤検知が多く、単語を一つずつ追加する必要があり、大きな
  ファイルにも弱い。cSpell の `checkLimit` は約 500 KB で通知なく検査を
  打ち切る。
- **日本語校正は別の世界に分かれている。** textlint/prh は Node.js
  ベースであり、一般的なスペルチェッカーは日本語をまったく扱わない。

一つの設定ファイルで両方の方式を提供し、共有辞書も使えるツールは存在しない。
この統合に、一括トリアージと大きなファイルを正直に扱う設計を加えたものが、
ayame-spell の存在理由である。

## アーキテクチャ

```
crates/
  ayame-spell-core   エンジン: トークナイザー、修正表、FST 単語リスト、
                     日本語チェック、設定
                     （ファイル読み込み以外の I/O は行わない）
  ayame-spell        CLI (check/fix/words/dict/init/config) + LSP サーバー
  xtask              レジストリ索引、シェル補完、CLI 文書の生成
editors/vscode       薄い LanguageClient + 一括トリアージ用 QuickPick UI
site/                GitHub Pages: ランディングページ、文書、辞書レジストリ
```

主な判断は次のとおり。

- **Rust 製の単一バイナリ。** LSP サーバーは別バイナリではなく
  `ayame-spell lsp` として動くため、インストールや同梱の対象は一つで済む。
- **辞書を引く前にトークナイザーで誤検知を抑える。** URL、メールアドレス、
  16 進数と `0x` リテラル、数字を含む長いトークン、Base64 やハッシュに
  見えるトークン、`\escape` シーケンスを除外する。camelCase と snake_case
  は頭字語を考慮して分割する。これは typos の重要な知見である。
- **修正データには `typos-dict` crate を使う。** MIT OR Apache-2.0、
  約 95,000 件、コンパイル済み PHF マップのため起動コストはない。コードでは
  日常的な識別子だが文章では誤字になりうる `ser`、`flate`、`referer` は、
  小さな組み込み許可リストで扱う。codespell、misspell、wordfreq のデータは
  CC BY-SA の継承条件を避けるため使わない。
- **単語リストには FST set を使う。** 検索は O(len) でメモリ消費が少なく、
  Levenshtein オートマトンによる候補提示も利用できる。プロジェクト・ユーザー
  単語のような可変ソースはハッシュセットに置き、エディターから単語を追加した
  ときに FST の再構築を不要にする。
- **形態素解析なしで日本語を扱う（v0.1）。** カタカナ列を抽出し、三種類の
  チェックを行う。
  - *一貫性（既定）*: 同じ文書に末尾長音の有無だけが異なる表記がある場合に
    限って指摘する。RedPen と同じ発想で、辞書を必要とせず誤検知がほぼない。
    スタイルは強制しない。JIS Z 8301:2019 では「ー」を省略する規則が廃止
    されたためである。
  - *スタイル指定（任意）*: curated pair table に基づき `long` または
    `short` を強制する。
  - *機械的変換*: 全角英数、半角カタカナ（濁点・半濁点を結合）、全角
    スペース（既定ではコードファイルのみ）を扱う。
- **レジストリは GitHub Pages 上の静的ファイル。** `cargo xtask registry`
  で生成する `index.json` とプレーンテキスト/TOML の辞書を置き、SHA-256 を
  検証する。`dict add` は `toml_edit` で書式を保ったまま
  `ayame-spell.toml` に参照を追加するため、チームは一行をコミットするだけで
  辞書を共有できる。`$AYAME_SPELL_REGISTRY` で非公開レジストリへ切り替えられる。
- **一括操作を優先する UX。** `words collect` は頻度順に収集し、
  `words triage` は複数選択した語をプロジェクト・ユーザー・ignore に
  振り分ける。VS Code の「Review Flagged Words」も LSP の
  `executeCommand` を通して同じ流れを提供する。辞書方式のチェッカーで最も多い
  不満を解消するための設計である。
- **通知のない劣化を許さない。** バイナリやサイズ超過でスキップしたファイルは
  集計して表示する。LSP は診断を一文書 1,000 件に制限した場合にログへ記録し、
  4 MB を超える文書は入力中ではなく保存時に再検査する。

## データの由来（MIT OR Apache-2.0 と両立するもののみ）

| データ | 出典 | ライセンス |
|---|---|---|
| 英語修正表（組み込み） | typos-dict crate | MIT OR Apache-2.0 |
| en-base 単語リスト（レジストリ） | SCOWL size 60 以下 + 略語 | SCOWL permissive |
| ja-variants（レジストリ） | SudachiDict 同義語辞書のカタカナ表記ゆれペア（form=0、abbr=0、flag 2→0） | Apache-2.0 |
| ja-tech-variants、python/rust/web（レジストリ） | このリポジトリで手作業により整備 | MIT OR Apache-2.0 |
| code-terms（組み込み、辞書モードのみ） | このリポジトリで手作業により整備 | MIT OR Apache-2.0 |

ライセンス上の理由から、codespell 辞書と Wikipedia の誤字リスト
（CC BY-SA）、wordfreq データ（CC BY-SA）、SUBTLEX（非商用）は採用しない。

## 計測した性能（v0.1、Ryzen クラスのデスクトップ）

- 35 MB、40 万行のテキストファイル: **実時間 1.26 秒、最大 RSS 56 MB**。
  仕込んだ誤字 40 件をすべて検出した。単一ファイルのため単一スレッドでの結果。
- リポジトリ走査は `ignore` crate の並列 walker を使ってファイル単位に
  並列化し、gitignore も考慮する。

## ロードマップ

- **v0.2 — 候補順位とレベル**: SymSpell の MIT 頻度リストを使った候補順位、
  codespell 型の修正レベル（`clear`、`rare`、`informal`）、
  `ayame-spell trace <word>`（どのソースが、なぜ許可・指摘したか）。
- **v0.3 — prh 互換**: prh YAML の `expected`、`pattern`、`specs`、
  `wordBoundary` を読み込み、既存の日本語ルール資産を利用する。
  ですます調・である調の一貫性、ら抜き言葉の一覧、著作権の対象外である内閣告示
  に基づく送り仮名ペアも扱う。
- **v0.4 — 形態素解析（任意）**: cargo feature の背後で lindera + IPADIC を
  利用する。lindera は crates.io で公開された MIT ライセンスだが、
  sudachi.rs は git 参照のみのため依存関係には使えない。同音異義語の混同表を
  使った誤変換候補の検出が可能になる。
- **配布**: GitHub Releases のビルド済みバイナリ、プラットフォーム別サーバーを
  同梱した VS Code Marketplace 公開、cargo-binstall メタデータ、Homebrew/AUR。
- **エディターの拡充**: LSP はすでに Neovim、Helix、Zed で動作するため、
  各エディター向けの設定と文書を提供する。
