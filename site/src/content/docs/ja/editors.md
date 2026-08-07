---
title: エディター設定
description: VS Code、Neovim、Helix、Zed、Emacs、Sublime Text、JetBrains IDE で ayame-spell の LSP サーバーを設定します。
---

すべての連携が同じサーバーを起動します。

```sh
ayame-spell lsp --stdio
```

慣例でオプションを追加するクライアント向けに `--stdio` を受け付けます。サーバー
は常に標準入出力で通信します。エディタープロセスの `PATH` からバイナリを
見つけられる状態にし、`ayame-spell.toml` または Git リポジトリを含む
フォルダーを開いてください。

## VS Code

[GitHub Releases](https://github.com/ayame-editor/ayame-spell/releases/latest) から
環境に合う VSIX をインストールします。ネイティブサーバーを同梱しているため、
Cargo による別インストールは不要です。

1. 対応する VSIX をダウンロードする。
2. **Extensions: Install from VSIX...** を実行する。
3. プロジェクトフォルダーを開く。

一括整理には **ayame-spell: Review Flagged Words** を使います。サーバーパスや
初期化オプションは
[拡張 README](https://github.com/ayame-editor/ayame-spell/tree/main/editors/vscode)
を参照してください。

## Neovim 0.11 以降

`init.lua` に次を追加します。対象バッファーに合わせて `filetypes` を増やして
ください。現在の `vim.lsp.config` API を使う設定です。

```lua
vim.lsp.config("ayame_spell", {
  cmd = { "ayame-spell", "lsp", "--stdio" },
  filetypes = {
    "markdown", "text", "gitcommit",
    "rust", "python", "javascript", "typescript",
  },
  root_markers = { "ayame-spell.toml", ".ayame-spell.toml", ".git" },
})

vim.lsp.enable("ayame_spell")
```

対象ファイルを開き、`:checkhealth vim.lsp` を実行します。診断表示に独自
キーマップは不要です。`vim.lsp.buf.code_action()` から置換や辞書操作を
選べます。

従来の `nvim-lspconfig` 設定を使う場合は次のとおりです。

```lua
local configs = require("lspconfig.configs")
local util = require("lspconfig.util")

if not configs.ayame_spell then
  configs.ayame_spell = {
    default_config = {
      cmd = { "ayame-spell", "lsp", "--stdio" },
      filetypes = { "markdown", "text", "gitcommit" },
      root_dir = util.root_pattern("ayame-spell.toml", ".ayame-spell.toml", ".git"),
    },
  }
end

require("lspconfig").ayame_spell.setup({})
```

従来の `require("lspconfig")` API は非推奨です。Neovim 0.11 以降では最初の
設定を使ってください。詳しくは
[Neovim LSP ドキュメント](https://neovim.io/doc/user/lsp.html)を参照してください。

## Helix

プロジェクトの `.helix/languages.toml` またはユーザー用 `languages.toml`
を作成します。

```toml
[language-server.ayame-spell]
command = "ayame-spell"
args = ["lsp", "--stdio"]

[[language]]
name = "markdown"
language-servers = ["ayame-spell"]

[[language]]
name = "rust"
language-servers = ["rust-analyzer", "ayame-spell"]

[[language]]
name = "python"
language-servers = ["pyright", "ayame-spell"]
```

ほかの言語は Helix 組み込みの言語名で `[[language]]` を追加します。既存の
言語サーバーがある場合は、例のように配列へ残してください。
`hx --health markdown` で確認し、設定変更後は `:lsp-restart` を実行します。
[Helix の言語サーバー設定](https://docs.helix-editor.com/master/languages.html)
も参照してください。

## Zed

Zed で言語サーバーを登録するには拡張のアダプターが必要です。任意のサーバー
コマンドを `settings.json` だけで登録することはできません。ayame-spell 拡張
が公開されるまでは、このリポジトリの `editors/zed` を開発拡張として導入します。

1. このリポジトリをクローンし、Zed を開く。
2. **zed: install dev extension** を実行する。
3. リポジトリ内の `editors/zed` を選ぶ。
4. `settings.json` で対象言語へアダプターを追加する。

```json
{
  "languages": {
    "Markdown": {
      "language_servers": ["ayame-spell", "..."]
    }
  }
}
```

アダプターは `PATH` 上の `ayame-spell lsp --stdio` を起動します。
[Zed の言語拡張ガイド](https://zed.dev/docs/extensions/languages)も参照してください。

## Emacs と Eglot

サーバー対応を追加し、対象バッファーで `M-x eglot` を実行します。

```elisp
(with-eval-after-load 'eglot
  (add-to-list
   'eglot-server-programs
   '(((markdown-mode :language-id "markdown")
      (gfm-mode :language-id "markdown")
      (text-mode :language-id "plaintext"))
     . ("ayame-spell" "lsp" "--stdio"))))

(add-hook 'markdown-mode-hook #'eglot-ensure)
```

ほかの LSP クライアントが担当していないプログラミングモードは、対応表へ追加
できます。修正には `M-x eglot-code-actions` を使います。
[Eglot のサーバー設定マニュアル](https://www.gnu.org/software/emacs/manual/html_node/eglot/Setting-Up-LSP-Servers.html)
も参照してください。

## Emacs と lsp-mode

```elisp
(with-eval-after-load 'lsp-mode
  (add-to-list 'lsp-language-id-configuration
               '(markdown-mode . "markdown"))
  (add-to-list 'lsp-language-id-configuration
               '(gfm-mode . "markdown"))
  (lsp-register-client
   (make-lsp-client
    :new-connection
    (lsp-stdio-connection '("ayame-spell" "lsp" "--stdio"))
    :activation-fn (lsp-activate-on "markdown" "plaintext")
    :multi-root t
    :server-id 'ayame-spell)))

(add-hook 'markdown-mode-hook #'lsp-deferred)
```

[lsp-mode のクライアント登録ガイド](https://emacs-lsp.github.io/lsp-mode/page/adding-new-language/)
も参照してください。

## Sublime Text

**LSP** パッケージを導入し、**Preferences → Package Settings → LSP →
Server Configurations** を開いて次を追加します。

```json
{
  "ayame-spell": {
    "enabled": true,
    "command": ["ayame-spell", "lsp", "--stdio"],
    "selector": "text, source"
  }
}
```

すべての文章・ソースバッファーを対象にしない場合は、セレクターを絞ります。
**Tools → Developer → Show Scope** で構文の基本スコープを確認できます。この
設定は
[Sublime LSP クライアント文書](https://lsp.sublimetext.io/client_configuration/)
にある標準入出力通信を使います。

## JetBrains IDE

IDE プラグインを作らずに任意の標準入出力サーバーを登録できる、無料の
[LSP4IJ プラグイン](https://plugins.jetbrains.com/plugin/23257-lsp4ij)
を使うのが直接的です。

1. LSP4IJ を導入して IDE を再起動する。
2. **Settings → Languages & Frameworks → Language Servers** を開く。
3. `ayame-spell` というユーザー定義サーバーを追加する。
4. **Command** を `ayame-spell lsp --stdio` にする。
5. **Mappings** で Markdown、`*.txt`、Rust、Python などの対象を追加する。
6. 設定を適用する。起動しない場合は **LSP Consoles** を確認する。

JetBrains 標準の LSP API は IDE プラグイン開発者向けです。IntelliJ 系製品を
利用者側で設定する場合は LSP4IJ が汎用的な方法です。

## クライアントごとの機能差

サーバーは push / pull 診断、差分同期、ローカライズしたルール hover、
quick fix、`source.fixAll.ayame-spell` に対応します。変更後は既定で 150 ms
debounce し、大きすぎる文書はエディターを止めずに警告します。
診断と置換コードアクションは標準 LSP メソッドを使います。
`workspace/executeCommand` を通知しないクライアントでも、診断と直接置換は
利用できます。コマンドが必要な辞書操作は表示されない場合があるため、その場合は
ターミナルで `ayame-spell words add` または `words triage` を実行してください。
