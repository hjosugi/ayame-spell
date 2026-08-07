# ayame-spell for VS Code

Fast, low-noise spell checking for code and prose in **English and Japanese**.
The platform-specific extension includes the Rust language server, so no Rust
toolchain or separate CLI installation is required.

![ayame-spell checks English corrections, Japanese notation, and bulk word review](https://raw.githubusercontent.com/ayame-editor/ayame-spell/main/editors/vscode/media/ayame-spell-demo.gif)

## Why ayame-spell?

- **Quiet by default** — corrections mode flags known misspellings, not every
  project-specific identifier.
- **Dictionary mode when you need it** — opt into unknown-word detection with
  team and personal wordlists.
- **Japanese-first checks** — find 表記ゆれ, fullwidth alphanumerics, halfwidth
  katakana, and unexpected fullwidth spaces.
- **Bulk workflows** — review every flagged word across open files and add or
  ignore selected terms in one pass.
- **One configuration** — `ayame-spell.toml` drives VS Code, the CLI, and CI.
- **Native speed** — diagnostics and fixes come from the bundled Rust server.

## Installation

Install `hjosugi.ayame-spell` from the VS Code Marketplace or Open VSX after the
listing is enabled. Until then, download the VSIX matching the machine where the
extension host runs from the
[latest GitHub release](https://github.com/ayame-editor/ayame-spell/releases/latest):

| Host                | VSIX target    |
| ------------------- | -------------- |
| Windows x64         | `win32-x64`    |
| Linux x64           | `linux-x64`    |
| Linux ARM64         | `linux-arm64`  |
| macOS Apple silicon | `darwin-arm64` |
| macOS Intel         | `darwin-x64`   |

Then run **Extensions: Install from VSIX...**. In SSH, dev container, or WSL
workspaces, install the package matching the _remote extension host_.

If no bundled server matches, install the CLI separately and configure
`ayame-spell.serverPath`:

```sh
cargo install ayame-spell
```

## Everyday workflow

Known misspellings and Japanese notation issues appear as VS Code diagnostics.
Use Quick Fix for one finding, or run **ayame-spell: Fix All Safe Issues in
File**.

![A high-confidence English correction](https://raw.githubusercontent.com/ayame-editor/ayame-spell/main/editors/vscode/media/corrections.png)

The status bar shows the active mode and current-file finding count. Click it to
cycle `corrections → dictionary → off`. Set `ayame-spell.mode` to `inherit` to
use the checked-in TOML value instead.

![Japanese notation consistency and mechanical checks](https://raw.githubusercontent.com/ayame-editor/ayame-spell/main/editors/vscode/media/japanese-variants.png)

Run **ayame-spell: Review Flagged Words (Bulk Add / Ignore)** to select many
terms, then send them to:

- `ayame-words.txt` for the project and team;
- the personal global wordlist for all projects; or
- `[words].ignore` in `ayame-spell.toml`.

![Bulk review of flagged words](https://raw.githubusercontent.com/ayame-editor/ayame-spell/main/editors/vscode/media/review-words.png)

## Commands

| Command                              | Purpose                                        |
| ------------------------------------ | ---------------------------------------------- |
| Fix All Safe Issues in File          | Apply unambiguous typo and mechanical fixes    |
| Review Flagged Words                 | Bulk add or ignore findings from open files    |
| Add Selection to Project Words       | Add selected terms to `ayame-words.txt`        |
| Add Selection to Global Words        | Add selected terms to your personal words      |
| Install Shared Dictionary            | Pick registry dictionaries and update config   |
| Open or Create Project Configuration | Open a starter `ayame-spell.toml`              |
| Toggle Checking Mode                 | Cycle corrections, dictionary, and off         |
| Restart Server                       | Restart after diagnosing configuration changes |

## Settings

| Setting                          | Default   | Meaning                               |
| -------------------------------- | --------- | ------------------------------------- |
| `ayame-spell.enable`             | `true`    | Start the language server             |
| `ayame-spell.serverPath`         | empty     | Bundled server, then `$PATH` fallback |
| `ayame-spell.mode`               | `inherit` | TOML, corrections, dictionary, or off |
| `ayame-spell.japanese.enabled`   | `true`    | Enable Japanese checks                |
| `ayame-spell.diagnosticSeverity` | `warning` | Diagnostic display severity           |
| `ayame-spell.fixOnSave`          | `false`   | Apply safe fixes before save          |
| `ayame-spell.trace.server`       | `off`     | LSP message tracing                   |

For team settings, prefer `ayame-spell.toml`:

```toml
[check]
mode = "corrections"

[words]
project = "ayame-words.txt"
dictionaries = []

[japanese]
enabled = true
katakana-style = "consistency"
```

The full reference is in the
[project README](https://github.com/ayame-editor/ayame-spell#configuration).

## Diagnostics and troubleshooting

Open **Output: ayame-spell** for the selected executable, server version,
configuration reload warnings, dictionary downloads, and LSP logs. The extension
warns when its major/minor version does not match a custom server.

If startup fails:

1. install the VSIX matching the extension host architecture;
2. check `ayame-spell.serverPath` if it is set;
3. run `ayame-spell --version` on that host; and
4. run **ayame-spell: Restart Server**.

The extension is limited in untrusted workspaces: diagnostics remain available,
while commands that write configuration or wordlists require a trusted
workspace. Virtual workspaces are not supported because the native server needs
a filesystem.

## Development and publishing

See the
[publishing runbook](https://github.com/ayame-editor/ayame-spell/blob/main/editors/vscode/PUBLISHING.md)
for local builds, integration tests, platform VSIX packaging, Marketplace/Open
VSX credentials, and the release checklist. Listing copy and asset inventory
live in the
[Marketplace kit](https://github.com/ayame-editor/ayame-spell/blob/main/editors/vscode/MARKETPLACE.md).

## License

MIT OR Apache-2.0. Bundled and derived data notices are in the
[project NOTICE](https://github.com/ayame-editor/ayame-spell/blob/main/NOTICE.md).
