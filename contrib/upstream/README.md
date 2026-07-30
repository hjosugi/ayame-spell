# Prepared upstream contributions

These patches keep the external-repository portions of the editor and schema
issues reproducible without making remote changes from this repository.

| Target | Patch | Prepared against |
| --- | --- | --- |
| `neovim/nvim-lspconfig` | `nvim-lspconfig-ayame-spell.patch` | `b7b920947f21339ee41fbb38c79d6445e12900aa` |
| `helix-editor/helix` | `helix-ayame-spell.patch` | `079a789e8cb08ead67f19e1971a1b7438b37354b` |
| `SchemaStore/schemastore` | `schemastore-ayame-spell.patch` | `8a7f1de10fb52fef096aa5f199fd5ba30abdba8a` |

Apply a patch from the root of a fresh clone:

```sh
git apply --check /path/to/contrib/upstream/nvim-lspconfig-ayame-spell.patch
git apply /path/to/contrib/upstream/nvim-lspconfig-ayame-spell.patch
```

The Neovim patch adds a complete `vim.lsp.config` definition. The Helix patch
registers the server so users can add `"ayame-spell"` to any language's
`language-servers` array. The SchemaStore patch registers the versioned,
self-hosted schema for both supported configuration filenames.

Publishing these patches requires a separate session rooted in the respective
upstream repository, or a manual pull request by the maintainer.
