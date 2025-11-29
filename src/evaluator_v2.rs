use crate::tag::{TagNode, Value};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use lazy_static::lazy_static;

type Handler = fn(&mut Evaluator, &Value) -> Result<Value, String>;

lazy_static! {
    static ref HANDLERS: HashMap<&'static str, Handler> = {
        let mut map = HashMap::new();
        map.insert("root", Evaluator::handle_root as Handler);
        map.insert("character", Evaluator::handle_character as Handler);
        // Note: "define" is handled specially in evaluate_tag, not through handlers
        map.insert("list", Evaluator::handle_list as Handler);
        map.insert("text", Evaluator::handle_text as Handler);
        map.insert("number", Evaluator::handle_number as Handler);
        map.insert("flag", Evaluator::handle_flag as Handler);
        map.insert("item", Evaluator::handle_item as Handler);
        // Note: "set" is handled specially in evaluate_tag, not through handlers
        map.insert("attribute", Evaluator::handle_attribute as Handler);
        map
    };
}

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
    pub defined_attributes: std::collections::HashSet<String>, // Track attributes defined during load
}

impl Evaluator {
    pub fn new(log_path: &str) -> std::io::Result<Self> {
        let log_file = File::create(log_path)?;
        Ok(Evaluator {
            store: HashMap::new(),
            frames: vec![Frame::new()], // Start with global frame
            log_file: Some(log_file),
            eval_counter: 0,
            defined_attributes: std::collections::HashSet::new(),
        })
    }

    fn current_frame(&mut self) -> &mut Frame {
        self.frames.last_mut().expect("frames never empty")
    }

    /// Extract operation name from an ltag (walk to innermost primitive if composite)
    fn extract_operation_name(&self, ltag: &TagNode) -> Result<String, String> {
        match ltag {
            TagNode::Primitive(prim) => {
                prim.as_text()
                    .ok_or_else(|| "Operation name must be text".to_string())
            }
            TagNode::Composite { ltag: inner_ltag, rtag: _inner_rtag } => {
                // Walk the ltag chain to find the innermost primitive
                self.extract_operation_name(inner_ltag)
            }
        }
    }

    /// Execute a define block: push scope, execute content, pop scope
    fn handle_define_block(&mut self, content: &TagNode) -> Result<Value, String> {
        // Push a new frame for this define block
        self.frames.push(Frame::new());
        
        // Execute the content within the new scope
        let result = self.evaluate_tag(content)?;
        
        // Pop the frame when done
        self.frames.pop();
        
        Ok(result)
    }

    /// Execute a set block: the ltag is [set: target], rtag is the value
    /// We need to extract the target from the set expression
    fn handle_set_block(&mut self, set_expr: &TagNode, value_tag: &TagNode) -> Result<Value, String> {
        // set_expr is [set: target], so we need to extract the rtag (target)
        let target = match set_expr {
            TagNode::Composite { ltag: _, rtag } => rtag.as_ref(),
            _ => {
                return Err("set expression must be composite [set: target]".to_string());
            }
        };
        
        // Evaluate the target to get a reference
        let target_value = self.evaluate_tag(target)?;
        
        // Evaluate the value to assign
        let value = self.evaluate_tag(value_tag)?;
        
        // Perform the assignment
        if let Value::Reference(name) = target_value {
            // Search up the frame stack to find where this is defined
            for frame in self.frames.iter_mut().rev() {
                if frame.attributes.contains_key(&name) {
                    frame.attributes.insert(name.clone(), value.clone());
                    return Ok(value);
                }
                if frame.variables.contains_key(&name) {
                    frame.variables.insert(name.clone(), value.clone());
                    return Ok(value);
                }
            }
            Err(format!("Cannot assign to undefined attribute/variable '{}'", name))
        } else {
            Err(format!("set target must resolve to a reference, got {:?}", target_value))
        }
    }

    fn writeln_log(&mut self, msg: &str) -> std::io::Result<()> {
        if let Some(ref mut file) = self.log_file {
            writeln!(file, "{}", msg)?;
            file.flush()?;
        }
        Ok(())
    }

    pub fn validate(&self, root: &TagNode) -> Result<(), String> {
        self.validate_tag(root, &std::collections::HashSet::new())
    }

