#!/usr/bin/env python3
"""Generate the deterministic benchmark corpus without checking it in."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--lines", type=int, default=400_000)
    parser.add_argument("--bytes", type=int, default=35 * 1024 * 1024)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    if args.lines <= 0 or args.bytes <= 0:
        raise SystemExit("--lines and --bytes must be positive")

    rows = [
        (
            "The benchmark validates reliable documentation and configuration "
            f"for project teams {index:06d}."
        ).encode()
        for index in range(args.lines)
    ]
    base_bytes = sum(len(row) + 1 for row in rows)
    if base_bytes > args.bytes:
        raise SystemExit(
            f"{args.bytes} bytes is too small for {args.lines} lines "
            f"(minimum {base_bytes})"
        )
    padding, remainder = divmod(args.bytes - base_bytes, args.lines)

    args.output.parent.mkdir(parents=True, exist_ok=True)
    digest = hashlib.sha256()
    with args.output.open("wb") as output:
        for index, row in enumerate(rows):
            rendered = row + b" " * (padding + (index < remainder)) + b"\n"
            output.write(rendered)
            digest.update(rendered)

    print(
        json.dumps(
            {
                "path": str(args.output),
                "bytes": args.output.stat().st_size,
                "lines": args.lines,
                "sha256": digest.hexdigest(),
            },
            sort_keys=True,
        )
    )


if __name__ == "__main__":
    main()
