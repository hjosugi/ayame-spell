---
title: FAQ
description: Common questions about modes, fixes, dictionaries, Japanese checks, CI, and editors.
---

## Do I need a configuration file?

No. With no config, ayame-spell uses corrections mode, the bundled correction
table, and the default Japanese checks. Add a config when you need dictionary
mode, exclusions, custom vocabulary, or a style policy.

## Which mode should I use?

Start with corrections mode for low-noise CI. Use dictionary mode in prose or
where finding novel typos is worth maintaining a wordlist. You can choose by
path with `[[overrides]]`.

## Does `fix` change unknown words?

No. It applies only single-candidate typos and mechanical Japanese width or
notation conversions. Unknown words and multi-candidate typos require a human
choice.

## Where should an accepted word go?

- Team or product term: project word file (`ayame-words.txt` by default).
- Personal name used across your projects: global word file.
- Deliberate misspelling that should stay visible in config: `[words].ignore`.
- Known typo with a preferred correction: `[corrections.words]`.
- Japanese alternate form with a preferred spelling: `[japanese.variants]`.

## Is matching case-sensitive?

English word and correction matching is case-insensitive. Corrections preserve
common case patterns where possible. Project and global word files therefore
usually need only one spelling.

## Why is a three-letter unknown word not reported?

The tokenizer default `min-word-len` is three, but dictionary-mode
unknown-word detection has an additional four-byte minimum and excludes
all-uppercase words. Known corrections can still report shorter words.

## Does ayame-spell understand identifiers?

It splits camel case and acronym boundaries and naturally treats punctuation
and underscores as boundaries. It also skips URLs, emails, hashes, UUID-like
values, and other data-like tokens.

## Why are `サーバ` and `サーバー` both accepted?

The default Japanese style is document consistency, not long or short style.
Either form is accepted alone; mixing both forms in one document reports the
minority. Choose `katakana-style = "long"` or `"short"` to enforce a direction.

## Does ayame-spell stop at a fixed file size?

Not by default: `max-file-size = 0` is unlimited. If you configure a limit,
skipped files are counted in the summary rather than silently ignored.

## Can CI run without network access?

Yes. The checker itself needs no network. Registry commands need HTTP, so
either prime the cache or vendor dictionary files and refer to relative paths.

## Can I use several language servers in one editor?

Yes, if the client supports multiple servers per buffer. Keep the programming
language server and add ayame-spell alongside it. Helix and Zed examples on the
[editor page](./editors/) show this explicitly.

## Is the LSP result different from the CLI?

Both use the same core checker and configuration. LSP editor initialization
options can deliberately override the discovered config, and editors check
open in-memory text while the CLI reads files from disk.

## How do I see the final configuration?

Run:

```sh
ayame-spell config
```

It prints the project root, loaded global/project files, and the fully defaulted
merged TOML.
