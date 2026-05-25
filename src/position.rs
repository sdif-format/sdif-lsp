//! UTF-16–safe LSP position utilities.
//!
//! LSP positions use UTF-16 code units for column offsets. Rust strings are
//! UTF-8, so naive byte indexing breaks on any non-ASCII character. All cursor
//! operations that touch line text must go through this module.

/// Return the text on `line` (0-based) that appears before the LSP cursor at
/// `character` (0-based, UTF-16 code units).
///
/// Returns an empty string when the line or position is out of range.
pub fn text_before_lsp_cursor<'a>(text: &'a str, line: u32, character: u32) -> &'a str {
    let line_text = text
        .split('\n')
        .nth(line as usize)
        .unwrap_or("")
        .trim_end_matches('\r');
    byte_prefix_up_to_utf16(line_text, character)
}

/// Convert a 1-based byte column from `sdif-rs` spans to a 0-based UTF-16 character offset.
///
/// `line` is 0-based; `byte_col_1based` is 1-based (as emitted by sdif-rs).
/// Returns 0 when the line or column is out of range.
pub fn sdif_span_col_to_lsp_character(text: &str, line: u32, byte_col_1based: u32) -> u32 {
    let line_str = text
        .split('\n')
        .nth(line as usize)
        .unwrap_or("")
        .trim_end_matches('\r');
    let byte_col = (byte_col_1based.saturating_sub(1) as usize).min(line_str.len());
    if !line_str.is_char_boundary(byte_col) {
        return 0;
    }
    line_str[..byte_col].encode_utf16().count() as u32
}

/// Return the text on `line` (0-based). Handles both `\n` and `\r\n`.
pub fn line_text(text: &str, line: u32) -> &str {
    text.split('\n')
        .nth(line as usize)
        .unwrap_or("")
        .trim_end_matches('\r')
}

/// Slice `s` to include only the bytes before UTF-16 column `character`.
///
/// If `character` falls inside a surrogate pair (SMP character), the slice
/// is clamped to before that character. If `character` is past the end, the
/// full string is returned.
fn byte_prefix_up_to_utf16(s: &str, character: u32) -> &str {
    let mut utf16_remaining = character;
    for (byte_pos, ch) in s.char_indices() {
        if utf16_remaining == 0 {
            return &s[..byte_pos];
        }
        let units = ch.len_utf16() as u32;
        if units > utf16_remaining {
            // Cursor lands inside a surrogate pair; clamp to before this char.
            return &s[..byte_pos];
        }
        utf16_remaining -= units;
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_cursor_returns_correct_prefix() {
        assert_eq!(text_before_lsp_cursor("@sdif 1.0\n", 0, 5), "@sdif");
    }

    #[test]
    fn emoji_cursor_uses_utf16_units() {
        // "kind \"😀\"" — 😀 is U+1F600, 2 UTF-16 units, 4 UTF-8 bytes
        // Layout:  k(0) i(1) n(2) d(3) (4) "(5) 😀(6-7 in UTF-16) "(8) → character=8 → before last "
        let text = "kind \"😀\"\n";
        assert_eq!(text_before_lsp_cursor(text, 0, 8), "kind \"😀");
    }

    #[test]
    fn accented_char_uses_utf16_units() {
        // "á" is U+00E1 — 1 UTF-16 unit, 2 UTF-8 bytes
        let text = "kind á\n";
        // character=6 → should include "kind á"
        assert_eq!(text_before_lsp_cursor(text, 0, 6), "kind á");
        // character=5 → "kind "
        assert_eq!(text_before_lsp_cursor(text, 0, 5), "kind ");
    }

    #[test]
    fn out_of_range_character_returns_full_line() {
        assert_eq!(text_before_lsp_cursor("abc\n", 0, 100), "abc");
    }

    #[test]
    fn zero_character_returns_empty() {
        assert_eq!(text_before_lsp_cursor("abc\n", 0, 0), "");
    }

    #[test]
    fn out_of_range_line_returns_empty() {
        assert_eq!(text_before_lsp_cursor("abc\n", 99, 0), "");
    }

    #[test]
    fn crlf_line_endings_stripped() {
        assert_eq!(text_before_lsp_cursor("abc\r\n", 0, 3), "abc");
    }
}
