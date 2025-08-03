# A simple json parser
A simple json parser written in pure rust without any dependencies. It has a pretty-printer which prints the parsed json value to s-expression.

# Json Grammar used 
The following is the EBNF like representation of Json grammar used. All the escape-sequences for the strings are not-handled. The floating-point type `f64` is used for all the numbers including integers.
- `value = dict | list | string | number | "true" | "false" | "null"`
- `list = "[" [value ("," value)*] "]"`
- `dict = "{" [pair ("," pair)*] "}"`
- `pair = string ":" value`

# Rules for converting json object to s-expression 
Json Grammer is transformed according to the following rules
- `dict = "(" [pair (" " pair)*] ")"`
- `pair = "(" key " " value ")"`
- `list = "(" [value (" "value)*] ")"`
- The keys are printed without quotes.
- The atoms `(string | number | "true" | "false" | "null")` are printed as is.

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
    let parser = Parser::new(lexer::tokens);
    let value = parser.parse().unwrap();
    println!("{}", pretty_print(value)); // this prints the s-expression for the json-object
```
