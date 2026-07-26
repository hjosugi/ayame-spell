# Third-party notices

ayame-spell is licensed under MIT OR Apache-2.0. It builds on the following
third-party data and libraries:

## typos-dict (built-in English corrections)

The built-in English known-misspelling table is provided by the
[`typos-dict`](https://crates.io/crates/typos-dict) crate from
[crate-ci/typos](https://github.com/crate-ci/typos), used as a Cargo dependency.

License: MIT OR Apache-2.0. Copyright the typos maintainers and contributors.

## SCOWL (registry dictionary `en-base`, not bundled in the binary)

The `en-base` wordlist distributed through the ayame-spell dictionary registry
is derived from SCOWL (Spell Checker Oriented Word Lists),
http://wordlist.aspell.net/, Copyright 2000-2026 by Kevin Atkinson. Permission
to use, copy, modify, distribute and sell these word lists and their derivatives
is granted without fee, subject to retention of the original copyright notices.
The full SCOWL copyright file accompanies the registry artifact.

## SymSpell frequency dictionary (registry artifact `en-freq`, not bundled)

Suggestion-ranking frequency data is derived from
[SymSpell](https://github.com/wolfgarbe/SymSpell)'s
`frequency_dictionary_en_82_765.txt` (MIT License, Copyright Wolf Garbe), which
was in turn created by intersecting Google Books Ngram data (CC BY 3.0,
https://books.google.com/ngrams/) with SCOWL.

## Sudachi synonym dictionary (registry artifact `ja-variants`, not bundled)

Japanese loanword variant pairs distributed through the registry are extracted
from the [SudachiDict](https://github.com/WorksApplications/SudachiDict) synonym
dictionary, Copyright Works Applications Co., Ltd., licensed under the Apache
License, Version 2.0. The extraction (variant → canonical pairs) is a
modification of the original file.
