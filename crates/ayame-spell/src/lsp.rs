//! LSP server. Runs over stdio (`ayame-spell lsp`).
//!
//! Design goals, learned from the shortcomings of existing tools:
//! - words are addable from the editor (project / global / ignore-list
//!   quick fixes) — no manual config editing;
//! - config and word-file changes hot-reload — no server restart;
//! - large documents degrade loudly (log message), never silently.

use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Context;
use ayame_spell_core::config::LoadedConfig;
use ayame_spell_core::{Checker, Issue, IssueKind};
use lsp_server::{Connection, Message, RequestId, Response};
use lsp_types::notification::Notification as _;
use lsp_types::request::Request as _;
use lsp_types::{
    CodeActionOrCommand, CodeActionParams, Command, Diagnostic, DiagnosticSeverity,
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    ExecuteCommandParams, InitializeParams, NumberOrString, Position, PublishDiagnosticsParams,
    Range, TextEdit, Url, WorkspaceEdit,
};
use serde::Deserialize;

const CMD_ADD_WORDS: &str = "ayame-spell.addWords";
const CMD_IGNORE_WORDS: &str = "ayame-spell.ignoreWords";
const CMD_FIX_ALL: &str = "ayame-spell.fixAll";

/// Diagnostics cap per document, so a giant generated file cannot flood
/// the editor. The cap is reported via a log message when hit.
const MAX_DIAGNOSTICS: usize = 1000;

/// Documents larger than this are only checked on open/save, not on every
/// keystroke.
const LARGE_DOC_BYTES: usize = 4 * 1024 * 1024;

struct Doc {
    text: String,
    issues: Vec<Issue>,
}

struct Server {
    connection: Connection,
    root: PathBuf,
    loaded: LoadedConfig,
    checker: Checker,
    docs: HashMap<Url, Doc>,
    next_request_id: i32,
}

pub fn run() -> anyhow::Result<i32> {
    let (connection, io_threads) = Connection::stdio();

    let (init_id, init_value) = connection.initialize_start()?;
    let init: InitializeParams = serde_json::from_value(init_value)?;
    let server_capabilities = serde_json::json!({
        "capabilities": {
            "textDocumentSync": 1,
            "codeActionProvider": {
                "codeActionKinds": ["quickfix", "source.fixAll.ayame-spell"]
            },
            "executeCommandProvider": {
                "commands": [CMD_ADD_WORDS, CMD_IGNORE_WORDS, CMD_FIX_ALL]
            }
        },
        "serverInfo": { "name": "ayame-spell", "version": env!("CARGO_PKG_VERSION") }
    });
    connection.initialize_finish(init_id, server_capabilities)?;

    #[allow(deprecated)]
    let root = init
        .root_uri
        .as_ref()
        .and_then(|u| u.to_file_path().ok())
        .or_else(|| std::env::current_dir().ok())
        .context("cannot determine workspace root")?;

    let loaded = ayame_spell_core::config::discover(&root)
        .unwrap_or_else(|_| ayame_spell_core::config::defaults(&root));
    let (checker, warnings) = Checker::new(&loaded);

    let mut server = Server {
        connection,
        root,
        loaded,
        checker,
        docs: HashMap::new(),
        next_request_id: 1_000_000,
    };
    for w in warnings {
        server.log(format!("warning: {w}"));
    }
    server.register_file_watchers();
    server.main_loop()?;
    // Drop the connection so the writer thread sees a closed channel;
    // otherwise join() waits forever.
    drop(server);
    io_threads.join()?;
    Ok(0)
}

