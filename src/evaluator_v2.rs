use crate::tag::{TagNode, Value};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

/// Execution frame tracks scope and defined variables/attributes
#[derive(Debug, Clone)]
struct Frame {
    variables: HashMap<String, Value>,
    attributes: HashMap<String, Value>,
}

impl Frame {
    fn new() -> Self {
        Frame {
            variables: HashMap::new(),
            attributes: HashMap::new(),
        }
    }
}

pub struct Evaluator {
    pub store: HashMap<String, Value>,
    frames: Vec<Frame>, // Call stack for scoping
    log_file: Option<File>,
    eval_counter: usize,
}

impl Evaluator {
    pub fn new(log_path: &str) -> std::io::Result<Self> {
        let log_file = File::create(log_path)?;
        Ok(Evaluator {
            store: HashMap::new(),
            frames: vec![Frame::new()], // Start with global frame
            log_file: Some(log_file),
            eval_counter: 0,
        })
    }

    fn current_frame(&mut self) -> &mut Frame {
        self.frames.last_mut().expect("frames never empty")
    }

    fn writeln_log(&mut self, msg: &str) -> std::io::Result<()> {
        if let Some(ref mut file) = self.log_file {
            writeln!(file, "{}", msg)?;
            file.flush()?;
        }
        Ok(())
    }

    pub fn evaluate_tags(&mut self, tags: &[TagNode]) -> Result<(), String> {
        self.writeln_log("=== Evaluation Trace ===\n")
            .map_err(|e| format!("Log error: {}", e))?;

        for tag in tags {
            self.evaluate_tag(tag)?;
        }

        self.writeln_log("\n=== Evaluation Complete ===")
            .map_err(|e| format!("Log error: {}", e))?;

        Ok(())
    }

    pub fn evaluate_tag(&mut self, tag: &TagNode) -> Result<Value, String> {
        self.eval_counter += 1;
        let eval_id = self.eval_counter;

        match tag {
            TagNode::Primitive(prim) => {
                let value = prim.to_value();
                self.writeln_log(&format!(
                    "[Eval {}] Primitive: {} => {}",
                    eval_id,
                    prim.as_display_string(),
                    value
                ))
                .map_err(|e| format!("Log error: {}", e))?;
                Ok(value)
            }
            TagNode::Composite { ltag, rtag } => {
                self.writeln_log(&format!("[Eval {}] Composite tag: [ltag: rtag]", eval_id))
                    .map_err(|e| format!("Log error: {}", e))?;

                // Evaluate ltag
                self.writeln_log(&format!("[Eval {}] Evaluating ltag...", eval_id))
                    .map_err(|e| format!("Log error: {}", e))?;
                let ltag_value = self.evaluate_tag(ltag)?;

                self.writeln_log(&format!("[Eval {}]   ltag evaluated to: {}", eval_id, ltag_value))
                    .map_err(|e| format!("Log error: {}", e))?;

                // Evaluate rtag
                self.writeln_log(&format!("[Eval {}] Evaluating rtag...", eval_id))
                    .map_err(|e| format!("Log error: {}", e))?;
                let rtag_value = self.evaluate_tag(rtag)?;

                self.writeln_log(&format!("[Eval {}]   rtag evaluated to: {}", eval_id, rtag_value))
                    .map_err(|e| format!("Log error: {}", e))?;

                // Dispatch handler based solely on operation name
                self.writeln_log(&format!(
                    "[Eval {}] Dispatching handler for operation: {}",
                    eval_id, ltag_value
                ))
                .map_err(|e| format!("Log error: {}", e))?;

                let result = self.execute(&ltag_value, &rtag_value)?;

                self.writeln_log(&format!("[Eval {}] Handler result: {}", eval_id, result))
                    .map_err(|e| format!("Log error: {}", e))?;

                Ok(result)
            }
        }
    }

    fn execute(&mut self, ltag: &Value, rtag: &Value) -> Result<Value, String> {
        // Handle assignment: if ltag is a reference, assign rtag to it
        if let Value::Reference(name) = ltag {
            let frame = self.current_frame();
            // Check if attribute exists
            if !frame.attributes.contains_key(name) && !frame.variables.contains_key(name) {
                return Err(format!("Cannot assign to undefined attribute/variable '{}'", name));
            }
            // Store the value
            if frame.attributes.contains_key(name) {
                frame.attributes.insert(name.clone(), rtag.clone());
            } else {
                frame.variables.insert(name.clone(), rtag.clone());
            }
            return Ok(rtag.clone());
        }

        // Extract operation name from ltag if it's text
        match ltag {
            Value::Text(ref op_name) => {
                // Dispatch based solely on operation name
                match op_name.as_str() {
                    "character" => self.handle_character(rtag),
                    "define" => self.handle_define(rtag),
                    "list" => self.handle_list(rtag),
                    "text" => self.handle_text(rtag),
                    "number" => self.handle_number(rtag),
                    "flag" => self.handle_flag(rtag),
                    "item" => self.handle_item(rtag),
                    "set" => self.handle_set(rtag),
                    "attribute" => self.handle_attribute(rtag),
                    _ => Err(format!("Unknown operation '{}'", op_name)),
                }
            }
            _ => {
                // If ltag isn't an operation name, just return rtag
                // This allows structure like [[def: ...]: [list: ...]] to work
                Ok(rtag.clone())
            }
        }
    }

    fn handle_character(&mut self, rtag: &Value) -> Result<Value, String> {
        if let Value::Text(name) = rtag {
            self.store.insert(name.clone(), Value::Item);
            Ok(Value::Text(format!("character:{}", name)))
        } else {
            Err("Character name must be text".to_string())
        }
    }

    fn handle_define(&mut self, _rtag: &Value) -> Result<Value, String> {
        // Push a new frame for the define block
        self.frames.push(Frame::new());
        Ok(Value::Item)
    }

    fn handle_list(&mut self, _rtag: &Value) -> Result<Value, String> {
        // List processing happens at the streaming parser level
        // Handlers just acknowledge it
        Ok(Value::Item)
    }

    fn handle_text(&mut self, rtag: &Value) -> Result<Value, String> {
        Ok(rtag.clone())
    }

    fn handle_number(&mut self, rtag: &Value) -> Result<Value, String> {
        Ok(rtag.clone())
    }

    fn handle_flag(&mut self, rtag: &Value) -> Result<Value, String> {
        Ok(rtag.clone())
    }

    fn handle_item(&mut self, _rtag: &Value) -> Result<Value, String> {
        Ok(Value::Item)
    }

    fn handle_set(&mut self, rtag: &Value) -> Result<Value, String> {
        // set returns a reference to indicate "this is a settable location"
        // The actual assignment happens when this reference is used as ltag in outer context
        match rtag {
            Value::Reference(name) => {
                // Just pass through the reference
                Ok(Value::Reference(name.clone()))
            }
            _ => Err(format!("set expects a reference, got {:?}", rtag)),
        }
    }

    fn handle_attribute(&mut self, rtag: &Value) -> Result<Value, String> {
        // attribute both declares (if needed) and returns a reference
        if let Value::Text(name) = rtag {
            let frame = self.current_frame();
            // Declare the attribute if it doesn't exist yet
            if !frame.attributes.contains_key(name) {
                frame.attributes.insert(name.clone(), Value::Item);
            }
            Ok(Value::Reference(name.clone()))
        } else {
            Err("Attribute name must be text".to_string())
        }
    }
}
