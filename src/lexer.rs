/// For string, number and boolean tokens, the value is also stored along with the
/// line-no but for all other tokens, only line-no is stored. This line-no is used
/// for reporting error while parsing
#[derive(Debug, PartialEq)]
pub enum Token {
    Str(String, usize), // string + line-no
    Number(f64, usize), // number + line-no
    LeftBracket(usize), // line-no
    RightBracket(usize),
    LeftBrace(usize),
    RightBrace(usize),
    Comma(usize),
    Colon(usize),
    Bool(bool, usize),
    Null(usize),
    EOF(usize), // End-of-file
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Token::Str(s, line) => write!(f, "'{}' at line: {}", &s, line),
            Token::Number(n, line) => write!(f, "'{}' at line: {}", *n, line),
            Token::LeftBracket(line) => write!(f, "'[' at line: {}", line),
            Token::RightBracket(line) => write!(f, "']' at line: {}", line),
            Token::LeftBrace(line) => write!(f, "'{{' at line: {}", line),
            Token::RightBrace(line) => write!(f, "'}}' at line: {}", line),
            Token::Comma(line) => write!(f, "',' at line: {}", line),
            Token::Colon(line) => write!(f, "':' at line: {}", line),
            Token::Bool(b, line) => write!(f, "'{}' at line: {}", b, line),
            Token::Null(line) => write!(f, "'null' at line: {}", line),
            Token::EOF(line) => write!(f, "'EOF' at line: {}", line),
        }
    }
}

pub struct Lexer<'a> {
    pub tokens: Vec<Token>,
    line: usize,
    source: &'a str, // json source
}

#[derive(Debug)]
pub enum LexError {
    UnterminatedString(String),
    UnknownSymbol(String),
    UnknownLiteral(String),
    InvalidNumber(String),
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            tokens: Vec::new(),
            line: 1,
            source,
        }
    }

    pub fn lex(&mut self) -> Result<(), LexError> {
        // peekable lets us peek the current character instead of
        // consuming it
        let mut source_iter = self.source.char_indices().peekable();

        'outer: while let Some((start, current)) = source_iter.next() {
            match current {
                '[' => self.tokens.push(Token::LeftBracket(self.line)),
                ']' => self.tokens.push(Token::RightBracket(self.line)),
                '{' => self.tokens.push(Token::LeftBrace(self.line)),
                '}' => self.tokens.push(Token::RightBrace(self.line)),
                ':' => self.tokens.push(Token::Colon(self.line)),
                ',' => self.tokens.push(Token::Comma(self.line)),
                '"' => {
                    // unicode is not handled
                    let string_start = self.line;
                    let mut string = String::new();
                    while let Some((_, current)) = source_iter.next() {
                        if current == '\n' {
                            string.push(current);
                            self.line += 1
                        } else if current == '\\' {
                            if let Some((_, current)) = source_iter.peek() {
                                match *current {
                                    'n' => string.push('\n'),
                                    't' => string.push('\t'),
                                    'r' => string.push('\r'),
                                    '\\' => string.push('\\'),
                                    c => string.push(c),
                                }
                            }
                        } else if current == '"' {
                            self.tokens.push(Token::Str(string, string_start));
                            continue 'outer;
                        } else {
                            string.push(current);
                            continue;
                        }
                    }
                    // must have reached EOF, so the string is unterminated
                    return Err(LexError::UnterminatedString(format!(
                        "Unterminated string at line: {}",
                        string_start
                    )));
                }
                // skip whitespace
                ' ' | '\r' | '\t' => continue,
                '\n' => self.line += 1,
                c if c.is_alphabetic() => {
                    while let Some((_, current)) = source_iter.peek() {
                        if current.is_alphanumeric() || *current == '_' {
                            source_iter.next().unwrap();
                            continue;
                        } else {
                            break;
                        }
                    }
                    let end = if let Some((end, _)) = source_iter.peek() {
                        *end
                    } else {
                        self.source.len()
                    };

                    if &self.source[start..end] == "true" {
                        self.tokens.push(Token::Bool(true, self.line));
                    } else if &self.source[start..end] == "false" {
                        self.tokens.push(Token::Bool(false, self.line));
                    } else if &self.source[start..end] == "null" {
                        self.tokens.push(Token::Null(self.line));
                    } else {
                        return Err(LexError::UnknownLiteral(format!(
                            "Unknown literal '{}' at line: {}",
                            &self.source[start..end],
                            self.line
                        )));
                    }
                }
                c if c.is_numeric() || c == '+' || c == '-' => {
                    while let Some((_, current)) = source_iter.peek() {
                        if current.is_numeric() || matches!(current, '.' | 'e' | 'E' | '+' | '-') {
                            source_iter.next().unwrap();
                            continue;
                        } else {
                            break;
                        }
                    }
                    let end = if let Some((end, _)) = source_iter.peek() {
                        *end
                    } else {
                        self.source.len()
                    };

                    match self.source[start..end].parse::<f64>() {
                        Ok(f) => self.tokens.push(Token::Number(f, self.line)),
                        Err(_) => {
                            return Err(LexError::InvalidNumber(format!(
                                "Invalid number {} at line: {}",
                                &self.source[start..end],
                                self.line
                            )))
                        }
                    }
                }
                invalid => {
                    return Err(LexError::UnknownSymbol(format!(
                        "Unknown symbol {} at line: {}",
                        invalid, self.line
                    )))
                }
            }
        }

        self.tokens.push(Token::EOF(self.line));
        Ok(()) // Lexing successful
    }
}

#[test]
fn test_print() {
    let token = Token::EOF(1);
    assert_eq!(format!("{}", token), "'EOF' at line: 1");
}

#[test]
fn test_json_lexer() {
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

    let expected = vec![
        Token::LeftBrace(2),
        Token::Str("name".to_string(), 3),
        Token::Colon(3),
        Token::Str("Alice".to_string(), 3),
        Token::Comma(3),
        Token::Str("age".to_string(), 4),
        Token::Colon(4),
        Token::Number(30.0, 4),
        Token::Comma(4),
        Token::Str("is_student".to_string(), 5),
        Token::Colon(5),
        Token::Bool(true, 5),
        Token::Comma(5),
        Token::Str("scores".to_string(), 6),
        Token::Colon(6),
        Token::LeftBracket(6),
        Token::Number(95.5, 6),
        Token::Comma(6),
        Token::Number(88.0, 6),
        Token::Comma(6),
        Token::Number(76.0, 6),
        Token::RightBracket(6),
        Token::Comma(6),
        Token::Str("address".to_string(), 7),
        Token::Colon(7),
        Token::Null(7),
        Token::RightBrace(8),
        Token::EOF(9),
    ];

    assert_eq!(lexer.tokens, expected);
}
