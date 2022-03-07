use mlang::{Reader, ParseError};
use core::cell::RefCell;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen(start)]
pub fn main() {
	mlang::init();
}

thread_local! {
	static READ: RefCell<String> = Default::default();
	static EVAL: RefCell<String> = Default::default();
	static PARSE_ERR: RefCell<String> = Default::default();
	static EVAL_ERR: RefCell<String> = Default::default();
}

#[wasm_bindgen]
pub fn read_and_eval(s: &str, sugar_syntax: bool) {
	let mut reader: Box<dyn Reader> = if sugar_syntax {
		Box::new(mlang::SugarReader::new(s))
	} else {
		Box::new(mlang::BaseReader::new(s))
	};
	READ.with(|x| x.borrow_mut().clear());
	EVAL.with(|x| x.borrow_mut().clear());
	PARSE_ERR.with(|x| x.borrow_mut().clear());
	EVAL_ERR.with(|x| x.borrow_mut().clear());
	loop {
		match reader.read() {
			Ok(op) => {
				READ.with(|x| x.borrow_mut().push_str(&format!("{op}\n")));
				match mlang::eval(op) {
					Ok(op) => {
						EVAL.with(|x| x.borrow_mut().push_str(&format!("{op}\n")));
					}
					Err(e) => {
						EVAL_ERR.with(|x| x.borrow_mut().push_str(&format!("{:?}", e)));
						break
					}
				}
			}
			Err(ParseError::Eof) => break,
			Err(e) => {
				PARSE_ERR.with(|x| x.borrow_mut().push_str(&format!("{:?}", e)));
				break
			}
		}
	}
}

#[wasm_bindgen(js_name = getParseResult)]
pub fn get_parse_result() -> String {
	READ.with(|x| x.borrow().clone())
}

#[wasm_bindgen(js_name = getEvalResult)]
pub fn get_eval_result() -> String {
	EVAL.with(|x| x.borrow().clone())
}

#[wasm_bindgen(js_name = getParseError)]
pub fn get_parse_error() -> String {
	PARSE_ERR.with(|x| x.borrow().clone())
}

#[wasm_bindgen(js_name = getEvalError)]
pub fn get_eval_error() -> String {
	EVAL_ERR.with(|x| x.borrow().clone())
}
