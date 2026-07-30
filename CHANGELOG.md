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

### Changed

- Pinned dependencies that had raised their compiler requirement so the
  declared Rust 1.80 MSRV remains buildable.

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
