# Contributing to ayame-spell

[日本語](CONTRIBUTING.ja.md)

Thank you for helping make ayame-spell faster, quieter, and easier to use.
Please open an issue before a large behavior or format change so the design can
be agreed before implementation.

## Development setup

Install stable Rust and Node.js 24. Clone the repository, then run:

```sh
cargo build --workspace
cargo test --workspace
npm ci --prefix site
npm run check --prefix site
```

The VS Code extension has its own Node.js workspace:

```sh
npm ci --prefix editors/vscode
npm run check --prefix editors/vscode
```

## Checks before a pull request

Run the same core checks as CI:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
cargo deny check
npm run check --prefix site
```

The workspace MSRV is the `rust-version` in the root `Cargo.toml` (currently
Rust 1.80). CI compiles the workspace with that exact toolchain in addition to
running lint and test jobs on stable Rust.

If your change affects generated content, regenerate it and commit the result:

```sh
cargo xtask registry
cargo xtask completions
cargo xtask cli-docs
```

CI rejects drift in the registry index, shell completions, and generated EN/JA
CLI references.

## Documentation and translations

English and Japanese documentation are maintained as explicit pairs:

- `README.md` and `README.ja.md`
- `DESIGN.md` and `DESIGN.ja.md`
- `CONTRIBUTING.md` and `CONTRIBUTING.ja.md`
- every page under `site/src/content/docs/` and its matching `ja/` page

Update both files in a pair in the same pull request and keep their heading
levels in the same order. Run `npm run check:i18n --prefix site` before
submitting. CI checks page presence, heading structure, landing-page anchors,
configuration/rule coverage, and whether both sides of every changed pair were
touched.

For a genuinely language-neutral change, such as formatting or a corrected
URL shared by generated output, put `i18n-skip: <reason>` in the pull request
description. The reason is required and the exception is visible in CI output.
Do not use the marker to postpone a translation.

The CLI reference is generated from Clap. Change the command parser or the EN/JA
preambles in `crates/xtask/src/main.rs`, then run `cargo xtask cli-docs`; do not
edit the generated CLI pages directly.

## Dictionary contributions

Registry sources live in `site/registry/registry.toml` and
`site/registry/dicts/`. Record the dictionary's source and license, keep entries
sorted and deduplicated, and follow the complete versioning, provenance,
licensing, size, and pull-request checklist in
[`CONTRIBUTING-dictionaries.md`](CONTRIBUTING-dictionaries.md). Then run:

```sh
cargo xtask registry
git diff -- site/registry/index.json
```

Only data whose license is compatible with MIT OR Apache-2.0 can be bundled or
served by the project. Update `NOTICE.md` when attribution is required.

## Commits and pull requests

Keep commits focused and use an imperative summary such as `Add dictionary
search`. Include tests for behavior changes and explain user-visible trade-offs
in the pull request. Link the issue with `Closes #123` only when the full
acceptance criteria are satisfied.

Do not commit generated build directories, editor settings, credentials, or
downloaded registry caches. Contributions are submitted under the repository's
MIT OR Apache-2.0 licensing terms.

By participating, follow the [Code of Conduct](CODE_OF_CONDUCT.md). Report
security vulnerabilities privately according to [SECURITY.md](SECURITY.md);
do not open a public issue for an undisclosed vulnerability.
