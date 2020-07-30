TARGET=--target wasm32-unknown-unknown
PANIC=-C panic=abort
OPT=-C opt-level=s
CRATE_TYPE=--crate-type=cdylib
DEBUG=-C debuginfo=0 -C debug-assertions=off
LINK_OPT=-C codegen-units=1 -C lto=fat
WASM_ARGS=$(TARGET) $(PANIC) $(OPT) $(CRATE_TYPE) $(DEBUG) $(LINK_OPT)
WASM_OUTPUT=wasm-demo/ime-engine.wasm
RUST_SRC=src/lib.rs src/autogen_hsk.rs src/main.rs

default: $(RUST_SRC)
	cargo build

run: $(RUST_SRC)
	cargo run --quiet

wasm: $(RUST_SRC)
	rustc $(WASM_ARGS) src/lib.rs -o $(WASM_OUTPUT)

# Note: ime-engine IS NOT thread safe, so must not run parallel tests
test: $(RUST_SRC)
	cargo test -- --test-threads 1

.PHONY: clean
clean:
	cargo clean
