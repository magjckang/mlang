mod sugar;

pub use sugar::SugarReader;
use crate::{Op, intern};
use core::iter::Peekable;
use core::str::{Chars, FromStr};

#[derive(Debug)]
pub enum Error {
	Continue,
	Eof,
	Unexpected(char),
	UnexpectedEof,
	UnsupportedChar(char),
}

pub trait Reader<'a> {
	fn chars(&mut self) -> &mut Peekable<Chars<'a>>;

	fn read_item(&mut self) -> Result<Op, Error>;

	fn read(&mut self) -> Result<Op, Error> {
		self.read_item()
	}

	fn read_list(&mut self, delimiter: char) -> Result<Op, Error> {
		let mut head = Op::null();
		let mut tail = head;
		loop {
			match self.read_item() {
				Ok(op) => {
					let op = Op::pair(op, Op::null());
					if head.is_null() {
						head = op;
					} else {
						tail.set_tail_unchecked(op);
					}
					tail = op;
				}
				Err(Error::Unexpected(',')) => {
					self.chars().next();
				}
				Err(Error::Unexpected(c)) if c == delimiter => {
					self.chars().next();
					break
				}
				Err(Error::Eof) => return Err(Error::UnexpectedEof),
				e @ _ => return e
			} 
		}
		if head.is_null() {
			head = Op::pair(Op::null(), Op::null());
		}
		Ok(head)
	}

	fn read_number(&mut self, lead: Option<char>) -> Result<Op, Error> {
		let chars = self.chars();
		let mut s = String::new();
		if let Some(c) = lead {
			s.push(c);
		} else {
			match *chars.peek().ok_or(Error::UnexpectedEof)? {
				c @ '0'..='9' => {
					chars.next();
					s.push(c);
				}
				_ => return Err(Error::Continue)
			}
		}
		while let Some(c) = chars.next_if(|&c| matches!(c, '0'..='9')) {
			s.push(c);
		}
		Ok(Op::long(isize::from_str(&s[..]).unwrap()))
	}

	fn read_symbol(&mut self, quoted: bool) -> Result<Op, Error> {
		let chars = self.chars();
		match *chars.peek().ok_or(Error::UnexpectedEof)? {
			'0'..='9' | 'A'..='Z' | 'a'..='z' | '_' => {
				let mut s = String::new();
				s.push(unsafe { chars.next().unwrap_unchecked() });
				while let Some(c) = chars.peek().cloned() {
					match c {
						'0'..='9' | 'A'..='Z' | 'a'..='z' | '_' => {
							chars.next();
							s.push(c);
						}
						'\'' => {
							if quoted {
								chars.next();
							}
							break
						}
						_ => {
							if quoted {
								chars.next();
								return Err(Error::UnsupportedChar(c))
							}
							break
						}
					}
				}
				Ok(intern(s))
			}
			c @ _ => Err(if quoted { Error::UnsupportedChar(c) } else { Error::Unexpected(c) })
		}
	}

	fn skip_spaces(&mut self) {
		while self.chars().next_if(char::is_ascii_whitespace).is_some() {}
	}
}

pub struct BaseReader<'a> {
	chars: Peekable<Chars<'a>>
}

impl<'a> BaseReader<'a> {
	pub fn new(input: &'a str) -> Self {
		Self { chars: input.chars().peekable() }
	}
}

impl<'a> Reader<'a> for BaseReader<'a> {
	#[inline]
	fn chars(& mut self) -> & mut Peekable<Chars<'a>> {
		&mut self.chars
	}

	fn read_item(&mut self) -> Result<Op, Error> {
		self.skip_spaces();
		loop {
			match *self.chars.peek().ok_or(Error::Eof)? {
				'(' => {
					self.chars.next();
					return self.read_list(')')
				}
				'[' => {
					self.chars.next();
					return self.read_list(']')
				}
				c @ '0'..='9' => {
					self.chars.next();
					return self.read_number(Some(c));
				}
				'\'' => {
					self.chars.next();
					return self.read_symbol(true)
				}
				_ => {
					return self.read_symbol(false)
				}
			}
		}
	}
}
