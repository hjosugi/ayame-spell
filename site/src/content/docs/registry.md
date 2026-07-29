---
title: Dictionary registry
description: Install, inspect, cache, mirror, and contribute shared ayame-spell dictionaries.
---

import RegistryTable from "../../components/RegistryTable.astro";

The registry publishes plain wordlists and notation rules behind a JSON index.
ayame-spell verifies every download against the SHA-256 digest in that index.

<RegistryTable locale="en" />

## Commands

```sh
ayame-spell dict list
ayame-spell dict add en-base python
ayame-spell dict add --cache-only rust
ayame-spell dict update
ayame-spell dict remove python
```

- `list` fetches the index and marks cached entries with `*`.
- `add` downloads and enables each dictionary in the correct config array.
- `--cache-only` downloads without editing the project config.
- `update` re-downloads every currently cached entry.
- `remove` deletes the cache file and removes its reference from the project
  config.

## Dictionary kinds

| Kind | File format | Enabled in |
| --- | --- | --- |
| `wordlist` | UTF-8 text, one word per line, `#` comments | `[words].dictionaries` |
| `corrections` | `typo<TAB>fix[,fix]` | `[corrections].extra` |
| `variants` | TOML `[variants]` map | `[japanese].variant-files` |

Registry references have the form `registry:name`. The current registry format
pins content by the index's SHA-256 value during download. Config-level
dictionary versions are not yet supported; vendor files when a build must
remain unchanged across future registry updates.

## Offline and hermetic use

Download a dictionary, copy it into the repository, and replace the registry
reference with a relative path:

```toml
[words]
dictionaries = ["dict/en-base.txt", "dict/team.txt"]
```

This avoids network access and makes the exact bytes reviewable with the
project.

## Private registry

Host an `index.json` and its files at one HTTP(S) base URL, then set:

```sh
export AYAME_SPELL_REGISTRY=https://docs.example.com/spelling/index.json
```

The index schema is:

```json
{
  "version": 1,
  "dictionaries": [
    {
      "name": "team",
      "language": "en",
      "kind": "wordlist",
      "description": "Company and product terms",
      "file": "dicts/team.txt",
      "sha256": "...",
      "entries": 120,
      "license": "Proprietary"
    }
  ]
}
```

## Contribute a dictionary

In a checkout of ayame-spell:

1. Add the UTF-8 data file under `site/registry/dicts/`.
2. Add a `[[dictionary]]` entry to `site/registry/registry.toml`.
3. Record provenance and license information in the file header and
   `NOTICE.md` when required.
4. Generate the index:

   ```sh
   cargo xtask registry
   ```

5. Run the repository checks:

   ```sh
   cargo test --workspace
   cargo run -p ayame-spell -- check .
   ```

Keep entries sorted, focused on one ecosystem or writing policy, and free of
secrets or private identifiers. A wordlist should contain terms that are useful
across multiple projects, not every identifier from one codebase.
