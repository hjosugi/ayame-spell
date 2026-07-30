#!/usr/bin/env python3
"""Measure another checker on the shared corpus with a recorded command."""

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
    parser.add_argument("--name", required=True)
    parser.add_argument("--version", required=True)
    parser.add_argument("--corpus", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--repeat", type=int, default=3)
    parser.add_argument(
        "--timeout",
        type=float,
        help="stop and record a lower bound if one run exceeds this many seconds",
    )
    parser.add_argument("--success-code", type=int, action="append", default=[0])
    parser.add_argument("command", nargs=argparse.REMAINDER)
    return parser.parse_args()


def peak_rss_mib() -> float:
    value = resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss
    if sys.platform == "darwin":
        return value / (1024 * 1024)
    return value / 1024


def main() -> None:
    args = parse_args()
    if args.repeat < 1:
        raise SystemExit("--repeat must be at least 1")
    if not args.command or args.command[0] != "--":
        raise SystemExit("pass the checker command after --")

    corpus = args.corpus.resolve()
    replacements = {
        "{corpus}": str(corpus),
        "{corpus-uri}": corpus.as_uri(),
    }
    command = [replacements.get(part, part) for part in args.command[1:]]
    if not command:
        raise SystemExit("checker command cannot be empty")

    samples = []
    exit_codes = []
    timed_out = False
    for _ in range(args.repeat):
        started = time.perf_counter()
        try:
            result = subprocess.run(
                command,
                capture_output=True,
                timeout=args.timeout,
            )
        except subprocess.TimeoutExpired:
            timed_out = True
            break
        samples.append(time.perf_counter() - started)
        exit_codes.append(result.returncode)
        if result.returncode not in args.success_code:
            raise SystemExit(
                f"benchmark command exited {result.returncode}\n"
                f"stdout:\n{result.stdout[-2000:].decode(errors='replace')}\n"
                f"stderr:\n{result.stderr[-2000:].decode(errors='replace')}"
            )

    corpus_bytes = corpus.read_bytes()
    median = statistics.median(samples) if samples else None
    record = {
        "schema": 1,
        "tool": args.name,
        "version": args.version,
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
        "exit_codes": exit_codes,
        "timed_out": timed_out,
        "timeout_seconds": args.timeout,
        "wall_seconds": {
            "samples": samples,
            "median": median,
            "minimum": min(samples) if samples else None,
            "lower_bound": args.timeout if timed_out else None,
        },
        "throughput_mib_per_second": (
            len(corpus_bytes) / (1024 * 1024) / median
            if median is not None
            else None
        ),
        "throughput_upper_bound_mib_per_second": (
            len(corpus_bytes) / (1024 * 1024) / args.timeout
            if timed_out and args.timeout
            else None
        ),
        "peak_rss_mib": peak_rss_mib(),
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(record, indent=2, sort_keys=True) + "\n")
    print(json.dumps(record, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
