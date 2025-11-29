use crate::lexer::{Token, TokenWithPos, Position};
use crate::tag::{TagNode, Primitive};

#[derive(Debug, Clone)]
struct TagFrame {
    ltag: Option<TagNode>,
    rtag: Option<TagNode>,
    ltag_token_pos: usize,
}

impl TagFrame {
    fn new(token_pos: usize) -> Self {
        TagFrame {
            ltag: None,
            rtag: None,
            ltag_token_pos: token_pos,
        }
    }

    fn is_complete(&self) -> bool {
        self.ltag.is_some() && self.rtag.is_some()
    }

    fn to_composite(self) -> Result<TagNode, String> {
        let ltag = self.ltag.ok_or("Missing ltag")?;
        let rtag = self.rtag.ok_or("Missing rtag")?;
        Ok(TagNode::Composite {
            ltag: Box::new(ltag),
            rtag: Box::new(rtag),
        })
    }
}

pub struct StreamingParser {
    tokens: Vec<TokenWithPos>,
    position: usize,
    source: String,
    tag_stack: Vec<TagFrame>,
    waiting_for_rtag: Vec<bool>,
}

impl StreamingParser {
    pub fn new(tokens: Vec<TokenWithPos>, source: String) -> Self {
        StreamingParser {
            tokens,
            position: 0,
            source,
            tag_stack: Vec::new(),
            waiting_for_rtag: Vec::new(),
        }
    }

    fn current(&self) -> &TokenWithPos {
        static EOF_WITH_POS: TokenWithPos = TokenWithPos {
            token: Token::Eof,
            pos: Position {
                byte_offset: 0,
                line: 0,
                column: 0,
            },
        };
        self.tokens.get(self.position).unwrap_or(&EOF_WITH_POS)
    }

    fn current_pos(&self) -> Position {
        self.current().pos
    }

    fn advance(&mut self) {
        if self.position < self.tokens.len() {
            self.position += 1;
        }
    }

    pub fn parse(&mut self) -> Result<TagNode, String> {
        let mut tags = Vec::new();

        while self.current().token != Token::Eof {
            let tag = self.parse_one_tag()?;
            tags.push(tag);
        }

        Self::create_root(tags)
    }

    fn create_root(tags: Vec<TagNode>) -> Result<TagNode, String> {
        Ok(TagNode::Composite {
            ltag: Box::new(TagNode::Primitive(Primitive::Keyword("root".to_string()))),
            rtag: Box::new(Self::create_list_node(tags)),
        })
    }

    fn create_list_node(mut tags: Vec<TagNode>) -> TagNode {
        if tags.is_empty() {
            // Empty program: return list with item keyword as placeholder
            TagNode::Composite {
                ltag: Box::new(TagNode::Primitive(Primitive::Keyword("list".to_string()))),
                rtag: Box::new(TagNode::Primitive(Primitive::Keyword("item".to_string()))),
            }
        } else if tags.len() == 1 {
            TagNode::Composite {
                ltag: Box::new(TagNode::Primitive(Primitive::Keyword("list".to_string()))),
                rtag: Box::new(tags.pop().unwrap()),
            }
        } else {
            tags.reverse();
            let mut list_node = tags.pop().unwrap();

            while let Some(tag) = tags.pop() {
                list_node = TagNode::Composite {
                    ltag: Box::new(tag),
                    rtag: Box::new(list_node),
                };
            }

            TagNode::Composite {
                ltag: Box::new(TagNode::Primitive(Primitive::Keyword("list".to_string()))),
                rtag: Box::new(list_node),
            }
        }
    }

