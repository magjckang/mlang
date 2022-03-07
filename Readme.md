# A LISP like language

with a small Web Assembly GUI.

## Run

Prerequisites
- Up-to-date Rust toolchain
- Node.js if you want to use the built-in file server

Add `wasm32-unknown-unknown` target
```bash
rustup target add wasm32-unknown-unknown
```

Install `wasm-bindgen`
```bash
cargo install wasm-bindgen-cli
```
**NOTE:** Please still run the install command to update it if already installed before,
having the exact same version of local and global installation is crucial for
the build process.

Build and run the file server
```bash
make build
make run
```
**NOTE:** Set `PORT` env var to change the default 80 port the server is listening to.

Use the lastest web browser that support [top level await](1).

[1]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/await#top_level_await

## Todo

- [ ] Basic garbage collection
- [ ] Quote and quasiquote
- [ ] PEG like interpreter builder
- [ ] Self hosting

## References

- LISP 1.5 Programmer's Manual
- [Maru](https://www.piumarta.com/software/maru/)
- [Ohm](https://ohmjs.org)
