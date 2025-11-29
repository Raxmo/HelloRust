use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    OpenBracket,
    CloseBracket,
    Colon,
    Comma,
    Arrow,
    Identifier(String),
    Number(f64),
    String(String),
    Keyword(String),
    Plus,
    Minus,
    Star,
    Slash,
    Eq,
    NotEq,
    Gt,
    Lt,
    GtEq,
    LtEq,
    And,
    Or,
    Not,
    Eof,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::OpenBracket => write!(f, "["),
            Token::CloseBracket => write!(f, "]"),
            Token::Colon => write!(f, ":"),
            Token::Comma => write!(f, ","),
            Token::Arrow => write!(f, "->"),
            Token::Identifier(s) => write!(f, "{}", s),
            Token::Number(n) => write!(f, "{}", n),
            Token::String(s) => write!(f, "\"{}\"", s),
            Token::Keyword(s) => write!(f, "{}", s),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Star => write!(f, "*"),
            Token::Slash => write!(f, "/"),
            Token::Eq => write!(f, "="),
            Token::NotEq => write!(f, "!="),
            Token::Gt => write!(f, ">"),
            Token::Lt => write!(f, "<"),
            Token::GtEq => write!(f, ">="),
            Token::LtEq => write!(f, "<="),
            Token::And => write!(f, "and"),
            Token::Or => write!(f, "or"),
            Token::Not => write!(f, "not"),
            Token::Eof => write!(f, "EOF"),
        }
    }
}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
}

impl Lexer {
    fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            position: 0,
        }
    }

    fn current(&self) -> Option<char> {
        if self.position < self.input.len() {
            Some(self.input[self.position])
        } else {
            None
        }
    }

    fn peek(&self, offset: usize) -> Option<char> {
        let pos = self.position + offset;
        if pos < self.input.len() {
            Some(self.input[pos])
        } else {
            None
        }
    }

    fn advance(&mut self) {
        self.position += 1;
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current() {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_line_comment(&mut self) {
        self.advance();
        self.advance();
        while let Some(ch) = self.current() {
            if ch == '\n' {
                self.advance();
                break;
            }
            self.advance();
        }
    }

    fn skip_block_comment(&mut self) -> Result<(), String> {
        self.advance();
        self.advance();
        while let Some(ch) = self.current() {
            if ch == '*' && self.peek(1) == Some('/') {
                self.advance();
                self.advance();
                return Ok(());
            }
            self.advance();
        }
        Err("Unclosed block comment".to_string())
    }

    fn read_identifier(&mut self) -> String {
        let mut result = String::new();
        while let Some(ch) = self.current() {
            if ch.is_alphanumeric() || ch == '_' || ch == '-' {
                result.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        result
    }

    fn read_number(&mut self) -> Result<f64, String> {
        let mut result = String::new();
        let mut has_dot = false;

        while let Some(ch) = self.current() {
            if ch.is_numeric() {
                result.push(ch);
                self.advance();
            } else if ch == '.' && !has_dot && self.peek(1).map_or(false, |c| c.is_numeric()) {
                has_dot = true;
                result.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        result.parse::<f64>().map_err(|_| "Invalid number".to_string())
    }

    fn read_string(&mut self) -> Result<String, String> {
        let quote = self.current().unwrap();
        self.advance();

        let mut result = String::new();
        while let Some(ch) = self.current() {
            if ch == quote {
                self.advance();
                return Ok(result);
            } else if ch == '\\' {
                self.advance();
                match self.current() {
                    Some('n') => result.push('\n'),
                    Some('t') => result.push('\t'),
                    Some('\\') => result.push('\\'),
                    Some('"') => result.push('"'),
                    Some('\'') => result.push('\''),
                    Some(c) => result.push(c),
                    None => return Err("Unterminated string".to_string()),
                }
                self.advance();
            } else {
                result.push(ch);
                self.advance();
            }
        }

        Err("Unterminated string".to_string())
    }

    fn next_token(&mut self) -> Result<Token, String> {
        self.skip_whitespace();

        match self.current() {
            None => Ok(Token::Eof),
            Some('[') => {
                self.advance();
                Ok(Token::OpenBracket)
            }
            Some(']') => {
                self.advance();
                Ok(Token::CloseBracket)
            }
            Some(':') => {
                self.advance();
                Ok(Token::Colon)
            }
            Some(',') => {
                self.advance();
                Ok(Token::Comma)
            }
            Some('-') => {
                if self.peek(1) == Some('>') {
                    self.advance();
                    self.advance();
                    Ok(Token::Arrow)
                } else if self.peek(1).map_or(false, |c| c.is_numeric()) {
                    self.advance();
                    let num = self.read_number()?;
                    Ok(Token::Number(-num))
                } else {
                    self.advance();
                    Ok(Token::Minus)
                }
            }
            Some('+') => {
                self.advance();
                Ok(Token::Plus)
            }
            Some('*') => {
                self.advance();
                Ok(Token::Star)
            }
            Some('/') => {
                if self.peek(1) == Some('/') {
                    self.skip_line_comment();
                    self.next_token()
                } else if self.peek(1) == Some('*') {
                    self.skip_block_comment()?;
                    self.next_token()
                } else {
                    self.advance();
                    Ok(Token::Slash)
                }
            }
            Some('=') => {
                self.advance();
                Ok(Token::Eq)
            }
            Some('!') => {
                if self.peek(1) == Some('=') {
                    self.advance();
                    self.advance();
                    Ok(Token::NotEq)
                } else {
                    Err("Unexpected '!'".to_string())
                }
            }
            Some('>') => {
                if self.peek(1) == Some('=') {
                    self.advance();
                    self.advance();
                    Ok(Token::GtEq)
                } else {
                    self.advance();
                    Ok(Token::Gt)
                }
            }
            Some('<') => {
                if self.peek(1) == Some('=') {
                    self.advance();
                    self.advance();
                    Ok(Token::LtEq)
                } else {
                    self.advance();
                    Ok(Token::Lt)
                }
            }
            Some('"') | Some('\'') => {
                let s = self.read_string()?;
                Ok(Token::String(s))
            }
            Some(ch) if ch.is_numeric() => {
                let num = self.read_number()?;
                Ok(Token::Number(num))
            }
            Some(ch) if ch.is_alphabetic() || ch == '_' => {
                let ident = self.read_identifier();
                let token = match ident.as_str() {
                    "on" | "off" | "and" | "or" | "not" => Token::Keyword(ident),
                    _ => Token::Identifier(ident),
                };
                Ok(token)
            }
            Some(ch) => {
                Err(format!("Unexpected character: '{}'", ch))
            }
        }
    }
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut lexer = Lexer::new(input);
    let mut tokens = Vec::new();

    loop {
        let token = lexer.next_token()?;
        let is_eof = token == Token::Eof;
        tokens.push(token);
        if is_eof {
            break;
        }
    }

    Ok(tokens)
}
