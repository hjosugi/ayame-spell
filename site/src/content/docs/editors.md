---
title: Editor setup
description: Configure the ayame-spell LSP server in VS Code, Neovim, Helix, Zed, Emacs, Sublime Text, and JetBrains IDEs.
---

All integrations run the same server:

```sh
ayame-spell lsp --stdio
```

The `--stdio` flag is accepted for clients that add it by convention. The
server always communicates over standard input and output. Ensure the binary is
on the editor process's `PATH`, then open a folder containing
`ayame-spell.toml` or a Git repository.

## VS Code

Install a platform-specific VSIX from
[GitHub Releases](https://github.com/hjosugi/ayame-spell/releases/latest). It
includes the native server, so a separate Cargo installation is unnecessary.

1. Download the VSIX for your platform.
2. Run **Extensions: Install from VSIX...**.
3. Open a project folder.

Use **ayame-spell: Review Flagged Words** for bulk triage. See the
[extension README](https://github.com/hjosugi/ayame-spell/tree/main/editors/vscode)
for server-path and initialization-option settings.

## Neovim 0.11+

Add this to `init.lua`. Extend `filetypes` for the buffers you want checked.
This uses Neovim's current `vim.lsp.config` API.

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

Open a matching file and run `:checkhealth vim.lsp`. Diagnostics work without
custom keymaps; `vim.lsp.buf.code_action()` exposes replacement and dictionary
actions.

For legacy `nvim-lspconfig` configurations:

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

The legacy `require("lspconfig")` API is deprecated; prefer the first snippet
on Neovim 0.11+. See the
[Neovim LSP documentation](https://neovim.io/doc/user/lsp.html).

## Helix

Create `.helix/languages.toml` in the project, or edit the user
`languages.toml`:

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

Repeat the `[[language]]` block for other built-in Helix language names. If a
language already has a server, keep it in the array as shown. Run
`hx --health markdown`, then use `:lsp-restart` after configuration changes.
See the [Helix language server configuration](https://docs.helix-editor.com/master/languages.html).

## Zed

Zed requires a language-server adapter supplied by an extension; arbitrary
server commands cannot be registered solely in `settings.json`. Until the
ayame-spell extension is published, install the development extension from
`editors/zed` in this repository:

1. Clone this repository and open Zed.
2. Run **zed: install dev extension**.
3. Select the repository's `editors/zed` directory.
4. Add the adapter to the desired language in `settings.json`:

```json
{
  "languages": {
    "Markdown": {
      "language_servers": ["ayame-spell", "..."]
    }
  }
}
```

The adapter launches `ayame-spell lsp --stdio` from `PATH`. See Zed's
[language extension guide](https://zed.dev/docs/extensions/languages).

## Emacs with Eglot

Add a server mapping, then run `M-x eglot` in a supported buffer:

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

Add programming modes to the mapping if another LSP client is not already
responsible for them. Use `M-x eglot-code-actions` for fixes. See the
[Eglot server setup manual](https://www.gnu.org/software/emacs/manual/html_node/eglot/Setting-Up-LSP-Servers.html).

## Emacs with lsp-mode

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

See the [lsp-mode client registration guide](https://emacs-lsp.github.io/lsp-mode/page/adding-new-language/).

## Sublime Text

Install the **LSP** package, then open **Preferences → Package Settings → LSP
→ Server Configurations** and add:

```json
{
  "ayame-spell": {
    "enabled": true,
    "command": ["ayame-spell", "lsp", "--stdio"],
    "selector": "text, source"
  }
}
```

Narrow the selector if you do not want every text and source buffer checked.
Use **Tools → Developer → Show Scope** to find a syntax's base scope. The
configuration uses the default stdio transport described in the
[Sublime LSP client documentation](https://lsp.sublimetext.io/client_configuration/).

## JetBrains IDEs

The most direct setup is the free
[LSP4IJ plugin](https://plugins.jetbrains.com/plugin/23257-lsp4ij), which
supports user-defined stdio servers without writing an IDE plugin.

1. Install LSP4IJ and restart the IDE.
2. Open **Settings → Languages & Frameworks → Language Servers**.
3. Add a user-defined server named `ayame-spell`.
4. Set **Command** to `ayame-spell lsp --stdio`.
5. In **Mappings**, add the languages, file types, or patterns to check, such
   as Markdown, `*.txt`, Rust, and Python.
6. Apply the settings and inspect **LSP Consoles** if the server does not start.

JetBrains' native LSP API is intended for IDE plugin authors. LSP4IJ is the
generic user-facing route across IntelliJ-based products.

## Client feature differences

Diagnostics and replacement code actions use standard LSP methods. Clients
that do not advertise `workspace/executeCommand` still receive diagnostics and
direct replacement edits; dictionary-management actions that require a
command may be unavailable. Run `ayame-spell words add` or `words triage` from
a terminal in that case.
