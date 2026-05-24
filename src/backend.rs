//! LSP backend: implements `LanguageServer` using the document store.

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::{
    CompletionOptions, CompletionParams, CompletionResponse, DidChangeTextDocumentParams,
    DidOpenTextDocumentParams, Hover, HoverParams, HoverProviderCapability, InitializeParams,
    InitializeResult, InitializedParams, ServerCapabilities, TextDocumentSyncCapability,
    TextDocumentSyncKind, Url,
};
use tower_lsp::{Client, LanguageServer};

use crate::diagnostics::to_lsp_diagnostic;
use crate::document::DocumentStore;

/// The LSP server backend.  One instance is created per client connection.
pub struct Backend {
    client: Client,
    docs: DocumentStore,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Backend { client, docs: DocumentStore::new() }
    }

    /// Re-publish diagnostics for `uri` based on the current document state.
    async fn publish_diagnostics(&self, uri: &Url) {
        let errors = self.docs.get_errors(uri).await;
        let diagnostics = errors.iter().map(to_lsp_diagnostic).collect();
        self.client.publish_diagnostics(uri.clone(), diagnostics, None).await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _params: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec!["@".to_string()]),
                    ..Default::default()
                }),
                // semantic_tokens_provider is added in Task 6
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _params: InitializedParams) {}

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;
        self.docs.update(&uri, text).await;
        self.publish_diagnostics(&uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        if let Some(change) = params.content_changes.into_iter().last() {
            self.docs.update(&uri, change.text).await;
            self.publish_diagnostics(&uri).await;
        }
    }

    async fn hover(&self, _params: HoverParams) -> Result<Option<Hover>> {
        // Filled in Task 7
        Ok(None)
    }

    async fn completion(
        &self,
        _params: CompletionParams,
    ) -> Result<Option<CompletionResponse>> {
        // Filled in Task 7
        Ok(None)
    }
}
