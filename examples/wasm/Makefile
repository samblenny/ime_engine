WASM=target/wasm32-unknown-unknown/debug/wasm.wasm
DEST_DIR=../../wasm-demo
DEST_WASM=$(DEST_DIR)/ime-engine.wasm

# Build the WebAssembly file. This isn't the interesting part yet.
.PHONY: build
build:
	cargo build

# Now, copy the build .wasm file to the ../../wasm-demo webpage directory.
# Using make is necessary because cargo does not yet provide post-build copy
.PHONY: install
install: build
	cp $(WASM) $(DEST_WASM)

# webserver is a shortcut for:
# 1. Change to wasm-demo directory
# 2. Run local webserver ruby script to host wasm demo on localhost:8000
.PHONY: webserver
webserver: install
	cd $(DEST_DIR) && ruby webserver.rb

.PHONY: clean
clean:
	cargo clean
