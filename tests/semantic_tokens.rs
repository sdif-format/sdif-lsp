//! Integration tests for semantic token generation from SDIF editor fixtures.

use sdif_lsp::semantic_tokens::{build_semantic_tokens_from_text, decode_tokens_to_json};
use std::path::PathBuf;
use tower_lsp::lsp_types::SemanticToken;

fn spec_fixture(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../sdif-spec/fixtures/editor")
        .join(relative)
}

fn decode(tokens: &[SemanticToken]) -> Vec<(u32, u32, u32, u32, u32)> {
    let mut out = Vec::new();
    let mut line = 0u32;
    let mut character = 0u32;
    for t in tokens {
        line += t.delta_line;
        character = if t.delta_line == 0 {
            character + t.delta_start
        } else {
            t.delta_start
        };
        out.push((
            line,
            character,
            t.length,
            t.token_type,
            t.token_modifiers_bitset,
        ));
    }
    out
}

fn has_token(
    decoded: &[(u32, u32, u32, u32, u32)],
    line: u32,
    character: u32,
    length: u32,
    token_type: u32,
) -> bool {
    decoded
        .iter()
        .any(|&(l, c, len, tt, _)| l == line && c == character && len == length && tt == token_type)
}

// Token type indices (must match semantic_tokens::TOKEN_TYPES order)
const TT_TYPE: u32 = 1;
const TT_KEYWORD: u32 = 12;
const TT_COMMENT: u32 = 14;
const TT_STRING: u32 = 15;
const TT_NUMBER: u32 = 16;
const TT_OPERATOR: u32 = 18;

#[test]
fn build_semantic_tokens_from_highlighting_fixture() {
    let path = spec_fixture("highlighting.sdif");
    assert!(path.exists(), "fixture must exist: {path:?}");

    let content = std::fs::read_to_string(&path).unwrap();
    sdif_rs::parser::parse_text(&content).expect("highlighting.sdif must parse cleanly");

    let tokens = build_semantic_tokens_from_text(&content);
    assert!(!tokens.is_empty(), "must produce tokens");

    let decoded = decode(&tokens);
    assert!(has_token(&decoded, 0, 0, 1, TT_OPERATOR), "@ is operator");
    assert!(
        has_token(&decoded, 0, 1, 4, TT_KEYWORD),
        "sdif directive is keyword"
    );
    assert!(
        has_token(&decoded, 2, 0, 19, TT_COMMENT),
        "comment is comment"
    );
    assert!(has_token(&decoded, 3, 0, 4, TT_KEYWORD), "kind is keyword");
}

#[test]
fn build_semantic_tokens_from_highlighting_ai_fixture() {
    let path = spec_fixture("highlighting.sdif.ai");
    assert!(path.exists(), "fixture must exist: {path:?}");

    let content = std::fs::read_to_string(&path).unwrap();
    sdif_rs::parser::parse_text(&content).expect("highlighting.sdif.ai must parse cleanly");

    let tokens = build_semantic_tokens_from_text(&content);
    assert!(!tokens.is_empty(), "must produce tokens");

    let decoded = decode(&tokens);
    assert!(
        has_token(&decoded, 0, 1, 7, TT_KEYWORD),
        "sdif.ai directive is keyword"
    );
}

#[test]
fn semantic_token_snapshot_is_valid_json() {
    let path = spec_fixture("highlighting.sdif");
    let content = std::fs::read_to_string(&path).unwrap();
    let tokens = build_semantic_tokens_from_text(&content);
    let json = decode_tokens_to_json(&tokens).expect("decode_tokens_to_json must succeed");
    let parsed: serde_json::Value =
        serde_json::from_str(&json).expect("snapshot must be valid JSON");
    assert!(parsed.is_array(), "snapshot must be a JSON array");
}

#[test]
fn snapshot_matches_committed_file() {
    let fixture_path = spec_fixture("highlighting.sdif");
    let snapshot_path = spec_fixture("snapshots/highlighting.sdif.semantic-tokens.json");

    if !snapshot_path.exists() {
        // Snapshot not yet committed — skip rather than fail.
        return;
    }

    let content = std::fs::read_to_string(&fixture_path).unwrap();
    let tokens = build_semantic_tokens_from_text(&content);
    let current = decode_tokens_to_json(&tokens).unwrap();

    let committed = std::fs::read_to_string(&snapshot_path).unwrap();
    let current_val: serde_json::Value = serde_json::from_str(&current).unwrap();
    let committed_val: serde_json::Value = serde_json::from_str(&committed).unwrap();

    assert_eq!(current_val, committed_val, "semantic token output changed — regenerate snapshot with: cargo run --release -- --dump-semantic-tokens ../sdif-spec/fixtures/editor/highlighting.sdif > ../sdif-spec/fixtures/editor/snapshots/highlighting.sdif.semantic-tokens.json");
}
