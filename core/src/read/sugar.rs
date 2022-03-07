use super::{Reader, Error};
use crate::{Op, intern};
use crate::eval::cons;
use core::iter::Peekable;
use core::str::Chars;

pub struct SugarReader<'a> {
	chars: Peekable<Chars<'a>>
}

impl<'a> SugarReader<'a> {
	pub fn new(input: &'a str) -> Self {
		Self { chars: input.chars().peekable() }
	}

	fn try_read_arg_list(&mut self) -> Result<Op, Error> {
		self.skip_spaces();
		if let Some('(') = self.chars.peek() {
			self.chars.next();
			return self.read_list(')')
		}
		Err(Error::Continue)
	}

	fn dollar_stmt(&mut self) -> Result<Op, Error> {
		let name = match self.read_number(None) {
			Ok(x) => x,
			Err(Error::Continue) => {
				self.read_symbol(false)?
			},
			e @ _ => return e
		};
		self.skip_spaces();
		match self.chars.peek() {
			Some('=') => {
				self.chars.next();
				let value = self.read_item()?;
				Ok(cons(intern("set_scope".into()), cons(name, cons(value, Op::null()))))
			}
			_ => {
				Ok(cons(intern("get_scope".into()), cons(name, Op::null())))
			}
		}
	}

	fn at_stmt(&mut self) -> Result<Op, Error> {
		match self.chars.next().ok_or(Error::UnexpectedEof)? {
			'[' => {
				self.read_list(']')
			}
			'-' => {
				match self.chars.next().ok_or(Error::UnexpectedEof)? {
					'>' => {
						self.skip_spaces();
						match self.chars.next().ok_or(Error::UnexpectedEof)? {
							'{' => {
								let mut body = self.read_list('}')?;
								// the `{}` is not an actual list when body contains only one item
								// e.g. { (add 1 2) }
								if body.get_tail_unchecked().is_null() {
									body = body.get_head_unchecked();
								}
								Ok(cons(intern("lambda_lambda".into()), cons(body, Op::null())))
							}
							c @ _ => Err(Error::Unexpected(c))
						}
					}
					c @ _ => Err(Error::Unexpected(c))
				}
			}
			c @ _ => Err(Error::Unexpected(c))
		}
	}
}

impl<'a> Reader<'a> for SugarReader<'a> {
	#[inline]
	fn chars(&mut self) -> &mut Peekable<Chars<'a>> {
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
				'$' => {
					self.chars.next();
					return self.dollar_stmt()
				}
				'@' => {
					self.chars.next();
					return self.at_stmt()
				}
				c @ '0'..='9' => {
					self.chars.next();
					return self.read_number(Some(c))
				}
				'\'' => {
					self.chars.next();
					return self.read_symbol(true)
				}
				_ => {
					let symbol = self.read_symbol(false)?;
					match self.try_read_arg_list() {
						Ok(list) => {
							return Ok(cons(symbol, list))
						}
						Err(Error::Continue) => return Ok(symbol),
						e @ _ => return e
					}
				}
			}
		}
	}
}
