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
ayame-spell dict add en-base@1.0.0
ayame-spell dict add --cache-only rust
ayame-spell dict update
ayame-spell dict update --check
ayame-spell dict vendor en-base
ayame-spell dict remove python
```

- `list` fetches the index and marks cached entries with `*`.
- `add` downloads and enables each dictionary in the correct config array.
- `--cache-only` downloads without editing the project config.
- `update` reports `up to date` or updates an unlocked entry from one version
  to another. `--check` writes nothing and exits `1` when updates exist.
- `vendor` copies verified bytes under `dict/` and rewrites the project config
  to that relative path.
- `remove` deletes the cache file and removes its reference from the project
  config.

## Dictionary kinds

| Kind | File format | Enabled in |
| --- | --- | --- |
| `wordlist` | UTF-8 text, one word per line, `#` comments | `[words].dictionaries` |
| `corrections` | `typo<TAB>fix[,fix]` | `[corrections].extra` |
| `variants` | TOML `[variants]` map | `[japanese].variant-files` |

Registry references use `registry:name` or the explicit
`registry:name@version` form. A normal `dict add` writes `ayame-spell.lock`
with the resolved version, immutable source file, and SHA-256 digest. Commit
that lockfile: another machine then downloads and verifies the same bytes even
when the registry has a newer current version.

Published versions remain in the index's `versions` array and their files are
never rewritten. An explicit `@version` pin is not advanced by `dict update`.
The checker verifies locked cache bytes before loading them.

## Offline and hermetic use

Vendor and rewrite a reference in one command:

```sh
ayame-spell dict vendor en-base
```

The resulting configuration uses a relative path:

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
ayame-spell dict --registry https://docs.example.com/spelling/index.json list
```

The index schema is:

```json
{
  "version": 2,
  "dictionaries": [
    {
      "name": "team",
      "version": "1.0.0",
      "language": "en",
      "kind": "wordlist",
      "description": "Company and product terms",
      "provenance": "Maintained by Example Corp",
      "file": "dicts/team.txt",
      "sha256": "...",
      "entries": 120,
      "versions": [
        {
          "version": "1.0.0",
          "file": "dicts/team.txt",
          "sha256": "...",
          "entries": 120
        }
      ],
      "license": "Proprietary"
    }
  ]
}
```

## Contribute a dictionary

Follow the complete
[dictionary contribution guide](https://github.com/hjosugi/ayame-spell/blob/main/CONTRIBUTING-dictionaries.md)
for formats, size limits, version immutability, provenance, licensing, and the
pull-request checklist. In a checkout of ayame-spell:

1. Add the UTF-8 data file under `site/registry/dicts/`.
2. Add a versioned `[[dictionary]]` entry to
   `site/registry/registry.toml`.
3. Record provenance and license information in the file header and
   `NOTICE.md` when required.
4. Generate the index:

   ```sh
   cargo xtask registry
   ```

5. Run the repository checks:

   ```sh
   cargo test --workspace
   cargo build -p ayame-spell --locked
   python3 contrib/quality/check_quality.py --binary target/debug/ayame-spell
   cargo run -p ayame-spell -- check .
   ```

Keep entries sorted, focused on one ecosystem or writing policy, and free of
secrets or private identifiers. Generation rejects duplicates and any
language-list entry already supplied by `en-base`. The quality fixtures run the
real CLI and guard representative terms from every shipped English wordlist,
clean prose, known corrections, and unknown-word reporting. They are a
deterministic regression suite, not a statistical accuracy claim for arbitrary
prose.
