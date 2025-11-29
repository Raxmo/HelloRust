use crate::lexer::Token;  // Token types from the lexer
use crate::tag::{TagNode, Primitive};  // Types we're building

// ============================================================================
// PARSER STATE MACHINE
// ============================================================================
// The parser uses a state machine to track parsing progress for a single tag
// A tag has two parts: [ltag: rtag]
// We need to know which part we're currently parsing

/// Parser state for building a single tag
/// #[derive] with Copy means this can be cheaply copied (like int in C++)
/// #[derive] Clone allows explicit cloning
/// PartialEq allows == comparisons
#[derive(Debug, Clone, Copy, PartialEq)]
enum TagParseState {
    /// Parsing the left tag (before colon) - the operation or function name
    ParsingLTag,
    /// Parsing the right tag (after colon) - the arguments/content
    ParsingRTag,
}

/// A tag being constructed - represents work in progress
/// This is stored on the tag_stack and represents one level of nesting
/// Example: while parsing [[set: x]: y], we have:
///   - Outer tag (in progress): ltag=None, rtag=None
///   - Inner tag (in progress): ltag=None, rtag=None
/// As we parse, these fill in one by one
struct TagInProgress {
    state: TagParseState,  // Are we parsing ltag or rtag?
    ltag: Option<TagNode>,  // Left tag (Some once we've parsed it)
    rtag: Option<TagNode>,  // Right tag (Some once we've parsed it)
}

impl TagInProgress {
    /// Create a new empty tag (ltag and rtag are None)
    /// Starts in ParsingLTag state - we expect ltag first
    fn new() -> Self {
        TagInProgress {
            state: TagParseState::ParsingLTag,
            ltag: None,
            rtag: None,
        }
    }

    /// Check if we have both ltag and rtag
    /// In Rust, .is_some() checks if Option<T> is Some(...)
    fn is_complete(&self) -> bool {
        self.ltag.is_some() && self.rtag.is_some()
    }

    /// Convert the completed parts into a composite TagNode
    /// Uses .clone() because we need to move the values out of Option
    /// .ok_or() converts None into an error: None.ok_or("msg") = Err("msg")
    /// The ? operator returns the error immediately if any step fails
    fn to_composite(&self) -> Result<TagNode, String> {
        // Extract values from Options, or return error if None
        let ltag = self.ltag.clone().ok_or("Missing ltag")?;
        let rtag = self.rtag.clone().ok_or("Missing rtag")?;
        // Wrap in Box (heap allocation) and return as composite
        Ok(TagNode::Composite {
            ltag: Box::new(ltag),
            rtag: Box::new(rtag),
        })
    }
}

// ============================================================================
// STREAMING PARSER
// ============================================================================
// Parses a token stream into a TagNode tree
// Uses a stack to handle nested tags
// Each item on the stack represents a tag being constructed at that nesting level

pub struct StreamingParser {
    tokens: Vec<Token>,              // Input token stream from lexer
    position: usize,                 // Current position in the token stream
    tag_stack: Vec<TagInProgress>,   // Stack of tags being constructed
                                     // When we see [, we push; when we see ], we pop
}

impl StreamingParser {
    /// Create a new parser with the given token stream
    /// Takes ownership of the tokens (no & means ownership transfer)
    pub fn new(tokens: Vec<Token>) -> Self {
        StreamingParser {
            tokens,
            position: 0,          // Start at beginning
            tag_stack: Vec::new(),  // Stack starts empty
        }
    }

    /// Get the current token without advancing
    /// .get() returns Option<&Token> (None if out of bounds)
    /// .unwrap_or() provides default value if None
    fn current(&self) -> &Token {
        self.tokens.get(self.position).unwrap_or(&Token::Eof)
    }

    /// Move to the next token
    /// Checks bounds to avoid panic
    fn advance(&mut self) {
        if self.position < self.tokens.len() {
            self.position += 1;
        }
    }

