use crate::ast::{Tag, PrimitiveValue};
use std::fs::File;
use std::io::Write;

pub fn trace_to_file(ast: &[Tag], filename: &str) -> std::io::Result<()> {
    let mut file = File::create(filename)?;
    
    writeln!(file, "=== Parse Tree Trace ===")?;
    writeln!(file)?;
    
    for (i, tag) in ast.iter().enumerate() {
        writeln!(file, "Tag {}:", i)?;
        trace_tag(&mut file, tag, 1)?;
        writeln!(file)?;
    }
    
    Ok(())
}

fn trace_tag(file: &mut File, tag: &Tag, depth: usize) -> std::io::Result<()> {
    let indent = "  ".repeat(depth);
    
    match tag {
        Tag::Primitive(prim) => {
            writeln!(file, "{}{}", indent, format_primitive(prim))?;
        }
        Tag::Composite { ltag, rtag } => {
            writeln!(file, "{}Composite {{", indent)?;
            write!(file, "{}  ltag: ", indent)?;
            trace_tag(file, ltag, depth + 2)?;
            write!(file, "{}  rtag: ", indent)?;
            trace_tag(file, rtag, depth + 2)?;
            writeln!(file, "{}}}", indent)?;
        }
    }
    
    Ok(())
}

fn format_primitive(prim: &PrimitiveValue) -> String {
    match prim {
        PrimitiveValue::Identifier(s) => format!("Identifier(\"{}\")", s),
        PrimitiveValue::Number(n) => {
            if n.fract() == 0.0 {
                format!("Number({})", *n as i64)
            } else {
                format!("Number({})", n)
            }
        }
        PrimitiveValue::String(s) => format!("String(\"{}\")", s),
        PrimitiveValue::Keyword(s) => format!("Keyword(\"{}\")", s),
    }
}
