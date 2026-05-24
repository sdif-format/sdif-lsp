//! Convert `sdif_rs::ParseError` values into `lsp_types::Diagnostic`.

use sdif_rs::ParseError;
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, NumberOrString, Position, Range};

/// Map a single `ParseError` to an LSP `Diagnostic`.
///
/// SDIF spans are 1-based; LSP positions are 0-based, so we subtract 1 from
/// each line and column value (saturating to avoid underflow on line 0).
pub fn to_lsp_diagnostic(e: &ParseError) -> Diagnostic {
    let start = Position {
        line: e.span.start_line.saturating_sub(1),
        character: e.span.start_col.saturating_sub(1),
    };
    let end = Position {
        line: e.span.end_line.saturating_sub(1),
        character: e.span.end_col.saturating_sub(1),
    };
    Diagnostic {
        range: Range { start, end },
        severity: Some(DiagnosticSeverity::ERROR),
        code: Some(NumberOrString::String(e.code.clone())),
        message: e.message.clone(),
        source: Some("sdif-lsp".to_string()),
        ..Default::default()
    }
}
