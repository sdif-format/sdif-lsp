//! Hover information for SDIF documents.
//!
//! Uses tree-sitter to locate the source node under the cursor. Semantic text
//! comes from the parsed `sdif` document where that can be resolved without
//! source spans.

use sdif::Document;
use tree_sitter::{Parser, Point};

/// Return a Markdown hover string for `(lsp_line, lsp_character)`.
///
/// Positions are 0-based LSP positions. The raw `text` is required because the
/// Rust SDIF AST intentionally does not own source spans.
pub fn hover_at(text: &str, doc: &Document, lsp_line: u32, lsp_character: u32) -> Option<String> {
    let line_text = text.lines().nth(lsp_line as usize)?;

    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_sdif::language()).ok()?;
    let tree = parser.parse(text, None)?;
    let point = Point {
        row: lsp_line as usize,
        column: lsp_character as usize,
    };
    tree.root_node()
        .named_descendant_for_point_range(point, point)?;

    hover_for_source_line(text, line_text, doc, lsp_line)
}

fn hover_for_source_line(
    text: &str,
    line_text: &str,
    doc: &Document,
    lsp_line: u32,
) -> Option<String> {
    let trimmed = line_text.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return None;
    }

    if let Some(name) = directive_name(trimmed) {
        return Some(directive_docs(name));
    }

    if let Some((name, _columns)) = table_header(trimmed) {
        if let Some(table) = doc.tables().find(|table| table.name == name) {
            return Some(format!(
                "Table **{}** — {} columns, {} rows\n\nColumns: `{}`",
                table.name,
                table.columns.len(),
                table.rows.len(),
                table.columns.join(", ")
            ));
        }
    }

    if let Some(table) = enclosing_table(text, doc, lsp_line) {
        return Some(format!(
            "Table **{}** — {} columns, {} rows\n\nColumns: `{}`",
            table.name,
            table.columns.len(),
            table.rows.len(),
            table.columns.join(", ")
        ));
    }

    if is_relation_line(text, lsp_line, trimmed) {
        return Some(format!("**Relation** `{}`", trimmed));
    }

    if trimmed.ends_with(':') && !trimmed.contains('[') {
        let key = trimmed.trim_end_matches(':');
        if key == "rel" {
            return Some("**Relation block**".to_string());
        }
        if key == "rules" {
            return Some("**Rules block**".to_string());
        }
        if let Some(obj) = doc.objects().find(|object| object.key == key) {
            return Some(format!(
                "**Object block** `{}`\n\n{} statements.",
                obj.key,
                obj.statements.len()
            ));
        }
    }

    if let Some(key) = field_key(trimmed) {
        if let Some(field) = doc.fields().find(|field| field.key == key) {
            return Some(format!(
                "**Field** `{}`\n\nString value: `{}`",
                field.key, field.value
            ));
        }
    }

    None
}

fn directive_name(trimmed: &str) -> Option<&str> {
    let rest = trimmed.strip_prefix('@')?;
    rest.split_whitespace().next()
}

fn table_header(trimmed: &str) -> Option<(&str, &str)> {
    let (name, rest) = trimmed.split_once('[')?;
    let columns = rest.strip_suffix(":")?.strip_suffix(']')?;
    if name.is_empty() {
        None
    } else {
        Some((name, columns))
    }
}

fn enclosing_table<'a>(text: &str, doc: &'a Document, lsp_line: u32) -> Option<&'a sdif::Table> {
    let lines: Vec<&str> = text.lines().collect();
    let current = *lines.get(lsp_line as usize)?;
    if current.trim().is_empty() || !current.starts_with(' ') {
        return None;
    }
    for prior in lines[..lsp_line as usize].iter().rev() {
        let trimmed = prior.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if !prior.starts_with(' ') {
            if let Some((name, _)) = table_header(trimmed) {
                return doc.tables().find(|table| table.name == name);
            }
            return None;
        }
    }
    None
}

fn is_relation_line(text: &str, lsp_line: u32, trimmed: &str) -> bool {
    if trimmed.ends_with(':') || trimmed.starts_with('@') || trimmed.contains('[') {
        return false;
    }
    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    if parts.len() != 3 {
        return false;
    }
    let lines: Vec<&str> = text.lines().collect();
    let current = match lines.get(lsp_line as usize) {
        Some(line) => *line,
        None => return false,
    };
    if !current.starts_with(' ') {
        return false;
    }
    for prior in lines[..lsp_line as usize].iter().rev() {
        let prior_trimmed = prior.trim();
        if prior_trimmed.is_empty() || prior_trimmed.starts_with('#') {
            continue;
        }
        return prior_trimmed == "rel:";
    }
    false
}

fn field_key(trimmed: &str) -> Option<&str> {
    if trimmed.ends_with(':') || trimmed.starts_with('@') || trimmed.contains('[') {
        return None;
    }
    trimmed.split_whitespace().next()
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
        let text = "@sdif 1.0\nname \"Alice\"\n";
        let doc = parse_text(text).unwrap();
        assert!(
            hover_at(text, &doc, 1, 0).is_some(),
            "expected hover on field key"
        );
    }

    #[test]
    fn test_hover_on_empty_line_returns_none() {
        let text = "@sdif 1.0\nname \"Alice\"\n\n";
        let doc = parse_text(text).unwrap();
        assert!(hover_at(text, &doc, 2, 0).is_none());
    }

    #[test]
    fn test_hover_on_directive_returns_some() {
        let text = "@sdif 1.0\nname \"Alice\"\n";
        let doc = parse_text(text).unwrap();
        assert!(hover_at(text, &doc, 0, 0).is_some());
        let content = hover_at(text, &doc, 0, 0).unwrap();
        assert!(content.contains("@sdif"));
    }
}