    fn validate_tag(&self, tag: &TagNode, _scope: &std::collections::HashSet<String>) -> Result<(), String> {
        match tag {
            TagNode::Primitive(_) => Ok(()),
            TagNode::Composite { ltag, rtag } => {
                // Validate both sides
                self.validate_tag(ltag, _scope)?;
                self.validate_tag(rtag, _scope)?;
                
                // Check operation is valid (extract operation name from ltag, handling nesting)
                if let Ok(op_name) = self.extract_operation_name(ltag) {
                    // "define" and "set" are handled specially, not through HANDLERS registry
                    if op_name != "define" && op_name != "set" && !HANDLERS.contains_key(op_name.as_str()) {
                        return Err(format!("Unknown operation: '{}'", op_name));
                    }
                }
                Ok(())
            }
        }
    }

    pub fn execute_root(&mut self, root: &TagNode) -> Result<Value, String> {
        self.writeln_log("=== Validation ===\n")
            .map_err(|e| format!("Log error: {}", e))?;
        
        self.validate(root)?;
        
        self.writeln_log("=== Evaluation Trace ===\n")
            .map_err(|e| format!("Log error: {}", e))?;

        let result = self.evaluate_tag(root)?;

        self.writeln_log("\n=== Evaluation Complete ===")
            .map_err(|e| format!("Log error: {}", e))?;

        Ok(result)
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

                // Extract operation name from ltag (walk to innermost primitive if ltag is composite)
                let op_name = self.extract_operation_name(ltag)?;

                self.writeln_log(&format!(
                    "[Eval {}] Operation: {}",
                    eval_id, op_name
                ))
                .map_err(|e| format!("Log error: {}", e))?;

                // Dispatch based on operation - some ops need special handling
                let result = match op_name.as_str() {
                    "define" => {
                        // define needs special handling - it manages its own scope and content execution
                        self.handle_define_block(rtag)?
                    }
                    "set" => {
                        // set needs special handling - it evaluates the target (ltag) and assigns to it
                        self.handle_set_block(ltag, rtag)?
                    }
                    _ => {
                        // For other operations, evaluate rtag normally and dispatch
                        self.writeln_log(&format!("[Eval {}] Evaluating rtag...", eval_id))
                            .map_err(|e| format!("Log error: {}", e))?;
                        let rtag_value = self.evaluate_tag(rtag)?;

                        self.writeln_log(&format!("[Eval {}]   rtag evaluated to: {}", eval_id, rtag_value))
                            .map_err(|e| format!("Log error: {}", e))?;

                        self.execute_operation(&op_name, &rtag_value)?
                    }
                };

                self.writeln_log(&format!("[Eval {}] Handler result: {}", eval_id, result))
                    .map_err(|e| format!("Log error: {}", e))?;

                Ok(result)
            }
        }
    }

    fn execute_operation(&mut self, op_name: &str, rtag_value: &Value) -> Result<Value, String> {
        // Handle assignment: if rtag_value contains a reference through evaluated rtag
        // (This is for cases where we've evaluated an assignment target)
        if let Value::Reference(name) = rtag_value {
            // Search up the frame stack to find where this is defined
            for frame in self.frames.iter_mut().rev() {
                if frame.attributes.contains_key(name) {
                    frame.attributes.insert(name.clone(), Value::Item);
                    return Ok(rtag_value.clone());
                }
                if frame.variables.contains_key(name) {
                    frame.variables.insert(name.clone(), Value::Item);
                    return Ok(rtag_value.clone());
                }
            }
            return Err(format!("Cannot assign to undefined attribute/variable '{}'", name));
        }

        // Dispatch based on handler registry
        if let Some(handler) = HANDLERS.get(op_name) {
            handler(self, rtag_value)
        } else {
            Err(format!("Unknown operation '{}'", op_name))
        }
    }

    fn handle_root(&mut self, rtag: &Value) -> Result<Value, String> {
        // Root just executes its content (the implicit list)
        // The actual execution happens through normal tag evaluation
        Ok(rtag.clone())
    }

    fn handle_character(&mut self, rtag: &Value) -> Result<Value, String> {
        if let Value::Text(name) = rtag {
            self.store.insert(name.clone(), Value::Item);
            Ok(Value::Text(format!("character:{}", name)))
        } else {
            Err("Character name must be text".to_string())
        }
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

    fn handle_attribute(&mut self, rtag: &Value) -> Result<Value, String> {
        // attribute both declares (if needed in current scope) and returns a reference
        if let Value::Text(name) = rtag {
            // Check if attribute exists in any scope first
            let exists = self.frames.iter().any(|f| f.attributes.contains_key(name));
            
            // If not found in any scope, declare it in current frame
            if !exists {
                self.current_frame().attributes.insert(name.clone(), Value::Item);
            }
            
            Ok(Value::Reference(name.clone()))
        } else {
            Err("Attribute name must be text".to_string())
        }
    }
}
