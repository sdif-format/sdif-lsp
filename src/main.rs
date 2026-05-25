//! Entry point for the SDIF Language Server Protocol server.
//!
//! Starts a tower-lsp server communicating over stdin/stdout.

use std::env;
use std::fs;

use tower_lsp::{LspService, Server};

mod backend;
mod completion;
mod diagnostics;
mod document;
mod hover;
mod position;
mod semantic_tokens;

use backend::Backend;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && args[1] == "--dump-semantic-tokens" {
        if args.len() < 3 {
            eprintln!("Usage: sdif-lsp --dump-semantic-tokens <file>");
            std::process::exit(1);
        }
        let file_path = &args[2];
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error reading file {}: {}", file_path, e);
                std::process::exit(1);
            }
        };
        let tokens = semantic_tokens::build_semantic_tokens_from_text(&content);
        let decoded_json = match semantic_tokens::decode_tokens_to_json(&tokens) {
            Ok(json) => json,
            Err(e) => {
                eprintln!("Error serializing semantic tokens to JSON: {}", e);
                std::process::exit(1);
            }
        };
        println!("{}", decoded_json);
        return;
    }

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let (service, socket) = LspService::new(|client| Backend::new(client));
    Server::new(stdin, stdout, socket).serve(service).await;
}
