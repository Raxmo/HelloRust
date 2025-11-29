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

// The Lexer struct holds the state of lexical analysis
// In C++, this would be a class with private members
pub struct Lexer {
    input: Vec<char>,   // The source code as a vector of characters
    position: usize,    // Current position in the input (like a file pointer)
}

impl Lexer {
    // In Rust, impl blocks define methods for a struct
    // This is similar to defining member functions in a C++ class
    
    // Private constructor (no pub keyword)
    // Creates a new Lexer from a string slice (&str)
    // Note: &str is a borrowed string reference (like const char* in C++)
    fn new(input: &str) -> Self {
        Lexer {
            // .chars() returns an iterator over characters
            // .collect() converts that iterator into a Vec<char>
            // This is more efficient than repeatedly indexing the original string
            input: input.chars().collect(),
            position: 0,  // Start at the beginning
        }
    }

    // Return the current character without advancing
    // Returns Option<char> (like Optional in C++17)
    // Option<T> is either Some(T) or None
    fn current(&self) -> Option<char> {
        if self.position < self.input.len() {
            Some(self.input[self.position])  // Return Some if in bounds
        } else {
            None  // Return None if we're past the end
        }
    }

    // Look ahead at character at current position + offset
    // Useful for detecting multi-character tokens like !=, >=, ->, etc
    fn peek(&self, offset: usize) -> Option<char> {
        let pos = self.position + offset;
        if pos < self.input.len() {
            Some(self.input[pos])
        } else {
            None
        }
    }

    // Move to the next character
    // &mut self means this method takes a mutable reference (can modify self)
    fn advance(&mut self) {
        self.position += 1;
    }

