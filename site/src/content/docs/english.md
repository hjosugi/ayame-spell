---
title: English locale policy
description: Enforce en-US or en-GB spelling while preserving quiet compound and acronym handling.
---

The default accepts both common regional spellings:

```toml
[check]
locale = "any"
```

Choose a house style when a repository needs one:

```toml
[check]
locale = "en-US" # colour → color
```

or:

```toml
[check]
locale = "en-GB" # color → colour
```

## What the locale rule reports

`en-variant` is separate from `typo`: both forms are real words, but one
conflicts with the configured policy. The built-in table covers common
technology-writing pairs such as color/colour, behavior/behaviour,
organization/organisation, and analyze/analyse.

`"any"` emits no locale findings. Project words and inline directives can
silence intentional product spellings without disabling the policy globally.

## Token behavior

Contractions such as `don't` remain intact. Hyphenated compounds such as
`state-of-the-art` are checked by component. Possessives use their base word,
and plural acronyms (`APIs`, `IDs`) plus ALL-CAPS acronyms are not reported as
unknown words.

CamelCase and PascalCase are split only when the active syntax profile checks
that source region. For example, `NmaeService` exposes `Nmae` in profile
`"all"`, while profile `"source"` masks an identifier outside comments and
strings.

