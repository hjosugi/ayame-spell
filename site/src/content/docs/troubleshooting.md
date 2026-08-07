---
title: Troubleshooting
description: Diagnose configuration discovery, missing dictionaries, noisy checks, editor startup, and unexpected skips.
---

## Start with these commands

```sh
ayame-spell --version
ayame-spell config
ayame-spell check path/to/file --format human
```

The first confirms which binary the shell finds. The second shows discovery and
merge results. The third narrows a repository-wide symptom to one file.

## My config is not loaded

- Run `ayame-spell config` and inspect `# root` and `# project config`.
- The file must be named exactly `ayame-spell.toml` or
  `.ayame-spell.toml`.
- Discovery walks upward from the first checked path. When checking paths from
  different projects in one invocation, the first path determines the config.
- Unknown keys cause an error with the file path.
- In an editor, ensure the workspace root contains the config. Restart the LSP
  client after changing root detection.

## A registry dictionary is missing

An entry such as `registry:en-base` resolves only to the local cache; it does
not download during a check.

```sh
ayame-spell dict add --cache-only en-base
ayame-spell dict list
```

If a custom registry is set, confirm `AYAME_SPELL_REGISTRY` in the environment
seen by the editor or CI. A checksum mismatch means the served file no longer
matches `index.json`; regenerate or repair the registry rather than bypassing
verification.

## Dictionary mode is too noisy

1. Confirm `en-base` is installed and configured.
2. Add relevant ecosystem dictionaries.
3. Run `words collect --min-count 2` to prioritize repeated findings.
4. Use `words triage` and commit the project word file.
5. Limit dictionary mode to prose with `[[overrides]]`.

Do not add apparent typos to the wordlist just to make the first run clean.

## An expected typo is not reported

- Corrections mode reports only known corrections. Try dictionary mode for
  novel misspellings.
- Check `[words].ignore`, project words, global words, and self-mapped
  corrections.
- Look for an inline directive.
- URLs, emails, hash-like values, backslash escapes, all-uppercase unknown
  words, and short unknown words are intentionally skipped.
- A matching later override may set `mode = "off"`.

## A file is not checked

- `.gitignore` and `[files].exclude` apply to directory walks.
- Hidden files require `include-hidden = true`.
- Binary detection skips files containing a NUL byte near the beginning.
- `max-file-size` may skip the file; the command summary reports the count.
- Explicitly pass the file path to isolate walk behavior.

## The editor shows no diagnostics

1. Run `ayame-spell lsp --stdio` in a terminal. It should wait silently for LSP
   input; exit with `Ctrl-C`.
2. Confirm the editor process can find `ayame-spell` on `PATH`. GUI apps may
   have a different environment from your shell.
3. Check the client-specific log: `:checkhealth vim.lsp`, `hx --health`,
   Zed.log, Eglot events, Sublime LSP logs, or JetBrains LSP Consoles.
4. Confirm the filetype/selector/mapping includes the current buffer.
5. Confirm the workspace root and config.
6. Restart the client after configuration changes.

Large open documents are checked on save rather than every edit.

## Code actions are missing

The client must request `textDocument/codeAction`. Direct replacements are
standard workspace edits. Project/global dictionary actions use
`workspace/executeCommand`; clients without that capability may omit or reject
them. Use `ayame-spell words add` or `words triage` in a terminal.

## JSON cannot be parsed as one value

`--format json` is JSON Lines, not a JSON array. Parse each line separately:

```sh
ayame-spell --format json | jq -c .
```

## `fix` leaves findings

This is expected for unknown words and corrections with several candidates.
Review them manually or use editor code actions. Run the check again to see
only remaining findings.

## Report a reproducible bug

Include:

- `ayame-spell --version`
- the smallest input that reproduces the problem
- the relevant effective config from `ayame-spell config`
- exact command or editor/client version
- human or JSON output

Remove secrets and private vocabulary before opening a
[GitHub issue](https://github.com/ayame-editor/ayame-spell/issues).
