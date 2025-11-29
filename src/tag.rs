use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    Text(String),
    Flag(bool),
    Item,
    Reference(String), // Reference to a variable/attribute name
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

/// A Packard Script tag: [ltag: rtag]
/// Both ltag and rtag can be nested tag structures
#[derive(Debug, Clone)]
pub enum TagNode {
    /// A composite tag with ltag and rtag
    Composite {
        ltag: Box<TagNode>,
        rtag: Box<TagNode>,
    },
    /// A primitive value (terminal node)
    Primitive(Primitive),
}

/// Represents a primitive value (terminal node)
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

    pub fn as_display_string(&self) -> String {
        match self {
            Primitive::Identifier(s) => s.clone(),
            Primitive::Number(n) => {
                if n.fract() == 0.0 {
                    format!("{}", *n as i64)
                } else {
                    format!("{}", n)
                }
            }
            Primitive::String(s) => format!("\"{}\"", s),
            Primitive::Keyword(s) => s.clone(),
        }
    }

    pub fn as_text(&self) -> Option<String> {
        match self {
            Primitive::Identifier(s) => Some(s.clone()),
            Primitive::String(s) => Some(s.clone()),
            Primitive::Keyword(kw) => Some(kw.clone()),
            Primitive::Number(_) => None,
        }
    }
}

impl TagNode {
    pub fn evaluate_ltag(&self) -> Result<Value, String> {
        match self {
            TagNode::Primitive(prim) => Ok(prim.to_value()),
            TagNode::Composite { ltag, rtag } => {
                let _ltag_val = ltag.evaluate_ltag()?;
                let _rtag_val = rtag.evaluate_ltag()?;
                // For now, just return a placeholder
                Ok(Value::Item)
            }
        }
    }

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

    pub fn to_display_string(&self) -> String {
        match self {
            TagNode::Primitive(prim) => prim.as_display_string(),
            TagNode::Composite { ltag, rtag } => {
                format!("[{}: {}]", ltag.to_display_string(), rtag.to_display_string())
            }
        }
    }
}
