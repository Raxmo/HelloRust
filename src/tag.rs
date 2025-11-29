use std::fmt;

// ============================================================================
// VALUE ENUM - Runtime values during execution
// ============================================================================
// These are the actual values that exist during execution
// They differ from Primitive (which is compile-time) in that they've been evaluated
// In C++, this would be like a tagged union (std::variant<double, string, bool, ...>)

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),                    // Floating point number
    Text(String),                   // String value
    Flag(bool),                     // Boolean (on/off in Packard script)
    Item,                           // Placeholder/unit value (like "null" or "()" in other languages)
    Reference(String),              // Points to a variable/attribute name (used for assignment)
}

// Implementing fmt::Display allows Value to be printed with {} (like operator<< in C++)
// Without this, you'd need to use {:?} (debug format)
impl fmt::Display for Value {
    // f is a mutable reference to a formatter (like std::ostream in C++)
    // This is why we have &mut - the formatter needs to write to itself
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Number(n) => {
                // If number is a whole number (like 5.0), print as integer
                // Otherwise print with decimals
                if n.fract() == 0.0 {
                    // Cast to i64 for cleaner display
                    // The * dereferences n since it's a reference
                    write!(f, "{}", *n as i64)
                } else {
                    write!(f, "{}", n)
                }
            }
            // Text values: wrap in quotes to distinguish from identifiers
            Value::Text(s) => write!(f, "\"{}\"", s),
            // Flag: print as "on" or "off" (the Packard script keywords)
            Value::Flag(b) => write!(f, "{}", if *b { "on" } else { "off" }),
            // Item: just print "item"
            Value::Item => write!(f, "item"),
            // Reference: print with & prefix to show it's a reference
            Value::Reference(name) => write!(f, "&{}", name),
        }
    }
}

// ============================================================================
// TAGNODE ENUM - The parse tree structure
// ============================================================================
// A TagNode represents the abstract syntax tree (AST) produced by the parser
// The fundamental structure: [ltag: rtag]
// Both ltag and rtag can be nested (recursive tree structure)
// Example: [[set: [attribute: name]]: [text: Alice]]
//          becomes a tree of TagNodes

#[derive(Debug, Clone)]
pub enum TagNode {
    /// Composite tag: [ltag: rtag]
    /// Both sides can be primitives or further composites (recursive)
    /// Example: Composite { 
    ///     ltag: Primitive(Keyword("set")),
    ///     rtag: Composite { ... }
    /// }
    Composite {
        // Box<T> is like unique_ptr<T> in C++ - allows heap allocation
        // We need Box because Composite can contain itself recursively
        // Without Box, the size would be infinite (self-referential)
        // Box gives us a pointer of fixed size (8 bytes on 64-bit)
        ltag: Box<TagNode>,
        rtag: Box<TagNode>,
    },
    /// Primitive value: a leaf node (doesn't contain other tags)
    /// Like "name", 42, "hello", or "on"
    Primitive(Primitive),
}

// ============================================================================
// PRIMITIVE ENUM - Static tokens from the parser
// ============================================================================
// These come directly from the lexer (Token types)
// They haven't been evaluated yet (unlike Value which is the result of evaluation)
// Primitive -> to_value() -> Value (evaluated result)

#[derive(Debug, Clone)]
pub enum Primitive {
    Identifier(String),    // Variable/operation name: "set", "name", "myvar"
    Number(f64),          // Numeric literal: 42, 3.14
    String(String),       // String literal: "hello world"
    Keyword(String),      // Keywords: "on", "off", "and", "or", "not", "root", "list"
}

impl Primitive {
    /// Convert a static Primitive to a runtime Value
    /// This happens during evaluation - we transform the parsed structure into actual values
    /// Example: Primitive::Keyword("on") -> Value::Flag(true)
    pub fn to_value(&self) -> Value {
        match self {
            // Identifier: treat as text (could be a variable name or operation)
            Primitive::Identifier(s) => Value::Text(s.clone()),
            // Number: directly convert to Value::Number
            // The * dereferences the &f64 to get the actual value
            Primitive::Number(n) => Value::Number(*n),
            // String: treat as text (the quotes are semantic, not part of the value)
            Primitive::String(s) => Value::Text(s.clone()),
            // Keyword: special handling for "on" and "off"
            Primitive::Keyword(kw) => match kw.as_str() {
                "on" => Value::Flag(true),      // Packard script boolean true
                "off" => Value::Flag(false),    // Packard script boolean false
                _ => Value::Text(kw.clone()),   // Other keywords are just text
            },
        }
    }

    /// Convert Primitive to a display string
    /// Used for printing the parse tree (like when you see the output of format_tag)
    /// This shows what was in the source, with appropriate formatting
    pub fn as_display_string(&self) -> String {
        match self {
            Primitive::Identifier(s) => s.clone(),
            Primitive::Number(n) => {
                // Print whole numbers without decimals for readability
                if n.fract() == 0.0 {
                    format!("{}", *n as i64)
                } else {
                    format!("{}", n)
                }
            }
            // Strings: add quotes back for display
            Primitive::String(s) => format!("\"{}\"", s),
            // Keywords: just the keyword itself
            Primitive::Keyword(s) => s.clone(),
        }
    }

    /// Try to extract text from a Primitive
    /// Returns Option<String> - Some if it's text-like, None if it's a number
    /// This is used to get operation names (which must be textual)
    pub fn as_text(&self) -> Option<String> {
        match self {
            Primitive::Identifier(s) => Some(s.clone()),
            Primitive::String(s) => Some(s.clone()),
            Primitive::Keyword(kw) => Some(kw.clone()),
            // Numbers have no text representation - return None
            Primitive::Number(_) => None,
        }
    }
}

impl TagNode {
    /// Evaluate only the ltag side of this tag
    /// This method is mostly unused - kept for reference/future use
    /// The real evaluation happens in evaluator_v2.rs which has full context
    #[allow(dead_code)]
    pub fn evaluate_ltag(&self) -> Result<Value, String> {
        match self {
            TagNode::Primitive(prim) => Ok(prim.to_value()),
            TagNode::Composite { ltag, rtag } => {
                // Note: the underscore prefix (_) tells Rust we're intentionally not using these
                let _ltag_val = ltag.evaluate_ltag()?;
                let _rtag_val = rtag.evaluate_ltag()?;
                // For now, just return a placeholder
                Ok(Value::Item)
            }
        }
    }

    /// Evaluate only the rtag side of this tag
    /// Similar to evaluate_ltag - mostly unused, real evaluation is in evaluator_v2.rs
    #[allow(dead_code)]
    pub fn evaluate_rtag(&self) -> Result<Value, String> {
        match self {
            TagNode::Primitive(prim) => Ok(prim.to_value()),
            TagNode::Composite { ltag, rtag } => {
                let _ltag_val = ltag.evaluate_rtag()?;
                let _rtag_val = rtag.evaluate_rtag()?;
                // For now, just return a placeholder
                Ok(Value::Item)
            }
        }
    }

    /// Convert TagNode tree to a compact display string
    /// This is for printing the tree in inline format (not pretty-printed)
    /// Unused - format_tag in main.rs does the pretty printing
    #[allow(dead_code)]
    pub fn to_display_string(&self) -> String {
        match self {
            TagNode::Primitive(prim) => prim.as_display_string(),
            TagNode::Composite { ltag, rtag } => {
                // Recursively format both sides with [ltag: rtag] structure
                format!("[{}: {}]", ltag.to_display_string(), rtag.to_display_string())
            }
        }
    }
}
