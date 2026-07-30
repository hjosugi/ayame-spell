# Fuzzing ayame-spell

The four cargo-fuzz targets cover tokenizer byte spans, Japanese line and
document checks, halfwidth-kana conversion, and configuration parsing.
Arbitrary byte input is decoded lossily first, matching the CLI's handling of
non-NUL, non-UTF-8 files.

```sh
cargo install cargo-fuzz --locked
cargo fuzz run fuzz_tokenizer
cargo fuzz run fuzz_japanese
cargo fuzz run fuzz_halfwidth
cargo fuzz run fuzz_config
```

CI runs every target for a short bounded session. When a crash is found, keep
the minimized input under `fuzz/corpus/<target>/` and add a deterministic Rust
regression test when practical. `fuzz/artifacts/` contains local crash output
and is not committed.
