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

`--format json` writes one JSON object per finding and no summary:

```json
{"path":"docs/guide.md","line":4,"column":3,"offset":42,"length":7,"word":"recieve","kind":"typo","suggestions":["receive"],"message":"`recieve` should be `receive`"}
```

Fields:

| Field | Type | Meaning |
| --- | --- | --- |
| `path` | string | Checked path as reported by the walker. |
| `line` | integer | One-based line. |
| `column` | integer | One-based character column. |
| `offset` | integer | Zero-based byte offset in the file text. |
| `length` | integer | Byte length of the finding. |
| `word` | string | Original text. |
| `kind` | string | Stable [issue code](./rules/). |
| `suggestions` | string array | Ordered replacement candidates. |
| `message` | string | Human-readable explanation. |

Read the stream line by line. The overall output is not a JSON array.

## Word collection output

`words collect` has separate `--plain` and `--json` switches:

```sh
ayame-spell words collect --plain
ayame-spell words collect --json
```

Its JSON Lines objects contain `word`, `count`, `kind`, and `example`.
