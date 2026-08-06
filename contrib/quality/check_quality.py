#!/usr/bin/env python3
"""Exercise every shipped English wordlist and representative error classes."""

from __future__ import annotations

import argparse
import json
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parent
CONFIG = ROOT / "ayame-spell.toml"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", type=Path, required=True)
    return parser.parse_args()


def check(binary: Path, fixture: str, expected_exit: int) -> list[dict[str, object]]:
    result = subprocess.run(
        [
            str(binary),
            "check",
            "--config",
            str(CONFIG),
            "--no-cache",
            "--format",
            "json",
            str(ROOT / fixture),
        ],
        capture_output=True,
        text=True,
    )
    if result.returncode != expected_exit:
        raise SystemExit(
            f"{fixture}: expected exit {expected_exit}, got {result.returncode}\n"
            f"stdout:\n{result.stdout}\nstderr:\n{result.stderr}"
        )
    try:
        records = [json.loads(line) for line in result.stdout.splitlines() if line]
    except json.JSONDecodeError as error:
        raise SystemExit(f"{fixture}: invalid JSON output: {error}") from error
    summaries = [record for record in records if record.get("type") == "summary"]
    if len(summaries) != 1:
        raise SystemExit(f"{fixture}: expected exactly one summary record")
    return records


def main() -> None:
    binary = parse_args().binary.resolve()
    clean = check(binary, "clean.md", 0)
    clean_issues = [record for record in clean if record.get("type") == "issue"]
    if clean_issues:
        raise SystemExit(f"clean.md produced false positives: {clean_issues}")

    issues = check(binary, "issues.md", 1)
    actual = {
        (str(record["word"]).lower(), str(record["kind"]))
        for record in issues
        if record.get("type") == "issue"
    }
    expected = {
        ("recieve", "typo"),
        ("publically", "typo"),
        ("zzqqy", "unknown-word"),
    }
    if actual != expected:
        raise SystemExit(
            "issues.md findings changed:\n"
            f"expected: {sorted(expected)}\n"
            f"actual:   {sorted(actual)}"
        )
    print(
        "dictionary quality fixtures passed: "
        "clean corpus has no findings and expected typo/unknown-word findings remain"
    )


if __name__ == "__main__":
    main()
