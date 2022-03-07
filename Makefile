TARGET := wasm32-unknown-unknown
NAME := mlang_web

check:
	@cargo check --release --target $(TARGET)
.PHONY: check

build:
	@cargo build --release --target $(TARGET)
	@wasm-bindgen --out-dir static --out-name main --target web --no-typescript target/$(TARGET)/release/$(NAME).wasm
.PHONY: build
