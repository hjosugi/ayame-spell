---
title: Inline directives
description: Suppress ayame-spell findings for a line, the next line, or an entire file.
---

Inline directives are literal, case-sensitive text recognized anywhere in the
file. They suppress both English and Japanese findings.

| Directive | Effect |
| --- | --- |
| `ayame-spell:ignore-line` | Skip the line containing the directive. |
| `ayame-spell:ignore-next-line` | Skip the directive line and the immediately following line. |
| `ayame-spell:ignore-file` | Skip the complete file. |

Use the comment syntax of the surrounding format:

```rust
let teh = 1; // ayame-spell:ignore-line

// ayame-spell:ignore-next-line
let recieve = 2;
```

```markdown
<!-- ayame-spell:ignore-file -->
```

## Matching details

- The directive can appear anywhere in its line.
- The directive line itself is always skipped.
- `ignore-next-line` applies to exactly one physical line, including an empty
  line.
- `ignore-file` is detected before line processing, wherever it appears.
- Similar text such as `Ayame-spell:ignore-line` does not match.
- There is no inline directive for a single word. Add the word to the project
  file, global file, or `[words].ignore` instead.

Prefer configuration or dictionaries for recurring intentional spellings.
Directives are best for fixtures, quoted mistakes, and generated examples.
