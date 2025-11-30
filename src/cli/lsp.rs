//! LSP server handler

use crate::lsp;

/// Run the Language Server Protocol server
pub fn handle_lsp() {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to create Tokio runtime")
        .block_on(lsp::run_server());
}
