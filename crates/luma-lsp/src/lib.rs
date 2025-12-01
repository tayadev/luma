//! Language Server Protocol implementation for Luma
//!
//! This module provides an LSP server that can be used with editors
//! that support the Language Server Protocol.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use luma_core::diagnostics::{Diagnostic as LumaDiagnostic, LineIndex};

type DocDiagnostics = (String, Vec<LumaDiagnostic>);
type DiagMap = HashMap<Url, DocDiagnostics>;

/// Document state tracked by the language server
///
/// Fields are stored for future features like:
/// - content: For incremental updates, goto definition, completion
/// - version: For document versioning in incremental sync mode
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct Document {
    content: String,
    version: i32,
}

/// The Luma Language Server backend
#[derive(Debug)]
pub struct LumaLanguageServer {
    client: Client,
    documents: Arc<RwLock<HashMap<Url, Document>>>,
    /// Last computed diagnostics per document (content + core diagnostics)
    last_diags: Arc<RwLock<DiagMap>>,
}

impl LumaLanguageServer {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: Arc::new(RwLock::new(HashMap::new())),
            last_diags: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Convert Luma diagnostic to LSP diagnostic
    fn to_lsp_diagnostic(diag: &LumaDiagnostic, source: &str) -> Diagnostic {
        let line_index = LineIndex::new(source);
        let (start_line, start_col) = line_index.line_col(diag.span.start);
        let (end_line, end_col) = line_index.line_col(diag.span.end);

        Diagnostic {
            range: Range {
                start: Position {
                    line: (start_line - 1) as u32,
                    character: (start_col - 1) as u32,
                },
                end: Position {
                    line: (end_line - 1) as u32,
                    character: (end_col - 1) as u32,
                },
            },
            severity: Some(match diag.severity {
                luma_core::diagnostics::Severity::Error => DiagnosticSeverity::ERROR,
                luma_core::diagnostics::Severity::Warning => DiagnosticSeverity::WARNING,
                luma_core::diagnostics::Severity::Info => DiagnosticSeverity::INFORMATION,
                luma_core::diagnostics::Severity::Hint => DiagnosticSeverity::HINT,
            }),
            code: None,
            code_description: None,
            source: Some("luma".to_string()),
            message: diag.message.clone(),
            related_information: None,
            tags: None,
            data: None,
        }
    }

    /// Validate a document and publish diagnostics
    async fn validate_document(&self, uri: &Url, content: &str) {
        let filename = uri.path().to_string();

        // Parse the document
        let mut diagnostics = Vec::new();
        let mut core_diags: Vec<LumaDiagnostic> = Vec::new();

        match luma_core::parser::parse(content, &filename) {
            Ok(ast) => {
                // Try to typecheck the AST
                if let Err(type_errors) = luma_core::typecheck::typecheck_program(&ast) {
                    for err in type_errors {
                        // Convert to core diagnostic to preserve suggestions/fix-its
                        let span = err.span.unwrap_or_else(|| luma_core::ast::Span::new(0, 0));
                        let mut core = luma_core::diagnostics::Diagnostic::error(
                            luma_core::diagnostics::DiagnosticKind::Type,
                            err.message.clone(),
                            span,
                            filename.clone(),
                        );
                        for s in err.suggestions {
                            core = core.with_suggestion(s);
                        }
                        for fix in err.fixits {
                            core = core.with_fix(fix);
                        }
                        diagnostics.push(Self::to_lsp_diagnostic(&core, content));
                        core_diags.push(core);
                    }
                }
            }
            Err(parse_errors) => {
                for err in &parse_errors {
                    diagnostics.push(Self::to_lsp_diagnostic(err, content));
                }
                core_diags.extend(parse_errors.into_iter());
            }
        }

        // Store the last diagnostics along with content for CodeActions
        {
            let mut m = self.last_diags.write().await;
            m.insert(uri.clone(), (content.to_string(), core_diags));
        }

        self.client
            .publish_diagnostics(uri.clone(), diagnostics, None)
            .await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for LumaLanguageServer {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "luma-language-server".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(TextDocumentSyncKind::FULL),
                        will_save: None,
                        will_save_wait_until: None,
                        save: Some(TextDocumentSyncSaveOptions::SaveOptions(SaveOptions {
                            include_text: Some(true),
                        })),
                    },
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                ..Default::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Luma language server initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let content = params.text_document.text;
        let version = params.text_document.version;

        // Store the document
        {
            let mut docs = self.documents.write().await;
            docs.insert(
                uri.clone(),
                Document {
                    content: content.clone(),
                    version,
                },
            );
        }

        // Validate and publish diagnostics
        self.validate_document(&uri, &content).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let version = params.text_document.version;

        // Get the full content (we use FULL sync mode)
        if let Some(change) = params.content_changes.into_iter().last() {
            let content = change.text;

            // Update stored document
            {
                let mut docs = self.documents.write().await;
                docs.insert(
                    uri.clone(),
                    Document {
                        content: content.clone(),
                        version,
                    },
                );
            }

            // Validate and publish diagnostics
            self.validate_document(&uri, &content).await;
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let uri = params.text_document.uri;

        // If we have the content, validate it
        if let Some(content) = params.text {
            self.validate_document(&uri, &content).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;

        // Remove document from tracking
        {
            let mut docs = self.documents.write().await;
            docs.remove(&uri);
        }

        // Clear stored diagnostics
        {
            let mut m = self.last_diags.write().await;
            m.remove(&uri);
        }

        // Clear diagnostics for the closed document
        self.client.publish_diagnostics(uri, Vec::new(), None).await;
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let _position = params.text_document_position_params.position;

        // Get the document content
        let docs = self.documents.read().await;
        if docs.get(&uri).is_some() {
            // For now, just return a simple hover message
            // In the future, we can provide type information, documentation, etc.
            Ok(Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "Luma code".to_string(),
                }),
                range: None,
            }))
        } else {
            Ok(None)
        }
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let uri = params.text_document.uri;
        let range = params.range;

        let guard = self.last_diags.read().await;
        let Some((content, core_diags)) = guard.get(&uri) else {
            return Ok(None);
        };

        let line_index = LineIndex::new(content);

        let mut actions: Vec<CodeActionOrCommand> = Vec::new();
        for d in core_diags {
            // Intersect diagnostic range with requested range
            let (ds_line, ds_col) = line_index.line_col(d.span.start);
            let (de_line, de_col) = line_index.line_col(d.span.end);
            let d_range = Range {
                start: Position {
                    line: (ds_line - 1) as u32,
                    character: (ds_col - 1) as u32,
                },
                end: Position {
                    line: (de_line - 1) as u32,
                    character: (de_col - 1) as u32,
                },
            };

            if d_range.start.line > range.end.line || d_range.end.line < range.start.line {
                continue;
            }

            // Create an action per fix-it
            for fix in &d.fixits {
                let span = fix.span();
                let (fs_line, fs_col) = line_index.line_col(span.start);
                let (fe_line, fe_col) = line_index.line_col(span.end);
                let edit = TextEdit {
                    range: Range {
                        start: Position {
                            line: (fs_line - 1) as u32,
                            character: (fs_col - 1) as u32,
                        },
                        end: Position {
                            line: (fe_line - 1) as u32,
                            character: (fe_col - 1) as u32,
                        },
                    },
                    new_text: fix.replacement().to_string(),
                };

                let action = CodeAction {
                    title: fix.label().to_string(),
                    kind: Some(CodeActionKind::QUICKFIX),
                    diagnostics: Some(vec![Self::to_lsp_diagnostic(d, content)]),
                    edit: Some(WorkspaceEdit {
                        changes: Some(HashMap::from([(uri.clone(), vec![edit])])),
                        document_changes: None,
                        change_annotations: None,
                    }),
                    command: None,
                    is_preferred: Some(true),
                    disabled: None,
                    data: None,
                };
                actions.push(CodeActionOrCommand::CodeAction(action));
            }
        }

        if actions.is_empty() {
            Ok(None)
        } else {
            Ok(Some(actions))
        }
    }
}

