//! Semantic token encoding for SDIF documents.
//!
//! Walks the AST and emits a flat `Vec<u32>` in the LSP delta-encoded format
//! (groups of 5: delta_line, delta_start_char, length, token_type, modifiers).

use tower_lsp::lsp_types::SemanticToken;
use sdif_rs::{
    Directive, Document, Field, Narrative, Relation, Statement, Table,
};
use sdif_rs::Span;

// ---------------------------------------------------------------------------
// Legend — order is significant (indices referenced in backend.rs)
// ---------------------------------------------------------------------------

pub const TOKEN_TYPES: &[&str] = &[
    "namespace",     // 0
    "type",          // 1
    "class",         // 2
    "enum",          // 3
    "interface",     // 4
    "struct",        // 5
    "typeParameter", // 6
    "parameter",     // 7
    "variable",      // 8
    "function",      // 9
    "method",        // 10
    "property",      // 11
    "keyword",       // 12
    "modifier",      // 13
    "comment",       // 14
    "string",        // 15
    "number",        // 16
    "regexp",        // 17
    "operator",      // 18
    "decorator",     // 19
];

pub const TOKEN_MODIFIERS: &[&str] = &[
    "declaration",   // bit 0
    "definition",    // bit 1
    "readonly",      // bit 2
    "static",        // bit 3
    "deprecated",    // bit 4
    "abstract",      // bit 5
    "async",         // bit 6
    "modification",  // bit 7
    "documentation", // bit 8
    "defaultLibrary",// bit 9
];

// Convenient index constants.
const TT_VARIABLE: u32 = 8;
const TT_PROPERTY: u32 = 11;
const TT_KEYWORD: u32 = 12;
const TT_COMMENT: u32 = 14;
const TT_STRING: u32 = 15;

const MOD_NONE: u32 = 0;
const MOD_DECLARATION: u32 = 1; // bit 0

// ---------------------------------------------------------------------------
// Internal raw token (absolute positions, not yet delta-encoded)
// ---------------------------------------------------------------------------

/// Intermediate token with absolute (line, col) before delta encoding.
#[derive(Debug)]
struct RawToken {
    line: u32,
    col: u32,
    len: u32,
    token_type: u32,
    modifiers: u32,
}

