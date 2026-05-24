//! Entry point for the SDIF Language Server Protocol server.
//!
//! Starts a tower-lsp server communicating over stdin/stdout.

use tower_lsp::{LspService, Server};

mod backend;
mod completion;
mod diagnostics;
mod document;
mod hover;
mod semantic_tokens;

use backend::Backend;

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let (service, socket) = LspService::new(|client| Backend::new(client));
    Server::new(stdin, stdout, socket).serve(service).await;
}
