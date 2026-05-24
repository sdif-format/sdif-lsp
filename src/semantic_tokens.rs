//! Semantic token encoding for SDIF documents.
//!
//! Structural/editorial highlighting is derived from `tree-sitter-sdif` and
//! its `queries/highlights.scm` query. The `sdif-rs` parser remains normative
//! for diagnostics and semantic language features; this module intentionally
//! avoids reimplementing SDIF highlighting rules from the AST.

use tower_lsp::lsp_types::SemanticToken;
use tree_sitter::{Node, Parser, Query, QueryCursor, StreamingIterator};

// ---------------------------------------------------------------------------
// Legend — order is significant (indices referenced by semantic tokens)
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
    "declaration",    // bit 0
    "definition",     // bit 1
    "readonly",       // bit 2
    "static",         // bit 3
    "deprecated",     // bit 4
    "abstract",       // bit 5
    "async",          // bit 6
    "modification",   // bit 7
    "documentation",  // bit 8
    "defaultLibrary", // bit 9
];

const TT_TYPE: u32 = 1;
const TT_ENUM: u32 = 3;
const TT_VARIABLE: u32 = 8;
const TT_PROPERTY: u32 = 11;
const TT_KEYWORD: u32 = 12;
const TT_COMMENT: u32 = 14;
const TT_STRING: u32 = 15;
const TT_OPERATOR: u32 = 18;

const MOD_NONE: u32 = 0;
const MOD_DECLARATION: u32 = 1; // bit 0

// ---------------------------------------------------------------------------
// Internal raw token (absolute UTF-16 LSP positions, not yet delta-encoded)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
struct RawToken {
    line: u32,
    character: u32,
    length: u32,
    token_type: u32,
    modifiers: u32,
}

impl RawToken {
    fn new(line: u32, character: u32, length: u32, token_type: u32, modifiers: u32) -> Self {
        Self {
            line,
            character,
            length,
            token_type,
            modifiers,
        }
    }

