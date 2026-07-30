#!/usr/bin/env python3
"""Fail unless every shipped integration matches the workspace version."""

from __future__ import annotations

import json
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def toml(path: str) -> dict:
    return tomllib.loads((ROOT / path).read_text())


def main() -> None:
    expected = toml("Cargo.toml")["workspace"]["package"]["version"]
    versions = {
        "VS Code": json.loads((ROOT / "editors/vscode/package.json").read_text())[
            "version"
        ],
        "npm": json.loads((ROOT / "packages/npm/package.json").read_text())["version"],
        "Zed Cargo": toml("editors/zed/Cargo.toml")["package"]["version"],
        "Zed extension": toml("editors/zed/extension.toml")["version"],
    }
    mismatches = {
        integration: version
        for integration, version in versions.items()
        if version != expected
    }
    if mismatches:
        rendered = ", ".join(
            f"{integration}={version}" for integration, version in mismatches.items()
        )
        raise SystemExit(f"version mismatch (workspace={expected}): {rendered}")
    print(f"all shipped integrations match {expected}")


if __name__ == "__main__":
    main()
