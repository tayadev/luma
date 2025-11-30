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

use crate::diagnostics::{Diagnostic as LumaDiagnostic, LineIndex};

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
}

impl LumaLanguageServer {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: Arc::new(RwLock::new(HashMap::new())),
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
                crate::diagnostics::Severity::Error => DiagnosticSeverity::ERROR,
                crate::diagnostics::Severity::Warning => DiagnosticSeverity::WARNING,
                crate::diagnostics::Severity::Info => DiagnosticSeverity::INFORMATION,
                crate::diagnostics::Severity::Hint => DiagnosticSeverity::HINT,
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

        match crate::parser::parse(content, &filename) {
            Ok(ast) => {
                // Try to typecheck the AST
                if let Err(type_errors) = crate::typecheck::typecheck_program(&ast) {
                    for err in type_errors {
                        let (start_line, start_col, end_line, end_col) =
                            if let Some(span) = err.span {
                                let line_index = LineIndex::new(content);
                                let (start_line, start_col) = line_index.line_col(span.start);
                                let (end_line, end_col) = line_index.line_col(span.end);
                                (start_line - 1, start_col - 1, end_line - 1, end_col - 1)
                            } else {
                                (0, 0, 0, 0)
                            };

                        diagnostics.push(Diagnostic {
                            range: Range {
                                start: Position {
                                    line: start_line as u32,
                                    character: start_col as u32,
                                },
                                end: Position {
                                    line: end_line as u32,
                                    character: end_col as u32,
                                },
                            },
                            severity: Some(DiagnosticSeverity::ERROR),
                            code: None,
                            code_description: None,
                            source: Some("luma".to_string()),
                            message: err.message,
                            related_information: None,
                            tags: None,
                            data: None,
                        });
                    }
                }
            }
            Err(parse_errors) => {
                for err in &parse_errors {
                    diagnostics.push(Self::to_lsp_diagnostic(err, content));
                }
            }
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
}

/// Run the LSP server
pub async fn run_server() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(LumaLanguageServer::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}