    /// Main entry point: parse all tokens into a TagNode tree
    /// Returns Result - either a single root TagNode or an error
    pub fn parse(&mut self) -> Result<TagNode, String> {
        let mut tags = Vec::new();

        // Parse tags until we hit EOF
        // Note: we don't use a for loop because parse_one_tag mutates self.position
        while self.current() != &Token::Eof {
            let tag = self.parse_one_tag()?;  // ? propagates any error
            tags.push(tag);
        }

        // Wrap all top-level tags in an implicit root list
        // This normalizes input: `[a] [b]` becomes `[root: [list: [a, [b, ...]]]]`
        Self::create_root(tags)
    }

    /// Create the implicit root wrapper around all top-level tags
    /// Every program becomes: [root: [list: ... tags ...]]
    /// This normalizes the structure so evaluator always sees a single root
    fn create_root(tags: Vec<TagNode>) -> Result<TagNode, String> {
        // Note: all three branches are identical (could be simplified)
        // They're kept separate for clarity about what's happening
        
        if tags.is_empty() {
            // Empty program: [root: [list: empty_list]]
            Ok(TagNode::Composite {
                ltag: Box::new(TagNode::Primitive(Primitive::Keyword("root".to_string()))),
                rtag: Box::new(Self::create_list_node(vec![])),
            })
        } else if tags.len() == 1 {
            // Single tag: [root: [list: tag]]
            Ok(TagNode::Composite {
                ltag: Box::new(TagNode::Primitive(Primitive::Keyword("root".to_string()))),
                rtag: Box::new(Self::create_list_node(tags)),
            })
        } else {
            // Multiple tags: [root: [list: [tag1, [tag2, ...]]]]
            Ok(TagNode::Composite {
                ltag: Box::new(TagNode::Primitive(Primitive::Keyword("root".to_string()))),
                rtag: Box::new(Self::create_list_node(tags)),
            })
        }
    }

    /// Create a list node structure from multiple tags
    /// Multiple tags get nested as: [tag1: [tag2: [tag3: item]]]
    /// This is how lists are represented in the tag language
    fn create_list_node(mut tags: Vec<TagNode>) -> TagNode {
        if tags.is_empty() {
            // Empty list: [list: item]
            TagNode::Composite {
                ltag: Box::new(TagNode::Primitive(Primitive::Keyword("list".to_string()))),
                rtag: Box::new(TagNode::Primitive(Primitive::Keyword("item".to_string()))),
            }
        } else if tags.len() == 1 {
            // Single item: [list: tag]
            TagNode::Composite {
                ltag: Box::new(TagNode::Primitive(Primitive::Keyword("list".to_string()))),
                rtag: Box::new(tags.pop().unwrap()),  // .pop() takes last element and returns Option
            }
        } else {
            // Multiple items: build nested structure
            // tags = [tag1, tag2, tag3]
            // reverse → [tag3, tag2, tag1]
            // pop until empty, building: tag1: [tag2: [tag3: item]]
            
            tags.reverse();  // Reverse so we can pop in correct order
            let mut list_node = tags.pop().unwrap();  // Start with last tag (becomes innermost)
            
            // Build the chain backwards
            // while let Some(tag) = ... is pattern matching in a loop
            // It pops, and if Some(tag), executes body; if None, exits loop
            while let Some(tag) = tags.pop() {
                list_node = TagNode::Composite {
                    ltag: Box::new(tag),          // Current tag becomes ltag
                    rtag: Box::new(list_node),    // Previous nesting becomes rtag
                };
            }
            
            // Wrap the whole thing in [list: ...]
            TagNode::Composite {
                ltag: Box::new(TagNode::Primitive(Primitive::Keyword("list".to_string()))),
                rtag: Box::new(list_node),
            }
        }
    }

