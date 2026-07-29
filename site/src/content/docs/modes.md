---
title: Checking modes
description: Choose corrections, dictionary, or off mode globally or by path.
---

ayame-spell separates high-confidence corrections from unknown-word detection.
The mode controls English word checking; Japanese checks are configured
independently.

## Corrections mode

```toml
[check]
mode = "corrections"
```

This is the default. It reports words found in the built-in correction table or
your configured correction tables. It does not assume every other identifier
is misspelled, so it is a good zero-configuration CI baseline.

Use corrections mode when false positives are more expensive than missed novel
typos.

## Dictionary mode

```toml
[check]
mode = "dictionary"

[words]
dictionaries = ["registry:en-base"]
```

Dictionary mode includes every corrections-mode check, then reports words not
found in the active project, global, or configured dictionaries. Suggestions
are ranked using edit distance.

Install registry dictionaries before referring to them:

```sh
ayame-spell dict add en-base python
```

Use dictionary mode for prose, public API documentation, and projects prepared
to maintain a vocabulary.

## Off mode

```toml
[check]
mode = "off"
```

English word checks are disabled. Japanese checks can still run. Set
`japanese = false` in an override if the path should have no checks at all.

## Mix modes by path

```toml
[check]
mode = "corrections"

[[overrides]]
paths = ["docs/**", "*.md"]
mode = "dictionary"

[[overrides]]
paths = ["tests/fixtures/**"]
mode = "off"
japanese = false
```

Overrides are evaluated in file order and later matching entries win. They can
change only `mode` and `japanese`; dictionaries and rule settings still come
from the merged global/project configuration.

## A practical rollout

1. Start the repository in corrections mode.
2. Install `en-base` and relevant ecosystem dictionaries.
3. Enable dictionary mode for documentation with `[[overrides]]`.
4. Run `words collect`, then `words triage`.
5. Expand dictionary mode only after the project word list is stable.
