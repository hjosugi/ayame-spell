---
title: CI recipes
description: Run ayame-spell in GitHub Actions, GitLab CI, or any shell-based build.
---

ayame-spell exits with `1` when findings remain, so a normal CI step fails
without extra shell logic. Commit `ayame-spell.toml`, `ayame-words.txt`, and any
local wordlists used by the configuration.

## GitHub Actions

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
      - run: ayame-spell check . --format brief
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

## JSON Lines for annotations

```sh
ayame-spell check . --format json > ayame-spell.jsonl
```

Each output line is an independent JSON object. A CI adapter can map `path`,
`line`, `column`, `message`, and `kind` to native annotations. Do not parse the
human format; use `brief` for compiler-style logs or `json` for automation.

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
