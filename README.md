<p align="center">
  <img src="https://raw.githubusercontent.com/sdif-format/.github/main/profile/assets/sdif-logo-t.png" alt="SDIF LSP" width="320">
</p>

<p align="center">
  <strong>SDIF Language Server (LSP)</strong>
</p>

<p align="center">
  Language Server Protocol (LSP) implementation for the Semantic Data Interchange Format (SDIF),<br>
  built with tower-lsp, sdif-rs, and tree-sitter-sdif.
</p>

<p align="center">
  <a href="#what-is-sdif-lsp">What is sdif-lsp?</a>
  ·
  <a href="#capabilities">Capabilities</a>
  ·
  <a href="#development">Development</a>
  ·
  <a href="#configuration">Configuration</a>
  ·
  <a href="#ecosystem">Ecosystem</a>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/language-Rust-2563eb?style=flat-square" alt="Rust">
  <img src="https://img.shields.io/badge/spec-v1.0.0-0f766e?style=flat-square" alt="Spec v1.0.0">
  <img src="https://img.shields.io/badge/status-active-16a34a?style=flat-square" alt="Status: Active">
</p>

<br>

<div align="center">

<table>
  <tr>
    <td align="center" width="25%">
      <strong>Compact</strong>
      <br><br>
      Less repeated structure.<br>
      Fewer wasted tokens.
    </td>
    <td align="center" width="25%">
      <strong>Semantic</strong>
      <br><br>
      Tables, relations,<br>
      metadata and intent.
    </td>
    <td align="center" width="25%">
      <strong>Canonical</strong>
      <br><br>
      Stable output for hashing,<br>
      signing and comparison.
    </td>
    <td align="center" width="25%">
      <strong>Auditable</strong>
      <br><br>
      Designed to be read,<br>
      reviewed and trusted.
    </td>
  </tr>
</table>

</div>

<br>

---

## What is sdif-lsp?

`sdif-lsp` is an LSP-compliant language server binary for SDIF files. It compiles down to a single binary that communicates with editors via stdin/stdout transport, providing real-time diagnostics, syntactic checks, and IDE features.

It leverages `sdif-rs` for semantic AST parsing and diagnostics, and uses `tree-sitter-sdif` captures to produce tokenized highlighting information.

<br>

---

## Capabilities

The server implements the following LSP capabilities:

- **Diagnostics** — Real-time syntax errors and policy limit violations on change.
- **Semantic Tokens** — Precision structural syntax highlighting via tree-sitter captures.
- **Hover** — View descriptions, fields, and structures details.
- **Completions** — Autocomplete support for fields, directives, and metadata.

<br>

---

## Development

Build and run cargo tests:

```bash
cargo build                  # compile debug binary
cargo build --release        # compile release binary -> target/release/sdif-lsp
cargo test                   # run test suite
```

<br>

---

## Configuration

Configure your editor to execute the compiled `sdif-lsp` binary. For example, using Neovim `nvim-lspconfig`:

```lua
vim.lsp.start({
  name = "sdif-lsp",
  cmd = { "/path/to/sdif-lsp" },
  filetypes = { "sdif", "sdif.ai" }
})
```

In VS Code, the [`vscode-sdif`](../vscode-sdif) extension acts as the client and automatically launches this binary if it is on your system `PATH`.

<br>

---

## Ecosystem

This GitHub organization hosts the official SDIF ecosystem: the core format, reference tooling, benchmarks, examples, libraries, and editor extensions.

<div align="center">

