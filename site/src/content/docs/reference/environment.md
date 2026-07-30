---
title: Environment variables and file locations
description: Registry overrides, project discovery, and XDG-compatible config and cache paths.
---

## Environment variables

| Variable | Default | Purpose |
| --- | --- | --- |
| `AYAME_SPELL_REGISTRY` | `https://hjosugi.github.io/ayame-spell/registry/index.json` | Replace the dictionary registry index URL. Dictionary file URLs are resolved relative to this index. |
| `AYAME_SPELL_CONFIG_DIR` | Platform config directory | Replace the directory containing the global `config.toml` and `words.txt`. |
| `AYAME_SPELL_CACHE_DIR` | Platform cache directory | Replace the root for registry, completion, and incremental scan caches. |

Example for an internal mirror:

```sh
export AYAME_SPELL_REGISTRY=https://docs.example.com/spelling/index.json
ayame-spell dict --registry https://docs.example.com/spelling/index.json list
ayame-spell dict list
```

ayame-spell does not currently read a variable for the project config path.
Choose the project root by placing `ayame-spell.toml` or
`.ayame-spell.toml`. LSP clients determine the discovery start through their
workspace root; editor initialization options can override `mode`,
`japaneseEnabled`, `diagnosticSeverity`, `debounceMs` (default 150), and
`locale`, but not the config path. The locale selects English or Japanese rule
explanations in hovers.

## Project files

| File | Location | Purpose |
| --- | --- | --- |
| `ayame-spell.toml` or `.ayame-spell.toml` | First matching ancestor of the checked path | Project configuration and ignore words. |
| `ayame-spell.lock` | Project root | Resolved registry versions, files, and sha256 digests; commit this file. |
| `ayame-words.txt` | Project root by default | Team wordlist; controlled by `[words].project`. |
| Relative dictionaries | Project-root relative | Wordlists, correction TSVs, and Japanese variant TOML files. |

Project discovery stops at the first matching config. If none exists, the
nearest `.git` ancestor is the root.

## User config and data

ayame-spell uses the platform directories returned by the Rust `dirs` crate.

| Purpose | Linux / BSD | macOS | Windows |
| --- | --- | --- | --- |
| Global config | `${XDG_CONFIG_HOME:-~/.config}/ayame-spell/config.toml` | `~/Library/Application Support/ayame-spell/config.toml` | `%APPDATA%\ayame-spell\config.toml` |
| Global words | `${XDG_CONFIG_HOME:-~/.config}/ayame-spell/words.txt` | `~/Library/Application Support/ayame-spell/words.txt` | `%APPDATA%\ayame-spell\words.txt` |
| Registry cache | `${XDG_CACHE_HOME:-~/.cache}/ayame-spell/dicts/` | `~/Library/Caches/ayame-spell/dicts/` | `%LOCALAPPDATA%\ayame-spell\dicts\` |
| Incremental scan cache | `${XDG_CACHE_HOME:-~/.cache}/ayame-spell/scan/` | `~/Library/Caches/ayame-spell/scan/` | `%LOCALAPPDATA%\ayame-spell\scan\` |

The global config is optional. `words add --global` creates parent directories
and the word file on demand. Registry commands create the cache directory on
demand.

## Incremental scan cache

Local scans cache each file's issues. Entries are keyed by path, size,
nanosecond modification time, content hash, effective configuration, loaded
dictionaries, lockfile, and ayame-spell version. Changing any of those inputs
invalidates the entry. A corrupt or missing entry is ignored and rebuilt.

Use `--no-cache` for a cold scan, `--cache-dir PATH` to select an explicit
location, and `-v` to display hit counts. Caching is disabled automatically
when `CI` or `GITHUB_ACTIONS` is set unless `--cache-dir` is supplied. This
avoids surprising persistence in CI while still allowing a deliberately
restored cache:

```sh
ayame-spell check . --cache-dir .cache/ayame-spell --format brief
```

## Reference resolution

- Absolute paths are used unchanged.
- Relative paths resolve against the project root, not the process working
  directory.
- `registry:name` resolves through `ayame-spell.lock` to cached
  `name@version.txt`; `registry:name@version` requests an explicit version.
- Locked bytes are sha256-verified before they are loaded.
- A missing referenced file produces a warning while the checker is built.

Use `ayame-spell config` to confirm the discovered root and config paths.
