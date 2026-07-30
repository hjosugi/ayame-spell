---
title: Reproducible benchmarks
description: Reproduce ayame-spell throughput, peak memory, comparison runs, and the CI regression guard.
---

Every number below comes from a checked-in generator and runner. The generated
corpus itself is not committed.

## Reference result

On 2026-07-30, ayame-spell 0.4.0 built in release mode checked the complete
35 MiB / 400,000-line corpus with no cache:

| Metric | Result |
| --- | ---: |
| Median of 3 runs | 1.598 s |
| Throughput | 21.90 MiB/s |
| Fastest run | 1.527 s |
| Peak RSS | 679.6 MiB |
| Files checked | 1 |
| Files skipped | 0 |

This replaces the earlier 56 MB memory claim, which was not backed by a
reproducible artifact. The
[raw result](https://github.com/hjosugi/ayame-spell/blob/main/benchmarks/results/2026-07-30-linux-x86_64.json)
records every sample, command, version, machine field, and CLI summary.

## Comparison

All tools received the same clean Markdown file on the same machine. Cache was
disabled, large-file limits were raised where necessary, output was suppressed,
and each completed tool ran three times.

| Tool | Version / rules | Median | Throughput | Peak RSS |
| --- | --- | ---: | ---: | ---: |
| ayame-spell | 0.4.0, default corrections + Japanese checks | 1.598 s | 21.90 MiB/s | 679.6 MiB |
| typos | 1.48.0, defaults | 1.348 s | 25.97 MiB/s | 61.4 MiB |
| cSpell | 10.0.1, defaults | 7.630 s | 4.59 MiB/s | 527.8 MiB |
| textlint | 15.7.1 + spellcheck-tech-word 5.0.0 | >60 s (timeout) | <0.58 MiB/s | 1003.2 MiB at stop |

Raw records:
[typos](https://github.com/hjosugi/ayame-spell/blob/main/benchmarks/results/2026-07-30-typos-linux-x86_64.json),
[cSpell](https://github.com/hjosugi/ayame-spell/blob/main/benchmarks/results/2026-07-30-cspell-linux-x86_64.json), and
[textlint](https://github.com/hjosugi/ayame-spell/blob/main/benchmarks/results/2026-07-30-textlint-linux-x86_64.json).

This is an end-to-end throughput comparison, not a correctness ranking.
The tools do not implement equivalent rule sets: typos uses a corrections
table, cSpell uses dictionaries, and the selected textlint rule checks curated
technical terms.

## Methodology

The machine reported Linux 7.1.4, x86_64, glibc 2.42, and Python 3.14.6.
The corpus is exactly 36,700,160 bytes with 400,000 newline-terminated lines.
Its SHA-256 is:

```text
d16dd8ec158f415c54d1b857fdf4f0cf620f50a8c905a8621c85defe3f7c640b
```

Each line is deterministic English prose plus a unique numeric identifier.
ayame-spell ran `check --no-config --no-cache --format json`. cSpell used a
`file://` input and `--max-file-size 100MB`; without those explicit settings it
would skip the absolute-path input instead of measuring it. The textlint run
uses a fixed 60-second timeout, recorded as a lower bound rather than a
fabricated completion time.

## Reproduce

From the repository root:

```sh
python3 contrib/bench/generate_corpus.py --output /tmp/ayame-corpus.md
cargo build --release --locked -p ayame-spell
python3 contrib/bench/run_benchmark.py \
  --binary target/release/ayame-spell \
  --corpus /tmp/ayame-corpus.md \
  --repeat 3 \
  --output benchmarks/results/local.json
```

The exact pinned comparison commands are in
[`contrib/bench/README.md`](https://github.com/hjosugi/ayame-spell/blob/main/contrib/bench/README.md).
Criterion microbenchmarks cover tokenization, correction lookup, dictionary
lookup and suggestions, and Japanese consistency:

```sh
cargo bench -p ayame-spell-core
```

## Regression guard

Pull-request CI builds both the candidate revision and `origin/main`, generates
one shared corpus, and records three no-cache runs for each binary. The job
fails when the candidate median is more than 35% slower. That tolerance absorbs
shared-runner noise while still rejecting a change that halves throughput.
