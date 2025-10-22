# Makefile for building PrepareBlindSign WASM module

INSTALL_DIR = $(shell pwd)/install
WASM_OUTPUT = blindsign.js
WASM_FILE = blindsign.wasm

# Emscripten compiler
EMCC = emcc

# Compiler flags
CFLAGS = -O3 \
	-I$(INSTALL_DIR)/include \
	-I$(INSTALL_DIR)/include/pbc \
	-s WASM=1 \
	-s EXPORTED_RUNTIME_METHODS='["ccall","cwrap","UTF8ToString"]' \
	-s EXPORTED_FUNCTIONS='["_init_pairing","_prepare_blind_sign","_cleanup_pairing","_malloc","_free"]' \
	-s ALLOW_MEMORY_GROWTH=1 \
	-s MODULARIZE=1 \
	-s EXPORT_NAME='createBlindSignModule' \
	-s ENVIRONMENT='web' \
	-s ASSERTIONS=1 \
	--no-entry

# Libraries
LDFLAGS = -L$(INSTALL_DIR)/lib -lpbc -lgmp -lcrypto

.PHONY: all clean deps

all: $(WASM_OUTPUT)

# Build dependencies first
deps:
	@echo "🔨 Building dependencies (GMP + PBC)..."
	@chmod +x build.sh
	@bash build.sh

# Compile WASM module
$(WASM_OUTPUT): blindsign_wasm.c
	@echo "🔧 Compiling PrepareBlindSign to WASM..."
	$(EMCC) blindsign_wasm.c $(CFLAGS) $(LDFLAGS) -o $(WASM_OUTPUT)
	@echo "✅ WASM module built: $(WASM_OUTPUT) and $(WASM_FILE)"

clean:
	rm -rf $(WASM_OUTPUT) $(WASM_FILE) build/ deps/ install/

test:
	@echo "🧪 Testing WASM module..."
	node test.js
