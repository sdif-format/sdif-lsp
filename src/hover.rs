//! Hover information for SDIF documents.
//!
//! Finds the innermost AST node whose span contains the cursor position and
//! formats a Markdown description of that node.

use sdif_rs::{Directive, Document, ObjectBlock, Span, Statement};

/// Return a Markdown hover string for the AST node at `(line, character)`.
///
/// `line` and `character` are 0-based LSP positions. Spans in sdif-rs are
/// 1-based, so we add 1 before comparing.
pub fn hover_at(doc: &Document, line: u32, character: u32) -> Option<String> {
    let lsp_line = line + 1;
    let lsp_col = character + 1;

    find_in_statements(doc.statements.iter(), lsp_line, lsp_col)
        .or_else(|| find_in_directives(doc.directives.iter(), lsp_line, lsp_col))
}

// ---------------------------------------------------------------------------
// Span helpers
// ---------------------------------------------------------------------------

/// Return true when `span` contains the position `(line, col)` (1-based).
///
/// `end_col` in sdif-rs Spans is exclusive, so a cursor sitting exactly at
/// end_col is already past the token and is not considered inside.
fn span_contains(span: &Span, line: u32, col: u32) -> bool {
    if span.start_line > line || span.end_line < line {
        return false;
    }
    if span.start_line == line && span.start_col > col {
        return false;
    }
    if span.end_line == line && span.end_col <= col {
        return false;
    }
    true
}

// ---------------------------------------------------------------------------
// Traversal
// ---------------------------------------------------------------------------

fn find_in_statements<'a>(
    stmts: impl Iterator<Item = &'a Statement>,
    line: u32,
    col: u32,
) -> Option<String> {
    for stmt in stmts {
        if let Some(s) = statement_hover(stmt, line, col) {
            return Some(s);
        }
    }
    None
}

fn find_in_directives<'a>(
    directives: impl Iterator<Item = &'a Directive>,
    line: u32,
    col: u32,
) -> Option<String> {
    for directive in directives {
        if span_contains(&directive.span, line, col) {
            return Some(format_directive(directive));
        }
    }
    None
}

fn statement_hover(stmt: &Statement, line: u32, col: u32) -> Option<String> {
    match stmt {
        Statement::Field(f) => {
            if span_contains(&f.span, line, col) {
                Some(format!("**{}**: {}", f.key, f.value))
            } else {
                None
            }
        }
        Statement::Table(t) => {
            if span_contains(&t.span, line, col) {
                Some(format!(
                    "Table **\"{}\"** — {} columns, {} rows",
                    t.name,
                    t.columns.len(),
                    t.rows.len()
                ))
            } else {
                None
            }
        }
        Statement::Narrative(n) => {
            if span_contains(&n.span, line, col) {
                Some(format!("Narrative **\"{}\"**", n.key))
            } else {
                None
            }
        }
        Statement::Relation(r) => {
            if span_contains(&r.span, line, col) {
                Some(format!("_{}_  {}  _{}_", r.subject, r.predicate, r.object))
            } else {
                None
            }
        }
        Statement::ObjectBlock(obj) => object_block_hover(obj, line, col),
        Statement::Rule(rule) => {
            if span_contains(&rule.span, line, col) {
                Some(format!("Rule: {}", rule.source))
            } else {
                None
            }
        }
    }
}

fn object_block_hover(obj: &ObjectBlock, line: u32, col: u32) -> Option<String> {
    if !span_contains(&obj.span, line, col) {
        return None;
    }
    // Check inner statements first for a more specific match.
    if let Some(inner) = find_in_statements(obj.statements.iter(), line, col) {
        return Some(inner);
    }
    Some(format!(
        "Object **{}** — {} statements",
        obj.key,
        obj.statements.len()
    ))
}

// ---------------------------------------------------------------------------
// Formatters
// ---------------------------------------------------------------------------

fn format_directive(d: &Directive) -> String {
    if d.args.is_empty() {
        format!("@{}", d.name)
    } else {
        format!("@{} {}", d.name, d.args.join(" "))
    }
}
