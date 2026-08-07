#!/usr/bin/env python3
"""Generate release-specific Homebrew, Scoop, and AUR manifests."""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path

REPOSITORY = "ayame-editor/ayame-spell"

# AUR expects this content in a file called `.SRCINFO`, but GitHub renames a
# dot-leading release asset to `default.SRCINFO`. Publish an unambiguous name
# instead and document the rename.
SRCINFO_ASSET = "ayame-spell-bin.SRCINFO"
TARGETS = {
    "linux-x64": ("x86_64-unknown-linux-gnu", "tar.gz"),
    "linux-arm64": ("aarch64-unknown-linux-gnu", "tar.gz"),
    "macos-x64": ("x86_64-apple-darwin", "tar.gz"),
    "macos-arm64": ("aarch64-apple-darwin", "tar.gz"),
    "windows-x64": ("x86_64-pc-windows-msvc", "zip"),
}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--tag", required=True)
    parser.add_argument("--checksums", type=Path, required=True)
    parser.add_argument("--output-dir", type=Path, required=True)
    return parser.parse_args()


def parse_checksums(path: Path) -> dict[str, str]:
    checksums: dict[str, str] = {}
    for line in path.read_text().splitlines():
        fields = line.split()
        if len(fields) == 2 and re.fullmatch(r"[0-9a-fA-F]{64}", fields[0]):
            checksums[fields[1].lstrip("*")] = fields[0].lower()
    return checksums


def asset(tag: str, key: str) -> str:
    target, extension = TARGETS[key]
    return f"ayame-spell-{tag}-{target}.{extension}"


def checksum(checksums: dict[str, str], name: str) -> str:
    try:
        return checksums[name]
    except KeyError as error:
        raise SystemExit(f"missing checksum for {name}") from error


def formula(tag: str, checksums: dict[str, str]) -> str:
    version = tag.removeprefix("v")
    base = f"https://github.com/{REPOSITORY}/releases/download/{tag}"
    names = {key: asset(tag, key) for key in TARGETS}
    sums = {key: checksum(checksums, name) for key, name in names.items()}
    return f'''class AyameSpell < Formula
  desc "Fast, low-noise English and Japanese spell checker"
  homepage "https://ayame-editor.github.io/ayame-spell/"
  version "{version}"
  license any_of: ["MIT", "Apache-2.0"]

  on_macos do
    if Hardware::CPU.arm?
      url "{base}/{names["macos-arm64"]}"
      sha256 "{sums["macos-arm64"]}"
    else
      url "{base}/{names["macos-x64"]}"
      sha256 "{sums["macos-x64"]}"
    end
  end

  on_linux do
    if Hardware::CPU.arm? && Hardware::CPU.is_64_bit?
      url "{base}/{names["linux-arm64"]}"
      sha256 "{sums["linux-arm64"]}"
    else
      url "{base}/{names["linux-x64"]}"
      sha256 "{sums["linux-x64"]}"
    end
  end

  def install
    bin.install "ayame-spell"
    man1.install "man/ayame-spell.1"
    bash_completion.install "completions/ayame-spell.bash" => "ayame-spell"
    zsh_completion.install "completions/_ayame-spell"
    fish_completion.install "completions/ayame-spell.fish"
  end

  test do
    assert_match version.to_s, shell_output("#{{bin}}/ayame-spell --version")
  end
end
'''


def scoop(tag: str, checksums: dict[str, str]) -> str:
    name = asset(tag, "windows-x64")
    version = tag.removeprefix("v")
    manifest = {
        "version": version,
        "description": "Fast, low-noise English and Japanese spell checker",
        "homepage": "https://ayame-editor.github.io/ayame-spell/",
        "license": "MIT OR Apache-2.0",
        "architecture": {
            "64bit": {
                "url": f"https://github.com/{REPOSITORY}/releases/download/{tag}/{name}",
                "hash": checksum(checksums, name),
            }
        },
        "extract_dir": f"ayame-spell-{tag}-x86_64-pc-windows-msvc",
        "bin": "ayame-spell.exe",
        "checkver": {"github": f"https://github.com/{REPOSITORY}"},
        "autoupdate": {
            "architecture": {
                "64bit": {
                    "url": (
                        f"https://github.com/{REPOSITORY}/releases/download/"
                        "v$version/ayame-spell-v$version-x86_64-pc-windows-msvc.zip"
                    )
                }
            }
        },
    }
    return json.dumps(manifest, indent=2) + "\n"


