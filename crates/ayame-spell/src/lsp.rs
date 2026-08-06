//! LSP server. Runs over stdio (`ayame-spell lsp`).
//!
//! Design goals, learned from the shortcomings of existing tools:
//! - words are addable from the editor (project / global / ignore-list
//!   quick fixes) — no manual config editing;
//! - config and word-file changes hot-reload — no server restart;
//! - large documents degrade loudly (log message), never silently.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use anyhow::Context;
use ayame_spell_core::config::{LoadedConfig, Mode};
use ayame_spell_core::{Checker, Issue, IssueKind};
use lsp_server::{Connection, Message, RequestId, Response};
use lsp_types::notification::Notification as _;
use lsp_types::request::Request as _;
use lsp_types::{
    CodeActionOrCommand, CodeActionParams, Command, Diagnostic, DiagnosticSeverity,
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    DocumentDiagnosticParams, DocumentDiagnosticReport, DocumentDiagnosticReportResult,
    ExecuteCommandParams, FullDocumentDiagnosticReport, Hover, HoverContents, HoverParams,
    InitializeParams, MarkupContent, MarkupKind, NumberOrString, Position,
    PublishDiagnosticsParams, Range, RelatedFullDocumentDiagnosticReport, TextEdit, Uri,
    WorkspaceDiagnosticParams, WorkspaceDiagnosticReport, WorkspaceDiagnosticReportResult,
    WorkspaceDocumentDiagnosticReport, WorkspaceEdit, WorkspaceFullDocumentDiagnosticReport,
};
use serde::Deserialize;

const CMD_ADD_WORDS: &str = "ayame-spell.addWords";
const CMD_IGNORE_WORDS: &str = "ayame-spell.ignoreWords";
const CMD_ADD_CORRECTION: &str = "ayame-spell.addCorrection";
const CMD_FIX_ALL: &str = "ayame-spell.server.fixAll";

/// Diagnostics cap per document, so a giant generated file cannot flood
/// the editor. The cap is reported via a log message when hit.
const MAX_DIAGNOSTICS: usize = 1000;

/// Documents larger than this are only checked on open/save, not on every
/// keystroke.
const LARGE_DOC_BYTES: usize = 4 * 1024 * 1024;
const DEFAULT_DEBOUNCE_MS: u64 = 150;

struct Doc {
    text: String,
    issues: Vec<Issue>,
    version: i32,
    large_warning_shown: bool,
}

