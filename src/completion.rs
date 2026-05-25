//! Completion suggestions for SDIF documents.
//!
//! Detects the context from the text before the cursor and returns a list of
//! `CompletionItem`s.

use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind, InsertTextFormat};

/// Return completion items for the cursor at `(line, character)` (0-based).
///
/// `text` is the full document text; `line` and `character` are the LSP
/// cursor position.
pub fn completions_at(text: &str, line: u32, character: u32) -> Vec<CompletionItem> {
    let before_cursor = crate::position::text_before_lsp_cursor(text, line, character);

    if before_cursor.trim_start().starts_with('@') {
        return directive_completions();
    }

    if before_cursor.trim_start().starts_with("alias[") {
        return alias_entry_completions();
    }

    if before_cursor.trim().is_empty() {
        return start_of_line_completions(text);
    }

    vec![]
}

// ---------------------------------------------------------------------------
// Item lists
// ---------------------------------------------------------------------------

fn directive_completions() -> Vec<CompletionItem> {
    vec![
        completion_item(
            "@sdif 1.0",
            "Format version directive",
            CompletionItemKind::KEYWORD,
            None,
        ),
        completion_item(
            "@sdif.ai 1.0",
            "AI projection format directive",
            CompletionItemKind::KEYWORD,
            None,
        ),
        completion_item(
            "@profile",
            "Profile directive",
            CompletionItemKind::KEYWORD,
            None,
        ),
        completion_item(
            "@namespace",
            "Namespace directive",
            CompletionItemKind::KEYWORD,
            None,
        ),
        completion_item(
            "@vocab",
            "Vocabulary directive",
            CompletionItemKind::KEYWORD,
            None,
        ),
        completion_item(
            "@base",
            "Base URI directive",
            CompletionItemKind::KEYWORD,
            None,
        ),
        completion_item(
            "@include",
            "Include directive",
            CompletionItemKind::KEYWORD,
            None,
        ),
    ]
}

fn alias_entry_completions() -> Vec<CompletionItem> {
    vec![
        completion_item(
            "k=kind",
            "Alias 'k' for kind",
            CompletionItemKind::VALUE,
            None,
        ),
        completion_item(
            "st=status",
            "Alias 'st' for status",
            CompletionItemKind::VALUE,
            None,
        ),
        completion_item(
            "e=rel",
            "Alias 'e' for rel",
            CompletionItemKind::VALUE,
            None,
        ),
    ]
}

fn start_of_line_completions(text: &str) -> Vec<CompletionItem> {
    let is_ai = text.lines().any(|l| l.trim_start().starts_with("@sdif.ai"));
    let mut items = directive_completions();
    items.extend(vec![
        completion_snippet(
            "table[col1,col2]:",
            "Table header",
            CompletionItemKind::CLASS,
            "${1:name}[${2:col1},${3:col2}]:",
        ),
        completion_snippet(
            "alias[k=kind]",
            "Alias header",
            CompletionItemKind::CLASS,
            "alias[${1:abbr}=${2:canonical}]",
        ),
        completion_snippet(
            "key\"\"\"",
            "Narrative block",
            CompletionItemKind::TEXT,
            "${1:key}\"\"\"\n${2:text}\n\"\"\"",
        ),
        completion_item("rel:", "Relation block", CompletionItemKind::KEYWORD, None),
        completion_item("rules:", "Rules block", CompletionItemKind::KEYWORD, None),
    ]);
    if is_ai {
        items.push(completion_snippet(
            "rel[subject]:",
            "Grouped relation block (AI)",
            CompletionItemKind::KEYWORD,
            "rel[${1:subject}]:",
        ));
    }
    items
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn completion_item(
    label: &str,
    detail: &str,
    kind: CompletionItemKind,
    insert: Option<&str>,
) -> CompletionItem {
    CompletionItem {
        label: label.to_string(),
        detail: Some(detail.to_string()),
        kind: Some(kind),
        insert_text: insert.map(str::to_string),
        ..Default::default()
    }
}

fn completion_snippet(
    label: &str,
    detail: &str,
    kind: CompletionItemKind,
    snippet: &str,
) -> CompletionItem {
    CompletionItem {
        label: label.to_string(),
        detail: Some(detail.to_string()),
        kind: Some(kind),
        insert_text: Some(snippet.to_string()),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        ..Default::default()
    }
}
