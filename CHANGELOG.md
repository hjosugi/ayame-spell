# Changelog

All notable changes to ayame-spell are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and the project uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.0] - 2026-08-07

### Added

- End-to-end dictionary quality fixtures covering every shipped English
  wordlist, clean prose, built-in and registry corrections, and unknown words.
- Peak-RSS regression enforcement alongside default and full-dictionary
  throughput comparison on pull requests and non-initial pushes to `main`.
- AUR `.SRCINFO` generation and aarch64 support for `ayame-spell-bin`.
- npm trusted publishing through GitHub Actions OIDC.

### Changed

- **Breaking:** the minimum supported Rust version is now 1.91, up from 1.80.
  The refreshed dependency set requires it: `typos-dict` and `dictgen` declare
  `rust-version = "1.91"`, and `clap_lex` needs the edition-2024 manifest
  format. Installing from crates.io, the `rust:` CI images, and the documented
  MSRV all move together.
- Refreshed 19 Rust dependencies, including `toml` 0.8 to 1.1, `ureq` 2 to 3,
  `lsp-types` 0.95 to 0.97, `sha2` 0.10 to 0.11, and `criterion` 0.5 to 0.8.
- Japanese number consistency now scans text as a stream instead of
  materializing every character and byte offset. On the reproducible 35 MiB
  corpus this reduced median wall time from 1.598 s to 0.817 s and peak RSS
  from 679.6 MiB to 121.1 MiB.
- VS Code store publication verifies and publishes the complete five-platform
  VSIX set.

### Fixed

- Removed a duplicate `postgres` entry from the bundled code-term list and
  added a normalization/uniqueness test.
- Variant rule files written by `ayame-spell import prh` load again. `toml` 1.x
  parses a bare value rather than a document through `FromStr`, which made
  every `[[rules]]` file fail to parse; the loader now parses a document and a
  round-trip test covers it.

## [0.4.0] - 2026-07-30

### Added

- Versioned JSON Lines issue and summary records, a documented additive
  stability policy, and a published machine-readable output schema.
- Contributor, security, conduct, issue-reporting, and pull-request guidance.
- Dependabot plus CI checks for Rust 1.80, rustdoc warnings, dependency
  advisories, source provenance, and licenses.
- A generated manual page in release archives and packaged-binary smoke tests
  on every release target.
- Cross-platform CLI integration tests covering every command, stable output,
  configuration discovery, fixes, directives, and an offline dictionary
  registry.
- Interactive dictionary discovery with filtered selection, `dict search`,
  `dict info`, and JSON registry listings.
- A guided `init --interactive` wizard with project detection, configuration
  preview, dictionary setup, and a ready-to-copy CI snippet.
- Reproducible per-run CLI overrides for configuration, mode, excludes,
  ignore handling, hidden files, colour, verbosity, standard input, file size,
  and worker threads.
- Cache-only dynamic completion for registry dictionaries, collected words,
  and word-list paths in Bash, Zsh, Fish, PowerShell, and Elvish.
- `fix --dry-run` unified-diff previews and `fix --interactive` per-finding
  review, including multi-candidate selection and dictionary additions.
- Native GitHub workflow annotations, SARIF 2.1.0 output with rule metadata,
  and automatic GitHub Actions format selection.
- Bilingual `explain` and `rules` commands backed by the same rule metadata as
  generated documentation and machine-readable reports.
- LSP pull and workspace diagnostics, incremental document sync, configurable
  debouncing, cancellation, and a visible large-document guard.
- Localized rule hovers plus quick fixes for global/project words, inline
  ignores, configured corrections, document-wide Japanese normalization, and
  `source.fixAll.ayame-spell`.
- Content-based baselines for adopting repositories incrementally, including
  full-audit and stale-entry pruning workflows.
- Searchable, paged, per-word triage with finding filters, bulk occurrence
  fixes, non-interactive safeguards, and exact changed-file summaries.
- A versioned configuration JSON Schema plus `config --schema` and
  suggestion-aware `config --validate` editor workflows.
