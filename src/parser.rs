use crate::lexer::Token;
use std::cell::Cell;
use std::collections::HashMap;

/// # Json Grammar
/// - `value = dict | list | string | number | "true" | "false" | "null"`
/// - `list = "[" [value ("," value)*] "]"`
/// - `dict = "{" [pair ("," pair)*] "}"`
/// - `pair = string ":" value`
#[derive(Debug, PartialEq)]
pub enum Value<'a> {
    Dict(HashMap<&'a str, Value<'a>>),
    List(Vec<Value<'a>>),
    Bool(bool),
    Str(&'a str),
    Number(f64),
    Null,
}

/// # Prints the s-expression representation of the json object
/// Json Grammer is transformed according to the following rules
/// - `dict = "(" [pair (" " pair)*] ")"`
/// - `pair = "(" key " " value ")"`
/// - `list = "(" [value (" "value)*] ")"`
/// - The keys are printed without quotes.
/// - The atoms `(string | number | "true" | "false" | "null")` are printed as is.
fn _pretty_print<'a>(val: &Value<'a>, indent: usize) -> String {
    let indent_str = "  ".repeat(indent);
    match val {
        Value::Dict(map) => {
            let mut result = String::from("(");
            for (key, value) in map {
                result.push_str(&format!(
                    "\n{}  ({} {})",
                    indent_str,
                    key,
                    _pretty_print(value, indent + 1)
                ));
            }
            result.push_str(&format!("\n{})", indent_str));
            result
        }
        Value::List(list) => {
            let mut result = String::from("(");
            for item in list {
                result.push_str(&format!(
                    "\n{}  {}",
                    indent_str,
                    _pretty_print(item, indent + 1)
                ));
            }
            result.push_str(&format!("\n{})", indent_str));
            result
        }
        Value::Str(s) => format!("\"{}\"", s),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
    }
}

pub fn pretty_print(value: &Value) -> String {
    _pretty_print(value, 0)
}

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken(String),
    InvalidKey(String),
}

pub struct Parser {
    tokens: Vec<Token>,
    current: Cell<usize>, // to allow interior mutability
}

impl Parser {
    // move the tokens emitted by the lexer
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: Cell::new(0),
        }
    }

    fn parse(&self) -> Result<Value, ParseError> {
        let value = self.parse_value()?;
        self.advance();
        // should be EOF
        if let Token::EOF(_) = &self.tokens[self.current.get()] {
            Ok(value)
        } else {
            Err(ParseError::UnexpectedToken(format!(
                "Expected EOF, got {}",
                &self.tokens[self.current.get()]
            )))
        }
    }

    /// `dict = "{" pair ("," pair)* "}"`
    /// `pair = string ":" value`
    fn parse_dict(&self) -> Result<HashMap<&str, Value>, ParseError> {
        let mut result: HashMap<&str, Value> = HashMap::new();
        loop {
            self.advance();
            if let Token::Str(s, _) = &self.tokens[self.current.get()] {
                self.advance();
                if let Token::Colon(_) = &self.tokens[self.current.get()] {
                    self.advance();
                } else {
                    return Err(ParseError::UnexpectedToken(format!(
                        "Expected ':', got {}",
                        &self.tokens[self.current.get()]
                    )));
                }
                let value = self.parse_value()?;
                self.advance();
                result.insert(&s, value);
            } else {
                return Err(ParseError::InvalidKey(format!(
                    "Expected string for key, got {}",
                    &self.tokens[self.current.get()]
                )));
            }
            if let Token::Comma(_) = &self.tokens[self.current.get()] {
                continue;
            } else {
                break;
            }
        }
        if let Token::RightBrace(_) = &self.tokens[self.current.get()] {
            Ok(result)
        } else {
            Err(ParseError::UnexpectedToken(format!(
                "Expected '}}', got {}",
                &self.tokens[self.current.get()]
            )))
        }
    }

    /// `list = "["value ("," value)*"]"`
    fn parse_list(&self) -> Result<Vec<Value>, ParseError> {
        let mut result = Vec::new();
        loop {
            self.advance();
            let value = self.parse_value()?;
            result.push(value);
            self.advance();
            if let Token::Comma(_) = &self.tokens[self.current.get()] {
                continue;
            } else {
                break;
            }
        }

        if let Token::RightBracket(_) = &self.tokens[self.current.get()] {
            Ok(result)
        } else {
            Err(ParseError::UnexpectedToken(format!(
                "Expected ']', got {}",
                &self.tokens[self.current.get()]
            )))
        }
    }

    fn advance(&self) {
        self.current.set(self.current.get() + 1);
    }

    /// `value = dict | list | string | number | "true" | "false" | "null"`
    fn parse_value(&self) -> Result<Value, ParseError> {
        match &self.tokens[self.current.get()] {
            // atoms
            Token::Str(s, _) => return Ok(Value::Str(&s)),
            Token::Bool(b, _) => return Ok(Value::Bool(*b)),
            Token::Number(n, _) => return Ok(Value::Number(*n)),
            Token::Null(_) => return Ok(Value::Null),
            // list
            Token::LeftBracket(_) => {
                // handle empty list
                if let Token::RightBracket(_) = &self.tokens[self.current.get() + 1] {
                    self.advance();
                    Ok(Value::List(Vec::new()))
                } else {
                    Ok(Value::List(self.parse_list()?))
                }
            }
            // dict
            Token::LeftBrace(_) => {
                // handle empty dict
                if let Token::RightBrace(_) = &self.tokens[self.current.get() + 1] {
                    self.advance();
                    Ok(Value::Dict(HashMap::new()))
                } else {
                    Ok(Value::Dict(self.parse_dict()?))
                }
            }
            unexpected_token => Err(ParseError::UnexpectedToken(format!(
                "Unexpected token {}",
                unexpected_token
            ))),
        }
    }
}

