use crate::{Object, Op};
use crate::GLOBALS;

#[derive(Debug)]
pub enum Error {
	Undefined(Op),
	CanNotApply(Op),
	RequireLong(Op),
	RequireSymbol(Op),
	TooFewArgs,
}

pub fn eval(op: Op, env: Op) -> Result<Op, Error> {
	println!("EVAL {:#?} IN {:#?}", op, env);
	match op.as_ref() {
		None => {
			Ok(Op::null())
		}
		Some(object) => match object {
			Object::Symbol { .. } => {
				let assoc_pair = assoc(op, env);
				if assoc_pair.is_null() {
					return Err(Error::Undefined(op))
				}
				Ok(cdr(assoc_pair))
			}
			Object::Pair { head, tail } => {
				let head = eval(head.into(), env)?;
				if head.is_subr() && head.get_is_fixed_unchecked() {
					apply(head, tail.into(), env)
				} else {
					apply(head, evlis(tail.into(), env)?, env)
				}
			}
			_ => {
				Ok(op)
			}
		}
	}
}

fn apply(fun: Op, args: Op, env: Op) -> Result<Op, Error> {
	println!("APPLY {:#?} TO {:#?} IN {:#?}", fun, args, env);
	match fun.as_ref_unchecked() {
		Object::Subr { imp, .. } => {
			imp(args, env)
		}
		Object::Expr { def, env } => {
			let def: Op = def.into();
			let env: Op = env.into();
			let env = pairlis(def.get_head_unchecked(), args, env)?;
			let mut body = def.get_tail_unchecked();
			let mut result = Op::null();
			while body.is_pair() {
				result = eval(body.get_head_unchecked(), env)?;
				body = body.get_tail_unchecked();
			}
			Ok(result)
		}
		_ => {
			Err(Error::CanNotApply(fun))
		}
	}
}

fn car(op: Op) -> Op {
	match op.as_ref() {
		Some(Object::Pair { head, .. }) => Op(*head),
		_ => Op::null()
	}
}

fn cdr(op: Op) -> Op {
	match op.as_ref() {
		Some(Object::Pair { tail, .. }) => Op(*tail),
		_ => Op::null()
	}
}

fn caar(op: Op) -> Op { car(car(op)) }
fn cadr(op: Op) -> Op { car(cdr(op)) }

pub fn define(name: Op, value: Op, env: Op) -> Op {
	let pair = Op::pair(name, value);
	let env_tail = Op::pair(pair, cdr(env));
	env.set_tail_unchecked(env_tail);
	pair
}

fn pairlis(mut names: Op, mut values: Op, mut env: Op) -> Result<Op, Error> {
	while names.is_pair() {
		let name = names.get_head_unchecked();
		if !name.is_symbol() {
			return Err(Error::RequireSymbol(name))
		}
		if !values.is_pair() {
			return Err(Error::TooFewArgs)
		}
		let value = values.get_head_unchecked();
		env = Op::pair(Op::pair(name, value), env);
		names = names.get_tail_unchecked();
		values = values.get_tail_unchecked();
	}
	if names.is_symbol() {
		env = Op::pair(Op::pair(names, values), env);
	}
	Ok(env)
}

fn assoc(key: Op, env: Op) -> Op {
	if caar(env) == key {
		return car(env)
	}
	let tail = cdr(env);
	if tail.is_null() {
		return Op::null()
	}
	assoc(key, tail)
}

fn evlis(op: Op, env: Op) -> Result<Op, Error> {
	if op.is_null() {
		return Ok(Op::null())
	}
	let head = eval(car(op), env)?;
	let tail = evlis(cdr(op), env)?;
	Ok(Op::pair(head, tail))
}

pub fn subr_define(args: Op, env: Op) -> Result<Op, Error> {
	let name = car(args);
	if !name.is_symbol() {
		return Err(Error::RequireSymbol(name))
	}
	let value = eval(cadr(args), env)?;
	define(name, value, unsafe { GLOBALS });
	Ok(value)
}

pub fn subr_lambda(args: Op, env: Op) -> Result<Op, Error> {
	Ok(Op::expr(args, env))
}

pub fn subr_add(args: Op, _env: Op) -> Result<Op, Error> {
	if !args.is_pair() {
		return Err(Error::TooFewArgs)
	}
	let lhs = args.get_head_unchecked();
	let args = args.get_tail_unchecked();
	if !args.is_pair() {
		return Err(Error::TooFewArgs)
	}
	let rhs = args.get_head_unchecked();
	if !lhs.is_long() {
		return Err(Error::RequireLong(lhs))
	}
	if !rhs.is_long() {
		return Err(Error::RequireLong(rhs))
	}
	Ok(Op::long(lhs.get_long_unchecked() + rhs.get_long_unchecked()))
}
