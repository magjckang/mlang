mod read;
mod eval;

use eval::{Error as EvalError};
use core::cell::RefCell;
use core::fmt::{self, Debug};
use core::ptr;

#[derive(Debug)]
enum Object {
	Long(isize),
	Symbol {
		s: String,
	},
	Pair {
		head: *const Object,
		tail: *const Object,
	},
	Expr {
		def: *const Object,
		env: *const Object,
	},
	Subr {
		imp: PrimFun,
		name: String,
		is_fixed: bool,
	},
}

type PrimFun = fn(Op, Op) -> Result<Op, EvalError>;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Op(*const Object);

#[allow(unused)]
impl Op {
	const fn null() -> Self {
		Self(ptr::null())
	}

	fn new(obj: Object) -> Self {
		OBJECTS.with(|cell| {
			let mut vec = cell.borrow_mut();
			vec.push(obj);
			Self(vec.last().unwrap() as _)
		})
	}

	fn long(n: isize) -> Self {
		Self::new(Object::Long(n))
	}

	fn symbol(s: String) -> Self {
		Self::new(Object::Symbol { s })
	}

	fn pair(head: Self, tail: Self) -> Self {
		Self::new(Object::Pair { head: head.0, tail: tail.0 })
	}

	fn expr(def: Self, env: Self) -> Self {
		Self::new(Object::Expr { def: def.0, env: env.0 })
	}

	fn subr(imp: PrimFun, name: String, is_fixed: bool) -> Self {
		Self::new(Object::Subr { imp, name, is_fixed })
	}

	#[inline]
	fn is_null(&self) -> bool {
		self.0.is_null()
	}

	fn as_ref<'a>(&self) -> Option<&'a Object> {
		if self.is_null() { None } else { unsafe { Some(&*self.0) } }
	}

	fn as_ref_unchecked<'a>(&self) -> &'a Object {
		unsafe { &*self.0 }
	}

	fn as_mut<'a>(&self) -> Option<&'a mut Object> {
		if self.is_null() { None } else { unsafe { Some(&mut *(self.0 as *mut _)) } }
	}

	fn as_mut_unchecked<'a>(&self) -> &'a mut Object {
		unsafe { &mut *(self.0 as *mut _) }
	}

	fn is_long(&self) -> bool {
		matches!( self.as_ref(), Some(Object::Long(..)) )
	}

	fn is_symbol(&self) -> bool {
		matches!( self.as_ref(), Some(Object::Symbol { .. }) )
	}

	fn is_pair(&self) -> bool {
		matches!( self.as_ref(), Some(Object::Pair { .. }) )
	}

	fn is_expr(&self) -> bool {
		matches!( self.as_ref(), Some(Object::Expr { .. }) )
	}

	fn is_subr(&self) -> bool {
		matches!( self.as_ref(), Some(Object::Subr { .. }) )
	}

	fn get_long_unchecked(&self) -> isize {
		match self.as_ref_unchecked() {
			Object::Long(n) => *n,
			_ => unsafe { core::hint::unreachable_unchecked() }
		}
	}

	fn get_symbol_unchecked<'a>(&self) -> &'a String {
		match self.as_ref_unchecked() {
			Object::Symbol { s } => s,
			_ => unsafe { core::hint::unreachable_unchecked() }
		}
	}

	fn get_head_unchecked(&self) -> Self {
		match self.as_ref_unchecked() {
			Object::Pair { head, .. } => { Self(*head) }
			_ => unsafe { core::hint::unreachable_unchecked() }
		}
	}

	fn set_head_unchecked(&self, op: Op) {
		match self.as_mut_unchecked() {
			Object::Pair { head, .. } => { *head = op.0 }
			_ => unsafe { core::hint::unreachable_unchecked() }
		}
	}

	fn get_tail_unchecked(&self) -> Self {
		match self.as_ref_unchecked() {
			Object::Pair { tail, .. } => { Self(*tail) }
			_ => unsafe { core::hint::unreachable_unchecked() }
		}
	}

	fn set_tail_unchecked(&self, op: Op) {
		match self.as_mut_unchecked() {
			Object::Pair { tail, .. } => { *tail = op.0 }
			_ => unsafe { core::hint::unreachable_unchecked() }
		}
	}