    fn end_character(&self) -> u32 {
        self.character + self.length
    }
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Build delta-encoded semantic tokens from SDIF source text.
///
/// Captures come from `tree-sitter-sdif::HIGHLIGHTS_QUERY`; this keeps the LSP
/// aligned with the tree-sitter grammar instead of maintaining a parallel
/// highlighter. Invalid/incomplete documents still return best-effort tokens,
/// matching editor expectations for incremental syntax highlighting.
pub fn build_semantic_tokens_from_text(text: &str) -> Vec<SemanticToken> {
    let mut parser = Parser::new();
    if parser.set_language(&tree_sitter_sdif::language()).is_err() {
        return Vec::new();
    }

    let Some(tree) = parser.parse(text, None) else {
        return Vec::new();
    };

    let query = match Query::new(
        &tree_sitter_sdif::language(),
        tree_sitter_sdif::HIGHLIGHTS_QUERY,
    ) {
        Ok(query) => query,
        Err(_) => return Vec::new(),
    };

    let line_index = LineIndex::new(text);
    let mut cursor = QueryCursor::new();
    let mut captures = cursor.captures(&query, tree.root_node(), text.as_bytes());
    let capture_names = query.capture_names();
    let mut tokens = Vec::new();

    loop {
        captures.advance();
        let Some((query_match, capture_index)) = captures.get() else {
            break;
        };
        let capture = query_match.captures[*capture_index];
        let capture_name = &capture_names[capture.index as usize];
        if let Some((token_type, modifiers)) = map_capture(capture_name) {
            push_node_tokens(
                &mut tokens,
                capture.node,
                &line_index,
                token_type,
                modifiers,
            );
        }
    }

    normalize_tokens(tokens)
}

// ---------------------------------------------------------------------------
// Capture mapping
// ---------------------------------------------------------------------------

fn map_capture(capture_name: &str) -> Option<(u32, u32)> {
    let base = capture_name.split('.').next().unwrap_or(capture_name);
    match base {
        "keyword" => Some((TT_KEYWORD, MOD_NONE)),
        "type" => Some((TT_TYPE, MOD_DECLARATION)),
        "property" => Some((TT_PROPERTY, MOD_NONE)),
        "variable" => Some((TT_VARIABLE, MOD_NONE)),
        "constant" => Some((TT_ENUM, MOD_NONE)),
        "comment" => Some((TT_COMMENT, MOD_NONE)),
        "string" => Some((TT_STRING, MOD_NONE)),
        "atom" => Some((TT_STRING, MOD_NONE)),
        "punctuation" => Some((TT_OPERATOR, MOD_NONE)),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Range conversion
// ---------------------------------------------------------------------------

fn push_node_tokens(
    tokens: &mut Vec<RawToken>,
    node: Node,
    line_index: &LineIndex,
    token_type: u32,
    modifiers: u32,
) {
    let start = node.start_position();
    let end = node.end_position();

    if start.row == end.row {
        if let Some(token) = line_index.token_for_byte_columns(
            start.row,
            start.column,
            end.column,
            token_type,
            modifiers,
        ) {
            tokens.push(token);
        }
        return;
    }

    for row in start.row..=end.row {
        let start_byte_col = if row == start.row { start.column } else { 0 };
        let end_byte_col = if row == end.row {
            end.column
        } else {
            line_index.line_byte_len(row)
        };
        if let Some(token) = line_index.token_for_byte_columns(
            row,
            start_byte_col,
            end_byte_col,
            token_type,
            modifiers,
        ) {
            tokens.push(token);
        }
    }
}

struct LineIndex<'a> {
    lines: Vec<&'a str>,
}

impl<'a> LineIndex<'a> {
    fn new(text: &'a str) -> Self {
        Self {
            lines: text.split('\n').collect(),
        }
    }

    fn line(&self, row: usize) -> Option<&'a str> {
        self.lines.get(row).copied()
    }

    fn line_byte_len(&self, row: usize) -> usize {
        self.line(row).map(str::len).unwrap_or(0)
    }

    fn token_for_byte_columns(
        &self,
        row: usize,
        start_byte_col: usize,
        end_byte_col: usize,
        token_type: u32,
        modifiers: u32,
    ) -> Option<RawToken> {
        let line = self.line(row)?;
        let start = start_byte_col.min(line.len());
        let end = end_byte_col.min(line.len());
        if end <= start || !line.is_char_boundary(start) || !line.is_char_boundary(end) {
            return None;
        }

        let character = utf16_units(&line[..start]);
        let length = utf16_units(&line[start..end]);
        if length == 0 {
            return None;
        }

        Some(RawToken::new(
            row as u32, character, length, token_type, modifiers,
        ))
    }
}

fn utf16_units(text: &str) -> u32 {
    text.encode_utf16().count() as u32
}

// ---------------------------------------------------------------------------
// Token normalization and delta encoding
// ---------------------------------------------------------------------------

fn normalize_tokens(mut tokens: Vec<RawToken>) -> Vec<SemanticToken> {
    tokens.sort_by(|a, b| {
        a.line
            .cmp(&b.line)
            .then(a.character.cmp(&b.character))
            .then((b.length).cmp(&a.length))
            .then(a.token_type.cmp(&b.token_type))
    });

    let mut filtered: Vec<RawToken> = Vec::with_capacity(tokens.len());
    for token in tokens {
        if token.length == 0 {
            continue;
        }
        if let Some(previous) = filtered.last() {
            if previous.line == token.line
                && previous.character == token.character
                && previous.length == token.length
            {
                continue;
            }
            if previous.line == token.line && token.character < previous.end_character() {
                continue;
            }
        }
        filtered.push(token);
    }

    delta_encode(filtered)
}

fn delta_encode(tokens: Vec<RawToken>) -> Vec<SemanticToken> {
    let mut result = Vec::with_capacity(tokens.len());
    let mut prev_line = 0u32;
    let mut prev_character = 0u32;

    for token in tokens {
        let delta_line = token.line - prev_line;
        let delta_start = if delta_line == 0 {
            token.character - prev_character
        } else {
            token.character
        };
        result.push(SemanticToken {
            delta_line,
            delta_start,
            length: token.length,
            token_type: token.token_type,
            token_modifiers_bitset: token.modifiers,
        });
        prev_line = token.line;
        prev_character = token.character;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, Eq)]
    struct DecodedToken {
        line: u32,
        character: u32,
        length: u32,
        token_type: u32,
    }

