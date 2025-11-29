mod lexer;
use lexer::tokenize;
use std::fs;

fn main() {
    let source = fs::read_to_string("test.psl").unwrap();
    let tokens = tokenize(&source).unwrap();
    for (i, token) in tokens.iter().enumerate() {
        println!("{}: {:?}", i, token);
    }
}
