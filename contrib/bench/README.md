# Reproducible performance measurement

Generate the corpus, build the exact revision, and record three cold scans:

```sh
python3 contrib/bench/generate_corpus.py --output /tmp/ayame-corpus.md
cargo build --release --locked -p ayame-spell
python3 contrib/bench/run_benchmark.py \
  --binary target/release/ayame-spell \
  --corpus /tmp/ayame-corpus.md \
  --repeat 3 \
  --output benchmarks/results/local.json
```

The generator writes exactly 35 MiB and 400,000 lines by default. The runner
records every wall-time sample, median throughput, peak child-process RSS,
machine details, commit, command, corpus SHA-256, and CLI summary. It always
uses `--no-cache`.

The pull-request CI job builds both `origin/main` and the proposed revision,
runs both against the same corpus, and fails above a 35% median slowdown.
This threshold tolerates shared-runner noise while ensuring a change that
halves throughput cannot merge.

## Comparison runs

Install the exact comparison versions, then use the generic runner so the
corpus digest, full command, wall-time samples, and peak RSS are recorded:

```sh
cargo install --locked typos-cli --version 1.48.0
npm install cspell@10.0.1 textlint@15.7.1 \
  textlint-rule-spellcheck-tech-word@5.0.0

python3 contrib/bench/run_external_benchmark.py \
  --name typos --version 1.48.0 --corpus /tmp/ayame-corpus.md \
  --repeat 3 --output benchmarks/results/typos.json \
  -- typos --format brief '{corpus}'
python3 contrib/bench/run_external_benchmark.py \
  --name cspell --version 10.0.1 --corpus /tmp/ayame-corpus.md \
  --repeat 3 --output benchmarks/results/cspell.json \
  -- cspell lint --no-progress --no-summary --no-color --no-issues \
  --no-exit-code --no-cache --max-file-size 100MB '{corpus-uri}'
python3 contrib/bench/run_external_benchmark.py \
  --name textlint --version 15.7.1 --corpus /tmp/ayame-corpus.md \
  --repeat 3 --timeout 60 --output benchmarks/results/textlint.json \
  -- textlint --config contrib/bench/textlint.config.json '{corpus}'
```

The comparison is an end-to-end throughput reference, not a claim that the
tools implement identical rules. `typos` and `cspell` use their default
spelling checks. `textlint` uses `spellcheck-tech-word`, which checks a curated
technical-term rule set rather than a general English dictionary.
The timeout is part of the methodology: a timed-out tool is recorded as a
lower-bound duration and upper-bound throughput instead of being assigned a
fabricated completion time.
