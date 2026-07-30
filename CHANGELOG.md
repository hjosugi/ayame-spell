# Changelog

All notable changes to ayame-spell are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and the project uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[Unreleased]: https://github.com/hjosugi/ayame-spell/compare/v0.3.1...HEAD
[0.3.1]: https://github.com/hjosugi/ayame-spell/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/hjosugi/ayame-spell/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/hjosugi/ayame-spell/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/hjosugi/ayame-spell/releases/tag/v0.1.0
