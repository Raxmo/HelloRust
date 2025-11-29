use crate::lexer::Token;
use std::collections::HashMap;

/// Represents a parsed tag that can be evaluated
#[derive(Debug, Clone)]
pub enum Tag {
    /// A primitive value: identifier, number, string, keyword
    Primitive(PrimitiveValue),
    /// A composite tag: [ltag: rtag]
    Composite {
        ltag: Box<Tag>,
        rtag: Box<Tag>,
    },
}

#[derive(Debug, Clone)]
pub enum PrimitiveValue {
    Identifier(String),
    Number(f64),
    String(String),
    Keyword(String),
}

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, position: 0 }
    }

    fn current(&self) -> &Token {
        self.tokens.get(self.position).unwrap_or(&Token::Eof)
    }

    fn advance(&mut self) {
        if self.position < self.tokens.len() {
            self.position += 1;
        }
    }

    fn expect(&mut self, expected: Token) -> Result<(), String> {
        if std::mem::discriminant(self.current()) == std::mem::discriminant(&expected) {
            self.advance();
            Ok(())
        } else {
            Err(format!("Expected {:?}, got {:?}", expected, self.current()))
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Tag>, String> {
        let mut tags = Vec::new();
        while self.current() != &Token::Eof {
            tags.push(self.parse_tag()?);
        }
        Ok(tags)
    }

    fn parse_tag(&mut self) -> Result<Tag, String> {
        self.expect(Token::OpenBracket)?;

        // Parse LTag
        let ltag = self.parse_ltag()?;
        self.expect(Token::Colon)?;

        // Parse RTag
        let rtag = self.parse_rtag()?;
        self.expect(Token::CloseBracket)?;

        Ok(Tag::Composite {
            ltag: Box::new(ltag),
            rtag: Box::new(rtag),
        })
    }

    fn parse_ltag(&mut self) -> Result<Tag, String> {
        // LTag can be an identifier or a nested tag
        if self.current() == &Token::OpenBracket {
            self.parse_tag()
        } else {
            self.parse_primitive()
        }
    }

    fn parse_rtag(&mut self) -> Result<Tag, String> {
        // RTag can be a primitive, a nested tag, or a list
        match self.current() {
            Token::OpenBracket => self.parse_tag(),
            Token::Identifier(name) => {
                let n = name.clone();
                self.advance();
                Ok(Tag::Primitive(PrimitiveValue::Identifier(n)))
            }
            Token::Number(n) => {
                let num = *n;
                self.advance();
                Ok(Tag::Primitive(PrimitiveValue::Number(num)))
            }
            Token::String(s) => {
                let string = s.clone();
                self.advance();
                Ok(Tag::Primitive(PrimitiveValue::String(string)))
            }
            Token::Keyword(kw) => {
                let keyword = kw.clone();
                self.advance();
                Ok(Tag::Primitive(PrimitiveValue::Keyword(keyword)))
            }
            _ => Err(format!("Expected RTag, got {:?}", self.current())),
        }
    }

    fn parse_primitive(&mut self) -> Result<Tag, String> {
        match self.current() {
            Token::Identifier(name) => {
                let n = name.clone();
                self.advance();
                Ok(Tag::Primitive(PrimitiveValue::Identifier(n)))
            }
            Token::Number(n) => {
                let num = *n;
                self.advance();
                Ok(Tag::Primitive(PrimitiveValue::Number(num)))
            }
            Token::String(s) => {
                let string = s.clone();
                self.advance();
                Ok(Tag::Primitive(PrimitiveValue::String(string)))
            }
            Token::Keyword(kw) => {
                let keyword = kw.clone();
                self.advance();
                Ok(Tag::Primitive(PrimitiveValue::Keyword(keyword)))
            }
            _ => Err(format!("Expected primitive value, got {:?}", self.current())),
        }
    }
}
