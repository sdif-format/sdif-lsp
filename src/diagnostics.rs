//! Convert `sdif_rs::ParseError` values into `lsp_types::Diagnostic`.

use sdif_rs::ParseError;
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, NumberOrString, Position, Range};

use crate::position::sdif_span_col_to_lsp_character;

/// Map a single `ParseError` to an LSP `Diagnostic`.
///
/// sdif-rs spans are 1-based with byte-based columns. LSP positions are
/// 0-based with UTF-16 character offsets. This function converts between
/// the two, using the source text to compute correct UTF-16 offsets even
/// for non-ASCII content.
///
/// When the parser reports a zero-width span (start == end), the range is
/// extended by one character so the squiggle is visible in the editor.
pub fn to_lsp_diagnostic(source: &str, e: &ParseError) -> Diagnostic {
    let start_line = e.span.start_line.saturating_sub(1);
    let end_line = e.span.end_line.saturating_sub(1);

    let start_char = sdif_span_col_to_lsp_character(source, start_line, e.span.start_col);
    let end_char = sdif_span_col_to_lsp_character(source, end_line, e.span.end_col);

    let (start_char, end_char) = if start_line == end_line && start_char == end_char {
        (start_char, start_char + 1)
    } else {
        (start_char, end_char)
    };

    Diagnostic {
        range: Range {
            start: Position {
                line: start_line,
                character: start_char,
            },
            end: Position {
                line: end_line,
                character: end_char,
            },
        },
        severity: Some(DiagnosticSeverity::ERROR),
        code: Some(NumberOrString::String(e.code.clone())),
        message: e.message.clone(),
        source: Some("sdif-lsp".to_string()),
        ..Default::default()
    }
}
