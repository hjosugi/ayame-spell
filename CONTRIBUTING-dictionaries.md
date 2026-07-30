# Contributing dictionaries

Dictionary contributions must be reviewable, reproducible, and legally safe.
The registry source of truth is `site/registry/registry.toml`; generated
metadata lives in `site/registry/index.json`.

## File formats

- `wordlist`: UTF-8 text, one lowercase entry per line. Blank lines and lines
  beginning with `#` are ignored.
- `corrections`: UTF-8 TSV, `typo<TAB>replacement[,replacement]`.
- `variants`: UTF-8 TOML with a `[variants]` string-to-string table.

Keep a focused dictionary below 10,000 entries unless the source and size are
discussed first. Wordlists must be sorted, deduplicated, and must not repeat an
entry already present in `en-base`; `cargo xtask registry` enforces this.

## Version, provenance, and license

Every `[[dictionary]]` record requires:

- a unique name and semantic `version`;
- language, kind, description, and immutable file path;
- a specific `provenance` statement;
- a license compatible with redistribution by this MIT OR Apache-2.0 project.

Do not change the bytes behind a published name/version. Add a new file and
version, and retain the old file through `[[dictionary.release]]` so pins keep
working. Never copy data from a website, standard, vendor, or package index
without confirming its redistribution terms. Update `NOTICE.md` whenever
attribution is required.

## Pull request checklist

- [ ] The file is UTF-8, focused, sorted, and deduplicated.
- [ ] `registry.toml` records version, provenance, and license.
- [ ] Published version files remain available and unchanged.
- [ ] `NOTICE.md` contains any required attribution.
- [ ] `cargo xtask registry` succeeds and regenerates `index.json`.
- [ ] `cargo test --workspace` succeeds.
- [ ] The English and Japanese registry documentation stays in sync.

Run:

```sh
cargo xtask registry
git diff --check
cargo test --workspace
```
