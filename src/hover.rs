//! Hover information for SDIF documents.

use sdif::{Document, Statement};

/// Return a Markdown hover string for the AST node at `(lsp_line, lsp_character)`.
///
/// `lsp_line` and `lsp_character` are 0-based LSP positions.
/// sdif-rs spans are 1-based, so we convert before calling `span.contains()`.
pub fn hover_at(doc: &Document, lsp_line: u32, lsp_character: u32) -> Option<String> {
    let line = lsp_line + 1;
    let col = lsp_character + 1;

    for directive in &doc.directives {
        if directive.span.contains(line, col) {
            return Some(directive_docs(&directive.name));
        }
    }

    for stmt in &doc.statements {
        match stmt {
            Statement::Field(f) if f.span.contains(line, col) => {
                return Some(format!("**Field** `{}`\n\nString value.", f.key));
            }
            Statement::Table(t) if t.span.contains(line, col) => {
                return Some(format!(
                    "**Table** `{}`\n\nColumns: {}",
                    t.name,
                    t.columns.join(", ")
                ));
            }
            Statement::Relation(r) if r.span.contains(line, col) => {
                return Some(format!(
                    "**Relation** `{} {} {}`",
                    r.subject, r.predicate, r.object
                ));
            }
            Statement::Narrative(n) if n.span.contains(line, col) => {
                return Some(format!(
                    "**Narrative** `{}`\n\nMultiline text block.",
                    n.key
                ));
            }
            Statement::Rule(r) if r.span.contains(line, col) => {
                return Some(format!("**Rule** `{}`", r.source));
            }
            Statement::ObjectBlock(ob) if ob.span.contains(line, col) => {
                return Some(format!("**Object block** `{}`", ob.key));
            }
            _ => {}
        }
    }

    None
}

fn directive_docs(name: &str) -> String {
    match name {
        "sdif" => "**`@sdif`** — Format version directive. Declares the SDIF format version (e.g., `@sdif 1.0`).".to_string(),
        "sdif.ai" => "**`@sdif.ai`** — AI profile directive. Declares the compact AI projection format.".to_string(),
        "schema" => "**`@schema`** — Schema directive. References a schema for validation.".to_string(),
        "namespace" => "**`@namespace`** — Namespace prefix declaration. Form: `@namespace prefix iri`.".to_string(),
        "include" => "**`@include`** — Include directive. Includes another local document. Disabled by default (requires explicit policy).".to_string(),
        "alias" => "**`@alias`** — Alias directive. Declares a short alias for a longer key.".to_string(),
        _ => format!("**`@{}`** — Directive.", name),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sdif::parse_text;

    #[test]
    fn test_hover_on_field_key_returns_some() {
        // "name" is at line 2 (1-based) → LSP line 1, col 0
        let doc = parse_text("@sdif 1.0\nname \"Alice\"\n").unwrap();
        assert!(hover_at(&doc, 1, 0).is_some(), "expected hover on field key");
    }

    #[test]
    fn test_hover_on_empty_line_returns_none() {
        let doc = parse_text("@sdif 1.0\nname \"Alice\"\n\n").unwrap();
        assert!(hover_at(&doc, 2, 0).is_none());
    }

    #[test]
    fn test_hover_on_directive_returns_some() {
        // "@sdif" is at line 1 (1-based) → LSP line 0, col 0
        let doc = parse_text("@sdif 1.0\nname \"Alice\"\n").unwrap();
        assert!(hover_at(&doc, 0, 0).is_some());
        let content = hover_at(&doc, 0, 0).unwrap();
        assert!(content.contains("@sdif"));
    }
}
