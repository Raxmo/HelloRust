use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub byte_offset: usize,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct TokenWithPos {
    pub token: Token,
    pub pos: Position,
}

macro_rules! tokens {
    (
        $( ($char_match:expr, $variant:ident, $display:expr, $handler:expr) ),* $(,)?
        ; $( $compound_variant:ident ),* $(,)?
    ) => {
        #[derive(Debug, Clone, PartialEq)]
        pub enum Token {
            $( $variant, )*
            $( $compound_variant, )*
            Identifier(String),
            Number(f64),
            String(String),
            Keyword(String),
            Eof,
        }

        impl fmt::Display for Token {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                match self {
                    $( Token::$variant => write!(f, $display), )*
                    Token::Identifier(s) => write!(f, "{}", s),
                    Token::Number(n) => write!(f, "{}", n),
                    Token::String(s) => write!(f, "\"{}\"", s),
                    Token::Keyword(s) => write!(f, "{}", s),
                    Token::NotEq => write!(f, "!="),
                    Token::GtEq => write!(f, ">="),
                    Token::LtEq => write!(f, "<="),
                    Token::RightArrow => write!(f, "->"),
                    Token::Eof => write!(f, "EOF"),
                }
            }
        }

        impl Lexer {
            fn dispatch_token(&mut self, ch: char) -> Result<Option<Token>, String> {
                match ch {
                    $( $char_match => $handler(self).map(Some), )*
                    _ => Ok(None),
                }
            }
        }
    };
}

// TODO: Refine token macro to better handle multi-character operators (!=, ->, >=, <=)
// Current approach works but could be cleaner. Consider:
// - Whether compound tokens should be generated differently
// - If handler logic for multi-char ops can be simplified
// - Better organization of single vs compound token metadata
tokens!(
    ('[', OpenBracket, "[", |this: &mut Lexer| {
        this.advance();
        Ok(Token::OpenBracket)
    }),
    (']', CloseBracket, "]", |this: &mut Lexer| {
        this.advance();
        Ok(Token::CloseBracket)
    }),
    (':', Colon, ":", |this: &mut Lexer| {
        this.advance();
        Ok(Token::Colon)
    }),
    (',', Comma, ",", |this: &mut Lexer| {
        this.advance();
        Ok(Token::Comma)
    }),
    ('+', Plus, "+", |this: &mut Lexer| {
        this.advance();
        Ok(Token::Plus)
    }),
    ('*', Star, "*", |this: &mut Lexer| {
        this.advance();
        Ok(Token::Star)
    }),
    ('=', Eq, "=", |this: &mut Lexer| {
        this.advance();
        Ok(Token::Eq)
    }),
    ('-', Minus, "-", |this: &mut Lexer| {
        if this.peek(1) == Some('>') {
            this.advance();
            this.advance();
            Ok(Token::RightArrow)
        } else if this.peek(1).map_or(false, |c| c.is_numeric()) {
            this.advance();
            let num = this.read_number()?;
            Ok(Token::Number(-num))
        } else {
            this.advance();
            Ok(Token::Minus)
        }
    }),
    ('/', Slash, "/", |this: &mut Lexer| {
        if this.peek(1) == Some('/') {
            this.skip_line_comment();
            this.next_token()
        } else if this.peek(1) == Some('*') {
            this.skip_block_comment()?;
            this.next_token()
        } else {
            this.advance();
            Ok(Token::Slash)
        }
    }),
    ('!', Bang, "!", |this: &mut Lexer| {
        if this.peek(1) == Some('=') {
            this.advance();
            this.advance();
            Ok(Token::NotEq)
        } else {
            Err("Unexpected '!'".to_string())
        }
    }),
    ('>', Gt, ">", |this: &mut Lexer| {
        if this.peek(1) == Some('=') {
            this.advance();
            this.advance();
            Ok(Token::GtEq)
        } else {
            this.advance();
            Ok(Token::Gt)
        }
    }),
    ('<', Lt, "<", |this: &mut Lexer| {
        if this.peek(1) == Some('=') {
            this.advance();
            this.advance();
            Ok(Token::LtEq)
        } else {
            this.advance();
            Ok(Token::Lt)
        }
    });
    NotEq, GtEq, LtEq, RightArrow
);

// Lexer converts source code to tokens
pub struct Lexer {
    input: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
        }
    }

    fn get_position(&self) -> Position {
        Position {
            byte_offset: self.position,
            line: self.line,
            column: self.column,
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
        if let Some(ch) = self.current() {
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
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
            Some(ch) => {
                // Try dispatch_token() first for known single-char tokens
                if let Some(token) = self.dispatch_token(ch)? {
                    return Ok(token);
                }

                // Handle special cases
                match ch {
                    '"' | '\'' => {
                        let s = self.read_string()?;
                        Ok(Token::String(s))
                    }
                    c if c.is_numeric() => {
                        let num = self.read_number()?;
                        Ok(Token::Number(num))
                    }
                    c if c.is_alphabetic() || c == '_' => {
                        let ident = self.read_identifier();
                        let token = match ident.as_str() {
                            "on" | "off" | "and" | "or" | "not" => Token::Keyword(ident),
                            _ => Token::Identifier(ident),
                        };
                        Ok(token)
                    }
                    c => {
                        Err(format!("Unexpected character: '{}'", c))
                    }
                }
            }
        }
    }
}

// Tokenize source code into a vector of tokens
pub fn tokenize(input: &str) -> Result<Vec<TokenWithPos>, String> {
    let mut lexer = Lexer::new(input);
    let mut tokens = Vec::new();

    loop {
        let pos = lexer.get_position();
        let token = lexer.next_token()?;
        let is_eof = token == Token::Eof;
        tokens.push(TokenWithPos { token, pos });
        if is_eof {
            break;
        }
    }

    Ok(tokens)
}