    /// Parse a single tag: [ltag: rtag]
    /// This is the core recursive parsing logic
    /// Uses the tag_stack to handle nested tags
    fn parse_one_tag(&mut self) -> Result<TagNode, String> {
        // Expect an opening bracket and consume it
        self.expect_open_bracket()?;
        // Push a new tag on the stack to track what we're parsing
        self.tag_stack.push(TagInProgress::new());

        // Main parsing loop - continues until tag is complete (we see ])
        loop {
            match self.current() {
                // Nested tag: recursively parse it
                Token::OpenBracket => {
                    let nested = self.parse_one_tag()?;  // Recursive call
                    // Get the current (top) tag on the stack
                    let current_tag = self.tag_stack.last_mut().ok_or("Tag stack empty")?;

                    // Depending on which part we're parsing, store the nested tag
                    match current_tag.state {
                        TagParseState::ParsingLTag => {
                            current_tag.ltag = Some(nested);
                        }
                        TagParseState::ParsingRTag => {
                            current_tag.rtag = Some(nested);
                        }
                    }
                }
                
                // End of this tag
                Token::CloseBracket => {
                    self.advance();
                    let tag = self.tag_stack.pop().ok_or("Tag stack empty")?;
                    // Verify we have both parts before completing
                    if tag.is_complete() {
                        return tag.to_composite();
                    } else {
                        return Err("Incomplete tag".to_string());
                    }
                }
                
                // Switch from ltag parsing to rtag parsing
                Token::Colon => {
                    self.advance();
                    let current_tag = self.tag_stack.last_mut().ok_or("Tag stack empty")?;
                    current_tag.state = TagParseState::ParsingRTag;
                }
                
                // Commas are ignored (they're just separators in lists)
                Token::Comma => {
                    self.advance();
                }
                
                // Identifier (variable name, operation name)
                Token::Identifier(name) => {
                    let name = name.clone();
                    self.advance();
                    let primitive = TagNode::Primitive(Primitive::Identifier(name));
                    let current_tag = self.tag_stack.last_mut().ok_or("Tag stack empty")?;

                    // Store in appropriate side
                    match current_tag.state {
                        TagParseState::ParsingLTag => {
                            current_tag.ltag = Some(primitive);
                        }
                        TagParseState::ParsingRTag => {
                            current_tag.rtag = Some(primitive);
                        }
                    }
                }
                
                // Number literal
                Token::Number(n) => {
                    let num = *n;  // Dereference the reference to get the value
                    self.advance();
                    let primitive = TagNode::Primitive(Primitive::Number(num));
                    let current_tag = self.tag_stack.last_mut().ok_or("Tag stack empty")?;

                    match current_tag.state {
                        TagParseState::ParsingLTag => {
                            current_tag.ltag = Some(primitive);
                        }
                        TagParseState::ParsingRTag => {
                            current_tag.rtag = Some(primitive);
                        }
                    }
                }
                
                // String literal
                Token::String(s) => {
                    let string = s.clone();
                    self.advance();
                    let primitive = TagNode::Primitive(Primitive::String(string));
                    let current_tag = self.tag_stack.last_mut().ok_or("Tag stack empty")?;

                    match current_tag.state {
                        TagParseState::ParsingLTag => {
                            current_tag.ltag = Some(primitive);
                        }
                        TagParseState::ParsingRTag => {
                            current_tag.rtag = Some(primitive);
                        }
                    }
                }
                
                // Keyword (on, off, and, or, not, root, list, etc)
                Token::Keyword(kw) => {
                    let keyword = kw.clone();
                    self.advance();
                    let primitive = TagNode::Primitive(Primitive::Keyword(keyword));
                    let current_tag = self.tag_stack.last_mut().ok_or("Tag stack empty")?;

                    match current_tag.state {
                        TagParseState::ParsingLTag => {
                            current_tag.ltag = Some(primitive);
                        }
                        TagParseState::ParsingRTag => {
                            current_tag.rtag = Some(primitive);
                        }
                    }
                }
                
                // Any other token is unexpected here
                _ => {
                    return Err(format!("Unexpected token: {:?}", self.current()));
                }
            }
        }
    }

    /// Helper: verify we have an opening bracket at current position
    /// Consumes the bracket if found
    fn expect_open_bracket(&mut self) -> Result<(), String> {
        if self.current() == &Token::OpenBracket {
            self.advance();
            Ok(())
        } else {
            Err(format!("Expected [, got {:?}", self.current()))
        }
    }
}
