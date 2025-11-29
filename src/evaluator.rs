use crate::ast::{Tag, PrimitiveValue};
use crate::trace;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    Text(String),
    Flag(bool),
    Item,
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
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
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Context {
    Global,
    Define,
    Set,
    List,
}

type TagHandler = fn(&mut Evaluator, &Tag) -> Result<Value, String>;

pub struct Evaluator {
    pub store: HashMap<String, Value>,
    scope_stack: Vec<String>,
    current_context: Context,
    handlers: HashMap<(Context, String), TagHandler>,
}

impl Evaluator {
    fn new() -> Self {
        let mut evaluator = Evaluator {
            store: HashMap::new(),
            scope_stack: Vec::new(),
            current_context: Context::Global,
            handlers: HashMap::new(),
        };
        evaluator.register_handlers();
        evaluator
    }

    fn register_handlers(&mut self) {
        // Global context handlers
        self.handlers
            .insert((Context::Global, "character".to_string()), handle_character);
        self.handlers
            .insert((Context::Global, "define".to_string()), handle_define);
        self.handlers
            .insert((Context::Global, "text".to_string()), handle_text);
        self.handlers
            .insert((Context::Global, "number".to_string()), handle_number);
        self.handlers
            .insert((Context::Global, "flag".to_string()), handle_flag);
        self.handlers
            .insert((Context::Global, "item".to_string()), handle_item);
        self.handlers
            .insert((Context::Global, "list".to_string()), handle_list);

        // Define context handlers
        self.handlers
            .insert((Context::Define, "set".to_string()), handle_define_set);
        self.handlers
            .insert((Context::Define, "attribute".to_string()), handle_define_attribute);

        // List context handlers
        self.handlers
            .insert((Context::List, "set".to_string()), handle_list_set);
        self.handlers
            .insert((Context::List, "text".to_string()), handle_text);
        self.handlers
            .insert((Context::List, "number".to_string()), handle_number);
        self.handlers
            .insert((Context::List, "attribute".to_string()), handle_list_attribute);
    }

    fn eval_tag(&mut self, tag: &Tag) -> Result<Value, String> {
        trace::trace_eval_tag(tag);
        match tag {
            Tag::Primitive(prim) => self.eval_primitive(prim),
            Tag::Composite { ltag, rtag } => self.eval_composite(ltag, rtag),
        }
    }

    fn eval_primitive(&mut self, prim: &PrimitiveValue) -> Result<Value, String> {
        match prim {
            PrimitiveValue::Identifier(name) => Ok(Value::Text(name.clone())),
            PrimitiveValue::Number(n) => Ok(Value::Number(*n)),
            PrimitiveValue::String(s) => Ok(Value::Text(s.clone())),
            PrimitiveValue::Keyword(kw) => match kw.as_str() {
                "on" => Ok(Value::Flag(true)),
                "off" => Ok(Value::Flag(false)),
                _ => Err(format!("Unknown keyword: {}", kw)),
            },
        }
    }

    fn eval_composite(&mut self, ltag: &Tag, rtag: &Tag) -> Result<Value, String> {
        // Extract the operation name from the LTag
        let op_name = self.extract_operation_name(ltag)?;

        // Look up handler
        let handler = self
            .handlers
            .get(&(self.current_context, op_name.clone()))
            .copied()
            .ok_or_else(|| {
                format!(
                    "Unknown tag '{}' in context {:?}",
                    op_name, self.current_context
                )
            })?;

        // Create a synthetic tag for the handler to work with
        let composite_tag = Tag::Composite {
            ltag: Box::new(Tag::Primitive(PrimitiveValue::Identifier(op_name.clone()))),
            rtag: Box::new(rtag.clone()),
        };

        trace::trace_enter(&format!("{:?}", self.current_context), &op_name, &composite_tag);
        let result = handler(self, &composite_tag);
        match &result {
            Ok(val) => trace::trace_exit(&format!("{:?}", val)),
            Err(e) => trace::trace_exit(&format!("ERROR: {}", e)),
        }
        result
    }

    fn extract_operation_name(&mut self, tag: &Tag) -> Result<String, String> {
        match tag {
            Tag::Primitive(PrimitiveValue::Identifier(name)) => Ok(name.clone()),
            Tag::Composite { ltag, .. } => {
                // Nested LTag - extract recursively
                self.extract_operation_name(ltag)
            }
            _ => Err("Invalid operation name in LTag".to_string()),
        }
    }
}

