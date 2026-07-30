---
title: CI recipes
description: Run ayame-spell in GitHub Actions, GitLab CI, or any shell-based build.
---

ayame-spell exits with `1` when findings remain, so a normal CI step fails
without extra shell logic. Commit `ayame-spell.toml`, `ayame-words.txt`, and any
local wordlists used by the configuration.

## GitHub Actions

The repository includes a composite Action. Pin its major release and the
checker version:

```yaml
      - uses: actions/checkout@v6
      - uses: hjosugi/ayame-spell@v1
        with:
          version: 1.0.0
```

It installs the exact requested crates.io version and emits native GitHub
annotations. Set `sarif: true` to upload a SARIF result instead; the calling
workflow must grant `security-events: write`.

The equivalent build-from-source recipe is:

```yaml
name: spelling

on:
  pull_request:
  push:
    branches: [main]

jobs:
  ayame-spell:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo install --locked ayame-spell
      - run: ayame-spell check . --format github
```

For faster builds, download a pinned release archive instead of compiling it.
Verify the archive against `SHA256SUMS` from the same release.

## GitLab CI

```yaml
spell:
  image: rust:1.80
  cache:
    paths:
      - .cargo/
  before_script:
    - cargo install --locked ayame-spell
  script:
    - ayame-spell check . --format brief
```

## CircleCI

```yaml
version: 2.1
jobs:
  spelling:
    docker:
      - image: cimg/rust:1.80
    steps:
      - checkout
      - run: cargo install --locked ayame-spell
      - run: ayame-spell check . --format brief
workflows:
  spelling:
    jobs: [spelling]
```

On any shell-based CI runner, the portable fallback is:

```sh
cargo install --locked ayame-spell
ayame-spell check . --format brief
```

## pre-commit

The repository exports both checking and manual fixing hooks:

```yaml
repos:
  - repo: https://github.com/hjosugi/ayame-spell
    rev: v1.0.0
    hooks:
      - id: ayame-spell
```

Use `id: ayame-spell-fix` with `stages: [manual]`, then run
`pre-commit run ayame-spell-fix --all-files` when you explicitly want files
rewritten.

## GitHub annotations and SARIF

```sh
ayame-spell check . --format github
```

The GitHub format emits native workflow commands and is selected automatically
when `GITHUB_ACTIONS=true`. For code scanning, generate and upload SARIF:

```yaml
      - name: Generate spelling SARIF
        run: ayame-spell check . --format sarif > ayame-spell.sarif
        continue-on-error: true
      - uses: github/codeql-action/upload-sarif@v4
        with:
          sarif_file: ayame-spell.sarif
```

Do not parse the human format; use `brief` for compiler-style logs or `json`
for automation.

## Adopt an existing repository without a cleanup freeze

Create and commit a content-based baseline once:

```sh
ayame-spell baseline .
git add ayame-spell-baseline.json
```

`ayame-spell check .` now suppresses those existing findings while still
failing on new ones. Fingerprints use the file path, rule, word, and surrounding
line content rather than a line number, so inserting lines does not invalidate
the baseline. Audit everything with `ayame-spell check --no-baseline .`.

As old findings are fixed, remove their entries and verify the committed file:

```sh
ayame-spell baseline --prune .
git diff --exit-code ayame-spell-baseline.json
```

## Registry dictionaries in CI

Registry references resolve from the local cache, so install them before the
check:

```sh
ayame-spell dict add --cache-only en-base python
ayame-spell check .
```

For hermetic or offline CI, vendor the wordlists in the repository and use
relative paths in `[words].dictionaries`.

## Check documentation freshness

This repository generates its CLI reference from Clap and then verifies there
is no diff:

```sh
cargo xtask cli-docs
git diff --exit-code -- site/src/content/docs/reference/cli.md \
  site/src/content/docs/ja/reference/cli.md
```
