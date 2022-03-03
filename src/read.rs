use crate::{Op, intern};
use core::iter::Peekable;
use core::str::{Chars, FromStr};

#[derive(Debug)]
pub enum Error {
	Continue,
	Eof,
	EndOfList,
	UnexpectedEof,
	UnsupportedChar(char),
}

pub fn read(chars: &mut Peekable<Chars<'_>>) -> Result<Op, Error> {
	while let Some(c) = chars.peek() {
		match c {
			' ' | ',' | '\r' | '\n' => {
				chars.next();
				continue
			}
			'(' => {
				chars.next();
				return read_list(chars, ')')
			}
			')' => {
				return Err(Error::EndOfList)
			}
			'[' => {
				chars.next();
				return read_list(chars, ']')
			}
			']' => {
				return Err(Error::EndOfList)
			}
			'0'..='9' => {
				let mut input = String::new();
				while let Some(c) = chars.peek() {
					match c {
						'0'..='9' => {
							input.push(unsafe { chars.next().unwrap_unchecked() })
						}
						_ => {
							return Ok(Op::long(isize::from_str(&input[..]).unwrap()))
						}
					}
				}
			}
			'\'' => {
				chars.next();
				return read_symbol(chars, true)
			}
			_ => {
				match read_symbol(chars, false) {
					Err(Error::Continue) => {}
					x @ _ => return x
				}
			}
		}
	}
	Err(Error::Eof)
}

fn read_symbol(chars: &mut Peekable<Chars<'_>>, quoted: bool) -> Result<Op, Error> {
	let c = unsafe { chars.next().unwrap_unchecked() };
	match c {
		'A'..='Z' | 'a'..='z' | '_' => {
			let mut s = String::new();
			s.push(c);
			while let Some(c) = chars.peek() {
				match c {
					'0'..='9' | 'A'..='Z' | 'a'..='z' | '_' => {
						s.push(unsafe { chars.next().unwrap_unchecked() })
					}
					'\'' => {
						if quoted {
							chars.next();
							return Ok(intern(s))
						}
						break
					}
					_ => {
						if quoted {
							let c = unsafe { chars.next().unwrap_unchecked() };
							return Err(Error::UnsupportedChar(c))
						}
						break
					}
				}
			}
			Ok(intern(s))
		}
		_ => {
			if quoted {
				return Err(Error::UnsupportedChar(c))
			}
			Err(Error::Continue)
		}
	}
}

fn read_list(chars: &mut Peekable<Chars<'_>>, delimiter: char) -> Result<Op, Error> {
	let mut head = Op::null();
	let mut tail: Op;
	match read(chars) {
		Ok(op) => {
			head = Op::pair(op, Op::null());
			tail = head;
			loop {
				match read(chars) {
					Ok(op) => {
						let op = Op::pair(op, Op::null());
						tail.set_tail_unchecked(op);
						tail = op;
					}
					Err(Error::EndOfList) => {
						break
					}
					Err(Error::Eof) => {
						return Err(Error::UnexpectedEof)
					}
					r @ _ => {
						return r
					}
				}
			}
		}
		Err(Error::Eof) => {}
		Err(err @ _) => {
			return Err(err)
		}
	}
	match chars.next() {
		None => return Err(Error::UnexpectedEof),
		Some(c) => {
			if c != delimiter {
				return Err(Error::UnexpectedEof)
			}
		}
	}
	Ok(head)
}
