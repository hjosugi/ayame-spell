use ayame_spell_core::corrections::Corrections;
use ayame_spell_core::dictionary::WordSets;
use ayame_spell_core::japanese::{consistency_issues, KatakanaOcc};
use ayame_spell_core::tokenizer::{words_in_line, TokenizerOptions};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::hint::black_box;

fn tokenizer(c: &mut Criterion) {
    let input = "The state-of-the-art AyameSpellService validates developer's APIs, \
                 documentation, URLs like https://example.com, and recieve fixtures.";
    let mut group = c.benchmark_group("tokenizer");
    group.throughput(Throughput::Bytes(input.len() as u64));
    group.bench_function("mixed_prose", |b| {
        b.iter(|| words_in_line(black_box(input), black_box(&TokenizerOptions::default())))
    });
    group.finish();
}

fn corrections(c: &mut Criterion) {
    let corrections = Corrections::new(true);
    let mut group = c.benchmark_group("corrections");
    for word in ["recieve", "receive", "ProjectIdentifier"] {
        group.bench_with_input(BenchmarkId::new("lookup", word), word, |b, word| {
            b.iter(|| corrections.check(black_box(word)))
        });
    }
    group.finish();
}

fn dictionary(c: &mut Criterion) {
    let words = (0..50_000)
        .map(|index| format!("benchmarkword{index:05}"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut dictionary = WordSets::default();
    dictionary.add_wordlist_str(&words);

    let mut group = c.benchmark_group("dictionary");
    group.bench_function("lookup_50k", |b| {
        b.iter(|| dictionary.contains(black_box("benchmarkword25000")))
    });
    group.bench_function("levenshtein_suggestion_50k", |b| {
        b.iter(|| dictionary.suggest(black_box("benchmarkword2500x"), black_box(5)))
    });
    group.finish();
}

fn japanese_consistency(c: &mut Criterion) {
    let occurrences = (0..20_000)
        .map(|index| {
            let form = if index % 10 == 0 {
                "サーバ"
            } else {
                "サーバー"
            };
            KatakanaOcc {
                form: form.to_string(),
                line: index + 1,
                col: 0,
                offset: index as usize * 16,
            }
        })
        .collect::<Vec<_>>();
    let mut group = c.benchmark_group("japanese");
    group.throughput(Throughput::Elements(occurrences.len() as u64));
    group.bench_function("katakana_consistency_20k", |b| {
        b.iter(|| consistency_issues(black_box(&occurrences)))
    });
    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(30);
    targets = tokenizer, corrections, dictionary, japanese_consistency
}
criterion_main!(benches);
