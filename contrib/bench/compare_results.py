#!/usr/bin/env python3
"""Fail when candidate throughput regresses materially from main."""

from __future__ import annotations

import argparse
import json
from pathlib import Path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("baseline", type=Path)
    parser.add_argument("candidate", type=Path)
    parser.add_argument("--max-slowdown-percent", type=float, default=35.0)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    baseline = json.loads(args.baseline.read_text())
    candidate = json.loads(args.candidate.read_text())
    if baseline["corpus"]["sha256"] != candidate["corpus"]["sha256"]:
        raise SystemExit("baseline and candidate used different corpora")

    baseline_time = baseline["wall_seconds"]["median"]
    candidate_time = candidate["wall_seconds"]["median"]
    slowdown = (candidate_time / baseline_time - 1) * 100
    print(
        f"main median: {baseline_time:.4f}s; "
        f"candidate median: {candidate_time:.4f}s; "
        f"slowdown: {slowdown:+.1f}% "
        f"(limit {args.max_slowdown_percent:.1f}%)"
    )
    if slowdown > args.max_slowdown_percent:
        raise SystemExit(
            "performance regression exceeds the noise-tolerant threshold"
        )


if __name__ == "__main__":
    main()
