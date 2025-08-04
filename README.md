# A Simple JSON Parser
A JSON parser written in pure Rust with no dependencies. It includes a pretty-printer that formats parsed JSON values into S-expressions.

# JSON Grammar Used
The following is an EBNF-like representation of the JSON grammar. Note: escape sequences in strings are **not** handled. All numbers (including integers) are parsed as `f64`.

- `value = dict | list | string | number | "true" | "false" | "null"`
- `list = "[" [value ("," value)*] "]"`
- `dict = "{" [pair ("," pair)*] "}"`
- `pair = string ":" value`

# JSON to S-Expression Conversion Rules
The parsed JSON is transformed into an S-expression as follows:

- `dict = "(" [pair (" " pair)*] ")"`
- `pair = "(" key " " value ")"`
- `list = "(" [value (" " value)*] ")"`
- Keys are printed without quotes.
- Atoms (`string`, `number`, `true`, `false`, `null`) are printed as-is.

# Example Usage
```rust
use json_parser::{Lexer, Parser, pretty_print};

let source = r#"
{
    "name": "Alice",
    "age": 30,
    "is_student": true,
    "scores": [95.5, 88.0, 76],
    "address": null
}
"#;

let mut lexer = Lexer::new(source);
let _ = lexer.lex().unwrap();
let parser = Parser::new(lexer.tokens);
let value = parser.parse().unwrap();

println!("{}", pretty_print(value)); // prints the S-expression for the JSON object
