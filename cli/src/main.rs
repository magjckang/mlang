use mlang::{Reader as _, BaseReader, ParseError};
use std::io;

fn rep(s: &str) {
	let mut reader = BaseReader::new(s);
	loop {
		match reader.read() {
			Ok(op) => {
				match mlang::eval(op) {
					Ok(op) => {
						println!(" => {:?}", op);
					}
					Err(e) => {
						println!("✗ {:?}", e);
					}
				}
			}
			Err(ParseError::Eof) => break,
			Err(e) => {
				println!("✗ {:?}", e);
			}
		}
	}
}

fn main() -> io::Result<()> {
	mlang::init();

	let stdin = io::stdin();
	let mut input = String::new();

	loop {
		stdin.read_line(&mut input)?;
		rep(&input[..]);
		input.clear();
	}
}