impl Server {
    fn main_loop(&mut self) -> anyhow::Result<()> {
        while let Ok(msg) = self.connection.receiver.recv() {
            match msg {
                Message::Request(req) => {
                    if self.connection.handle_shutdown(&req)? {
                        return Ok(());
                    }
                    let id = req.id.clone();
                    let result = match req.method.as_str() {
                        lsp_types::request::CodeActionRequest::METHOD => {
                            let params: CodeActionParams = serde_json::from_value(req.params)?;
                            Some(serde_json::to_value(self.code_actions(params))?)
                        }
                        lsp_types::request::ExecuteCommand::METHOD => {
                            let params: ExecuteCommandParams = serde_json::from_value(req.params)?;
                            if let Err(e) = self.execute_command(params) {
                                self.log(format!("command failed: {e:#}"));
                            }
                            Some(serde_json::Value::Null)
                        }
                        _ => None,
                    };
                    match result {
                        Some(value) => self.respond(Response::new_ok(id, value)),
                        None => self.respond(Response::new_err(
                            id,
                            lsp_server::ErrorCode::MethodNotFound as i32,
                            format!("unhandled method {}", req.method),
                        )),
                    }
                }
                Message::Notification(note) => match note.method.as_str() {
                    lsp_types::notification::DidOpenTextDocument::METHOD => {
                        let p: DidOpenTextDocumentParams = serde_json::from_value(note.params)?;
                        self.docs.insert(
                            p.text_document.uri.clone(),
                            Doc {
                                text: p.text_document.text,
                                issues: Vec::new(),
                            },
                        );
                        self.publish(&p.text_document.uri);
                    }
                    lsp_types::notification::DidChangeTextDocument::METHOD => {
                        let p: DidChangeTextDocumentParams = serde_json::from_value(note.params)?;
                        if let Some(change) = p.content_changes.into_iter().last() {
                            let uri = p.text_document.uri;
                            let large = change.text.len() > LARGE_DOC_BYTES;
                            if let Some(doc) = self.docs.get_mut(&uri) {
                                doc.text = change.text;
                            }
                            // Large docs re-check on save only.
                            if !large {
                                self.publish(&uri);
                            }
                        }
                    }
                    lsp_types::notification::DidSaveTextDocument::METHOD => {
                        #[derive(Deserialize)]
                        struct MinimalSave {
                            #[serde(rename = "textDocument")]
                            text_document: lsp_types::TextDocumentIdentifier,
                        }
                        let p: MinimalSave = serde_json::from_value(note.params)?;
                        self.publish(&p.text_document.uri);
                    }
                    lsp_types::notification::DidCloseTextDocument::METHOD => {
                        let p: DidCloseTextDocumentParams = serde_json::from_value(note.params)?;
                        self.docs.remove(&p.text_document.uri);
                        self.notify::<lsp_types::notification::PublishDiagnostics>(
                            PublishDiagnosticsParams {
                                uri: p.text_document.uri,
                                diagnostics: Vec::new(),
                                version: None,
                            },
                        );
                    }
                    lsp_types::notification::DidChangeWatchedFiles::METHOD => {
                        self.reload_config();
                    }
                    _ => {}
                },
                Message::Response(_) => {}
            }
        }
        Ok(())
    }

    fn reload_config(&mut self) {
        self.loaded = ayame_spell_core::config::discover(&self.root)
            .unwrap_or_else(|_| ayame_spell_core::config::defaults(&self.root));
        let (checker, warnings) = Checker::new(&self.loaded);
        self.checker = checker;
        for w in warnings {
            self.log(format!("warning: {w}"));
        }
        let uris: Vec<Url> = self.docs.keys().cloned().collect();
        for uri in uris {
            self.publish(&uri);
        }
    }

    fn rel_path(&self, uri: &Url) -> Option<PathBuf> {
        let p = uri.to_file_path().ok()?;
        Some(
            p.strip_prefix(&self.root)
                .map(|r| r.to_path_buf())
                .unwrap_or(p),
        )
    }

    fn publish(&mut self, uri: &Url) {
        let rel = self.rel_path(uri);
        let Some(doc) = self.docs.get_mut(uri) else {
            return;
        };
        let issues = self.checker.check(&doc.text, rel.as_deref());
        let lines: Vec<&str> = doc.text.split('\n').collect();
        let capped = issues.len() > MAX_DIAGNOSTICS;
        let diagnostics: Vec<Diagnostic> = issues
            .iter()
            .take(MAX_DIAGNOSTICS)
            .map(|issue| Diagnostic {
                range: issue_range(&lines, issue),
                severity: Some(match issue.kind {
                    IssueKind::Typo => DiagnosticSeverity::WARNING,
                    _ => DiagnosticSeverity::INFORMATION,
                }),
                code: Some(NumberOrString::String(issue.kind.code().to_string())),
                source: Some("ayame-spell".to_string()),
                message: issue.message(),
                ..Diagnostic::default()
            })
            .collect();
        doc.issues = issues;
        let uri = uri.clone();
        if capped {
            self.log(format!(
                "{uri}: more than {MAX_DIAGNOSTICS} findings; diagnostics were capped"
            ));
        }
        self.notify::<lsp_types::notification::PublishDiagnostics>(PublishDiagnosticsParams {
            uri,
            diagnostics,
            version: None,
        });
    }

