use crate::lexer::Token;
use std::cell::Cell;
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
enum Value<'a> {
    Dict(HashMap<&'a str, Box<Value<'a>>>),
    List(Vec<Box<Value<'a>>>),
    Bool(bool),
    Str(&'a str),
    Number(f64),
    Null,
}

#[derive(Debug)]
enum ParseError {
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

    fn parse_dict(&self) -> Result<HashMap<&str, Box<Value>>, ParseError> {
        let mut result: HashMap<&str, Box<Value>> = HashMap::new();
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
                result.insert(&s, Box::new(value));
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

    fn parse_list(&self) -> Result<Vec<Box<Value>>, ParseError> {
        let mut result = Vec::new();
        loop {
            self.advance();
            let value = self.parse_value()?;
            result.push(Box::new(value));
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
fn test_atoms() {
    use crate::lexer::*;
    let source = "3.14";
    let mut lexer = Lexer::new(source);
    let _ = lexer.lex().unwrap();
    let parser = Parser::new(lexer.tokens);
    let value = parser.parse().unwrap();
    assert_eq!(value, Value::Number(3.14));
}
