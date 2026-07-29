---
title: Environment variables and file locations
description: Registry overrides, project discovery, and XDG-compatible config and cache paths.
---

## Environment variables

| Variable | Default | Purpose |
| --- | --- | --- |
| `AYAME_SPELL_REGISTRY` | `https://hjosugi.github.io/ayame-spell/registry/index.json` | Replace the dictionary registry index URL. Dictionary file URLs are resolved relative to this index. |

Example for an internal mirror:

```sh
export AYAME_SPELL_REGISTRY=https://docs.example.com/spelling/index.json
ayame-spell dict list
```

ayame-spell does not currently read a variable for the project config path.
Choose the project root by placing `ayame-spell.toml` or
`.ayame-spell.toml`. LSP clients determine the discovery start through their
workspace root; editor initialization options can override `mode`,
`japaneseEnabled`, and `diagnosticSeverity`, but not the config path.

## Project files

| File | Location | Purpose |
| --- | --- | --- |
| `ayame-spell.toml` or `.ayame-spell.toml` | First matching ancestor of the checked path | Project configuration and ignore words. |
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

The global config is optional. `words add --global` creates parent directories
and the word file on demand. Registry commands create the cache directory on
demand.

## Reference resolution

- Absolute paths are used unchanged.
- Relative paths resolve against the project root, not the process working
  directory.
- `registry:name` resolves to the cached `name.txt` inside the registry cache.
- A missing referenced file produces a warning while the checker is built.

Use `ayame-spell config` to confirm the discovered root and config paths.
