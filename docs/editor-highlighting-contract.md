# SDIF LSP Editor Contract

## Sources of truth

- `tree-sitter-sdif` owns syntax highlighting structure.
- `sdif-rs` owns normative parsing and diagnostics.
- `sdif-lsp` adapts both into LSP features.
- The VS Code extension owns final visual presentation through semantic tokens, scopes and theme rules.

## Required editor features

- Diagnostics on open/change/close.
- Hover for directives, fields, tables, narratives, relations, object blocks and rules.
- Completion for directives and common SDIF constructs.
- Semantic tokens for directives, keys, tables, columns, values, comments, strings, numbers and operators.
- Column modifiers `sdifColumn0`–`sdifColumn7` distinguish table column positions visually.
- Reproducible token dump through `sdif-lsp --dump-semantic-tokens`.

## Semantic token type mapping

| Token type  | SDIF construct                                    |
|-------------|---------------------------------------------------|
| `keyword`   | Directive names (`sdif`, `sdif.ai`), structural keys (`kind`, `rel`, `rules`, `alias`) |
| `type`      | Table names, narrative keys, block identifiers    |
| `property`  | Table column headers, alias entries               |
| `variable`  | Table row identifiers, grouped relation subjects  |
| `enum`      | `null`, `true`, `false`, domain constants         |
| `string`    | Quoted strings, unquoted scalars, narrative text  |
| `number`    | Numeric literals                                  |
| `operator`  | `@`, `"""`, brackets, delimiters                  |
| `comment`   | Lines starting with `#`                           |

## Column modifier bit layout

Bits 10–17 in the token modifier bitset encode the column index (0-based).
Column 8 and beyond use modifier `sdifColumn7`.

```
bit 10 → sdifColumn0
bit 11 → sdifColumn1
...
bit 17 → sdifColumn7
```

## Non-goals for this slice

- Full schema-aware validation.
- Cross-file include resolution.
- Rename/reference support.
- Formatting.
- Semantic diff or merge.
