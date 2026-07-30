# 🌸 ayame-spell

[English README](README.md) · [ドキュメント](https://hjosugi.github.io/ayame-spell/ja/) · [設計メモ](DESIGN.ja.md) · [コントリビューション](CONTRIBUTING.ja.md)

**コードと文章のための高速・低ノイズなスペルチェッカー(英語・日本語対応)。**
1つの設定ファイルで CLI(CI 用)・LSP サーバ・VS Code 拡張がすべて同じ結果を返します。

```console
$ ayame-spell
docs/design.md:4:3: サーバ → サーバー [ja-variant]
docs/design.md:5:1: ＡＢＣ１２３ → ABC123 [fullwidth-alnum]
src/main.rs:1:4: recieve → receive [typo]
src/main.rs:2:20: Nmae → Name [typo]
4 issue(s) in 2 file(s) — 214 file(s) checked
```

## なぜ作ったか

既存ツールはどれも一長一短でした:

- **typos / codespell**(既知タイポ表方式): 誤検出ほぼゼロで CI に最適。
  でも辞書の共有機構がなく、未知のタイポは見逃す。日本語非対応。
- **cSpell / Harper**(辞書方式): 網羅的だが `Kadane` や `heapq` のような
  識別子に延々と赤線が出る。単語追加は1個ずつ。cSpell は約 500KB で
  **黙って**チェックを打ち切る。日本語非対応。
- **textlint / prh**: 日本語校正の定番だが Node 製で重く、英語タイポは守備範囲外。

ayame-spell はこの3つの良いところを1つの Rust バイナリに統合しました。

- **corrections モード(既定)** — 既知のスペルミスだけを指摘
  ([typos-dict](https://github.com/crate-ci/typos) の約9.5万語テーブル)。
  設定ゼロで即 CI に入れられます。
- **dictionary モード(オプトイン)** — 加えてワードリストにない単語も検出。
  一括トリアージがあるので現実的に運用できます。
- **日本語** — 表記ゆれは既定で「文書内一貫性」方式:
  同じ文書に サーバ と サーバー が混在したときだけ少数派を指摘します。
  全角英数(１２３ＡＢＣ)・半角カナ(ｶﾀｶﾅ)・全角スペースも検出。
- **高速** — 35MB / 40万行のテキストを約1.3秒・ピーク56MBで全行チェック。
  スキップしたファイルは必ず件数報告します(黙って打ち切らない)。

## インストール

```sh
cargo install ayame-spell           # CLI + LSP サーバ
```

VS Code: **ayame-spell** 拡張をインストールしてください。
[GitHub Releases](https://github.com/hjosugi/ayame-spell/releases/latest) の
プラットフォーム別 VSIX にはネイティブサーバーが同梱されるため、Rust の
インストールは不要です。詳しくは[拡張ガイド](editors/vscode/README.md)を参照してください。

### シェル補完

bash、zsh、fish、PowerShell、Elvish 用の補完スクリプトを生成できます。

```sh
# bash (bash-completion)
mkdir -p ~/.local/share/bash-completion/completions
ayame-spell completions bash > ~/.local/share/bash-completion/completions/ayame-spell

# zsh (~/.zshrc で compinit より前に ~/.zfunc を fpath へ追加)
mkdir -p ~/.zfunc
ayame-spell completions zsh > ~/.zfunc/_ayame-spell
# ~/.zshrc:
fpath=(~/.zfunc $fpath)
autoload -Uz compinit && compinit

# fish
mkdir -p ~/.config/fish/completions
ayame-spell completions fish > ~/.config/fish/completions/ayame-spell.fish
```

PowerShell は次の行を `$PROFILE` に追加します。

```powershell
ayame-spell completions powershell | Out-String | Invoke-Expression
```

Elvish は次の行を `~/.config/elvish/rc.elv` に追加します。

```elvish
eval (ayame-spell completions elvish | slurp)
```

リリースアーカイブにも生成済みスクリプトを `completions/` として同梱します。

## クイックスタート

```sh
ayame-spell                  # カレントディレクトリをチェック(設定不要)
ayame-spell fix              # 安全な修正を一括適用
ayame-spell init             # ayame-spell.toml の雛形を生成

# 共有辞書つき dictionary モード:
ayame-spell dict add en-base python   # ダウンロード + 設定に自動追記
ayame-spell words collect             # 未知語を頻度順に一覧
ayame-spell words triage              # 対話式の一括トリアージ
```

**「いちいち単語を追加したくない」への答えが `words triage` です。**
フラグされた単語を複数選択して、プロジェクト辞書・グローバル辞書・
ignore リストへ一括で振り分けられます。VS Code では
**ayame-spell: Review Flagged Words** コマンドが同じ操作です。

## 設定

プロジェクトルートの `ayame-spell.toml`(または `.ayame-spell.toml`)。
ユーザー全体の設定 `~/.config/ayame-spell/config.toml` に上書きマージされます。
[完全な設定リファレンス](https://hjosugi.github.io/ayame-spell/ja/reference/configuration/)
に、全キー、既定値、マージ規則、上書きの優先順位を掲載しています。

```toml
[check]
mode = "corrections"     # "corrections" | "dictionary" | "off"

[files]
exclude = ["vendor/**"]  # .gitignore + ロックファイル等の既定除外に追加

[words]
project = "ayame-words.txt"     # チーム辞書(git にコミット)
ignore = ["exmaple"]            # どのモードでも指摘しない
dictionaries = ["registry:en-base"]

[corrections.words]
teh = "the"                     # インライン修正の追加
neet = "neet"                   # 自分自身への修正 = 許可リスト入り

[japanese]
katakana-style = "consistency"  # "consistency" | "long" | "short" | "off"
variant-files = ["registry:ja-tech-variants"]
fullwidth-space = "code"        # コードファイルのみ全角スペースを指摘
[japanese.variants]
"インタフェース" = "インターフェース"

[[overrides]]                   # glob 単位の上書き(後勝ち)
paths = ["docs/**"]
mode = "dictionary"
```

### 追加した単語はどこに入る?

| 操作 | ファイル | 共有範囲 |
|---|---|---|
| プロジェクト辞書に追加 | `ayame-words.txt`(コミット対象) | チーム全員 |
| グローバル辞書に追加 | `~/.config/ayame-spell/words.txt` | 自分の全プロジェクト |
| ignore | `ayame-spell.toml` の `[words].ignore` | チーム全員 |

照合は常に大文字小文字を無視し、修正は元のケースを保持します
(`Teh` → `The`、`TEH` → `THE`)。

### インラインディレクティブ

```text
ayame-spell:ignore-line        (行内のどこかに書く)
ayame-spell:ignore-next-line
ayame-spell:ignore-file        (ファイル内のどこかに書く)
```

## 共有辞書レジストリ

辞書は [GitHub Pages](https://hjosugi.github.io/ayame-spell/) から配信され、
ダウンロード時に sha256 検証、`~/.cache/ayame-spell/` にキャッシュされます。

```console
$ ayame-spell dict list
  en-base            en  wordlist    120531  英語ベース辞書 (SCOWL ≤60)
  python             en  wordlist       126  Python エコシステム用語
  rust               en  wordlist        81  Rust エコシステム用語
  web                en  wordlist       101  Web 開発用語
  ja-variants        ja  variants      3173  カタカナ表記ゆれ (SudachiDict 由来)
  ja-tech-variants   ja  variants        42  技術文書向けカタカナ規則(厳選)

$ ayame-spell dict add en-base    # DL + ayame-spell.toml へ自動追記
```

設定には `"registry:en-base"` と記録されるので、チームメイトは同じコマンドを
一度実行するだけ。社内レジストリを立てたい場合は `$AYAME_SPELL_REGISTRY` で
index.json の URL を差し替えられます。

## 日本語チェック詳細

| チェック | 例 | 既定 |
|---|---|---|
| 表記ゆれ(一貫性) | 文書内で サーバ / サーバー 混在 → 少数派を指摘 | on |
| 表記ゆれ(スタイル強制) | long(サーバー)/ short(サーバ)を強制 | opt-in |
| 変種ルール | インタフェース → インターフェース | 辞書で追加 |
| 全角英数 | １２３ＡＢＣ → 123ABC | on |
| 半角カナ | ﾃﾞｰﾀ → データ | on |
| 全角スペース | ソースコード中の U+3000 | on(コードのみ) |

JIS Z 8301:2019 で「3音以上は長音記号を省く」規則が廃止された経緯を踏まえ、
既定ではどちらの方向も強制しません(一貫性のみ確認)。

## 終了コードと出力形式

`0` 問題なし · `1` 指摘あり · `2` エラー。
`--format human`（既定）、`--format brief`、`--format json`
（JSON Lines）を利用できます。

## ライセンス

MIT OR Apache-2.0。同梱・派生データの出典は [NOTICE.md](NOTICE.md) を参照
(typos-dict / SCOWL / SudachiDict 同義語辞書)。
