# Security Policy

## Supported versions

Security fixes target the latest released version. Older pre-1.0 releases may
receive a fix only when it is practical; upgrade before reporting a result as
unfixed.

| Version | Support |
| --- | --- |
| Latest release | Supported |
| Older releases | Best effort |

## Threat model

ayame-spell reads source text, configuration, and dictionaries; its `fix`
command can rewrite explicitly selected files. The language server receives
document contents from the editor. The project does not intentionally collect
telemetry or send checked document contents to a service.

Registry operations download an index and dictionary artifacts over HTTPS and
verify each artifact against the SHA-256 digest in the index. This detects an
artifact that differs from the fetched index, but it does not protect against a
registry origin that is itself compromised. For stronger supply-chain control,
review and vendor dictionaries or point `AYAME_SPELL_REGISTRY` at a registry
you operate.

Security-sensitive areas include:

- path traversal or unintended file writes in `fix`, configuration, caches, or
  registry installation;
- tampered registry artifacts, digest bypasses, and unsafe archive handling;
- denial of service or memory exhaustion from untrusted documents or
  dictionaries;
- LSP messages that crash the server or expose document contents; and
- vulnerable or incorrectly licensed dependencies and release artifacts.

Project configuration, locally supplied dictionaries, editor extensions, and
the machine running ayame-spell are trust boundaries controlled by the user.

## Reporting a vulnerability

Use [GitHub private vulnerability reporting](https://github.com/hjosugi/ayame-spell/security/advisories/new).
Do not open a public issue for an undisclosed vulnerability.

Include the affected version, operating system, impact, minimal reproduction,
and any known workaround. Remove unrelated private data and credentials.
Maintainers will acknowledge and triage reports on a best-effort basis, discuss
coordinated disclosure with the reporter, and credit reporters who want public
credit. Please allow time for a fixed release before public disclosure.

## 日本語

セキュリティ修正は最新リリースを対象とします。古い pre-1.0 リリースは best
effort です。ayame-spell はソース、設定、辞書を読み、`fix` は明示的に指定した
ファイルを書き換えます。意図的な telemetry 収集や、チェック対象文書の外部送信は
行いません。

レジストリは HTTPS で index と辞書を取得し、index 内の SHA-256 digest と照合
します。この検証は取得した index 自体が侵害された場合の真正性を保証しません。
より強い supply-chain 管理が必要な場合は辞書を review して vendor するか、
自分で運用するレジストリを `AYAME_SPELL_REGISTRY` に指定してください。

未公開の脆弱性は
[GitHub の非公開 vulnerability report](https://github.com/hjosugi/ayame-spell/security/advisories/new)
で報告し、public issue には投稿しないでください。対象 version、OS、影響、
最小再現、既知の回避策を含め、無関係な個人情報と認証情報は除いてください。
