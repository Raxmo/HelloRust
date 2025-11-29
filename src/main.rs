mod lexer;
mod tag;
mod streaming_parser;
mod evaluator_v2;

use std::fs;
use std::env;
use lexer::tokenize;
use streaming_parser::StreamingParser;
use evaluator_v2::Evaluator;

fn main() {
    // Collect command-line arguments into a Vec. args[0] is the program name, args[1] is the filename.
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: packard <script.psl>");
        std::process::exit(1);
    }

    let filename = &args[1];
    let source = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            std::process::exit(1);
        }
    };

    // Stage 1: Tokenize (see src/lexer.rs)
    let tokens = match tokenize(&source) {
        Ok(tokens) => tokens,
        Err(e) => {
            eprintln!("Lexer error: {}", e);
            std::process::exit(1);
        }
    };

    println!("Tokens ({} total):", tokens.len());
    for (i, token_with_pos) in tokens.iter().enumerate() {
        println!(
            "  {}: {:?} at line {}, col {}",
            i, token_with_pos.token, token_with_pos.pos.line, token_with_pos.pos.column
        );
    }

    // Stage 2: Parse (see src/streaming_parser.rs)
    let mut parser = StreamingParser::new(tokens, source.clone());
    let root = match parser.parse() {
        Ok(root) => root,
        Err(e) => {
            let error_msg = format!("Parse error: {}", e);
            eprintln!("{}", error_msg);
            
            // Log error to file
            if let Err(io_err) = std::fs::write("parse_error.log", &error_msg) {
                eprintln!("Could not write error log: {}", io_err);
            }
            
            std::process::exit(1);
        }
    };

    println!("\nParsed root tag:");
    println!("  {}", format_tag(&root, 2));

    // Stage 3 & 4: Validate and Evaluate (see src/evaluator_v2.rs)
    match Evaluator::new("eval_trace.log") {
        Ok(mut evaluator) => {
            match evaluator.execute_root(&root) {
                Ok(result) => {
                    println!("\nEvaluation trace written to eval_trace.log");
                    println!("Result: {}", result);
                    println!("Variable store:");
                    for (key, value) in &evaluator.store {
                        println!("  {}: {}", key, value);
                    }
                }
                Err(e) => {
                    let error_msg = format!("Evaluation error: {}", e);
                    eprintln!("{}", error_msg);
                    
                    // Log error to file
                    if let Err(io_err) = std::fs::write("eval_error.log", &error_msg) {
                        eprintln!("Could not write error log: {}", io_err);
                    }
                    
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Could not create log file: {}", e);
            std::process::exit(1);
        }
    }
}

// Format a TagNode tree for pretty-printing with indentation.
// Shows ltag/rtag traversal at each level.
fn format_tag(tag: &tag::TagNode, indent: usize) -> String {
    let ind = " ".repeat(indent);
    match tag {
        tag::TagNode::Primitive(prim) => prim.as_display_string(),
        tag::TagNode::Composite { ltag, rtag } => {
            let ltag_str = format_tag(ltag, indent + 2);
            let rtag_str = format_tag(rtag, indent + 2);
            format!(
                "[\n{}ltag: {}\n{}rtag: {}\n{}]",
                ind, ltag_str, ind, rtag_str,
                " ".repeat(if indent > 0 { indent - 2 } else { 0 })
            )
        }
    }
}
