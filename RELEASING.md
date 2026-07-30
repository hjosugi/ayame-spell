# Releasing ayame-spell

This repository publishes the Rust CLI and platform-specific VS Code packages
from the `Release` GitHub Actions workflow.

## Before tagging

1. Move the release entries out of `Unreleased` in `CHANGELOG.md` and add a
   dated `## [X.Y.Z]` section.
2. Keep the workspace and VS Code extension versions in lockstep.
3. Run the same checks as CI, including the Rust 1.80 MSRV check.
4. Merge or push the release commit to `main`, wait for CI, then create and push
   `vX.Y.Z`.

The workflow extracts the matching CHANGELOG section as the GitHub Release
body. A missing or empty section fails before the release is created.

## Artifact guarantees

- Rust builds use the committed lockfile via `cargo build --locked`.
- Each CLI archive includes the binary, shell completions, manual page,
  `LICENSE-MIT`, `LICENSE-APACHE`, and `NOTICE.md`.
- Each archive is unpacked and smoke-tested on its target runner before upload.
  The test checks `--version` and a fixture that must report a known typo.
- `SHA256SUMS` covers every CLI archive and VSIX attached to the release.

## cargo-dist evaluation

Evaluated against dist 0.32 in July 2026. dist can generate multi-platform
archives and installers, include extra files, build extra artifacts, produce
checksums, and derive release notes from CHANGELOG files.

Adoption is deferred until the broader installer work in issue #31. The current
workflow also builds a target-specific VSIX containing the matching Rust server
binary, validates Cargo/npm version lockstep, optionally publishes each VSIX to
two extension registries, and supports repairing a single Intel macOS asset.
Replacing only the CLI archive matrix now would leave most release orchestration
custom while adding a generated workflow and another pinned release tool.

Re-evaluate dist when implementing installers or attestations. At that point,
prototype the VSIX packages as extra artifacts and verify that the repair flow
and registry publishing remain independently rerunnable.

References:

- https://axodotdev.github.io/cargo-dist/book/
- https://axodotdev.github.io/cargo-dist/book/reference/config.html
