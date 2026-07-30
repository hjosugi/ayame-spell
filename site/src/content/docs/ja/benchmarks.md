---
title: 再現可能なベンチマーク
description: ayame-spell の処理速度、ピークメモリ、比較計測、CI の性能劣化判定を再現します。
---

以下の数値はすべて、リポジトリに含めた generator と runner から得ています。
生成した corpus 自体はコミットしません。

## 基準結果

2026-07-30 に release build した ayame-spell 0.4.0 は、cache を使わず
35 MiB / 40 万行の corpus 全体を確認しました。

| 指標 | 結果 |
| --- | ---: |
| 3 回の中央値 | 1.598 s |
| throughput | 21.90 MiB/s |
| 最速 | 1.527 s |
| peak RSS | 679.6 MiB |
| 確認ファイル | 1 |
| skip ファイル | 0 |

これは、再現可能な artifact がなかった従来の 56 MB というメモリ表記を
置き換えるものです。
[生の結果](https://github.com/hjosugi/ayame-spell/blob/main/benchmarks/results/2026-07-30-linux-x86_64.json)
には、全 sample、command、version、machine 情報、CLI summary を記録しています。

## 比較

同じ machine 上で、全 tool に同じ問題のない Markdown file を渡しました。
cache を無効化し、必要な tool では大きな file の上限を引き上げ、出力を抑え、
完了した tool は 3 回ずつ実行しています。

| Tool | version / rule | 中央値 | throughput | peak RSS |
| --- | --- | ---: | ---: | ---: |
| ayame-spell | 0.4.0、既定の corrections + 日本語確認 | 1.598 s | 21.90 MiB/s | 679.6 MiB |
| typos | 1.48.0、既定設定 | 1.348 s | 25.97 MiB/s | 61.4 MiB |
| cSpell | 10.0.1、既定設定 | 7.630 s | 4.59 MiB/s | 527.8 MiB |
| textlint | 15.7.1 + spellcheck-tech-word 5.0.0 | >60 s（timeout） | <0.58 MiB/s | 停止時 1003.2 MiB |

生の記録:
[typos](https://github.com/hjosugi/ayame-spell/blob/main/benchmarks/results/2026-07-30-typos-linux-x86_64.json)、
[cSpell](https://github.com/hjosugi/ayame-spell/blob/main/benchmarks/results/2026-07-30-cspell-linux-x86_64.json)、
[textlint](https://github.com/hjosugi/ayame-spell/blob/main/benchmarks/results/2026-07-30-textlint-linux-x86_64.json)。

これは end-to-end の処理速度比較であり、正確さの順位ではありません。tool の
rule set は同等ではなく、typos は corrections table、cSpell は辞書、選択した
textlint rule は curated technical term を確認します。

## 計測方法

machine は Linux 7.1.4、x86_64、glibc 2.42、Python 3.14.6 と報告されました。
corpus は改行で終わる 40 万行、正確に 36,700,160 bytes です。SHA-256 は
次のとおりです。

```text
d16dd8ec158f415c54d1b857fdf4f0cf620f50a8c905a8621c85defe3f7c640b
```

各行は決定的な英語 prose と一意な数値 ID で構成します。ayame-spell は
`check --no-config --no-cache --format json` で実行しました。cSpell には
`file://` input と `--max-file-size 100MB` を指定しています。これらがないと
絶対 path の file を確認せず skip するためです。textlint は 60 秒で停止し、
架空の完了時間ではなく下限として記録します。

## 再現手順

リポジトリ root で実行します。

```sh
python3 contrib/bench/generate_corpus.py --output /tmp/ayame-corpus.md
cargo build --release --locked -p ayame-spell
python3 contrib/bench/run_benchmark.py \
  --binary target/release/ayame-spell \
  --corpus /tmp/ayame-corpus.md \
  --repeat 3 \
  --output benchmarks/results/local.json
```

version を固定した比較 command は
[`contrib/bench/README.md`](https://github.com/hjosugi/ayame-spell/blob/main/contrib/bench/README.md)
にあります。Criterion microbenchmark は tokenizer、correction lookup、
dictionary lookup と suggestion、日本語 consistency を対象にします。

```sh
cargo bench -p ayame-spell-core
```

## 性能劣化の防止

pull request CI は candidate と `origin/main` の両方を build し、同じ corpus
に対して cache なしで各 3 回計測します。candidate の中央値が 35% を超えて
遅くなると失敗します。この幅で shared runner の揺らぎを吸収しつつ、
throughput が半減する変更は reject します。