    fn parse_one_tag(&mut self) -> Result<TagNode, String> {
        self.expect_open_bracket()?;
        let tag_pos = self.position;
        self.tag_stack.push(TagFrame::new(tag_pos));
        self.waiting_for_rtag.push(false);

        loop {
            match &self.current().token {
                Token::OpenBracket => {
                    let nested = self.parse_one_tag()?;
                    self.push_to_current(nested)?;
                }
                Token::CloseBracket => {
                    self.advance();
                    return self.close_current_tag();
                }
                Token::Colon => {
                    if self.tag_stack.last().ok_or("Stack empty")?.ltag.is_none() {
                        return Err(self.error_with_context(
                            "Colon with no ltag",
                            self.current_pos(),
                        ));
                    }
                    let idx = self.waiting_for_rtag.len() - 1;
                    self.waiting_for_rtag[idx] = true;
                    self.advance();
                }
                Token::Comma => {
                    self.advance();
                }
                Token::Identifier(name) => {
                    let name = name.clone();
                    self.advance();
                    let primitive = TagNode::Primitive(Primitive::Identifier(name));
                    self.push_to_current(primitive)?;
                }
                Token::Number(n) => {
                    let num = *n;
                    self.advance();
                    let primitive = TagNode::Primitive(Primitive::Number(num));
                    self.push_to_current(primitive)?;
                }
                Token::String(s) => {
                    let string = s.clone();
                    self.advance();
                    let primitive = TagNode::Primitive(Primitive::String(string));
                    self.push_to_current(primitive)?;
                }
                Token::Keyword(kw) => {
                    let keyword = kw.clone();
                    self.advance();
                    let primitive = TagNode::Primitive(Primitive::Keyword(keyword));
                    self.push_to_current(primitive)?;
                }
                Token::Eof => {
                    return Err(self.error_with_context(
                        "Unexpected EOF while parsing tag (expected ']')",
                        self.current_pos(),
                    ));
                }
                _ => {
                    return Err(self.error_with_context(
                        &format!("Unexpected token: {:?}", self.current().token),
                        self.current_pos(),
                    ));
                }
            }
        }
    }

    fn push_to_current(&mut self, node: TagNode) -> Result<(), String> {
        let pos = self.current_pos();
        let waiting_rtag = *self.waiting_for_rtag.last().ok_or("Waiting stack empty")?;
        let current = self.tag_stack.last_mut().ok_or("Stack empty")?;

        if !waiting_rtag {
            if current.ltag.is_some() {
                return Err(self.error_with_context(
                    "ltag already set, expected colon",
                    pos,
                ));
            }
            current.ltag = Some(node);
        } else {
            if current.rtag.is_some() {
                return Err(self.error_with_context("rtag already set", pos));
            }
            current.rtag = Some(node);
        }
        Ok(())
    }

    fn close_current_tag(&mut self) -> Result<TagNode, String> {
        let close_pos = if self.position > 0 {
            self.tokens[self.position - 1].pos
        } else {
            Position {
                byte_offset: 0,
                line: 1,
                column: 1,
            }
        };

        let frame = self.tag_stack.pop().ok_or_else(|| {
            self.error_with_context("Unexpected ']': no open tag", close_pos)
        })?;
        self.waiting_for_rtag.pop();

        if !frame.is_complete() {
            return Err(self.error_with_context(
                "Incomplete tag: missing ltag or rtag",
                self.tokens[frame.ltag_token_pos].pos,
            ));
        }

        Ok(frame.to_composite()?)
    }

    fn expect_open_bracket(&mut self) -> Result<(), String> {
        if self.current().token == Token::OpenBracket {
            self.advance();
            Ok(())
        } else {
            Err(self.error_with_context(
                &format!("Expected '[', got {:?}", self.current().token),
                self.current_pos(),
            ))
        }
    }

    fn error_with_context(&self, message: &str, pos: Position) -> String {
        let depth = self.tag_stack.len();
        let depth_marker = if depth > 0 {
            format!(" (inside {} nested tag{})", depth, if depth == 1 { "" } else { "s" })
        } else {
            String::new()
        };

        let context = if depth > 0 {
            let frame = &self.tag_stack[depth - 1];
            format!(
                "\nContext: ltag={} rtag={}",
                if frame.ltag.is_some() { "set" } else { "empty" },
                if frame.rtag.is_some() { "set" } else { "empty" }
            )
        } else {
            String::new()
        };

        format!(
            "Parse error at line {}, column {} (byte {}){}: {}{}",
            pos.line, pos.column, pos.byte_offset, depth_marker, message, context
        )
    }
}
