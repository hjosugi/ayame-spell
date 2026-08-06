---
title: Getting started
description: Install ayame-spell, run the first check, and add a project configuration.
---

## Install without Rust

The shell installers select the matching GitHub Release archive, verify it
against `SHA256SUMS`, and place the binary in `~/.local/bin`:

```sh
curl --proto '=https' --tlsv1.2 -LsSf \
  https://raw.githubusercontent.com/hjosugi/ayame-spell/main/install.sh | sh
```

On Windows PowerShell:

```powershell
irm https://raw.githubusercontent.com/hjosugi/ayame-spell/main/install.ps1 | iex
```

Release-specific Homebrew and Scoop manifests are attached to each release:

```sh
brew install --formula \
  https://github.com/hjosugi/ayame-spell/releases/latest/download/ayame-spell.rb
```

```powershell
scoop install https://github.com/hjosugi/ayame-spell/releases/latest/download/ayame-spell.json
```

Node projects can run the release-attached, checksum-verifying native wrapper
without a registry account:

```sh
npx https://github.com/hjosugi/ayame-spell/releases/download/v0.4.0/ayame-spell-npm-v0.4.0.tgz check .
```

The shorter `npx ayame-spell check .` command will become available when the
same wrapper is published to npm.

For CI containers:

```sh
docker run --rm -v "$PWD:/work" -w /work \
  ghcr.io/hjosugi/ayame-spell:0.4.0 check .
```

An x86_64/aarch64 `ayame-spell-bin` `PKGBUILD` and matching `.SRCINFO` are
attached to each release for AUR maintainers and direct `makepkg -si` use. See the
[distribution source](https://github.com/hjosugi/ayame-spell/tree/main/packaging)
for the reproducible manifest generator.

## Install with Rust

Install from crates.io with Rust 1.91 or newer:

```sh
cargo install ayame-spell
ayame-spell --version
```

You can also download the archive for your platform from
[GitHub Releases](https://github.com/hjosugi/ayame-spell/releases/latest),
put `ayame-spell` (or `ayame-spell.exe`) on `PATH`, and optionally install a
shell completion from the archive's `completions/` directory.

## Run the first check

From a project directory:

```sh
ayame-spell
```

With no subcommand or path, ayame-spell checks the current directory in
corrections mode. It respects `.gitignore`, skips common generated lock files,
and reports known misspellings plus enabled Japanese notation issues.

Check selected paths or apply safe fixes:

```sh
ayame-spell check README.md docs/
ayame-spell fix README.md docs/
```

`fix` only applies unambiguous replacements and mechanical width conversions.
Unknown words and ambiguous corrections remain for review.

## Create a configuration

```sh
ayame-spell init
ayame-spell config
```

`init` writes `ayame-spell.toml`. `config` prints the effective configuration
after global and project settings are merged and defaults are applied.

For a quiet baseline, the generated corrections-mode config is enough. To find
words outside a dictionary:

```sh
ayame-spell dict add en-base
```

Then set:

```toml
[check]
mode = "dictionary"
```

Use [modes](./modes/) and the
[configuration reference](./reference/configuration/) to tune the result.

## Add project words

```sh
ayame-spell words add ProjectName APIName
ayame-spell words collect
ayame-spell words triage
```

Project words go to `ayame-words.txt` by default. Commit this file so the CLI
and every editor use the same vocabulary.

## Next steps

- Set up your [editor](./editors/).
- Add ayame-spell to [CI](./ci/).
- Review all [issue codes](./reference/rules/).
- Learn the [Japanese checks](./japanese/).
