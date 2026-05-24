//! Completion suggestions for SDIF documents.
//!
//! Detects the context from the text before the cursor and returns a list of
//! `CompletionItem`s.  This is an MVP implementation covering directive and
//! start-of-line contexts.

use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind};

/// Return completion items for the cursor at `(line, character)` (0-based).
///
/// `text` is the full document text; `line` and `character` are the LSP
/// cursor position.
pub fn completions_at(text: &str, line: u32, character: u32) -> Vec<CompletionItem> {
    let line_text = text.lines().nth(line as usize).unwrap_or("");
    let char_limit = (character as usize).min(line_text.len());
    let before_cursor = &line_text[..char_limit];

    if before_cursor.trim_start().starts_with('@') {
        return directive_completions();
    }

    if before_cursor.trim().is_empty() {
        return start_of_line_completions();
    }

    vec![]
}

// ---------------------------------------------------------------------------
// Item lists
// ---------------------------------------------------------------------------

fn directive_completions() -> Vec<CompletionItem> {
    vec![
        completion_item("@sdif 1.0", "Format version directive", CompletionItemKind::KEYWORD),
        completion_item("@sdif.ai 1.0", "AI projection format directive", CompletionItemKind::KEYWORD),
        completion_item("@profile", "Profile directive", CompletionItemKind::KEYWORD),
        completion_item("@namespace", "Namespace directive", CompletionItemKind::KEYWORD),
        completion_item("@vocab", "Vocabulary directive", CompletionItemKind::KEYWORD),
        completion_item("@base", "Base URI directive", CompletionItemKind::KEYWORD),
    ]
}

fn start_of_line_completions() -> Vec<CompletionItem> {
    let mut items = directive_completions();
    items.extend(vec![
        completion_item("rel:", "Relation block", CompletionItemKind::KEYWORD),
        completion_item("rules:", "Rules block", CompletionItemKind::KEYWORD),
    ]);
    items
}

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn completion_item(label: &str, detail: &str, kind: CompletionItemKind) -> CompletionItem {
    CompletionItem {
        label: label.to_string(),
        detail: Some(detail.to_string()),
        kind: Some(kind),
        ..Default::default()
    }
}
