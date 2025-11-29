// Module declarations - each of these corresponds to a .rs file
// (lexer.rs, tag.rs, streaming_parser.rs, evaluator_v2.rs)
// In Rust, modules are how you organize code (similar to namespaces in C++)
mod lexer;
mod tag;
mod streaming_parser;
mod evaluator_v2;

// Standard library imports
// std::fs - filesystem operations (like C++ <fstream>)
// std::env - environment and command-line args (like C++ argv)
use std::fs;
use std::env;
// These are specific items from our modules we'll use directly
use lexer::tokenize;
use streaming_parser::StreamingParser;
use evaluator_v2::Evaluator;

fn main() {
    // env::args() returns an iterator over command-line arguments as strings
    // .collect() consumes the iterator and creates a Vec<String> (like C++ vector<string>)
    // Example: if user runs "packard test.psl", args = ["packard", "test.psl"]
    let args: Vec<String> = env::args().collect();
    
    // In Rust, we need at least 2 args: program name + filename
    // Check the length before accessing by index (prevents panic/crash)
    if args.len() < 2 {
        // eprintln! writes to stderr (like C++ cerr)
        eprintln!("Usage: packard <script.psl>");
        std::process::exit(1);  // Exit with error code 1
    }

    // Get the filename (args[0] is program name, args[1] is first real argument)
    // The & means we're borrowing the string, not taking ownership
    // (In Rust, if you move ownership, original variable can't be used anymore)
    let filename = &args[1];
    
    // Read file contents into a String
    // Result<T, E> is Rust's error handling (like C++ exceptions but explicit)
    // match statement unwraps the Result:
    //   - Ok(value) means it succeeded, use the value
    //   - Err(e) means it failed, handle the error
    // fs::read_to_string() returns Result<String, io::Error>
    let source = match fs::read_to_string(filename) {
        Ok(content) => content,  // Extract the String from Ok
        Err(e) => {              // Extract the Error from Err
            eprintln!("Error reading file: {}", e);
            std::process::exit(1);
        }
    };

    // ============================================================================
    // STAGE 1: LEXER (tokenize.rs)
    // Convert the source code string into a Vec<Token>
    // This breaks down the raw text into meaningful units (keywords, operators, etc)
    // ============================================================================
    let tokens = match tokenize(&source) {
        Ok(tokens) => tokens,
        Err(e) => {
            eprintln!("Lexer error: {}", e);
            std::process::exit(1);
        }
    };

    // Print tokens for debugging
    println!("Tokens ({} total):", tokens.len());
    // .iter() creates an iterator, .enumerate() gives us (index, item) pairs
    // In Rust, you iterate explicitly like this (no foreach like C++11)
    for (i, token) in tokens.iter().enumerate() {
        // {:?} is the debug format specifier (prints with Debug trait)
        println!("  {}: {:?}", i, token);
    }

    // ============================================================================
    // STAGE 2: PARSER (streaming_parser.rs)
    // Convert tokens into a tree structure (TagNode)
    // Each tag is [ltag: rtag] - a binary tree structure
    // ============================================================================
    let mut parser = StreamingParser::new(tokens);
    // mut means mutable - parser will modify its internal position as it reads tokens
    // (In Rust, variables are immutable by default, unlike C++)
    
    let root = match parser.parse() {
        Ok(root) => root,  // Returns a TagNode tree
        Err(e) => {
            eprintln!("Parse error: {}", e);
            std::process::exit(1);
        }
    };

    println!("\nParsed root tag:");
    // format_tag recursively pretty-prints the tree structure
    println!("  {}", format_tag(&root, 2));

    // ============================================================================
    // STAGE 3 & 4: VALIDATOR + EVALUATOR (evaluator_v2.rs)
    // Validate the tree, then execute it
    // ============================================================================
    match Evaluator::new("eval_trace.log") {
        Ok(mut evaluator) => {
            // Evaluator needs to be mut because execute_root() modifies its internal state
            // (frame stack, variable store, log file, eval counter, etc)
            
            match evaluator.execute_root(&root) {
                Ok(result) => {
                    // Execution succeeded!
                    println!("\nEvaluation trace written to eval_trace.log");
                    println!("Result: {}", result);
                    
                    // Print the global variable store (HashMap of variable names → values)
                    println!("Variable store:");
                    // We use & to borrow evaluator.store, not take ownership
                    // This lets us print it without consuming it
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

// Helper function to pretty-print a TagNode tree with indentation
// This is a recursive function that walks the entire tree
// Parameters:
//   tag - reference to a TagNode (& means we borrow, don't take ownership)
//   indent - current indentation level (number of spaces)
// Returns: String containing the pretty-printed tree
fn format_tag(tag: &tag::TagNode, indent: usize) -> String {
    // Create a string of spaces for indentation
    // .repeat(n) repeats the string n times (useful for formatting)
    let ind = " ".repeat(indent);
    
    // Match on the tag type (pattern matching - like switch in C++ but more powerful)
    match tag {
        // Case 1: Primitive value (like "name", 42, "text", etc)
        // These are leaf nodes in the tree
        tag::TagNode::Primitive(prim) => prim.as_display_string(),
        
        // Case 2: Composite tag with ltag and rtag
        // The { ltag, rtag } syntax destructures the struct fields
        // (like unpacking a tuple in C++17)
        tag::TagNode::Composite { ltag, rtag } => {
            // format! is like sprintf in C++ - builds a string from template + args
            // \n is a newline
            format!(
                "[\n{}ltag: {}\n{}rtag: {}\n{}]",
                ind,
                // Recursively call format_tag on ltag with increased indent
                format_tag(ltag, indent + 2),
                ind,
                // Recursively call format_tag on rtag with increased indent
                format_tag(rtag, indent + 2),
                " ".repeat(indent - 2)
            )
        }
    }
}