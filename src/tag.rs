use std::fmt;

// Runtime values during execution
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    Text(String),
    Flag(bool),
    Item,
    Reference(String), // Points to a variable/attribute name
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Number(n) => {
                if n.fract() == 0.0 {
                    write!(f, "{}", *n as i64)
                } else {
                    write!(f, "{}", n)
                }
            }
            Value::Text(s) => write!(f, "\"{}\"", s),
            Value::Flag(b) => write!(f, "{}", if *b { "on" } else { "off" }),
            Value::Item => write!(f, "item"),
            Value::Reference(name) => write!(f, "&{}", name),
        }
    }
}

// Parse tree: [ltag: rtag] structure (recursive)
#[derive(Debug, Clone)]
pub enum TagNode {
    Composite {
        ltag: Box<TagNode>,  // Box allows recursive self-reference
        rtag: Box<TagNode>,
    },
    Primitive(Primitive),
}

// Static tokens from parser - converted to Values during evaluation
#[derive(Debug, Clone)]
pub enum Primitive {
    Identifier(String),
    Number(f64),
    String(String),
    Keyword(String),
}

impl Primitive {
    pub fn to_value(&self) -> Value {
        match self {
            Primitive::Identifier(s) => Value::Text(s.clone()),
            Primitive::Number(n) => Value::Number(*n),
            Primitive::String(s) => Value::Text(s.clone()),
            Primitive::Keyword(kw) => match kw.as_str() {
                "on" => Value::Flag(true),
                "off" => Value::Flag(false),
                _ => Value::Text(kw.clone()),
            },
        }
    }
}

impl TagNode {
    // Extract ltag from a composite tag
    pub fn ltag(&self) -> Option<&TagNode> {
        match self {
            TagNode::Composite { ltag, .. } => Some(ltag),
            TagNode::Primitive(_) => None,
        }
    }

    // Extract rtag from a composite tag
    pub fn rtag(&self) -> Option<&TagNode> {
        match self {
            TagNode::Composite { rtag, .. } => Some(rtag),
            TagNode::Primitive(_) => None,
        }
    }
}
