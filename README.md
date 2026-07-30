# 🌸 ayame-spell

[日本語 README](README.ja.md) · [Documentation](https://hjosugi.github.io/ayame-spell/) · [Design notes](DESIGN.md) · [Contributing](CONTRIBUTING.md)

[![CI](https://github.com/hjosugi/ayame-spell/actions/workflows/ci.yml/badge.svg)](https://github.com/hjosugi/ayame-spell/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/ayame-spell.svg)](https://crates.io/crates/ayame-spell)
[![docs.rs](https://img.shields.io/docsrs/ayame-spell)](https://docs.rs/ayame-spell)
[![VS Marketplace](https://img.shields.io/visual-studio-marketplace/v/hjosugi.ayame-spell)](https://marketplace.visualstudio.com/items?itemName=hjosugi.ayame-spell)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)

**Fast, low-noise spell checker for code and prose — English & Japanese.**
One config file drives the CLI (CI), the LSP server, and the VS Code
extension.

```console
$ ayame-spell
docs/design.md:4:3: サーバ → サーバー [ja-variant]
docs/design.md:5:1: ＡＢＣ１２３ → ABC123 [fullwidth-alnum]
src/main.rs:1:4: recieve → receive [typo]
src/main.rs:2:20: Nmae → Name [typo]
4 issue(s) in 2 file(s) — 214 file(s) checked
```

## Why another spell checker?

Every existing tool picks one side of a trade-off:

| | corrections-table tools<br>(typos, codespell) | dictionary tools<br>(cSpell, Harper) |
|---|---|---|
| False positives | ~zero | constant (`Kadane`, `heapq`, …) |
| False negatives | misses unknown typos | ~zero |
| Best at | CI | interactive editing |
| Shared dictionaries | ✗ | ✓ (cSpell) |
| Bulk add/ignore | ✗ | weak (one word at a time) |
| Huge files | fast | cSpell silently stops at ~500 KB |
| Japanese | ✗ | ✗ |

ayame-spell unifies both modes behind one config, adds first-class bulk
workflows, a shared dictionary registry, and Japanese notation checking —
in a single fast Rust binary.

- **corrections mode (default)** — only *known* misspellings are flagged
  (built on the 95k-entry [typos-dict] table). Safe for CI, zero setup.
- **dictionary mode (opt-in)** — additionally flags words not in the active
  wordlists, with Levenshtein suggestions. Made livable by bulk triage.
- **Japanese** — 表記ゆれ detection that is quiet by default: it only
  reports when one document mixes two spellings of the same katakana word
  (サーバ vs サーバー). Plus fullwidth-alphanumeric, halfwidth-katakana,
  and fullwidth-space checks.
- **Fast** — 35 MB / 400k lines checked in ~1.3 s with 56 MB peak RSS.
  No silent truncation, ever: skipped files are counted and reported.

[typos-dict]: https://github.com/crate-ci/typos

## Install

```sh
cargo install ayame-spell           # CLI + LSP server
```

VS Code: install the **ayame-spell** extension. Platform-specific VSIX builds
from [GitHub Releases](https://github.com/hjosugi/ayame-spell/releases/latest)
include the native server, so no Rust installation is required. See the
[extension guide](editors/vscode/README.md).

### Shell completions

Generate a completion script for bash, zsh, fish, PowerShell, or Elvish:

```sh
# bash (bash-completion)
mkdir -p ~/.local/share/bash-completion/completions
ayame-spell completions bash > ~/.local/share/bash-completion/completions/ayame-spell

# zsh (add ~/.zfunc to fpath before running compinit in ~/.zshrc)
mkdir -p ~/.zfunc
ayame-spell completions zsh > ~/.zfunc/_ayame-spell
# ~/.zshrc:
fpath=(~/.zfunc $fpath)
autoload -Uz compinit && compinit

# fish
mkdir -p ~/.config/fish/completions
ayame-spell completions fish > ~/.config/fish/completions/ayame-spell.fish
```

For PowerShell, add the following line to `$PROFILE`:

```powershell
ayame-spell completions powershell | Out-String | Invoke-Expression
```

For Elvish, add the following line to `~/.config/elvish/rc.elv`:

```elvish
eval (ayame-spell completions elvish | slurp)
```

Release archives also contain pre-generated scripts in `completions/`.

## Quick start

```sh
ayame-spell                  # check current directory, zero config
ayame-spell fix              # apply safe fixes in place
ayame-spell init             # write a starter ayame-spell.toml

# dictionary mode with shared wordlists:
ayame-spell dict add en-base python   # download + enable in config
ayame-spell words collect             # unknown words, ranked by frequency
ayame-spell words triage              # interactive bulk add/ignore
```

`ayame-spell words triage` is the answer to "I don't want to add words one
by one": multi-select any number of flagged words and send them to the
project dictionary, your global dictionary, or the ignore list in one pass.
In VS Code the same flow is **ayame-spell: Review Flagged Words**.

## Configuration

`ayame-spell.toml` (or `.ayame-spell.toml`) at the project root, merged
over the per-user global config `~/.config/ayame-spell/config.toml`.
All keys are optional; the defaults are what you saw above.
See the [complete configuration reference](https://hjosugi.github.io/ayame-spell/reference/configuration/)
for every key, default, merge rule, and override precedence rule.

```toml
[check]
mode = "corrections"     # "corrections" | "dictionary" | "off"
min-word-len = 3
max-token-len = 40       # longer digit-bearing tokens = hashes, skipped

[files]
exclude = ["vendor/**"]  # on top of .gitignore + built-in lock-file excludes
include-hidden = false
max-file-size = 0        # bytes; 0 = unlimited (skips are always reported)

[words]
project = "ayame-words.txt"     # team dictionary, committed to git
ignore = ["exmaple"]            # never flagged, any mode
dictionaries = ["registry:en-base", "./team-words.txt"]

[corrections]
builtin = true                  # 95k-entry English table (typos-dict)
extra = ["registry:my-corrections", "./fixes.tsv"]  # typo<TAB>fix[,fix]
[corrections.words]
teh = "the"                     # inline corrections
neet = "neet"                   # fix == typo → whitelist

[japanese]
enabled = true
katakana-style = "consistency"  # "consistency" | "long" | "short" | "off"
variant-files = ["registry:ja-tech-variants"]
flag-fullwidth-alnum = true     # １２３ＡＢＣ → 123ABC
flag-halfwidth-kana = true      # ｶﾀｶﾅ → カタカナ
fullwidth-space = "code"        # "code" | "always" | "never"
[japanese.variants]
"インタフェース" = "インターフェース"

[[overrides]]                   # per-glob settings; later entries win
paths = ["docs/**"]
mode = "dictionary"
```

### Where do added words go?

| Action | File | Shared with |
|---|---|---|
| Add to **project** words | `ayame-words.txt` (committed) | your team |
| Add to **global** words | `~/.config/ayame-spell/words.txt` | all your projects |
| **Ignore** | `[words].ignore` in `ayame-spell.toml` | your team |

One rule everywhere: matching is case-insensitive, fixes preserve case
(`Teh` → `The`, `TEH` → `THE`).

### Inline directives

```text
ayame-spell:ignore-line        (anywhere in the line)
ayame-spell:ignore-next-line
ayame-spell:ignore-file        (anywhere in the file)
```

## Dictionary registry

Shared dictionaries are plain text files served from
[GitHub Pages](https://hjosugi.github.io/ayame-spell/), sha256-verified on
download and cached in `~/.cache/ayame-spell/`:

```console
$ ayame-spell dict list
  en-base            en  wordlist    120531  English base wordlist (SCOWL ≤60)
  python             en  wordlist       126  Python ecosystem terms
  rust               en  wordlist        81  Rust ecosystem terms
  web                en  wordlist       101  Web development terms
  ja-variants        ja  variants      3173  Katakana notation variants (SudachiDict)
  ja-tech-variants   ja  variants        42  Curated tech-writing katakana rules

$ ayame-spell dict add en-base    # downloads + wires into ayame-spell.toml
```

Because the config records `"registry:en-base"`, teammates just run
`ayame-spell dict add en-base` once — or you can vendor the file and point
at a path instead. Registry URL is overridable via `$AYAME_SPELL_REGISTRY`
(point it at your company's own index.json).

## Japanese checks in detail

| Check | Example | Default |
|---|---|---|
| 表記ゆれ (consistency) | doc mixes サーバ & サーバー → flag minority | on |
| 表記ゆれ (style) | enforce long (サーバー) or short (サーバ) | opt-in |
| variant rules | インタフェース → インターフェース | via dictionaries |
| fullwidth alnum | １２３ＡＢＣ → 123ABC | on |
| halfwidth kana | ﾃﾞｰﾀ → データ | on |
| fullwidth space | U+3000 in source code | on (code files only) |

The consistency default follows JIS Z 8301:2019 (which abolished the old
"omit the long vowel" rule): ayame-spell does not impose a direction unless
you ask for one.

## Exit codes & output formats

`0` clean · `1` issues found · `2` error.
`--format human` (default), `--format brief`, `--format json` (JSON lines).

## License

MIT OR Apache-2.0. Bundled/derived data: see [NOTICE.md](NOTICE.md)
(typos-dict, SCOWL, SudachiDict synonym dictionary).
