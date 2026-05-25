//! Document store: holds text, parsed AST, and parse errors per URI.

use std::collections::HashMap;
use std::sync::Arc;

use lsp_types::Url;
use sdif::{Document, ParseError};
use tokio::sync::RwLock;

/// Per-document state kept in memory while the editor has the file open.
pub struct DocState {
    pub text: String,
    pub doc: Option<Document>,
    pub errors: Vec<ParseError>,
    pub version: Option<i32>,
}

/// Thread-safe store for all open documents.
pub struct DocumentStore {
    inner: Arc<RwLock<HashMap<String, DocState>>>,
}

impl DocumentStore {
    pub fn new() -> Self {
        DocumentStore {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Re-parse `text` and store the result under `uri`.
    pub async fn update(&self, uri: &Url, text: String) {
        let (doc, errors) = match sdif::parser::parse_text(&text) {
            Ok(d) => (Some(d), vec![]),
            Err(e) => (None, vec![e]),
        };
        let state = DocState {
            text,
            doc,
            errors,
            version: None,
        };
        self.inner.write().await.insert(uri.to_string(), state);
    }

    /// Return the parse errors for `uri`, or an empty vec if not found.
    pub async fn get_errors(&self, uri: &Url) -> Vec<ParseError> {
        self.inner
            .read()
            .await
            .get(&uri.to_string())
            .map(|s| s.errors.clone())
            .unwrap_or_default()
    }

    /// Return the parsed `Document` for `uri` if parsing succeeded.
    pub async fn get_doc(&self, uri: &Url) -> Option<Document> {
        self.inner
            .read()
            .await
            .get(&uri.to_string())
            .and_then(|s| s.doc.clone())
    }

    /// Return the raw source text for `uri`, or `None` if not found.
    pub async fn get_text(&self, uri: &Url) -> Option<String> {
        self.inner
            .read()
            .await
            .get(&uri.to_string())
            .map(|s| s.text.clone())
    }

    /// Remove the document for `uri` from the store.
    pub async fn remove(&self, uri: &Url) {
        self.inner.write().await.remove(&uri.to_string());
    }
}