    // Skip whitespace characters (space, tab, newline, etc)
    fn skip_whitespace(&mut self) {
        // while let Some(ch) = ... is Rust's way of looping while Option is Some
        // This is like a do-while loop with pattern matching in C++
        while let Some(ch) = self.current() {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;  // Stop when we hit non-whitespace
            }
        }
    }

    // Skip a line comment (// to end of line)
    fn skip_line_comment(&mut self) {
        self.advance();  // Skip first /
        self.advance();  // Skip second /
        while let Some(ch) = self.current() {
            if ch == '\n' {
                self.advance();  // Skip the newline
                break;
            }
            self.advance();
        }
    }

    // Skip a block comment (/* to */)
    // Returns Result to signal if we found the closing */ or hit EOF
    fn skip_block_comment(&mut self) -> Result<(), String> {
        self.advance();  // Skip /
        self.advance();  // Skip *
        while let Some(ch) = self.current() {
            if ch == '*' && self.peek(1) == Some('/') {
                // Found closing */
                self.advance();
                self.advance();
                return Ok(());  // Success
            }
            self.advance();
        }
        // If we get here, we hit EOF without finding */
        Err("Unclosed block comment".to_string())
    }

    // Read an identifier (variable/operation name)
    // Valid characters: alphanumeric, underscore, hyphen
    fn read_identifier(&mut self) -> String {
        let mut result = String::new();
        while let Some(ch) = self.current() {
            if ch.is_alphanumeric() || ch == '_' || ch == '-' {
                result.push(ch);      // Add character to result
                self.advance();       // Move to next character
            } else {
                break;                // Stop at first invalid character
            }
        }
        result
    }

    // Read a number (integer or floating point)
    // Returns Result to signal invalid format
    fn read_number(&mut self) -> Result<f64, String> {
        let mut result = String::new();
        let mut has_dot = false;  // Track if we've seen a decimal point

        while let Some(ch) = self.current() {
            if ch.is_numeric() {
                result.push(ch);
                self.advance();
            } else if ch == '.' && !has_dot && self.peek(1).map_or(false, |c| c.is_numeric()) {
                // Only allow one dot, and only if followed by a digit
                // .map_or(false, |c| c.is_numeric()) means "if peek returns Some(c), check if it's numeric, else false"
                has_dot = true;
                result.push(ch);
                self.advance();
            } else {
                break;  // Stop at first non-numeric character
            }
        }

        // Try to parse the accumulated string as f64
        // .map_err() transforms the error type (convert parse error to our String error type)
        result.parse::<f64>().map_err(|_| "Invalid number".to_string())
    }

    // Read a quoted string, handling escape sequences
    // Returns Result to signal unterminated string
    fn read_string(&mut self) -> Result<String, String> {
        let quote = self.current().unwrap();  // Get the opening quote character
        self.advance();  // Skip the opening quote

        let mut result = String::new();
        while let Some(ch) = self.current() {
            if ch == quote {
                // Found closing quote
                self.advance();
                return Ok(result);
            } else if ch == '\\' {
                // Escape sequence
                self.advance();
                match self.current() {
                    Some('n') => result.push('\n'),      // \n = newline
                    Some('t') => result.push('\t'),      // \t = tab
                    Some('\\') => result.push('\\'),     // \\ = backslash
                    Some('"') => result.push('"'),       // \" = quote
                    Some('\'') => result.push('\''),     // \' = apostrophe
                    Some(c) => result.push(c),           // Unknown escape: just include the char
                    None => return Err("Unterminated string".to_string()),
                }
                self.advance();
            } else {
                result.push(ch);
                self.advance();
            }
        }

        // If we get here, we hit EOF without finding closing quote
        Err("Unterminated string".to_string())
    }

    // The main tokenization method - returns the next token
    // This uses exhaustive pattern matching to handle every possible input character
    fn next_token(&mut self) -> Result<Token, String> {
        self.skip_whitespace();  // Skip leading whitespace

        // match on Option<char> - pattern matching is fundamental to Rust
        // This is much more powerful than switch in C++
        match self.current() {
            // None = reached EOF
            None => Ok(Token::Eof),
            
            // Single character tokens
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
            
            // Minus: could be -, ->, or negative number
            Some('-') => {
                if self.peek(1) == Some('>') {
                    // It's an arrow ->
                    self.advance();
                    self.advance();
                    Ok(Token::Arrow)
                } else if self.peek(1).map_or(false, |c| c.is_numeric()) {
                    // It's a negative number
                    self.advance();
                    let num = self.read_number()?;
                    Ok(Token::Number(-num))
                } else {
                    // It's just minus operator
                    self.advance();
                    Ok(Token::Minus)
                }
            }
            
            // Arithmetic operators
            Some('+') => {
                self.advance();
                Ok(Token::Plus)
            }
            Some('*') => {
                self.advance();
                Ok(Token::Star)
            }
            
            // Slash: could be /, //, or /*
            Some('/') => {
                if self.peek(1) == Some('/') {
                    // Line comment
                    self.skip_line_comment();
                    self.next_token()  // Recursively get next real token
                } else if self.peek(1) == Some('*') {
                    // Block comment
                    self.skip_block_comment()?;  // ? operator propagates errors
                    self.next_token()
                } else {
                    // Just division operator
                    self.advance();
                    Ok(Token::Slash)
                }
            }
            
            Some('=') => {
                self.advance();
                Ok(Token::Eq)
            }
            
            // Exclamation: must be !=
            Some('!') => {
                if self.peek(1) == Some('=') {
                    self.advance();
                    self.advance();
                    Ok(Token::NotEq)
                } else {
                    Err("Unexpected '!'".to_string())
                }
            }
            
            // Greater than: could be > or >=
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
            
            // Less than: could be < or <=
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
            
            // String literals: " or '
            Some('"') | Some('\'') => {
                let s = self.read_string()?;
                Ok(Token::String(s))
            }
            
            // Numbers
            Some(ch) if ch.is_numeric() => {
                let num = self.read_number()?;
                Ok(Token::Number(num))
            }
            
            // Identifiers and keywords
            // The "if guard" (if ch.is_alphabetic() || ...) restricts this pattern
            Some(ch) if ch.is_alphabetic() || ch == '_' => {
                let ident = self.read_identifier();
                let token = match ident.as_str() {
                    // These special identifiers are keywords, not generic identifiers
                    "on" | "off" | "and" | "or" | "not" => Token::Keyword(ident),
                    _ => Token::Identifier(ident),
                };
                Ok(token)
            }
            
            // Unexpected character
            Some(ch) => {
                Err(format!("Unexpected character: '{}'", ch))
            }
        }
    }
}

// Public API for the lexer
// This is the function called from main.rs
// It takes a string slice and returns either a Vec<Token> or an error
pub fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut lexer = Lexer::new(input);
    let mut tokens = Vec::new();  // Like C++ vector<Token>

    // Keep tokenizing until we hit EOF
    loop {
        let token = lexer.next_token()?;  // The ? operator: if error, return it immediately
        let is_eof = token == Token::Eof;  // Check if we're done
        tokens.push(token);                 // Add to vector
        if is_eof {
            break;  // Stop after EOF token
        }
    }

    Ok(tokens)  // Return the complete token list
}
