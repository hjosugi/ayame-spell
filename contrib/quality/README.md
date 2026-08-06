# Dictionary quality regression fixtures

This suite runs the real CLI in dictionary mode. `clean.md` covers the base
English list, the project list, and at least one representative term from every
shipped English ecosystem wordlist. `issues.md` covers a built-in correction,
an extra registry correction, and an unknown word.

Run it from the repository root:

```sh
cargo build -p ayame-spell --locked
python3 contrib/quality/check_quality.py --binary target/debug/ayame-spell
```

This is a deterministic regression gate, not a statistical precision or recall
claim for arbitrary prose. Add a minimized fixture whenever a real false
positive or false negative is fixed.