    fn decode(tokens: &[SemanticToken]) -> Vec<DecodedToken> {
        let mut decoded = Vec::new();
        let mut line = 0;
        let mut character = 0;

        for token in tokens {
            line += token.delta_line;
            character = if token.delta_line == 0 {
                character + token.delta_start
            } else {
                token.delta_start
            };
            decoded.push(DecodedToken {
                line,
                character,
                length: token.length,
                token_type: token.token_type,
            });
        }

        decoded
    }

    fn has_token(
        decoded: &[DecodedToken],
        line: u32,
        character: u32,
        length: u32,
        token_type: u32,
    ) -> bool {
        decoded.iter().any(|token| {
            token.line == line
                && token.character == character
                && token.length == length
                && token.token_type == token_type
        })
    }

    #[test]
    fn semantic_tokens_follow_tree_sitter_highlights_query_for_core_sdif() {
        let text = "@sdif 1.0\n# hello\nkind Example\nitems[name,value$]:\nalpha\t\"one\"\nnotes\"\"\"body\"\"\"\n";

        let decoded = decode(&build_semantic_tokens_from_text(text));

        assert!(
            has_token(&decoded, 0, 1, 4, TT_KEYWORD),
            "directive identifier is keyword"
        );
        assert!(
            has_token(&decoded, 1, 0, 7, TT_COMMENT),
            "comment line is comment"
        );
        assert!(has_token(&decoded, 3, 0, 5, TT_TYPE), "table name is type");
        assert!(
            has_token(&decoded, 3, 6, 4, TT_PROPERTY),
            "table column is property"
        );
        assert!(
            has_token(&decoded, 4, 0, 11, TT_STRING),
            "table row is string"
        );
        assert!(
            has_token(&decoded, 5, 0, 5, TT_TYPE),
            "narrative key is type"
        );
        assert!(
            has_token(&decoded, 5, 8, 4, TT_STRING),
            "narrative body is string"
        );
    }

    #[test]
    fn semantic_tokens_follow_tree_sitter_highlights_query_for_ai_profile() {
        let text = "@sdif.ai 1.0\nalias[k=kind,st=status]\nrel[item-1]:\n  depends_on item-2\n";

        let decoded = decode(&build_semantic_tokens_from_text(text));

        assert!(
            has_token(&decoded, 0, 1, 7, TT_KEYWORD),
            "sdif.ai directive is keyword"
        );
        assert!(
            has_token(&decoded, 1, 0, 5, TT_KEYWORD),
            "alias header keyword is keyword"
        );
        assert!(
            has_token(&decoded, 1, 6, 6, TT_PROPERTY),
            "alias entry is property"
        );
        assert!(
            has_token(&decoded, 2, 0, 3, TT_KEYWORD),
            "grouped relation keyword is keyword"
        );
        assert!(
            has_token(&decoded, 2, 4, 6, TT_VARIABLE),
            "grouped relation subject is variable"
        );
        assert!(
            has_token(&decoded, 3, 0, 19, TT_STRING),
            "grouped relation row is string"
        );
    }

    #[test]
    fn semantic_tokens_use_utf16_columns_for_non_ascii_strings() {
        let text = "kind \"😀\"\n";

        let decoded = decode(&build_semantic_tokens_from_text(text));

        assert!(
            has_token(&decoded, 0, 0, 4, TT_PROPERTY),
            "identifier length is UTF-16"
        );
        assert!(
            has_token(&decoded, 0, 5, 4, TT_STRING),
            "quoted value length is UTF-16"
        );
    }
}
