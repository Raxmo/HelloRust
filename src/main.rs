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

    // Stage 1: Tokenize
    let tokens = match tokenize(&source) {
        Ok(tokens) => tokens,
        Err(e) => {
            eprintln!("Lexer error: {}", e);
            std::process::exit(1);
        }
    };

    println!("Tokens ({} total):", tokens.len());
    for (i, token) in tokens.iter().enumerate() {
        println!("  {}: {:?}", i, token);
    }

    // Stage 2: Parse
    let mut parser = StreamingParser::new(tokens);
    let root = match parser.parse() {
        Ok(root) => root,
        Err(e) => {
            eprintln!("Parse error: {}", e);
            std::process::exit(1);
        }
    };

    println!("\nParsed root tag:");
    println!("  {}", format_tag(&root, 2));

    // Stage 3 & 4: Validate and Evaluate
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
                    eprintln!("Evaluation error: {}", e);
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

fn format_tag(tag: &tag::TagNode, indent: usize) -> String {
    let ind = " ".repeat(indent);
    match tag {
        tag::TagNode::Primitive(prim) => prim.as_display_string(),
        tag::TagNode::Composite { ltag, rtag } => {
            format!(
                "[\n{}ltag: {}\n{}rtag: {}\n{}]",
                ind,
                format_tag(ltag, indent + 2),
                ind,
                format_tag(rtag, indent + 2),
                " ".repeat(indent - 2)
            )
        }
    }
}