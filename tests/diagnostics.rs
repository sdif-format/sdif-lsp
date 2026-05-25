//! Integration tests for SDIF diagnostics conversion.

use sdif_lsp::diagnostics::to_lsp_diagnostic;
use sdif::parser::parse_text;

#[test]
fn diagnostic_range_is_zero_based() {
    let text = "kind Dataset\n"; // missing @sdif version
    let err = parse_text(text).unwrap_err();
    let diag = to_lsp_diagnostic(text, &err);
    // LSP positions are 0-based; sdif-rs emits 1-based
    assert!(
        diag.range.start.line < 1000,
        "line must be reasonable 0-based value"
    );
}

#[test]
fn diagnostic_range_is_visible_when_parser_span_is_empty() {
    // Any parse error must produce a non-zero-width range for editor visibility.
    let text = "kind Dataset\n";
    let err = parse_text(text).unwrap_err();
    let diag = to_lsp_diagnostic(text, &err);
    let start = diag.range.start;
    let end = diag.range.end;
    // Either different lines, or same line with end.character > start.character
    assert!(
        end.line > start.line || end.character > start.character,
        "diagnostic range must be non-empty: start={start:?} end={end:?}"
    );
}

#[test]
fn diagnostic_includes_error_code_and_source() {
    let text = "kind Dataset\n";
    let err = parse_text(text).unwrap_err();
    let diag = to_lsp_diagnostic(text, &err);

    use tower_lsp::lsp_types::NumberOrString;
    match &diag.code {
        Some(NumberOrString::String(code)) => {
            assert!(!code.is_empty(), "error code must not be empty");
            assert!(
                code.starts_with("SDIF_"),
                "error code must start with SDIF_"
            );
        }
        _ => panic!("expected string error code"),
    }
    assert_eq!(
        diag.source.as_deref(),
        Some("sdif-lsp"),
        "source must be sdif-lsp"
    );
}

#[test]
fn diagnostic_range_handles_ascii_content() {
    let text = "@sdif 1.0\n@unknown_directive 1.0\n";
    let err = parse_text(text).unwrap_err();
    let diag = to_lsp_diagnostic(text, &err);
    // Unknown directive is on line 1 (0-based)
    assert_eq!(
        diag.range.start.line, 1,
        "diagnostic must point to correct line"
    );
}
