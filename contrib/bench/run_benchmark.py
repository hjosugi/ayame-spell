#!/usr/bin/env python3
"""Measure end-to-end ayame-spell wall time and peak resident memory."""

from __future__ import annotations

import argparse
import hashlib
import json
import platform
import resource
import statistics
import subprocess
import sys
import time
from pathlib import Path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--corpus", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--repeat", type=int, default=3)
    parser.add_argument("--revision")
    return parser.parse_args()


def revision(explicit: str | None) -> str:
    if explicit:
        return explicit
    result = subprocess.run(
        ["git", "rev-parse", "HEAD"],
        check=False,
        capture_output=True,
        text=True,
    )
    return result.stdout.strip() if result.returncode == 0 else "unknown"


def peak_rss_mib() -> float:
    value = resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss
    if sys.platform == "darwin":
        return value / (1024 * 1024)
    return value / 1024


def main() -> None:
    args = parse_args()
    if args.repeat < 1:
        raise SystemExit("--repeat must be at least 1")
    binary = args.binary.resolve()
    corpus = args.corpus.resolve()
    command = [
        str(binary),
        "check",
        "--no-config",
        "--no-cache",
        "--format",
        "json",
        str(corpus),
    ]

    samples = []
    summary = None
    for _ in range(args.repeat):
        started = time.perf_counter()
        result = subprocess.run(command, capture_output=True, text=True)
        samples.append(time.perf_counter() - started)
        if result.returncode != 0:
            raise SystemExit(
                f"benchmark command exited {result.returncode}\n"
                f"stdout:\n{result.stdout[-2000:]}\n"
                f"stderr:\n{result.stderr[-2000:]}"
            )
        records = [json.loads(line) for line in result.stdout.splitlines() if line]
        summary = records[-1] if records else None

    corpus_bytes = corpus.read_bytes()
    version = subprocess.run(
        [str(binary), "--version"],
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    median = statistics.median(samples)
    result = {
        "schema": 1,
        "revision": revision(args.revision),
        "version": version,
        "machine": {
            "platform": platform.platform(),
            "architecture": platform.machine(),
            "processor": platform.processor() or "unknown",
            "python": platform.python_version(),
        },
        "command": command,
        "corpus": {
            "path": str(corpus),
            "bytes": len(corpus_bytes),
            "lines": corpus_bytes.count(b"\n"),
            "sha256": hashlib.sha256(corpus_bytes).hexdigest(),
        },
        "repeat": args.repeat,
        "wall_seconds": {
            "samples": samples,
            "median": median,
            "minimum": min(samples),
        },
        "throughput_mib_per_second": len(corpus_bytes) / (1024 * 1024) / median,
        "peak_rss_mib": peak_rss_mib(),
        "summary": summary,
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n")
    print(json.dumps(result, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