impl RawToken {
    fn new(line: u32, col: u32, len: u32, token_type: u32, modifiers: u32) -> Self {
        Self { line, col, len, token_type, modifiers }
    }
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Build delta-encoded semantic tokens from a parsed `Document`.
pub fn build_semantic_tokens(doc: &Document) -> Vec<SemanticToken> {
    let mut tokens: Vec<RawToken> = Vec::new();

    for directive in &doc.directives {
        collect_directive(&mut tokens, directive);
    }

    for stmt in &doc.statements {
        collect_statement(&mut tokens, stmt);
    }

    // LSP requires tokens in document order.
    tokens.sort_by(|a, b| a.line.cmp(&b.line).then(a.col.cmp(&b.col)));

    delta_encode(tokens)
}

// ---------------------------------------------------------------------------
// Span helpers
// ---------------------------------------------------------------------------

/// Convert a 1-based span into a 0-based (line, col, len) triple.
///
/// Only valid for single-line spans; multi-line spans are approximated to
/// their start line with length reaching to end_col.
fn span_to_lsp(span: &Span) -> (u32, u32, u32) {
    let line = span.start_line.saturating_sub(1);
    let col = span.start_col.saturating_sub(1);
    let len = if span.end_col > span.start_col {
        span.end_col - span.start_col
    } else {
        1
    };
    (line, col, len)
}

fn push_span(
    tokens: &mut Vec<RawToken>,
    span: &Span,
    token_type: u32,
    modifiers: u32,
) {
    let (line, col, len) = span_to_lsp(span);
    tokens.push(RawToken::new(line, col, len, token_type, modifiers));
}

// ---------------------------------------------------------------------------
// Collectors per node type
// ---------------------------------------------------------------------------

fn collect_directive(tokens: &mut Vec<RawToken>, directive: &Directive) {
    // Emit the "@name" part as a keyword.  The directive span covers the whole
    // line; the name begins at column 1 (0-based) with length = @+name chars.
    let line = directive.span.start_line.saturating_sub(1);
    let col = 0u32; // '@' is always the first character
    let len = (directive.name.len() as u32) + 1; // +1 for '@'
    tokens.push(RawToken::new(line, col, len, TT_KEYWORD, MOD_NONE));
}

fn collect_field(tokens: &mut Vec<RawToken>, field: &Field) {
    // Key → property
    push_span(tokens, &field.key_span, TT_PROPERTY, MOD_NONE);

    // Value → string if quoted, variable if bare
    let value_type = if field.quoted { TT_STRING } else { TT_VARIABLE };
    push_span(tokens, &field.value_span, value_type, MOD_NONE);
}

fn collect_table(tokens: &mut Vec<RawToken>, table: &Table) {
    // Emit the entire header line as a single property+declaration token.
    // The header contains "name[col1\tcol2…]:" so we highlight the whole line.
    push_span(tokens, &table.header_span, TT_PROPERTY, MOD_DECLARATION);

    // Emit each data row as a single string token spanning the whole row line.
    // Row spans are not individually tracked in the AST; we reconstruct them
    // from the table span, skipping the header line.
    let header_line = table.header_span.start_line;
    let table_start = table.span.start_line;
    let table_end = table.span.end_line;

    // Data rows occupy lines after the header line.
    let first_row_line = header_line + 1;
    let num_rows = table.rows.len() as u32;

    for i in 0..num_rows {
        let abs_line = first_row_line + i;
        if abs_line > table_end {
            break;
        }
        let lsp_line = abs_line.saturating_sub(1);
        // We don't know exact row lengths; use a sentinel that covers a
        // reasonable row width.  LSP clients clip to actual line length.
        let row_text_len: u32 = table.rows[i as usize]
            .iter()
            .map(|c| c.len() as u32 + 1) // +1 for tab separator
            .sum::<u32>()
            .saturating_sub(1)
            .max(1);
        tokens.push(RawToken::new(lsp_line, 0, row_text_len, TT_STRING, MOD_NONE));
        // Suppress unused warning from table_start
        let _ = table_start;
    }
}

fn collect_narrative(tokens: &mut Vec<RawToken>, narrative: &Narrative) {
    // The narrative span covers "key> text…".  Approximate:
    //   - First line: the key (property + declaration) up to the '>' separator.
    //   - Remaining lines: comment tokens.
    let start_line = narrative.span.start_line;
    let end_line = narrative.span.end_line;
    let lsp_start = start_line.saturating_sub(1);

    // Key portion: length of key + 1 for '>'
    let key_len = (narrative.key.len() as u32) + 1;
    tokens.push(RawToken::new(lsp_start, 0, key_len, TT_PROPERTY, MOD_DECLARATION));

    // Body lines: everything after the first line
    for abs_line in (start_line + 1)..=end_line {
        let lsp_line = abs_line.saturating_sub(1);
        // Use a non-zero length; clients clip to actual line content.
        tokens.push(RawToken::new(lsp_line, 0, 1, TT_COMMENT, MOD_NONE));
    }

    // If it is a single-line narrative, also emit the body text on same line.
    if start_line == end_line && !narrative.text.is_empty() {
        // body starts after "key> " — column = key_len + 1 (space after '>').
        let body_col = key_len + 1;
        let body_len = (narrative.text.len() as u32).max(1);
        tokens.push(RawToken::new(lsp_start, body_col, body_len, TT_COMMENT, MOD_NONE));
    }
}

fn collect_relation(tokens: &mut Vec<RawToken>, relation: &Relation) {
    // The Relation AST has no sub-spans; parse token positions from the span
    // start line and the stored string lengths.
    let line = relation.span.start_line.saturating_sub(1);

    let subject_len = relation.subject.len() as u32;
    let predicate_len = relation.predicate.len() as u32;

    // subject starts at column 0
    tokens.push(RawToken::new(line, 0, subject_len, TT_VARIABLE, MOD_NONE));

    // predicate follows subject + 1 space
    let predicate_col = subject_len + 1;
    tokens.push(RawToken::new(line, predicate_col, predicate_len, TT_VARIABLE, MOD_NONE));

    // object follows predicate + 1 space (quotes not counted in stored string)
    let object_col = predicate_col + predicate_len + 1;
    let object_len = relation.object.len() as u32;
    let object_type = if relation.object_quoted { TT_STRING } else { TT_VARIABLE };
    tokens.push(RawToken::new(line, object_col, object_len, object_type, MOD_NONE));
}

fn collect_statement(tokens: &mut Vec<RawToken>, stmt: &Statement) {
    match stmt {
        Statement::Field(f) => collect_field(tokens, f),
        Statement::Table(t) => collect_table(tokens, t),
        Statement::Narrative(n) => collect_narrative(tokens, n),
        Statement::Relation(r) => collect_relation(tokens, r),
        Statement::ObjectBlock(ob) => {
            // Emit the block key as a property + declaration on its span line.
            let line = ob.span.start_line.saturating_sub(1);
            let key_len = (ob.key.len() as u32).max(1);
            tokens.push(RawToken::new(line, 0, key_len, TT_PROPERTY, MOD_DECLARATION));
            // Recurse into nested statements.
            for inner in &ob.statements {
                collect_statement(tokens, inner);
            }
        }
        Statement::Rule(_) => {
            // Rules are left un-highlighted for now (no sub-span data).
        }
    }
}

// ---------------------------------------------------------------------------
// Delta encoding
// ---------------------------------------------------------------------------

fn delta_encode(tokens: Vec<RawToken>) -> Vec<SemanticToken> {
    let mut result = Vec::with_capacity(tokens.len());
    let mut prev_line = 0u32;
    let mut prev_col = 0u32;

    for t in tokens {
        let delta_line = t.line - prev_line;
        let delta_start = if delta_line == 0 { t.col - prev_col } else { t.col };
        result.push(SemanticToken {
            delta_line,
            delta_start,
            length: t.len,
            token_type: t.token_type,
            token_modifiers_bitset: t.modifiers,
        });
        prev_line = t.line;
        prev_col = t.col;
    }

    result
}