    fn code_actions(&self, params: CodeActionParams) -> Vec<CodeActionOrCommand> {
        let uri = params.text_document.uri;
        let Some(doc) = self.docs.get(&uri) else {
            return Vec::new();
        };
        let lines: Vec<&str> = doc.text.split('\n').collect();
        let mut actions = Vec::new();

        let overlapping: Vec<&Issue> = doc
            .issues
            .iter()
            .filter(|i| ranges_overlap(&issue_range(&lines, i), &params.range))
            .take(3)
            .collect();

        for issue in &overlapping {
            let range = issue_range(&lines, issue);
            for (n, suggestion) in issue.suggestions.iter().take(5).enumerate() {
                let edit = WorkspaceEdit {
                    changes: Some(
                        [(
                            uri.clone(),
                            vec![TextEdit {
                                range,
                                new_text: suggestion.clone(),
                            }],
                        )]
                        .into_iter()
                        .collect(),
                    ),
                    ..WorkspaceEdit::default()
                };
                actions.push(CodeActionOrCommand::CodeAction(lsp_types::CodeAction {
                    title: format!("Change to \"{suggestion}\""),
                    kind: Some(lsp_types::CodeActionKind::QUICKFIX),
                    edit: Some(edit),
                    is_preferred: Some(n == 0 && issue.suggestions.len() == 1),
                    ..lsp_types::CodeAction::default()
                }));
            }
            if matches!(
                issue.kind,
                IssueKind::Typo | IssueKind::UnknownWord | IssueKind::JaVariant
            ) {
                let word = issue.word.clone();
                let word_arg = serde_json::json!({ "words": [word] });
                actions.push(CodeActionOrCommand::Command(Command {
                    title: format!("Add \"{word}\" to project words"),
                    command: CMD_ADD_WORDS.to_string(),
                    arguments: Some(vec![serde_json::json!({
                        "words": [word], "scope": "project"
                    })]),
                }));
                actions.push(CodeActionOrCommand::Command(Command {
                    title: format!("Add \"{word}\" to global words"),
                    command: CMD_ADD_WORDS.to_string(),
                    arguments: Some(vec![serde_json::json!({
                        "words": [word], "scope": "global"
                    })]),
                }));
                actions.push(CodeActionOrCommand::Command(Command {
                    title: format!("Ignore \"{word}\" in this project (ayame-spell.toml)"),
                    command: CMD_IGNORE_WORDS.to_string(),
                    arguments: Some(vec![word_arg]),
                }));
            }
        }

        if doc.issues.iter().any(|i| i.safe_fix().is_some()) {
            actions.push(CodeActionOrCommand::Command(Command {
                title: "ayame-spell: fix all safe issues in file".to_string(),
                command: CMD_FIX_ALL.to_string(),
                arguments: Some(vec![serde_json::json!({ "uri": uri })]),
            }));
        }
        actions
    }

