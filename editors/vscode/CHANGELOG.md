# Changelog

All notable VS Code extension changes are documented here.

## 0.3.1

- Add the complete Japanese design notes and cross-link the English and Japanese
  project documentation.
- Enforce matching EN/JA page sets, heading structures, and changed-file pairs
  in CI, with an explicit documented escape hatch for language-neutral edits.
- Keep the generated English and Japanese CLI overviews structurally aligned.

## 0.3.0

- Add a complete English/Japanese documentation site with search, language
  routing, generated CLI reference, and editor setup guides.
- Accept the conventional `ayame-spell lsp --stdio` invocation used by generic
  language-server clients.
- Document Neovim, Helix, Zed, Emacs, Sublime Text, and JetBrains setup, and
  include a development Zed extension adapter.

## 0.2.0

- Ship platform-specific VSIX packages with the ayame-spell server for Windows
  x64, Linux x64/ARM64, and macOS Intel/Apple silicon.
- Add status-bar mode and finding count, with one-click mode switching.
- Add workspace overrides for mode, Japanese checks, severity, server tracing,
  and fix-on-save.
- Add bulk review, selection-to-dictionary, shared dictionary installation,
  configuration creation, server restart, and safe fix-all commands.
- Add a dedicated log output channel, reactive configuration reload, actionable
  startup errors, and extension/server compatibility reporting.
- Add English and Japanese manifest localization, a getting-started walkthrough,
  Marketplace icon and screenshots, and complete publishing docs.
- Add linting, formatting, platform unit tests, VS Code-hosted integration
  tests, package validation, and automated multi-platform release packaging.

## 0.1.0

- Initial language client with diagnostics, quick fixes, project/global word
  additions, ignore actions, bulk word review, and CLI fallback.