	fn get_env_unchecked(&self) -> Self {
		match self.as_ref_unchecked() {
			Object::Expr { env, .. } => { Self(*env) }
			_ => unsafe { core::hint::unreachable_unchecked() }
		}
	}

	fn set_env_unchecked(&self, op: Op) {
		match self.as_mut_unchecked() {
			Object::Expr { env, .. } => { *env = op.0 }
			_ => unsafe { core::hint::unreachable_unchecked() }
		}
	}

	fn get_is_fixed_unchecked(&self) -> bool {
		match self.as_ref_unchecked() {
			Object::Subr { is_fixed, .. } => *is_fixed,
			_ => unsafe { core::hint::unreachable_unchecked() }
		}
	}
}

impl Debug for Op {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		if *self == unsafe { GLOBALS } {
			return write!(f, "<globals>")
		}
		match self.as_ref() {
			None => {
				write!(f, "nil")
			}
			Some(obj) => match obj {
				Object::Long(n) => {
					write!(f, "Long {:?}", n)
				}
				Object::Symbol { s } => {
					write!(f, "Symbol {:?}", s)
				}
				Object::Pair { head, tail } => {
					f.debug_struct("Pair")
						.field("head", &Self(*head))
						.field("tail", &Self(*tail))
						.finish()
				}
				Object::Expr { def, env } => {
					f.debug_struct("Expr")
						.field("def", &Self(*def))
						.field("env", &Self(*env))
						.finish()
				}
				Object::Subr { name, .. } => {
					write!(f, "Subr {:?}", name)
				}
			}
		}
	}
}

impl From<*const Object> for Op {
	fn from(ptr: *const Object) -> Self {
		Self(ptr)
	}
}

impl From<&*const Object> for Op {
	fn from(ptr: &*const Object) -> Self {
		Self(*ptr)
	}
}

thread_local! {
	static OBJECTS: RefCell<Vec<Object>> = RefCell::new(Vec::with_capacity(1000));
}

static mut SYMBOLS: Op = Op::null();
static mut GLOBALS: Op = Op::null();

fn intern(s: String) -> Op {
	let mut list = unsafe { SYMBOLS };
	while list.is_pair() {
		let symbol = list.get_head_unchecked();
		if symbol.get_symbol_unchecked() == &s {
			return symbol
		}
		list = list.get_tail_unchecked();
	}
	let symbol = Op::symbol(s);
	unsafe { SYMBOLS = Op::pair(symbol, SYMBOLS) }
	symbol
}

fn repl(s: &str) {
	let mut chars = s.chars().peekable();

	match read::read(&mut chars) {
		Ok(op) => {
			match eval::eval(op, unsafe { GLOBALS }) {
				Ok(op) => {
					println!(" => {:#?}", op);
				}
				Err(e) => {
					println!("{:?}", e);
				}
			}
		}
		Err(err) => {
			println!("{:?}", err);
		}
	}
}

fn main() {
	let global_var = Op::pair(intern("globals".into()), Op::null());
	unsafe {
		GLOBALS = Op::pair(global_var, GLOBALS);
		global_var.set_tail_unchecked(GLOBALS)
	}

	let sub_routes: [(&str, PrimFun, bool); 14] = [
		("define", eval::subr_define, true),
		("lambda", eval::subr_lambda, true),
		("lambda_lambda", eval::subr_lambda_lambda, true),
		("set_scope", eval::subr_set_scope, true),
		("get_scope", eval::subr_get_scope, false),
		("apply", eval::subr_apply, false),
		("lambda_apply", eval::subr_apply, false),
		("add", eval::subr_add, false),
		("new_list", eval::subr_new_list, false),
		("list_append", eval::subr_list_append, false),
		("list_prepend", eval::subr_list_prepend, false),
		("list_count", eval::subr_list_count, false),
		("list_index", eval::subr_list_index, false),
		("list_map", eval::subr_list_map, false),
	];

	for (name, fun, is_fixed) in sub_routes {
		let subr = Op::subr(fun, name.into(), is_fixed);
		eval::define(intern(name.into()), subr, unsafe { GLOBALS });
	}

	use std::io;

	let stdin = io::stdin();
	let mut input = String::new();

	loop {
		match stdin.read_line(&mut input) {
			Ok(_) => {
				repl(&input[..]);
				input.clear();
			}
			Err(error) => {
				println!("Read line error: {}", error);
			}
		}
	}
}