#[test]
fn test_string() {
    use crate::lexer::Lexer;
    let source = "\"test\"";
    let mut lexer = Lexer::new(source);
    lexer.lex().unwrap();
    let parser = Parser::new(lexer.tokens);
    let value = parser.parse().unwrap();
    assert_eq!(value, Value::Str("test"));
}

#[test]
fn test_number() {
    use crate::lexer::Lexer;
    let source = "3.14e-8";
    let mut lexer = Lexer::new(source);
    lexer.lex().unwrap();
    let parser = Parser::new(lexer.tokens);
    let value = parser.parse().unwrap();
    assert_eq!(value, Value::Number(3.14e-8));
}

#[test]
fn test_true() {
    use crate::lexer::Lexer;
    let source = "true";
    let mut lexer = Lexer::new(source);
    lexer.lex().unwrap();
    let parser = Parser::new(lexer.tokens);
    let value = parser.parse().unwrap();
    assert_eq!(value, Value::Bool(true));
}

#[test]
fn test_nested_structures() {
    use crate::lexer::Lexer;
    let input = r#"
        {
            "description": "The test case description",
            "schema": { "type": "string" },
            "tests": [
                {
                    "description": "a test with a valid instance",
                    "data": "a string",
                    "valid": true
                },
                {
                    "description": "a test with an invalid instance",
                    "data": 15,
                    "valid": false
                }
            ]
        }
        "#;

    let mut lexer = Lexer::new(input);
    lexer.lex().unwrap();
    let parser = Parser::new(lexer.tokens);
    let parsed = parser.parse().unwrap();

    let mut expected_map = std::collections::HashMap::new();

    expected_map.insert("description", Value::Str("The test case description"));

    let mut schema_map = std::collections::HashMap::new();
    schema_map.insert("type", Value::Str("string"));
    expected_map.insert("schema", Value::Dict(schema_map));

    let mut test1 = std::collections::HashMap::new();
    test1.insert("description", Value::Str("a test with a valid instance"));
    test1.insert("data", Value::Str("a string"));
    test1.insert("valid", Value::Bool(true));

    let mut test2 = std::collections::HashMap::new();
    test2.insert("description", Value::Str("a test with an invalid instance"));
    test2.insert("data", Value::Number(15.0));
    test2.insert("valid", Value::Bool(false));

    let test_list = vec![Value::Dict(test1), Value::Dict(test2)];

    expected_map.insert("tests", Value::List(test_list));

    let expected = Value::Dict(expected_map);

    assert_eq!(parsed, expected);
}
