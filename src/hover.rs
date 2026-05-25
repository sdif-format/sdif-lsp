//! Hover information for SDIF documents.
//!
//! NOTE: This module previously used span fields on AST nodes for
//! position-based lookup. Those span fields have been removed from AST nodes
//! (spans are now only on ParseError, for diagnostics). This hover
//! implementation is a stub pending the tree-sitter rewrite in Task 15.

use sdif::Document;

/// Return a Markdown hover string for the AST node at `(line, character)`.
///
/// `line` and `character` are 0-based LSP positions.
///
/// Currently returns `None` — position-based lookup will be reimplemented
/// using tree-sitter in Task 15.
pub fn hover_at(_doc: &Document, _line: u32, _character: u32) -> Option<String> {
    None
}
