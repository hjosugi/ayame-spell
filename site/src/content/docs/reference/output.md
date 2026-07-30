---
title: Exit codes and output formats
description: Integrate ayame-spell exit status, human, brief, and JSON Lines output.
---

## Exit codes

| Code | Meaning |
| --- | --- |
| `0` | The command succeeded and no unfixed findings remain. |
| `1` | The check succeeded but one or more findings remain. |
| `2` | Usage, configuration, file, network, or other operational error. |

For `fix`, successfully applied findings do not cause code `1`; only findings
remaining after safe fixes do.

## Human format

`--format human` is the default:

```text
docs/guide.md:4:3: recieve → receive [typo]
```

When standard output is a terminal, the word and suggestions are colored. A
summary is written to standard error:

```text
1 issue(s) in 1 file(s) — 12 file(s) checked
```

The summary also counts fixed findings, skipped binaries, and files skipped by
`max-file-size`.

## Brief format

`--format brief` produces compiler-style, colorless records:

```text
docs/guide.md:4:3: recieve -> receive
```

Use this for CI logs that recognize `path:line:column`.

## JSON Lines format

`--format json` writes one JSON object per finding, followed by one summary
record:

```json
{"version":1,"type":"issue","path":"docs/guide.md","line":4,"column":3,"offset":42,"length":7,"word":"recieve","kind":"typo","suggestions":["receive"],"fix":"receive","message":"`recieve` should be `receive`"}
{"version":1,"type":"summary","issues":1,"files_with_issues":1,"files_checked":12,"fixed":0,"skipped_binary":0,"skipped_large":0}
```

Every record has a numeric `version` and a `type` discriminator. Consumers
must reject unsupported versions and ignore fields they do not understand.
Within version 1, existing fields and meanings will not change; new fields and
new `kind` values may be added. A removal, rename, type change, or semantic
change requires version 2.

The machine-readable schema is published at
[`schema/v1/ayame-spell-output.json`](https://hjosugi.github.io/ayame-spell/schema/v1/ayame-spell-output.json).

Issue fields:

| Field | Type | Meaning |
| --- | --- | --- |
| `version` | integer | JSON Lines contract version; currently `1`. |
| `type` | string | Always `"issue"` for a finding. |
| `path` | string | Checked path as reported by the walker. |
| `line` | integer | One-based line. |
| `column` | integer | One-based character column. |
| `offset` | integer | Zero-based byte offset in the file text. |
| `length` | integer | Byte length of the finding. |
| `word` | string | Original text. |
| `kind` | string | Stable [issue code](./rules/). |
| `suggestions` | string array | Ordered replacement candidates. |
| `fix` | string or null | Safe non-interactive replacement, or `null` when review is required. |
| `message` | string | Human-readable explanation. |

Summary fields:

| Field | Type | Meaning |
| --- | --- | --- |
| `version` | integer | JSON Lines contract version; currently `1`. |
| `type` | string | Always `"summary"`. |
| `issues` | integer | Findings still present after optional fixes. |
| `files_with_issues` | integer | Files containing remaining findings. |
| `files_checked` | integer | Text files checked. |
| `fixed` | integer | Findings safely fixed during this run. |
| `skipped_binary` | integer | Files skipped because they appear binary. |
| `skipped_large` | integer | Files skipped by `max-file-size`. |

The summary is emitted even when there are no findings, so a successful empty
stream is distinguishable from a command that did not run. Read the output
line by line; the overall stream is not a JSON array.

## Word collection output

`words collect` has separate `--plain` and `--json` switches:

```sh
ayame-spell words collect --plain
ayame-spell words collect --json
```

Its JSON Lines objects contain `word`, `count`, `kind`, and `example`.
