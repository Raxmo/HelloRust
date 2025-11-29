use crate::ast::{Tag, PrimitiveValue};
use std::sync::atomic::{AtomicUsize, Ordering};

static DEPTH: AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

pub fn trace_enter(context: &str, op_name: &str, tag: &Tag) {
    let depth = DEPTH.load(Ordering::SeqCst);
    let indent = "  ".repeat(depth);
    
    let tag_preview = match tag {
        Tag::Primitive(p) => format!("{:?}", p),
        Tag::Composite { ltag, rtag } => {
            format!("Composite[ltag: {:?}, rtag: ...]", ltag)
        }
    };
    
    println!("{}→ [{}] {} | {}", indent, context, op_name, tag_preview);
    DEPTH.store(depth + 1, Ordering::SeqCst);
}

pub fn trace_exit(result: &str) {
    let depth = DEPTH.load(Ordering::SeqCst);
    if depth > 0 {
        DEPTH.store(depth - 1, Ordering::SeqCst);
    }
    let indent = "  ".repeat(depth.saturating_sub(1));
    println!("{}← {}", indent, result);
}

pub fn trace_eval_tag(tag: &Tag) {
    let depth = DEPTH.load(Ordering::SeqCst);
    let indent = "  ".repeat(depth);
    
    match tag {
        Tag::Primitive(p) => {
            println!("{}eval_tag(Primitive[{:?}])", indent, p);
        }
        Tag::Composite { ltag, rtag } => {
            println!("{}eval_tag(Composite[", indent);
            println!("{}  ltag: {:?}", indent, ltag);
            println!("{}  rtag: {:?}", indent, rtag);
            println!("{}])", indent);
        }
    }
}