def pkgbuild(tag: str, checksums: dict[str, str]) -> str:
    x64_name = asset(tag, "linux-x64")
    arm64_name = asset(tag, "linux-arm64")
    version = tag.removeprefix("v")
    x64_sha = checksum(checksums, x64_name)
    arm64_sha = checksum(checksums, arm64_name)
    return f'''# Maintainer: ayame-spell contributors
pkgname=ayame-spell-bin
pkgver={version}
pkgrel=1
pkgdesc="Fast, low-noise English and Japanese spell checker"
arch=('x86_64' 'aarch64')
url="https://ayame-editor.github.io/ayame-spell/"
license=('MIT' 'Apache-2.0')
depends=('gcc-libs' 'glibc')
provides=("ayame-spell=$pkgver")
conflicts=('ayame-spell')
source_x86_64=("{x64_name}::https://github.com/{REPOSITORY}/releases/download/{tag}/{x64_name}")
sha256sums_x86_64=('{x64_sha}')
source_aarch64=("{arm64_name}::https://github.com/{REPOSITORY}/releases/download/{tag}/{arm64_name}")
sha256sums_aarch64=('{arm64_sha}')

package() {{
  local target
  case "$CARCH" in
    x86_64) target=x86_64-unknown-linux-gnu ;;
    aarch64) target=aarch64-unknown-linux-gnu ;;
  esac
  local root="$srcdir/ayame-spell-{tag}-$target"
  install -Dm755 "$root/ayame-spell" "$pkgdir/usr/bin/ayame-spell"
  install -Dm644 "$root/man/ayame-spell.1" "$pkgdir/usr/share/man/man1/ayame-spell.1"
  install -Dm644 "$root/LICENSE-MIT" "$pkgdir/usr/share/licenses/$pkgname/LICENSE-MIT"
  install -Dm644 "$root/LICENSE-APACHE" "$pkgdir/usr/share/licenses/$pkgname/LICENSE-APACHE"
  install -Dm644 "$root/completions/ayame-spell.bash" \\
    "$pkgdir/usr/share/bash-completion/completions/ayame-spell"
  install -Dm644 "$root/completions/_ayame-spell" \\
    "$pkgdir/usr/share/zsh/site-functions/_ayame-spell"
  install -Dm644 "$root/completions/ayame-spell.fish" \\
    "$pkgdir/usr/share/fish/vendor_completions.d/ayame-spell.fish"
}}
'''


def srcinfo(tag: str, checksums: dict[str, str]) -> str:
    x64_name = asset(tag, "linux-x64")
    arm64_name = asset(tag, "linux-arm64")
    version = tag.removeprefix("v")
    base = f"https://github.com/{REPOSITORY}/releases/download/{tag}"
    return f'''pkgbase = ayame-spell-bin
\tpkgdesc = Fast, low-noise English and Japanese spell checker
\tpkgver = {version}
\tpkgrel = 1
\turl = https://ayame-editor.github.io/ayame-spell/
\tarch = x86_64
\tarch = aarch64
\tlicense = MIT
\tlicense = Apache-2.0
\tdepends = gcc-libs
\tdepends = glibc
\tprovides = ayame-spell={version}
\tconflicts = ayame-spell
\tsource_x86_64 = {x64_name}::{base}/{x64_name}
\tsha256sums_x86_64 = {checksum(checksums, x64_name)}
\tsource_aarch64 = {arm64_name}::{base}/{arm64_name}
\tsha256sums_aarch64 = {checksum(checksums, arm64_name)}

pkgname = ayame-spell-bin
'''


def main() -> None:
    args = parse_args()
    tag = args.tag if args.tag.startswith("v") else f"v{args.tag}"
    checksums = parse_checksums(args.checksums)
    args.output_dir.mkdir(parents=True, exist_ok=True)
    (args.output_dir / "ayame-spell.rb").write_text(formula(tag, checksums))
    (args.output_dir / "ayame-spell.json").write_text(scoop(tag, checksums))
    (args.output_dir / "PKGBUILD").write_text(pkgbuild(tag, checksums))
    # Not named `.SRCINFO`: GitHub renames a dot-leading release asset to
    # `default.SRCINFO`, so publish an explicit name and let the consumer
    # save it as `.SRCINFO` next to the PKGBUILD.
    (args.output_dir / SRCINFO_ASSET).write_text(srcinfo(tag, checksums))


if __name__ == "__main__":
    main()