    fn execute_command(&mut self, params: ExecuteCommandParams) -> anyhow::Result<()> {
        match params.command.as_str() {
            CMD_ADD_WORDS => {
                #[derive(Deserialize)]
                struct Args {
                    words: Vec<String>,
                    #[serde(default)]
                    scope: Option<String>,
                }
                let args: Args = first_arg(&params)?;
                let path = if args.scope.as_deref() == Some("global") {
                    ayame_spell_core::global_words_path()
                        .context("cannot determine the global config directory")?
                } else {
                    self.loaded.project_words_path()
                };
                crate::words::append_words(&path, &args.words)?;
                self.checker.allow_words(&args.words);
                self.republish_all();
            }
            CMD_IGNORE_WORDS => {
                #[derive(Deserialize)]
                struct Args {
                    words: Vec<String>,
                }
                let args: Args = first_arg(&params)?;
                crate::words::add_to_string_array(&self.loaded, "words", "ignore", &args.words)?;
                self.checker.allow_words(&args.words);
                self.republish_all();
            }
            CMD_FIX_ALL => {
                #[derive(Deserialize)]
                struct Args {
                    uri: Url,
                }
                let args: Args = first_arg(&params)?;
                let Some(doc) = self.docs.get(&args.uri) else {
                    return Ok(());
                };
                let lines: Vec<&str> = doc.text.split('\n').collect();
                let edits: Vec<TextEdit> = doc
                    .issues
                    .iter()
                    .filter_map(|i| {
                        i.safe_fix().map(|fix| TextEdit {
                            range: issue_range(&lines, i),
                            new_text: fix.to_string(),
                        })
                    })
                    .collect();
                if edits.is_empty() {
                    return Ok(());
                }
                let edit = WorkspaceEdit {
                    changes: Some([(args.uri, edits)].into_iter().collect()),
                    ..WorkspaceEdit::default()
                };
                self.request::<lsp_types::request::ApplyWorkspaceEdit>(
                    lsp_types::ApplyWorkspaceEditParams { label: None, edit },
                );
            }
            other => anyhow::bail!("unknown command {other}"),
        }
        Ok(())
    }

    fn republish_all(&mut self) {
        let uris: Vec<Url> = self.docs.keys().cloned().collect();
        for uri in uris {
            self.publish(&uri);
        }
    }

    fn register_file_watchers(&mut self) {
        let words_file = self
            .loaded
            .project_words_path()
            .file_name()
            .and_then(|n| n.to_str().map(str::to_string))
            .unwrap_or_else(|| "ayame-words.txt".to_string());
        let pattern = format!("**/{{ayame-spell.toml,.ayame-spell.toml,{words_file}}}");
        let options = serde_json::json!({
            "watchers": [{ "globPattern": pattern }]
        });
        self.request::<lsp_types::request::RegisterCapability>(lsp_types::RegistrationParams {
            registrations: vec![lsp_types::Registration {
                id: "ayame-spell-watch".to_string(),
                method: "workspace/didChangeWatchedFiles".to_string(),
                register_options: Some(options),
            }],
        });
    }

    fn respond(&self, response: Response) {
        let _ = self.connection.sender.send(Message::Response(response));
    }

    fn notify<N: lsp_types::notification::Notification>(&self, params: N::Params) {
        let note = lsp_server::Notification::new(N::METHOD.to_string(), params);
        let _ = self.connection.sender.send(Message::Notification(note));
    }

    fn request<R: lsp_types::request::Request>(&mut self, params: R::Params) {
        let id = RequestId::from(self.next_request_id);
        self.next_request_id += 1;
        let req = lsp_server::Request::new(id, R::METHOD.to_string(), params);
        let _ = self.connection.sender.send(Message::Request(req));
    }

    fn log(&self, message: String) {
        self.notify::<lsp_types::notification::LogMessage>(lsp_types::LogMessageParams {
            typ: lsp_types::MessageType::WARNING,
            message,
        });
    }
}

fn first_arg<T: serde::de::DeserializeOwned>(params: &ExecuteCommandParams) -> anyhow::Result<T> {
    let value = params
        .arguments
        .first()
        .cloned()
        .context("missing command argument")?;
    Ok(serde_json::from_value(value)?)
}

fn utf16_len(s: &str) -> u32 {
    s.encode_utf16().count() as u32
}

fn issue_range(lines: &[&str], issue: &Issue) -> Range {
    let line_idx = issue.line.saturating_sub(1);
    let line_text = lines.get(line_idx as usize).copied().unwrap_or("");
    let col = issue.col.min(line_text.len());
    let end = (issue.col + issue.len).min(line_text.len());
    let start_char = utf16_len(&line_text[..col]);
    let end_char = start_char + utf16_len(&line_text[col..end]);
    Range {
        start: Position {
            line: line_idx,
            character: start_char,
        },
        end: Position {
            line: line_idx,
            character: end_char,
        },
    }
}

fn ranges_overlap(a: &Range, b: &Range) -> bool {
    let starts_before_end = (a.start.line, a.start.character) <= (b.end.line, b.end.character);
    let ends_after_start = (a.end.line, a.end.character) >= (b.start.line, b.start.character);
    starts_before_end && ends_after_start
}