struct Server {
    connection: Connection,
    root: PathBuf,
    loaded: LoadedConfig,
    checker: Checker,
    editor_options: EditorOptions,
    docs: HashMap<Uri, Doc>,
    pending: HashMap<Uri, Instant>,
    cancelled_requests: HashSet<RequestId>,
    next_request_id: i32,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EditorOptions {
    mode: Option<Mode>,
    japanese_enabled: Option<bool>,
    diagnostic_severity: Option<EditorDiagnosticSeverity>,
    debounce_ms: Option<u64>,
    locale: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
enum EditorDiagnosticSeverity {
    Error,
    Warning,
    Information,
    Hint,
}

impl EditorDiagnosticSeverity {
    fn lsp(self) -> DiagnosticSeverity {
        match self {
            Self::Error => DiagnosticSeverity::ERROR,
            Self::Warning => DiagnosticSeverity::WARNING,
            Self::Information => DiagnosticSeverity::INFORMATION,
            Self::Hint => DiagnosticSeverity::HINT,
        }
    }
}

fn apply_editor_options(loaded: &mut LoadedConfig, options: &EditorOptions) {
    if let Some(mode) = options.mode {
        loaded.config.check.mode = mode;
    }
    if let Some(enabled) = options.japanese_enabled {
        loaded.config.japanese.enabled = enabled;
    }
}

pub fn run() -> anyhow::Result<i32> {
    let (connection, io_threads) = Connection::stdio();

    let (init_id, init_value) = connection.initialize_start()?;
    let init: InitializeParams = serde_json::from_value(init_value)?;
    let server_capabilities = serde_json::json!({
        "capabilities": {
            "textDocumentSync": {
                "openClose": true,
                "change": 2,
                "save": true
            },
            "diagnosticProvider": {
                "identifier": "ayame-spell",
                "interFileDependencies": true,
                "workspaceDiagnostics": true
            },
            "hoverProvider": true,
            "codeActionProvider": {
                "codeActionKinds": ["quickfix", "source.fixAll.ayame-spell"]
            },
            "executeCommandProvider": {
                "commands": [
                    CMD_ADD_WORDS,
                    CMD_IGNORE_WORDS,
                    CMD_ADD_CORRECTION,
                    CMD_FIX_ALL
                ]
            }
        },
        "serverInfo": { "name": "ayame-spell", "version": env!("CARGO_PKG_VERSION") }
    });
    connection.initialize_finish(init_id, server_capabilities)?;

    #[allow(deprecated)]
    let root = init
        .workspace_folders
        .as_ref()
        .and_then(|folders| folders.first())
        .and_then(|folder| crate::file_uri::to_path(&folder.uri))
        .or_else(|| init.root_uri.as_ref().and_then(crate::file_uri::to_path))
        .or_else(|| std::env::current_dir().ok())
        .context("cannot determine workspace root")?;

    let editor_options: EditorOptions = init
        .initialization_options
        .and_then(|value| serde_json::from_value(value).ok())
        .unwrap_or_default();
    let mut loaded = ayame_spell_core::config::discover(&root)
        .unwrap_or_else(|_| ayame_spell_core::config::defaults(&root));
    apply_editor_options(&mut loaded, &editor_options);
    let (checker, warnings) = Checker::new(&loaded);

    let mut server = Server {
        connection,
        root,
        loaded,
        checker,
        editor_options,
        docs: HashMap::new(),
        pending: HashMap::new(),
        cancelled_requests: HashSet::new(),
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
        loop {
            let msg = if self.pending.is_empty() {
                match self.connection.receiver.recv() {
                    Ok(message) => message,
                    Err(_) => return Ok(()),
                }
            } else {
                let now = Instant::now();
                let wait = self
                    .pending
                    .values()
                    .min()
                    .map_or(Duration::ZERO, |deadline| {
                        deadline.saturating_duration_since(now)
                    });
                match self.connection.receiver.recv_timeout(wait) {
                    Ok(message) => message,
                    Err(crossbeam_channel::RecvTimeoutError::Timeout) => {
                        self.publish_due();
                        continue;
                    }
                    Err(crossbeam_channel::RecvTimeoutError::Disconnected) => return Ok(()),
                }
            };
            match msg {
                Message::Request(req) => {
                    if self.connection.handle_shutdown(&req)? {
                        return Ok(());
                    }
                    let id = req.id.clone();
                    if self.cancelled_requests.remove(&id) {
                        self.respond(Response::new_err(
                            id,
                            lsp_server::ErrorCode::RequestCanceled as i32,
                            "request cancelled".to_string(),
                        ));
                        continue;
                    }
                    let response = match req.method.as_str() {
                        lsp_types::request::CodeActionRequest::METHOD => {
                            let params: CodeActionParams = serde_json::from_value(req.params)?;
                            Response::new_ok(id, self.code_actions(params))
                        }
                        lsp_types::request::ExecuteCommand::METHOD => {
                            let params: ExecuteCommandParams = serde_json::from_value(req.params)?;
                            match self.execute_command(params) {
                                Ok(()) => Response::new_ok(id, serde_json::Value::Null),
                                Err(error) => Response::new_err(
                                    id,
                                    lsp_server::ErrorCode::InvalidParams as i32,
                                    format!("{error:#}"),
                                ),
                            }
                        }
                        lsp_types::request::HoverRequest::METHOD => {
                            let params: HoverParams = serde_json::from_value(req.params)?;
                            Response::new_ok(id, self.hover(params))
                        }
                        lsp_types::request::DocumentDiagnosticRequest::METHOD => {
                            let params: DocumentDiagnosticParams =
                                serde_json::from_value(req.params)?;
                            Response::new_ok(id, self.document_diagnostics(params))
                        }
                        lsp_types::request::WorkspaceDiagnosticRequest::METHOD => {
                            let params: WorkspaceDiagnosticParams =
                                serde_json::from_value(req.params)?;
                            Response::new_ok(id, self.workspace_diagnostics(params))
                        }
                        _ => Response::new_err(
                            id,
                            lsp_server::ErrorCode::MethodNotFound as i32,
                            format!("unhandled method {}", req.method),
                        ),
                    };
                    self.respond(response);
                }
                Message::Notification(note) => match note.method.as_str() {
                    lsp_types::notification::DidOpenTextDocument::METHOD => {
                        let p: DidOpenTextDocumentParams = serde_json::from_value(note.params)?;
                        let version = p.text_document.version;
                        self.docs.insert(
                            p.text_document.uri.clone(),
                            Doc {
                                text: p.text_document.text,
                                issues: Vec::new(),
                                version,
                                large_warning_shown: false,
                            },
                        );
                        self.publish(&p.text_document.uri);
                    }
                    lsp_types::notification::DidChangeTextDocument::METHOD => {
                        let p: DidChangeTextDocumentParams = serde_json::from_value(note.params)?;
                        let uri = p.text_document.uri;
                        if let Some(doc) = self.docs.get_mut(&uri) {
                            if let Err(error) =
                                apply_content_changes(&mut doc.text, p.content_changes)
                            {
                                let uri_text = uri.as_str();
                                self.log(format!(
                                    "{uri_text}: invalid incremental edit: {error:#}"
                                ));
                            } else {
                                doc.version = p.text_document.version;
                                doc.large_warning_shown = false;
                                self.schedule(uri);
                            }
                        }
                    }
                    lsp_types::notification::DidSaveTextDocument::METHOD => {
                        let p: lsp_types::DidSaveTextDocumentParams =
                            serde_json::from_value(note.params)?;
                        if let (Some(doc), Some(text)) =
                            (self.docs.get_mut(&p.text_document.uri), p.text)
                        {
                            doc.text = text;
                            doc.large_warning_shown = false;
                        }
                        self.publish(&p.text_document.uri);
                    }
                    lsp_types::notification::DidCloseTextDocument::METHOD => {
                        let p: DidCloseTextDocumentParams = serde_json::from_value(note.params)?;
                        self.pending.remove(&p.text_document.uri);
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
                    lsp_types::notification::Cancel::METHOD => {
                        let params: lsp_types::CancelParams = serde_json::from_value(note.params)?;
                        let id = match params.id {
                            NumberOrString::Number(id) => RequestId::from(id),
                            NumberOrString::String(id) => RequestId::from(id),
                        };
                        self.cancelled_requests.insert(id);
                    }
                    lsp_types::notification::Exit::METHOD => return Ok(()),
                    _ => {}
                },
                Message::Response(_) => {}
            }
            self.publish_due();
        }
    }

    fn reload_config(&mut self) {
        self.loaded = ayame_spell_core::config::discover(&self.root)
            .unwrap_or_else(|_| ayame_spell_core::config::defaults(&self.root));
        apply_editor_options(&mut self.loaded, &self.editor_options);
        let (checker, warnings) = Checker::new(&self.loaded);
        self.checker = checker;
        for w in warnings {
            self.log(format!("warning: {w}"));
        }
        let uris: Vec<Uri> = self.docs.keys().cloned().collect();
        for uri in uris {
            self.publish(&uri);
        }
    }

    fn schedule(&mut self, uri: Uri) {
        let debounce = self
            .editor_options
            .debounce_ms
            .unwrap_or(DEFAULT_DEBOUNCE_MS)
            .min(5_000);
        self.pending
            .insert(uri, Instant::now() + Duration::from_millis(debounce));
    }

    fn publish_due(&mut self) {
        let now = Instant::now();
        let uris: Vec<Uri> = self
            .pending
            .iter()
            .filter(|(_, deadline)| **deadline <= now)
            .map(|(uri, _)| uri.clone())
            .collect();
        for uri in uris {
            self.pending.remove(&uri);
            self.publish(&uri);
        }
    }

    fn rel_path(&self, uri: &Uri) -> Option<PathBuf> {
        let p = crate::file_uri::to_path(uri)?;
        Some(
            p.strip_prefix(&self.root)
                .map(|r| r.to_path_buf())
                .unwrap_or(p),
        )
    }

    fn analyze_document(&mut self, uri: &Uri) -> Vec<Diagnostic> {
        let rel = self.rel_path(uri);
        let max_size = if self.loaded.config.files.max_file_size == 0 {
            LARGE_DOC_BYTES
        } else {
            usize::try_from(self.loaded.config.files.max_file_size).unwrap_or(usize::MAX)
        };
        let Some(document_len) = self.docs.get(uri).map(|doc| doc.text.len()) else {
            return Vec::new();
        };
        if document_len > max_size {
            let doc = self.docs.get_mut(uri).expect("document still open");
            doc.issues.clear();
            let should_warn = !doc.large_warning_shown;
            doc.large_warning_shown = true;
            if should_warn {
                self.show_warning(format!(
                    "{}: skipped {} byte document (LSP limit: {max_size} bytes)",
                    uri.as_str(),
                    document_len
                ));
            }
            return Vec::new();
        }
        let doc = self.docs.get_mut(uri).expect("document still open");
        doc.large_warning_shown = false;
        let issues = self.checker.check(&doc.text, rel.as_deref());
        let capped = issues.len() > MAX_DIAGNOSTICS;
        let diagnostics =
            diagnostics_for(&doc.text, &issues, self.editor_options.diagnostic_severity);
        doc.issues = issues;
        if capped {
            self.log(format!(
                "{}: more than {MAX_DIAGNOSTICS} findings; diagnostics were capped",
                uri.as_str()
            ));
        }
        diagnostics
    }

    fn publish(&mut self, uri: &Uri) {
        let diagnostics = self.analyze_document(uri);
        let version = self.docs.get(uri).map(|doc| doc.version);
        self.notify::<lsp_types::notification::PublishDiagnostics>(PublishDiagnosticsParams {
            uri: uri.clone(),
            diagnostics,
            version,
        });
    }

    fn document_diagnostics(
        &mut self,
        params: DocumentDiagnosticParams,
    ) -> DocumentDiagnosticReportResult {
        let uri = params.text_document.uri;
        self.pending.remove(&uri);
        let items = self.analyze_document(&uri);
        let result_id = self.docs.get(&uri).map(|doc| doc.version.to_string());
        DocumentDiagnosticReportResult::Report(DocumentDiagnosticReport::Full(
            RelatedFullDocumentDiagnosticReport {
                related_documents: None,
                full_document_diagnostic_report: FullDocumentDiagnosticReport { result_id, items },
            },
        ))
    }

    fn workspace_diagnostics(
        &mut self,
        _params: WorkspaceDiagnosticParams,
    ) -> WorkspaceDiagnosticReportResult {
        let uris: Vec<Uri> = self.docs.keys().cloned().collect();
        let mut items = Vec::with_capacity(uris.len());
        for uri in uris {
            self.pending.remove(&uri);
            let diagnostics = self.analyze_document(&uri);
            let version = self.docs.get(&uri).map(|doc| i64::from(doc.version));
            items.push(WorkspaceDocumentDiagnosticReport::Full(
                WorkspaceFullDocumentDiagnosticReport {
                    uri,
                    version,
                    full_document_diagnostic_report: FullDocumentDiagnosticReport {
                        result_id: version.map(|value| value.to_string()),
                        items: diagnostics,
                    },
                },
            ));
        }
        WorkspaceDiagnosticReportResult::Report(WorkspaceDiagnosticReport { items })
    }

    fn hover(&mut self, params: HoverParams) -> Option<Hover> {
        let uri = params.text_document_position_params.text_document.uri;
        self.pending.remove(&uri);
        self.analyze_document(&uri);
        let doc = self.docs.get(&uri)?;
        let lines: Vec<&str> = doc.text.split('\n').collect();
        let position = params.text_document_position_params.position;
        let issue = doc
            .issues
            .iter()
            .find(|issue| range_contains(&issue_range(&lines, issue), position))?;
        let japanese = self
            .editor_options
            .locale
            .as_deref()
            .is_some_and(|locale| locale.to_ascii_lowercase().starts_with("ja"));
        let info = issue.kind.info(japanese);
        let docs = format!(
            "https://hjosugi.github.io/ayame-spell/{}/reference/rules/#{}",
            if japanese { "ja" } else { "" },
            issue.kind.code()
        )
        .replace("//reference", "/reference");
        let value = if japanese {
            format!(
                "### {} (`{}`)\n\n{}\n\n{}\n\n**設定:** `{}`  \n**例:** `{}`  \n**無視する方法:** {}  \n[ルール詳細]({docs})",
                info.title,
                issue.kind.code(),
                info.summary,
                info.explanation,
                info.config_key,
                info.example,
                info.silence
            )
        } else {
            format!(
                "### {} (`{}`)\n\n{}\n\n{}\n\n**Configuration:** `{}`  \n**Example:** `{}`  \n**How to silence:** {}  \n[Rule details]({docs})",
                info.title,
                issue.kind.code(),
                info.summary,
                info.explanation,
                info.config_key,
                info.example,
                info.silence
            )
        };
        Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value,
            }),
            range: Some(issue_range(&lines, issue)),
        })
    }

    fn code_actions(&mut self, params: CodeActionParams) -> Vec<CodeActionOrCommand> {
        let uri = params.text_document.uri;
        self.pending.remove(&uri);
        self.analyze_document(&uri);
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
            if issue.kind == IssueKind::JaVariant {
                if let Some(replacement) = issue.suggestions.first() {
                    let edits: Vec<TextEdit> = doc
                        .issues
                        .iter()
                        .filter(|candidate| {
                            candidate.kind == IssueKind::JaVariant
                                && candidate.word == issue.word
                                && candidate.suggestions.first() == Some(replacement)
                        })
                        .map(|candidate| TextEdit {
                            range: issue_range(&lines, candidate),
                            new_text: replacement.clone(),
                        })
                        .collect();
                    if !edits.is_empty() {
                        actions.push(CodeActionOrCommand::CodeAction(lsp_types::CodeAction {
                            title: format!(
                                "Normalise every \"{}\" to \"{}\" in this document",
                                issue.word, replacement
                            ),
                            kind: Some(lsp_types::CodeActionKind::QUICKFIX),
                            edit: Some(WorkspaceEdit {
                                changes: Some([(uri.clone(), edits)].into_iter().collect()),
                                ..WorkspaceEdit::default()
                            }),
                            ..lsp_types::CodeAction::default()
                        }));
                    }
                }
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
            if let Some(replacement) = issue.suggestions.first() {
                actions.push(CodeActionOrCommand::Command(Command {
                    title: format!(
                        "Add correction \"{}\" → \"{}\" to ayame-spell.toml",
                        issue.word, replacement
                    ),
                    command: CMD_ADD_CORRECTION.to_string(),
                    arguments: Some(vec![serde_json::json!({
                        "word": issue.word,
                        "replacement": replacement
                    })]),
                }));
            }

            let file_directive = directive_line(&uri, "ayame-spell:ignore-file");
            actions.push(CodeActionOrCommand::CodeAction(lsp_types::CodeAction {
                title: "Ignore findings in this file".to_string(),
                kind: Some(lsp_types::CodeActionKind::QUICKFIX),
                edit: Some(WorkspaceEdit {
                    changes: Some(
                        [(
                            uri.clone(),
                            vec![TextEdit {
                                range: Range::new(Position::new(0, 0), Position::new(0, 0)),
                                new_text: file_directive,
                            }],
                        )]
                        .into_iter()
                        .collect(),
                    ),
                    ..WorkspaceEdit::default()
                }),
                ..lsp_types::CodeAction::default()
            }));

            let line = issue.line.saturating_sub(1);
            let line_text = lines.get(line as usize).copied().unwrap_or("");
            let line_end = Position::new(line, utf16_len(line_text));
            actions.push(CodeActionOrCommand::CodeAction(lsp_types::CodeAction {
                title: "Ignore this line".to_string(),
                kind: Some(lsp_types::CodeActionKind::QUICKFIX),
                edit: Some(WorkspaceEdit {
                    changes: Some(
                        [(
                            uri.clone(),
                            vec![TextEdit {
                                range: Range::new(line_end, line_end),
                                new_text: directive_suffix(&uri).to_string(),
                            }],
                        )]
                        .into_iter()
                        .collect(),
                    ),
                    ..WorkspaceEdit::default()
                }),
                ..lsp_types::CodeAction::default()
            }));
        }

        if doc.issues.iter().any(|i| i.safe_fix().is_some()) {
            actions.push(CodeActionOrCommand::CodeAction(lsp_types::CodeAction {
                title: "ayame-spell: fix all safe issues in file".to_string(),
                kind: Some(lsp_types::CodeActionKind::new("source.fixAll.ayame-spell")),
                command: Some(Command {
                    title: "ayame-spell: fix all safe issues in file".to_string(),
                    command: CMD_FIX_ALL.to_string(),
                    arguments: Some(vec![serde_json::json!({ "uri": uri })]),
                }),
                ..lsp_types::CodeAction::default()
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
            CMD_ADD_CORRECTION => {
                #[derive(Deserialize)]
                struct Args {
                    word: String,
                    replacement: String,
                }
                let args: Args = first_arg(&params)?;
                crate::words::set_correction(&self.loaded, &args.word, &args.replacement)?;
                self.reload_config();
            }
            CMD_FIX_ALL => {
                #[derive(Deserialize)]
                struct Args {
                    uri: Uri,
                }
                let args: Args = first_arg(&params)?;
                self.pending.remove(&args.uri);
                self.analyze_document(&args.uri);
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
        let uris: Vec<Uri> = self.docs.keys().cloned().collect();
        for uri in uris {
            self.pending.remove(&uri);
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

    fn show_warning(&self, message: String) {
        self.notify::<lsp_types::notification::ShowMessage>(lsp_types::ShowMessageParams {
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

fn diagnostics_for(
    text: &str,
    issues: &[Issue],
    severity: Option<EditorDiagnosticSeverity>,
) -> Vec<Diagnostic> {
    let lines: Vec<&str> = text.split('\n').collect();
    issues
        .iter()
        .take(MAX_DIAGNOSTICS)
        .map(|issue| Diagnostic {
            range: issue_range(&lines, issue),
            severity: Some(
                severity
                    .map(EditorDiagnosticSeverity::lsp)
                    .unwrap_or_else(|| match issue.kind {
                        IssueKind::Typo => DiagnosticSeverity::WARNING,
                        _ => DiagnosticSeverity::INFORMATION,
                    }),
            ),
            code: Some(NumberOrString::String(issue.kind.code().to_string())),
            source: Some("ayame-spell".to_string()),
            message: issue.message(),
            ..Diagnostic::default()
        })
        .collect()
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

fn range_contains(range: &Range, position: Position) -> bool {
    (range.start.line, range.start.character) <= (position.line, position.character)
        && (position.line, position.character) <= (range.end.line, range.end.character)
}

fn ranges_overlap(a: &Range, b: &Range) -> bool {
    let starts_before_end = (a.start.line, a.start.character) <= (b.end.line, b.end.character);
    let ends_after_start = (a.end.line, a.end.character) >= (b.start.line, b.start.character);
    starts_before_end && ends_after_start
}

fn apply_content_changes(
    text: &mut String,
    changes: Vec<lsp_types::TextDocumentContentChangeEvent>,
) -> anyhow::Result<()> {
    for change in changes {
        if let Some(range) = change.range {
            let start = position_to_offset(text, range.start)
                .context("incremental edit start is outside the document")?;
            let end = position_to_offset(text, range.end)
                .context("incremental edit end is outside the document")?;
            anyhow::ensure!(start <= end, "incremental edit range is reversed");
            text.replace_range(start..end, &change.text);
        } else {
            *text = change.text;
        }
    }
    Ok(())
}

fn position_to_offset(text: &str, position: Position) -> Option<usize> {
    let mut line_start = 0usize;
    for _ in 0..position.line {
        let newline = text[line_start..].find('\n')?;
        line_start += newline + 1;
    }
    let line_end = text[line_start..]
        .find('\n')
        .map_or(text.len(), |offset| line_start + offset);
    let line = &text[line_start..line_end];
    let mut utf16 = 0u32;
    for (offset, character) in line.char_indices() {
        if utf16 == position.character {
            return Some(line_start + offset);
        }
        utf16 += character.len_utf16() as u32;
        if utf16 > position.character {
            return None;
        }
    }
    (utf16 == position.character).then_some(line_end)
}

fn directive_line(uri: &Uri, directive: &str) -> String {
    match uri
        .path()
        .as_str()
        .rsplit_once('.')
        .map(|(_, extension)| extension.to_ascii_lowercase())
        .as_deref()
    {
        Some("md" | "mdx" | "html" | "xml") => format!("<!-- {directive} -->\n"),
        Some("py" | "rb" | "sh" | "bash" | "zsh" | "yaml" | "yml" | "toml") => {
            format!("# {directive}\n")
        }
        _ => format!("// {directive}\n"),
    }
}

fn directive_suffix(uri: &Uri) -> &'static str {
    match uri
        .path()
        .as_str()
        .rsplit_once('.')
        .map(|(_, extension)| extension.to_ascii_lowercase())
        .as_deref()
    {
        Some("md" | "mdx" | "html" | "xml") => " <!-- ayame-spell:ignore-line -->",
        Some("py" | "rb" | "sh" | "bash" | "zsh" | "yaml" | "yml" | "toml") => {
            " # ayame-spell:ignore-line"
        }
        _ => " // ayame-spell:ignore-line",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editor_initialization_options_override_discovered_config() {
        let options: EditorOptions = serde_json::from_value(serde_json::json!({
            "mode": "dictionary",
            "japaneseEnabled": false,
            "diagnosticSeverity": "hint",
            "debounceMs": 25,
            "locale": "ja-JP"
        }))
        .unwrap();
        let mut loaded = ayame_spell_core::config::defaults(std::path::Path::new("."));

        apply_editor_options(&mut loaded, &options);

        assert_eq!(loaded.config.check.mode, Mode::Dictionary);
        assert!(!loaded.config.japanese.enabled);
        assert_eq!(
            options.diagnostic_severity,
            Some(EditorDiagnosticSeverity::Hint)
        );
        assert_eq!(
            options.diagnostic_severity.unwrap().lsp(),
            DiagnosticSeverity::HINT
        );
        assert_eq!(options.debounce_ms, Some(25));
        assert_eq!(options.locale.as_deref(), Some("ja-JP"));
    }

    #[test]
    fn incremental_changes_use_utf16_positions() {
        let mut text = "a😀b\nteh\n".to_string();
        apply_content_changes(
            &mut text,
            vec![
                lsp_types::TextDocumentContentChangeEvent {
                    range: Some(Range::new(Position::new(0, 1), Position::new(0, 3))),
                    range_length: Some(2),
                    text: "🙂".to_string(),
                },
                lsp_types::TextDocumentContentChangeEvent {
                    range: Some(Range::new(Position::new(1, 0), Position::new(1, 3))),
                    range_length: Some(3),
                    text: "the".to_string(),
                },
            ],
        )
        .unwrap();
        assert_eq!(text, "a🙂b\nthe\n");

        let mut invalid = text.clone();
        assert!(apply_content_changes(
            &mut invalid,
            vec![lsp_types::TextDocumentContentChangeEvent {
                range: Some(Range::new(Position::new(0, 2), Position::new(0, 3))),
                range_length: None,
                text: String::new(),
            }]
        )
        .is_err());
    }
}
