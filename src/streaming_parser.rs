use crate::lexer::Token;
use crate::tag::{TagNode, Primitive};

#[derive(Debug, Clone, Copy, PartialEq)]
enum TagParseState {
    ParsingLTag,
    ParsingRTag,
}

struct TagInProgress {
    state: TagParseState,
    ltag: Option<TagNode>,
    rtag: Option<TagNode>,
}

impl TagInProgress {
    fn new() -> Self {
        TagInProgress {
            state: TagParseState::ParsingLTag,
            ltag: None,
            rtag: None,
        }
    }

    fn is_complete(&self) -> bool {
        self.ltag.is_some() && self.rtag.is_some()
    }

    fn to_composite(&self) -> Result<TagNode, String> {
        let ltag = self.ltag.clone().ok_or("Missing ltag")?;
        let rtag = self.rtag.clone().ok_or("Missing rtag")?;
        Ok(TagNode::Composite {
            ltag: Box::new(ltag),
            rtag: Box::new(rtag),
        })
    }
}

pub struct StreamingParser {
    tokens: Vec<Token>,
    position: usize,
    tag_stack: Vec<TagInProgress>,
}

impl StreamingParser {
    pub fn new(tokens: Vec<Token>) -> Self {
        StreamingParser {
            tokens,
            position: 0,
            tag_stack: Vec::new(),
        }
    }

    fn current(&self) -> &Token {
        self.tokens.get(self.position).unwrap_or(&Token::Eof)
    }

    fn advance(&mut self) {
        if self.position < self.tokens.len() {
            self.position += 1;
        }
    }

    pub fn parse(&mut self) -> Result<TagNode, String> {
        let mut tags = Vec::new();

        while self.current() != &Token::Eof {
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
        self.tag_stack.push(TagInProgress::new());

        loop {
            match self.current() {
                Token::OpenBracket => {
                    let nested = self.parse_one_tag()?;
                    let current_tag = self.tag_stack.last_mut().ok_or("Tag stack empty")?;
                    match current_tag.state {
                        TagParseState::ParsingLTag => current_tag.ltag = Some(nested),
                        TagParseState::ParsingRTag => current_tag.rtag = Some(nested),
                    }
                }
                Token::CloseBracket => {
                    self.advance();
                    let tag = self.tag_stack.pop().ok_or("Tag stack empty")?;
                    if tag.is_complete() {
                        return tag.to_composite();
                    } else {
                        return Err("Incomplete tag".to_string());
                    }
                }
                Token::Colon => {
                    self.advance();
                    let current_tag = self.tag_stack.last_mut().ok_or("Tag stack empty")?;
                    current_tag.state = TagParseState::ParsingRTag;
                }
                Token::Comma => {
                    self.advance();
                }
                Token::Identifier(name) => {
                    let name = name.clone();
                    self.advance();
                    let primitive = TagNode::Primitive(Primitive::Identifier(name));
                    let current_tag = self.tag_stack.last_mut().ok_or("Tag stack empty")?;
                    match current_tag.state {
                        TagParseState::ParsingLTag => current_tag.ltag = Some(primitive),
                        TagParseState::ParsingRTag => current_tag.rtag = Some(primitive),
                    }
                }
                Token::Number(n) => {
                    let num = *n;
                    self.advance();
                    let primitive = TagNode::Primitive(Primitive::Number(num));
                    let current_tag = self.tag_stack.last_mut().ok_or("Tag stack empty")?;
                    match current_tag.state {
                        TagParseState::ParsingLTag => current_tag.ltag = Some(primitive),
                        TagParseState::ParsingRTag => current_tag.rtag = Some(primitive),
                    }
                }
                Token::String(s) => {
                    let string = s.clone();
                    self.advance();
                    let primitive = TagNode::Primitive(Primitive::String(string));
                    let current_tag = self.tag_stack.last_mut().ok_or("Tag stack empty")?;
                    match current_tag.state {
                        TagParseState::ParsingLTag => current_tag.ltag = Some(primitive),
                        TagParseState::ParsingRTag => current_tag.rtag = Some(primitive),
                    }
                }
                Token::Keyword(kw) => {
                    let keyword = kw.clone();
                    self.advance();
                    let primitive = TagNode::Primitive(Primitive::Keyword(keyword));
                    let current_tag = self.tag_stack.last_mut().ok_or("Tag stack empty")?;
                    match current_tag.state {
                        TagParseState::ParsingLTag => current_tag.ltag = Some(primitive),
                        TagParseState::ParsingRTag => current_tag.rtag = Some(primitive),
                    }
                }
                _ => {
                     return Err(format!("Unexpected token: {:?}", self.current()));
                 }
            }
        }
    }

    fn expect_open_bracket(&mut self) -> Result<(), String> {
        if self.current() == &Token::OpenBracket {
            self.advance();
            Ok(())
        } else {
            Err(format!("Expected [, got {:?}", self.current()))
        }
    }
}