- A composite GitHub Action with annotations and optional SARIF upload,
  pre-commit check/fix hooks, and portable GitLab and CircleCI recipes.
- Reproducible registry resolution through `ayame-spell.lock`, explicit
  `name@version` pins, checksum verification, update-only checks, and
  project-local vendoring.
- Fifteen independently curated technology, business, finance, correction,
  and Japanese-variant dictionaries, with provenance and license metadata.
- Per-file incremental scan caching with content/configuration/dictionary
  invalidation, CI opt-in paths, cold-run controls, and stable output.
- Automated, dry-run-first migration from cSpell, typos, and a documented
  Rust-regex-compatible subset of prh, including untranslated-setting reports.
- Syntax-aware Markdown and source-code profiles with per-glob overrides.
- Explicit `en-US` / `en-GB` policy checks plus compound, possessive, and
  acronym-aware English tokenization.
- Japanese compatibility-character, kanji/okurigana, number/unit, and
  punctuation consistency checks plus regex-based project rules.
- Reproducible Criterion and end-to-end benchmarks, checked reference results,
  and a pull-request throughput regression guard.
- Fuzz targets for tokenization, Japanese checking, width conversion, and
  configuration parsing plus UTF-8 and fix-idempotence property tests.
- A branded favicon and social preview card, localized Open Graph/Twitter
  metadata, and a reproducible terminal demo of triage and fix workflows.
- Checksum-verifying shell and PowerShell installers, generated Homebrew,
  Scoop, and AUR manifests, a native npm wrapper, and multi-architecture GHCR
  images.

### Changed

- Pinned dependencies that had raised their compiler requirement so the
  declared Rust 1.80 MSRV remains buildable.
- Release builds now use `Cargo.lock`, and GitHub Release notes come from the
  matching version section in this changelog.
- Global configuration and cache locations can be isolated with
  `AYAME_SPELL_CONFIG_DIR` and `AYAME_SPELL_CACHE_DIR`.
- Registry indexes are cached for 24 hours and remain available offline;
  completion never performs network I/O.
- The dictionary registry index is now versioned and retains immutable
  historical releases; registry generation validates versions, formats,
  duplicates, base-list overlap, provenance, and licenses.
- Fixes abort instead of overwriting a file that changed after it was scanned.

## [0.3.1] - 2026-07-30

### Added

- Complete Japanese design notes and English/Japanese contributing guides.
- CI safeguards for localized page sets, heading structures, links, generated
  CLI documentation, and changed documentation pairs.

## [0.3.0] - 2026-07-29

### Added

- English and Japanese documentation site with search, language routing,
  editor setup guides, and generated CLI references.
- Generic `ayame-spell lsp --stdio` compatibility and a development Zed
  extension adapter.
- Setup documentation for Neovim, Helix, Zed, Emacs, Sublime Text, and
  JetBrains IDEs.

## [0.2.0] - 2026-07-26

### Added

- Platform-specific CLI archives and VSIX packages for Windows x64, Linux
  x64/ARM64, and macOS Intel/Apple silicon.
- VS Code mode and finding status, workspace settings, fix-on-save, bulk word
  review, dictionary installation, configuration, restart, and logging tools.
- English and Japanese extension localization, walkthrough, artwork,
  integration tests, package validation, and multi-platform release builds.
- Shell completions plus license and notice files in CLI archives.

## [0.1.0] - 2026-07-26

### Added

- Initial Rust CLI, spell-checking engine, language server, dictionary
  registry, Japanese notation checks, and VS Code language client.

[Unreleased]: https://github.com/hjosugi/ayame-spell/compare/v0.5.0...HEAD
[0.5.0]: https://github.com/hjosugi/ayame-spell/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/hjosugi/ayame-spell/compare/v0.3.1...v0.4.0
[0.3.1]: https://github.com/hjosugi/ayame-spell/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/hjosugi/ayame-spell/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/hjosugi/ayame-spell/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/hjosugi/ayame-spell/releases/tag/v0.1.0
