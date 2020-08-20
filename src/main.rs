#![allow(dead_code)]
mod lexer;
mod syntax;

use lexer::Lexer;
use syntax::*;

fn main() {
	println!("Hello, world!");
	
	let file = std::fs::read("valid/example").unwrap();
	let lexer = Lexer::new(&file).unwrap();
	let mut parser = Parser::new(lexer);
	let file = File::try_parse(&mut parser);

	println!("{:#?}, {:?}", file, parser.lexeme);
}