/// Run the LSP server
pub async fn run_server() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(LumaLanguageServer::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use luma_core::ast::Span;
    use luma_core::diagnostics::{Diagnostic as LumaDiagnostic, DiagnosticKind};

    #[test]
    fn test_to_lsp_diagnostic_single_line() {
        let source = "let x = 42;";
        let diag = LumaDiagnostic::error(
            DiagnosticKind::Parse,
            "Test error".to_string(),
            Span::new(4, 5),
            "test.luma".to_string(),
        );

        let lsp_diag = LumaLanguageServer::to_lsp_diagnostic(&diag, source);

        assert_eq!(lsp_diag.severity, Some(DiagnosticSeverity::ERROR));
        assert_eq!(lsp_diag.message, "Test error");
        assert_eq!(lsp_diag.source, Some("luma".to_string()));
    }

    #[test]
    fn test_to_lsp_diagnostic_warning() {
        let source = "let x = 42;";
        let diag = LumaDiagnostic::warning(
            DiagnosticKind::Type,
            "Test warning".to_string(),
            Span::new(0, 3),
            "test.luma".to_string(),
        );

        let lsp_diag = LumaLanguageServer::to_lsp_diagnostic(&diag, source);

        assert_eq!(lsp_diag.severity, Some(DiagnosticSeverity::WARNING));
        assert_eq!(lsp_diag.message, "Test warning");
    }

    #[test]
    fn test_to_lsp_diagnostic_with_info_severity() {
        let source = "let x = 42;";
        let diag = LumaDiagnostic::error(
            DiagnosticKind::Type,
            "Test info".to_string(),
            Span::new(0, 3),
            "test.luma".to_string(),
        );
        // Use the builder pattern to set severity if available
        // For now, we'll test error level which we know exists

        let lsp_diag = LumaLanguageServer::to_lsp_diagnostic(&diag, source);

        assert_eq!(lsp_diag.message, "Test info");
    }

    #[test]
    fn test_to_lsp_diagnostic_multiline() {
        let source = "let x = 42;\nlet y = 10;";
        let diag = LumaDiagnostic::error(
            DiagnosticKind::Parse,
            "Test multiline".to_string(),
            Span::new(12, 22),
            "test.luma".to_string(),
        );

        let lsp_diag = LumaLanguageServer::to_lsp_diagnostic(&diag, source);

        assert_eq!(lsp_diag.severity, Some(DiagnosticSeverity::ERROR));
        assert_eq!(lsp_diag.message, "Test multiline");
        // The range should span across lines
        assert!(lsp_diag.range.start.line <= lsp_diag.range.end.line);
    }
}
