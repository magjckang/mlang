use crate::{Object, Op};
use crate::GLOBALS;

#[derive(Debug)]
pub enum Error {
	Undefined(Op),
	CanNotApply(Op),
	RequireLong(Op),
	RequireSymbol(Op),
	RequirePair(Op),
	RequireExpr(Op),
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
				let (should_apply, should_eval_tail) = match head.as_ref() {
					Some(Object::Subr { is_fixed, .. }) => {
						(true, !*is_fixed)
					}
					Some(Object::Expr { .. }) => {
						(true, true)
					}
					_ => {
						(false, true)
					}
				};
				let tail = if should_eval_tail {
					evlis(tail.into(), env)?
				} else {
					tail.into()
				};
				if should_apply {
					apply(head, tail, env)
				} else {
					Ok(cons(head, tail))
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
	if fun.is_null() {
		return Err(Error::CanNotApply(fun))
	}
	match fun.as_ref_unchecked() {
		Object::Subr { imp, .. } => {
			imp(args, env)
		}
		Object::Expr { def, env } => {
			let def: Op = def.into();
			let env: Op = env.into();
			let env = pairlis(def.get_head_unchecked(), args, env)?;
			eval(def.get_tail_unchecked(), env)
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

#[inline]
fn cons(head: Op, tail: Op) -> Op {
	Op::pair(head, tail)
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

macro_rules! check_args {
	($args:ident, let $name:ident $(, $check:ident, $error:ident)?) => {
		if !$args.is_pair() {
			return Err(Error::TooFewArgs)
		}
		let $name = $args.get_head_unchecked();
		$( if !$name.$check() {
			return Err(Error::$error($name))
		} )?
	};
	($args:ident, let $name:ident $(, $check:ident, $error:ident)? $(let $r_name:ident $(, $r_check:ident, $r_error:ident)?)+) => {
		check_args!($args, let $name $(, $check, $error )? );
		let $args = $args.get_tail_unchecked();
		check_args!($args, $( let $r_name $(, $r_check, $r_error )? )+);
	};
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
	check_args! {
		args,
		let symbols
		let body
	}
	Ok(Op::expr(cons(symbols, body), env))
}

// lambda that omit the arg list
pub fn subr_lambda_lambda(args: Op, env: Op) -> Result<Op, Error> {
	check_args! {
		args,
		let body
	}
	Ok(Op::expr(cons(Op::null(), body), env))
}

pub fn subr_apply(args: Op, env: Op) -> Result<Op, Error> {
	let fun = car(args);
	let args = cdr(args);
	let fun_args = car(args);
	let args = cdr(args);
	let mut ctx = car(args);
	if ctx.is_null() {
		ctx = env;
	}
	apply(fun, fun_args, ctx)
}

pub fn subr_set_scope(args: Op, env: Op) -> Result<Op, Error> {
	subr_define(args, env)
}

pub fn subr_get_scope(args: Op, env: Op) -> Result<Op, Error> {
	check_args! {
		args,
		let name
	};
	if name.is_long() {
		let n = name.get_long_unchecked();
		let mut env = env;
		while !env.is_null() {
			let pair = car(env);
			let key = car(pair);
			if key.is_long() && key.get_long_unchecked() == n {
				return Ok(cdr(pair))
			}
			env = cdr(env);
		}
	}
	Ok(name)
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

pub fn subr_new_list(_args: Op, _env: Op) -> Result<Op, Error> {
	Ok(Op::pair(Op::null(), Op::null()))
}

pub fn subr_list_append(args: Op, _env: Op) -> Result<Op, Error> {
	check_args! {
		args,
		let list, is_pair, RequirePair
		let elem
	};
	if list.get_head_unchecked().is_null() {
		list.set_head_unchecked(elem);
		return Ok(list)
	}
	let mut tail = list;
	loop {
		let maybe_tail = tail.get_tail_unchecked();
		if maybe_tail.is_null() {
			tail.set_tail_unchecked(cons(elem, Op::null()));
			return Ok(list)
		}
		if !maybe_tail.is_pair() {
			return Err(Error::RequirePair(tail))
		}
		tail = maybe_tail
	}
}

pub fn subr_list_prepend(args: Op, _env: Op) -> Result<Op, Error> {
	check_args! {
		args,
		let list, is_pair, RequirePair
		let elem
	}
	if list.get_head_unchecked().is_null() {
		list.set_head_unchecked(elem);
		return Ok(list)
	}
	let head = list.get_head_unchecked();
	let tail = list.get_tail_unchecked();
	list.set_head_unchecked(elem);
	list.set_tail_unchecked(cons(head, tail));
	Ok(list)
}

pub fn subr_list_count(args: Op, _env: Op) -> Result<Op, Error> {
	check_args! {
		args,
		let list, is_pair, RequirePair
	}
	if list.get_head_unchecked().is_null() {
		return Ok(Op::long(0))
	}
	let mut tail = list;
	let mut count = 1;
	loop {
		let maybe_tail = tail.get_tail_unchecked();
		if maybe_tail.is_null() {
			break
		}
		if !maybe_tail.is_pair() {
			return Err(Error::RequirePair(maybe_tail))
		}
		count += 1;
		tail = maybe_tail;
	}
	Ok(Op::long(count))
}

pub fn subr_list_index(args: Op, _env: Op) -> Result<Op, Error> {
	check_args! {
		args,
		let list, is_pair, RequirePair
		let index, is_long, RequireLong
	};
	let mut tail = list;
	let mut index = index.get_long_unchecked();
	loop {
		if index == 0 {
			break
		}
		let maybe_tail = tail.get_tail_unchecked();
		if maybe_tail.is_null() {
			return Ok(Op::null())
		}
		if !maybe_tail.is_pair() {
			return Err(Error::RequirePair(maybe_tail))
		}
		index -= 1;
		tail = maybe_tail;
	}
	Ok(tail.get_head_unchecked())
}

pub fn subr_list_map(args: Op, _env: Op) -> Result<Op, Error> {
	check_args! {
		args,
		let list, is_pair, RequirePair
		let fun, is_expr, RequireExpr // TODO: allow Subr
	};
	let new_list = cons(Op::null(), Op::null());
	if list.get_head_unchecked().is_null() {
		return Ok(new_list)
	}
	let mut tail = list;
	let mut new_tail = new_list;
	loop {
		let elem = tail.get_head_unchecked();
		// call closure with modified context then restore it.
		let original_env = fun.get_env_unchecked();
		fun.set_env_unchecked(cons(cons(Op::long(0), elem), original_env));
		let new_elem = apply(fun, cons(elem, Op::null()), Op::null())?;
		fun.set_env_unchecked(original_env);
		new_tail.set_head_unchecked(new_elem);
		tail = tail.get_tail_unchecked();
		if tail.is_null() {
			break
		}
		if !tail.is_pair() {
			return Err(Error::RequirePair(tail))
		}
		let tmp = cons(Op::null(), Op::null());
		new_tail.set_tail_unchecked(tmp);
		new_tail = tmp;
	}
	Ok(new_list)
}
