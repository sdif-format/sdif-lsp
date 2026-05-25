//! Integration tests for SDIF completion suggestions.

use sdif_lsp::completion::completions_at;

#[test]
fn completion_directives_after_at_sign() {
    let items = completions_at("@", 0, 1);
    let labels: Vec<_> = items.iter().map(|i| i.label.as_str()).collect();
    assert!(labels.contains(&"@sdif 1.0"), "must include @sdif 1.0");
    assert!(
        labels.contains(&"@sdif.ai 1.0"),
        "must include @sdif.ai 1.0"
    );
}

#[test]
fn completion_sdif_structures_on_empty_line() {
    let text = "@sdif 1.0\n";
    let items = completions_at(text, 1, 0);
    let labels: Vec<_> = items.iter().map(|i| i.label.as_str()).collect();
    assert!(!items.is_empty(), "empty line must return completions");
    assert!(
        labels.iter().any(|l| l.contains("table")),
        "must include table snippet"
    );
}

#[test]
fn completion_ai_only_items_when_ai_directive_present() {
    let text = "@sdif.ai 1.0\n";
    let items = completions_at(text, 1, 0);
    let labels: Vec<_> = items.iter().map(|i| i.label.as_str()).collect();
    assert!(
        labels.iter().any(|l| l.contains("rel[")),
        "must include AI grouped rel snippet"
    );
}

#[test]
fn completion_ai_only_items_absent_without_ai_directive() {
    let text = "@sdif 1.0\n";
    let items = completions_at(text, 1, 0);
    let labels: Vec<_> = items.iter().map(|i| i.label.as_str()).collect();
    assert!(
        !labels.iter().any(|l| l.contains("rel[")),
        "must not include AI grouped rel without @sdif.ai"
    );
}

#[test]
fn completion_does_not_break_with_utf16_cursor_past_emoji() {
    // "kind \"😀\"" — 😀 is 2 UTF-16 units; cursor at character=8 (after emoji, before closing ")
    let text = "kind \"😀\"\n";
    // Should not panic regardless of what it returns
    let _items = completions_at(text, 0, 8);
}

#[test]
fn completion_inside_alias_header() {
    let items = completions_at("alias[", 0, 6);
    let labels: Vec<_> = items.iter().map(|i| i.label.as_str()).collect();
    assert!(!items.is_empty(), "alias[ must trigger alias completions");
    assert!(
        labels.iter().any(|l| l.contains('=')),
        "alias entries must contain '='"
    );
}
