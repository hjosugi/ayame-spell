---
title: 環境変数とファイル配置
description: レジストリの差し替え、プロジェクト検出、XDG 対応の設定・キャッシュパス。
---

## 環境変数

| 変数 | 既定値 | 用途 |
| --- | --- | --- |
| `AYAME_SPELL_REGISTRY` | `https://hjosugi.github.io/ayame-spell/registry/index.json` | 辞書レジストリのインデックス URL を差し替える。辞書ファイルの URL はこのインデックスからの相対位置。 |
| `AYAME_SPELL_CONFIG_DIR` | OS 標準の設定ディレクトリ | ユーザー共通の `config.toml` と `words.txt` を置くディレクトリを差し替える。 |
| `AYAME_SPELL_CACHE_DIR` | OS 標準のキャッシュディレクトリ | レジストリ、補完、差分スキャンのキャッシュルートを差し替える。 |

社内ミラーの例です。

```sh
export AYAME_SPELL_REGISTRY=https://docs.example.com/spelling/index.json
ayame-spell dict --registry https://docs.example.com/spelling/index.json list
ayame-spell dict list
```

プロジェクト設定パスを指定する環境変数は現在ありません。
`ayame-spell.toml` または `.ayame-spell.toml` を置いてプロジェクトルートを
決めます。LSP クライアントではワークスペースルートが検出開始位置です。
エディター初期化オプションで `mode`、`japaneseEnabled`、
`diagnosticSeverity`、`debounceMs`（既定 150）、`locale` は上書きできますが、
設定パスは変更できません。locale は hover のルール説明言語を選びます。

## プロジェクトファイル

| ファイル | 配置 | 用途 |
| --- | --- | --- |
| `ayame-spell.toml` または `.ayame-spell.toml` | チェック対象から上位へ進んで最初に一致する場所 | プロジェクト設定と無視単語。 |
| `ayame-spell.lock` | プロジェクトルート | 解決済みのレジストリ version、ファイル、sha256。commit 対象。 |
| `ayame-words.txt` | 既定ではプロジェクトルート | チーム単語リスト。`[words].project` で変更。 |
| 相対パスの辞書 | プロジェクトルート基準 | 単語リスト、修正 TSV、日本語表記ゆれ TOML。 |

最初に一致する設定でプロジェクト検出を止めます。設定がなければ最寄りの `.git`
祖先がルートです。

## ユーザー設定とデータ

ayame-spell は Rust の `dirs` クレートが返す OS 標準ディレクトリを使います。

| 用途 | Linux / BSD | macOS | Windows |
| --- | --- | --- | --- |
| ユーザー設定 | `${XDG_CONFIG_HOME:-~/.config}/ayame-spell/config.toml` | `~/Library/Application Support/ayame-spell/config.toml` | `%APPDATA%\ayame-spell\config.toml` |
| ユーザー単語 | `${XDG_CONFIG_HOME:-~/.config}/ayame-spell/words.txt` | `~/Library/Application Support/ayame-spell/words.txt` | `%APPDATA%\ayame-spell\words.txt` |
| レジストリキャッシュ | `${XDG_CACHE_HOME:-~/.cache}/ayame-spell/dicts/` | `~/Library/Caches/ayame-spell/dicts/` | `%LOCALAPPDATA%\ayame-spell\dicts\` |
| 差分スキャンキャッシュ | `${XDG_CACHE_HOME:-~/.cache}/ayame-spell/scan/` | `~/Library/Caches/ayame-spell/scan/` | `%LOCALAPPDATA%\ayame-spell\scan\` |

ユーザー設定は任意です。`words add --global` は親ディレクトリと単語ファイルを
必要時に作成します。レジストリコマンドも必要時にキャッシュを作ります。

## 差分スキャンキャッシュ

ローカルスキャンでは、ファイルごとの指摘をキャッシュします。path、size、
ナノ秒精度の更新時刻、内容 hash、適用後の設定、読み込んだ辞書、lockfile、
ayame-spell version のいずれかが変わると無効化します。壊れた、または存在しない
entry は無視して再生成します。

コールドスキャンには `--no-cache`、明示した配置には `--cache-dir PATH`、
hit 数の確認には `-v` を使います。`CI` または `GITHUB_ACTIONS` が設定済みの
場合、`--cache-dir` を指定しない限り自動で無効になります。CI で意図的に復元
した cache を使う例です。

```sh
ayame-spell check . --cache-dir .cache/ayame-spell --format brief
```

## 参照パスの解決

- 絶対パスはそのまま使う。
- 相対パスはプロセスの作業ディレクトリではなく、プロジェクトルート基準。
- `registry:name` は `ayame-spell.lock` を通してキャッシュ内の
  `name@version.txt` に解決。`registry:name@version` は明示 version を指定。
- lock 済みの内容はロード前に sha256 を検証。
- 参照ファイルがなければ、チェッカー構築時に警告を出す。

`ayame-spell config` で検出したルートと設定パスを確認できます。
