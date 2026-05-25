//! Integration tests for SDIF hover information.

use sdif_lsp::hover::hover_at;
use sdif_rs::parser::parse_text;

fn doc(text: &str) -> sdif_rs::Document {
    parse_text(text).expect("test input must parse")
}

#[test]
fn hover_directive_sdif() {
    let text = "@sdif 1.0\n";
    let d = doc(text);
    let result = hover_at(&d, 0, 1); // cursor on "sdif"
    assert!(result.is_some(), "must return hover for @sdif");
    let content = result.unwrap();
    assert!(
        content.contains("@sdif"),
        "hover must reference directive name"
    );
    assert!(
        content.contains("version"),
        "hover must describe version semantics"
    );
}

#[test]
fn hover_directive_sdif_ai() {
    let text = "@sdif.ai 1.0\n";
    let d = doc(text);
    let result = hover_at(&d, 0, 1);
    assert!(result.is_some(), "must return hover for @sdif.ai");
    let content = result.unwrap();
    assert!(content.contains("@sdif.ai"), "hover must reference sdif.ai");
}

#[test]
fn hover_table_header_lists_columns() {
    let text = "@sdif 1.0\nitems[name,value$]:\n  r1\tdata\n";
    let d = doc(text);
    let result = hover_at(&d, 1, 2); // cursor on table header line
    assert!(result.is_some(), "must return hover for table header");
    let content = result.unwrap();
    assert!(content.contains("items"), "hover must mention table name");
    assert!(content.contains("name"), "hover must list columns");
}

#[test]
fn hover_table_body_reports_table_context() {
    let text = "@sdif 1.0\nitems[name,value$]:\n  r1\tdata\n";
    let d = doc(text);
    // sdif-rs Table span covers only the header line (end_col=1 on row line).
    // Cursor on the header line (1, 2) falls inside the span.
    let result = hover_at(&d, 1, 2);
    assert!(result.is_some(), "must return hover on table header line");
    let content = result.unwrap();
    assert!(content.contains("items"), "hover must mention table name");
}

#[test]
fn hover_relation_formats_subject_predicate_object() {
    let text = "@sdif 1.0\nrel:\n  A depends_on B\n";
    let d = doc(text);
    let result = hover_at(&d, 2, 2); // cursor on relation row
    assert!(result.is_some(), "must return hover for relation");
    let content = result.unwrap();
    assert!(content.contains("A"), "hover must include subject");
    assert!(
        content.contains("depends_on"),
        "hover must include predicate"
    );
    assert!(content.contains("B"), "hover must include object");
}

#[test]
fn hover_object_block_returns_block_info() {
    let text = "@sdif 1.0\nconfig:\n  key value\n";
    let d = doc(text);
    // sdif-rs ObjectBlock span ends at end_col=1 of the inner line; cursor on
    // the header line (1, 0) is reliably inside the block span.
    let result = hover_at(&d, 1, 0);
    assert!(
        result.is_some(),
        "must return hover for object block header"
    );
    let content = result.unwrap();
    assert!(
        content.contains("config"),
        "hover must name the object block"
    );
}

#[test]
fn hover_returns_none_outside_any_node() {
    let text = "@sdif 1.0\n";
    let d = doc(text);
    let result = hover_at(&d, 99, 0); // past end of document
    assert!(result.is_none(), "must return None when no node found");
}
