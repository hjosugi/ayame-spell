# ayame-spell for VS Code

Fast, low-noise spell checking for code and prose — English & Japanese.

- **Typo corrections** (typos-style): only *known* misspellings are flagged,
  so identifiers like `Kadane`, `heapq`, or `subarray` never light up.
- **Dictionary mode** (opt-in): unknown-word detection with shared wordlists.
- **Japanese**: 表記ゆれ detection (サーバ/サーバー mixing), fullwidth
  alphanumerics, halfwidth katakana, fullwidth spaces.
- **Bulk workflows**: quick fixes to add words to the project/global
  dictionary, and *ayame-spell: Review Flagged Words* to multi-select many
  words and add/ignore them all at once.
- **Fast**: Rust LSP server; multi-megabyte files are fine.

## Requirements

The extension needs the `ayame-spell` binary: bundled builds are used when
present; otherwise install it and make sure it is on your `PATH`:

```sh
cargo install ayame-spell
```

or point `ayame-spell.serverPath` at the binary.

## Configuration

Project settings live in `ayame-spell.toml` at the repository root — shared
with the CLI and CI. See the
[project README](https://github.com/hjosugi/ayame-spell) for the full
reference.
