TARGET=--target wasm32-unknown-unknown
PANIC=-C panic=abort
# 'link-args=-s' removes 10KB+ of debug symbols from linked core::*
OPT=-C opt-level=s -C link-args=-s
CRATE_TYPE=--crate-type=cdylib
DEBUG=-C debuginfo=0 -C debug-assertions=off
LINK_OPT=-C codegen-units=1 -C lto=fat
WASM_ARGS=$(TARGET) $(PANIC) $(OPT) $(CRATE_TYPE) $(DEBUG) $(LINK_OPT)
WASM_OUTPUT=wasm-demo/ime-engine.wasm

.PHONY: default
default:
	cargo build

.PHONY: run
run:
	cargo run --quiet

.PHONY: wasm
wasm:
	rustc $(WASM_ARGS) src/lib.rs -o $(WASM_OUTPUT)

# ime-engine IS NOT thread safe, so turn off parallel tests
# Point of --lib is to turn off messages about doc tests
.PHONY: test
test:
	cargo test --lib -- --test-threads 1

.PHONY: clean
clean:
	cargo clean