// Global context handlers
fn handle_character(eval: &mut Evaluator, tag: &Tag) -> Result<Value, String> {
    if let Tag::Composite { rtag, .. } = tag {
        if let Value::Text(name) = eval.eval_tag(rtag)? {
            eval.store.insert(name.clone(), Value::Item);
            Ok(Value::Text(format!("character:{}", name)))
        } else {
            Err("Character name must be text".to_string())
        }
    } else {
        Err("Invalid character tag".to_string())
    }
}

fn handle_define(eval: &mut Evaluator, tag: &Tag) -> Result<Value, String> {
    // In the new architecture, the actual tag structure passed here is:
    // Tag::Composite { ltag: Identifier("define"), rtag: [the body] }
    // But the real nested LTag [define: [character: alice]] was already evaluated
    // to get the operation name.
    
    // We need access to the original nested structure. For now, this is a limitation.
    // We should redesign so handlers get access to the original tag structure.
    
    Err("define not yet implemented in new architecture".to_string())
}

fn handle_text(_eval: &mut Evaluator, tag: &Tag) -> Result<Value, String> {
    if let Tag::Composite { rtag, .. } = tag {
        match &**rtag {
            Tag::Primitive(PrimitiveValue::String(s)) => Ok(Value::Text(s.clone())),
            Tag::Primitive(PrimitiveValue::Identifier(s)) => Ok(Value::Text(s.clone())),
            _ => Err("text expects a string or identifier".to_string()),
        }
    } else {
        Err("Invalid text tag".to_string())
    }
}

fn handle_number(_eval: &mut Evaluator, tag: &Tag) -> Result<Value, String> {
    if let Tag::Composite { rtag, .. } = tag {
        if let Tag::Primitive(PrimitiveValue::Number(n)) = &**rtag {
            Ok(Value::Number(*n))
        } else {
            Err("number expects a numeric value".to_string())
        }
    } else {
        Err("Invalid number tag".to_string())
    }
}

fn handle_flag(_eval: &mut Evaluator, tag: &Tag) -> Result<Value, String> {
    if let Tag::Composite { rtag, .. } = tag {
        if let Tag::Primitive(PrimitiveValue::Keyword(kw)) = &**rtag {
            match kw.as_str() {
                "on" => Ok(Value::Flag(true)),
                "off" => Ok(Value::Flag(false)),
                _ => Err("flag expects 'on' or 'off'".to_string()),
            }
        } else {
            Err("flag expects 'on' or 'off'".to_string())
        }
    } else {
        Err("Invalid flag tag".to_string())
    }
}

fn handle_item(_eval: &mut Evaluator, _tag: &Tag) -> Result<Value, String> {
    Ok(Value::Item)
}

fn handle_list(eval: &mut Evaluator, tag: &Tag) -> Result<Value, String> {
    if let Tag::Composite { rtag, .. } = tag {
        let prev_context = eval.current_context;
        eval.current_context = Context::List;
        let result = eval.eval_tag(rtag)?;
        eval.current_context = prev_context;
        Ok(result)
    } else {
        Err("Invalid list tag".to_string())
    }
}

// Define context handlers
fn handle_define_set(eval: &mut Evaluator, tag: &Tag) -> Result<Value, String> {
    if let Tag::Composite { ltag: _, rtag } = tag {
        eval.eval_tag(rtag)
    } else {
        Err("Invalid set tag in define context".to_string())
    }
}

fn handle_define_attribute(eval: &mut Evaluator, tag: &Tag) -> Result<Value, String> {
    if let Tag::Composite { rtag, .. } = tag {
        eval.eval_tag(rtag)
    } else {
        Err("Invalid attribute tag".to_string())
    }
}

// List context handlers
fn handle_list_set(eval: &mut Evaluator, tag: &Tag) -> Result<Value, String> {
    if let Tag::Composite { ltag: _, rtag } = tag {
        eval.eval_tag(rtag)
    } else {
        Err("Invalid set tag in list context".to_string())
    }
}

fn handle_list_attribute(eval: &mut Evaluator, tag: &Tag) -> Result<Value, String> {
    if let Tag::Composite { rtag, .. } = tag {
        eval.eval_tag(rtag)
    } else {
        Err("Invalid attribute tag in list context".to_string())
    }
}

pub fn evaluate(ast: Vec<Tag>) -> Result<HashMap<String, Value>, String> {
    let mut evaluator = Evaluator::new();
    for tag in &ast {
        evaluator.eval_tag(tag)?;
    }
    Ok(evaluator.store)
}
