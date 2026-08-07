#!/bin/sh
set -eu

repository="ayame-editor/ayame-spell"
version=${AYAME_SPELL_VERSION:-latest}
install_dir=${AYAME_SPELL_INSTALL_DIR:-"${HOME}/.local/bin"}

usage() {
  cat <<'EOF'
Install ayame-spell from a checksum-verified GitHub release archive.

Usage: install.sh [--version VERSION] [--install-dir DIRECTORY]

Environment:
  AYAME_SPELL_VERSION       Release version (default: latest)
  AYAME_SPELL_INSTALL_DIR   Binary directory (default: $HOME/.local/bin)
EOF
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --version)
      version=${2:?--version requires a value}
      shift 2
      ;;
    --install-dir)
      install_dir=${2:?--install-dir requires a value}
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

require() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "required command not found: $1" >&2
    exit 1
  }
}

require curl
require install
require tar

if [ "$version" = "latest" ]; then
  latest_url=$(curl -fsSLI -o /dev/null -w '%{url_effective}' \
    "https://github.com/${repository}/releases/latest")
  tag=${latest_url##*/}
else
  case "$version" in
    v*) tag=$version ;;
    *) tag="v$version" ;;
  esac
fi

case "$(uname -s)" in
  Linux) system=unknown-linux-gnu ;;
  Darwin) system=apple-darwin ;;
  *)
    echo "unsupported operating system: $(uname -s)" >&2
    exit 1
    ;;
esac

case "$(uname -m)" in
  x86_64|amd64) machine=x86_64 ;;
  arm64|aarch64) machine=aarch64 ;;
  *)
    echo "unsupported architecture: $(uname -m)" >&2
    exit 1
    ;;
esac

target="${machine}-${system}"
archive="ayame-spell-${tag}-${target}.tar.gz"
base_url="https://github.com/${repository}/releases/download/${tag}"
temporary=$(mktemp -d "${TMPDIR:-/tmp}/ayame-spell-install.XXXXXX")
trap 'rm -rf "$temporary"' EXIT HUP INT TERM

curl -fL --retry 3 --output "$temporary/$archive" "$base_url/$archive"
curl -fL --retry 3 --output "$temporary/SHA256SUMS" "$base_url/SHA256SUMS"

expected=$(awk -v archive="$archive" '
  $2 == archive || $2 == "*" archive { print $1; exit }
' "$temporary/SHA256SUMS")
if [ -z "$expected" ]; then
  echo "release checksum is missing for $archive" >&2
  exit 1
fi

if command -v sha256sum >/dev/null 2>&1; then
  actual=$(sha256sum "$temporary/$archive" | awk '{print $1}')
elif command -v shasum >/dev/null 2>&1; then
  actual=$(shasum -a 256 "$temporary/$archive" | awk '{print $1}')
else
  echo "sha256sum or shasum is required to verify the download" >&2
  exit 1
fi

if [ "$actual" != "$expected" ]; then
  echo "checksum mismatch for $archive" >&2
  exit 1
fi

tar -xzf "$temporary/$archive" -C "$temporary"
source_binary="$temporary/ayame-spell-${tag}-${target}/ayame-spell"
test -f "$source_binary"
mkdir -p "$install_dir"
install -m 0755 "$source_binary" "$install_dir/ayame-spell"

echo "installed ayame-spell ${tag#v} to $install_dir/ayame-spell"
case ":${PATH}:" in
  *":${install_dir}:"*) ;;
  *) echo "add $install_dir to PATH to run ayame-spell" ;;
esac
