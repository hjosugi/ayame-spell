---
title: CI recipes
description: Run ayame-spell in GitHub Actions, GitLab CI, or any shell-based build.
---

ayame-spell exits with `1` when findings remain, so a normal CI step fails
without extra shell logic. Commit `ayame-spell.toml`, `ayame-words.txt`, and any
local wordlists used by the configuration.

## GitHub Actions

The repository includes a composite Action. Pin the release and checker
version:

```yaml
      - uses: actions/checkout@v7
      - uses: ayame-editor/ayame-spell@v0.5.0
        with:
          version: 0.5.0
```

It downloads the exact checksum-verified GitHub Release and emits native GitHub
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
      - uses: actions/checkout@v7
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
  image: rust:1.91
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
      - image: cimg/rust:1.91
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
  - repo: https://github.com/ayame-editor/ayame-spell
    rev: v0.5.0
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

Registry references resolve from the local cache. Commit `ayame-spell.lock`,
then restore its exact versions before the check:

```sh
ayame-spell dict add --cache-only en-base python
ayame-spell check .
```

For hermetic or offline CI, run `ayame-spell dict vendor <name>`, commit the
copied files and rewritten config, and use relative paths.

## Restore the incremental scan cache

CI disables the scan cache unless its location is explicit. Restore the same
directory through the CI provider, then pass it to ayame-spell:

```sh
ayame-spell check . --cache-dir .cache/ayame-spell --format brief
```

The cache validates file content, effective configuration, dictionaries,
lockfile, and binary version before reuse. Do not cache only by path. Use
`--no-cache` when measuring a cold run.

## Check documentation freshness

This repository generates its CLI reference from Clap and then verifies there
is no diff:

```sh
cargo xtask cli-docs
git diff --exit-code -- site/src/content/docs/reference/cli.md \
  site/src/content/docs/ja/reference/cli.md
```
