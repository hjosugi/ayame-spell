use zed_extension_api as zed;

struct AyameSpellExtension;

impl zed::Extension for AyameSpellExtension {
    fn new() -> Self {
        Self
    }

    fn language_server_command(
        &mut self,
        _language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> zed::Result<zed::Command> {
        let command = worktree.which("ayame-spell").ok_or_else(|| {
            "ayame-spell was not found on PATH; install it with `cargo install ayame-spell`"
                .to_string()
        })?;
        Ok(zed::Command {
            command,
            args: vec!["lsp".to_string(), "--stdio".to_string()],
            env: worktree.shell_env(),
        })
    }
}

zed::register_extension!(AyameSpellExtension);
