# ayame-spell for Zed

This development extension connects Zed's built-in languages to an
`ayame-spell` binary on `PATH`.

1. Install the server:

   ```sh
   cargo install ayame-spell
   ```

2. In Zed, run **zed: install dev extension** and select this `editors/zed`
   directory.
3. If a language already has an LSP server, prioritize ayame-spell alongside
   it in `settings.json`:

   ```json
   {
     "languages": {
       "Markdown": {
         "language_servers": ["ayame-spell", "..."]
       }
     }
   }
   ```

The extension starts `ayame-spell lsp --stdio`. Project configuration is
discovered from `ayame-spell.toml`, `.ayame-spell.toml`, or the Git root.
