# Marketplace listing kit

This file is the canonical copy-and-asset handoff for the VS Code Marketplace
and Open VSX listings.

## Identity

| Field                 | Value                                           |
| --------------------- | ----------------------------------------------- |
| Extension ID          | `hjosugi.ayame-spell`                           |
| Package name          | `ayame-spell`                                   |
| Display name          | ayame-spell — English & Japanese Spell Checker  |
| Publisher / namespace | `hjosugi`                                       |
| Categories            | Linters, Programming Languages, Other           |
| License               | MIT OR Apache-2.0                               |
| Support               | <https://github.com/hjosugi/ayame-spell/issues> |
| Homepage              | <https://hjosugi.github.io/ayame-spell/>        |

## Short description

Fast, low-noise spell checking for English and Japanese, powered by a bundled
Rust language server.

## Listing introduction

ayame-spell catches high-confidence English misspellings and Japanese notation
inconsistencies without turning project identifiers into noise. Start in quiet
corrections mode, opt into full dictionary checking where it helps, and manage
team or personal words in bulk. Every supported desktop package includes the
native language server.

## Search terms

`spell`, `spellcheck`, `typo`, `linter`, `japanese`, `校正`, `表記ゆれ`

## Asset inventory

| Asset                 | File                          | Use                          |
| --------------------- | ----------------------------- | ---------------------------- |
| Store icon            | `icon.png`                    | 128×128 Marketplace icon     |
| Editable icon source  | `media/icon-source.png`       | Future crops and exports     |
| Animated overview     | `media/ayame-spell-demo.gif`  | README hero                  |
| English correction    | `media/corrections.png`       | README and walkthrough       |
| Japanese checks       | `media/japanese-variants.png` | README and walkthrough       |
| Bulk review           | `media/review-words.png`      | README and walkthrough       |
| Deterministic sources | `media/source/*.svg`          | Regenerate exact screenshots |

All published images are PNG or GIF. The SVG sources are excluded from VSIX
packages because the Marketplace rejects user-provided SVG content.

## Gallery presentation

- Banner color: `#1f2459`
- Banner theme: dark
- Q&A: Marketplace
- Untrusted workspace support: limited
- Virtual workspace support: unavailable
- Extension kind: workspace

## Release announcement copy

> ayame-spell 0.3.1 completes the Japanese design notes and adds CI safeguards
> that keep English and Japanese project documentation in sync. Platform-matched
> packages continue to bundle the Rust language server for Windows x64, Linux
> x64/ARM64, and macOS Intel/Apple silicon.

## Listing QA

- Confirm each store shows the icon, banner, localized title, README images,
  changelog, repository, license, support URL, and five platform packages.
- Install one package from each platform family and confirm the server starts
  without `cargo install`.
- Confirm the walkthrough renders all three PNG assets.
- Confirm Marketplace and Open VSX versions match the Git tag, Cargo workspace,
  and extension manifest.
- Confirm no token, local path, source map, test fixture, SVG, or extra platform
  binary appears in a VSIX.
