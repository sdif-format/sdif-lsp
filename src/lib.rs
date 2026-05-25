//! Public API surface for sdif-lsp — exposed for integration tests.
//!
//! The binary (main.rs) compiles its own module graph independently.
//! This library re-exports the same source files so integration tests
//! in tests/ can import them via `sdif_lsp::`.

#[path = "completion.rs"]
pub mod completion;

#[path = "diagnostics.rs"]
pub mod diagnostics;

#[path = "hover.rs"]
pub mod hover;

#[path = "position.rs"]
pub mod position;

#[path = "semantic_tokens.rs"]
pub mod semantic_tokens;