<table>
  <tr>
    <td width="33%" valign="top">
      <p><sub>PYTHON CLIENT & CLI</sub></p>
      <h3>sdif-py</h3>
      <p>
        Specification, parser, canonicalizer, and CLI.<br>
        The normative reference implementation.
      </p>
      <p><a href="https://github.com/sdif-format/sdif-py"><strong>Explore sdif-py →</strong></a></p>
    </td>
    <td width="33%" valign="top">
      <p><sub>SPECIFICATION (SSOT)</sub></p>
      <h3>sdif-spec</h3>
      <p>
        Official format specification, canonicalization rules,<br>
        and portable conformance test suite.
      </p>
      <p><a href="https://github.com/sdif-format/sdif-spec"><strong>View specification →</strong></a></p>
    </td>
    <td width="33%" valign="top">
      <p><sub>BENCHMARKS</sub></p>
      <h3>sdif-benchmarks</h3>
      <p>
        Reproducible benchmark datasets and reports comparing SDIF with JSON, YAML, XML, and CSV.
      </p>
      <p><a href="https://github.com/sdif-format/sdif-benchmarks"><strong>View benchmarks →</strong></a></p>
    </td>
  </tr>
  <tr>
    <td width="33%" valign="top">
      <p><sub>RUST IMPLEMENTATION</sub></p>
      <h3>sdif-rs</h3>
      <p>
        Pure Rust parser implementation with a span-annotated AST designed for editor tooling.
      </p>
      <p><a href="https://github.com/sdif-format/sdif-rs"><strong>Explore sdif-rs →</strong></a></p>
    </td>
    <td width="33%" valign="top">
      <p><sub>LANGUAGE SERVER (LSP)</sub></p>
      <h3>sdif-lsp</h3>
      <p>
        LSP language server binary (tower-lsp) providing real-time diagnostics and IDE features.
      </p>
      <p><a href="https://github.com/sdif-format/sdif-lsp"><strong>View sdif-lsp →</strong></a></p>
    </td>
    <td width="33%" valign="top">
      <p><sub>EDITOR INTEGRATION</sub></p>
      <h3>vscode-sdif</h3>
      <p>
        VS Code extension client providing syntax highlighting, diagnostics, and LSP configuration.
      </p>
      <p><a href="https://github.com/sdif-format/vscode-sdif"><strong>Open extension →</strong></a></p>
    </td>
  </tr>
  <tr>
    <td width="33%" valign="top">
      <p><sub>GRAMMAR FOUNDATION</sub></p>
      <h3>tree-sitter-sdif</h3>
      <p>
        Tree-sitter grammar foundation for syntax highlighting and incremental parsing.
      </p>
      <p><a href="https://github.com/sdif-format/tree-sitter-sdif"><strong>Open grammar →</strong></a></p>
    </td>
    <td width="33%" valign="top">
      <p><sub>DOCUMENTATION</sub></p>
      <h3>sdif-format.github.io</h3>
      <p>
        Official documentation website containing specification guides, tutorials, and examples.
      </p>
      <p><a href="https://github.com/sdif-format/sdif-format.github.io"><strong>Read docs →</strong></a></p>
    </td>
    <td width="33%" valign="top">
      <p><sub>ORGANIZATION META</sub></p>
      <h3>.github</h3>
      <p>
        Organization profile, assets, and shared community configuration files.
      </p>
      <p><a href="https://github.com/sdif-format/.github"><strong>View profile →</strong></a></p>
    </td>
  </tr>
</table>

</div>

<br>

<details>
  <summary><strong>Repository map</strong></summary>

<br>

| Repository | Purpose |
| --------------------------------------------------------------------- | ---------------------------------------------------------------- |
| [`sdif-py`](https://github.com/sdif-format/sdif-py)                   | Core Python parser, validator, canonicalizer, and CLI |
| [`sdif-spec`](https://github.com/sdif-format/sdif-spec)               | Official format specification and conformance test suite (SSOT) |
| [`sdif-benchmarks`](https://github.com/sdif-format/sdif-benchmarks)   | Benchmark datasets, reports, and comparison tooling |
| [`sdif-rs`](https://github.com/sdif-format/sdif-rs)                   | Rust parser crate with span-annotated AST |
| [`sdif-lsp`](https://github.com/sdif-format/sdif-lsp)                 | LSP language server binary |
| [`tree-sitter-sdif`](https://github.com/sdif-format/tree-sitter-sdif) | Tree-sitter grammar foundation for syntax highlighting |
| [`vscode-sdif`](https://github.com/sdif-format/vscode-sdif)           | VS Code extension client for SDIF |
| [`sdif-format.github.io`](https://github.com/sdif-format/sdif-format.github.io) | Public documentation website (Docusaurus) |
| [`.github`](https://github.com/sdif-format/.github)                   | Organization profile, assets, and shared GitHub community files |

</details>

<br>

## License

See the [`sdif-spec`](../sdif-spec) repository for license information.
