# Build and publishing runbook

The extension version, Cargo workspace version, Git tag, and server version are
kept in lockstep. A release such as `0.2.0` uses tag `v0.2.0`.

## Prerequisites

- Node.js 22
- Rust 1.91 or newer
- Linux desktop libraries required by VS Code Electron tests
- `xvfb-run` for headless Linux integration tests

No Marketplace token belongs in this repository or a local config file.

## Local validation

From the repository root:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build -p ayame-spell

cd editors/vscode
npm ci
npm run check
npm run lint
npm run format:check
npm audit
AYAME_SPELL_SERVER_PATH=../../target/debug/ayame-spell xvfb-run -a npm test
```

## Build a local Linux x64 VSIX

```sh
cd editors/vscode
npm ci
npm run build:production
mkdir -p server/linux-x64
cp ../../target/release/ayame-spell server/linux-x64/
npx vsce package --target linux-x64 --out ayame-spell-linux-x64.vsix
```

The release workflow performs the equivalent operation separately on every
native runner:

| Rust target                 | VS Code target |
| --------------------------- | -------------- |
| `x86_64-unknown-linux-gnu`  | `linux-x64`    |
| `aarch64-unknown-linux-gnu` | `linux-arm64`  |
| `x86_64-apple-darwin`       | `darwin-x64`   |
| `aarch64-apple-darwin`      | `darwin-arm64` |
| `x86_64-pc-windows-msvc`    | `win32-x64`    |

## One-time Marketplace setup

These account-level operations must be completed by the repository owner. They
may involve platform terms or organization policy and are intentionally not
automated here.

1. In the Visual Studio Marketplace, create or verify publisher ID `hjosugi`.
2. In Open VSX, create or claim namespace `hjosugi`.
3. Create a Visual Studio Marketplace publishing credential and an Open VSX
   access token using the minimum publishing scope.
4. Add repository Actions secrets:
   - `VSCE_PAT`
   - `OVSX_PAT`
5. Add repository Actions variable `PUBLISH_VSCODE=true` only after both
   namespaces and secrets are ready.

Until the variable is set, GitHub releases still receive installable VSIX assets
and store publishing is safely skipped.

Azure DevOps global PATs are scheduled for retirement on December 1, 2026.
Before then, migrate the Marketplace step to `vsce --azure-credential` and a
repository-owner-approved Microsoft Entra workload identity. The current
`VSCE_PAT` path is kept as the immediately usable bootstrap described by the
[official publishing guide](https://code.visualstudio.com/api/working-with-extensions/publishing-extension).

## Automated release

Pushing `v*` runs `.github/workflows/release.yml`:

1. compile five native server binaries;
2. create CLI archives with licenses and shell completions;
3. build five target-specific VSIX packages;
4. attach every archive and VSIX to the GitHub release;
5. publish the five VSIX packages to both stores only when
   `PUBLISH_VSCODE=true`.

The publishing job first verifies all five expected filenames, then passes the
complete package set directly to each CLI:

```sh
npx vsce publish --packagePath path/to/package-1.vsix path/to/package-2.vsix
npx ovsx publish --packagePath path/to/package-1.vsix path/to/package-2.vsix
```

`vsce` reads `VSCE_PAT`; `ovsx` reads `OVSX_PAT`.

## Release checklist

- [ ] `Cargo.toml`, `Cargo.lock`, and `editors/vscode/package.json` use the same
      `major.minor.patch`.
- [ ] `CHANGELOG.md` and `MARKETPLACE.md` describe the release.
- [ ] Rust formatting, Clippy, unit tests, extension typecheck, lint, format,
      integration tests, audit, and VSIX package validation pass.
- [ ] The working tree is clean and `main` CI is green.
- [ ] Tag `vX.Y.Z` points at the intended `main` commit.
- [ ] The GitHub release contains five CLI archives and five VSIX packages.
- [ ] Every archive checksum is recorded in `SHA256SUMS`.
- [ ] Install and smoke-test at least one packaged VSIX.
- [ ] If store publishing is enabled, verify all five targets and both listings.
- [ ] Do not accept a CLA, license, Marketplace agreement, or other legal terms
      on someone else's behalf.

## Failure recovery

- Re-run a failed matrix job before changing the tag.
- The manual release workflow can rebuild the Intel macOS CLI and VSIX assets
  for an existing tag.
- If one store rejects a package, leave the GitHub release intact, disable
  `PUBLISH_VSCODE`, fix the manifest or credential, and republish the same
  already-built VSIX only when the registry permits it.
- Never print tokens or pass them as command-line arguments in logs.
