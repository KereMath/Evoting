#!/bin/bash
set -e

echo "🔨 Building PBC + GMP for WebAssembly"

# Directories
BUILD_DIR="$(pwd)/build"
DEPS_DIR="$(pwd)/deps"
INSTALL_DIR="$(pwd)/install"

mkdir -p "$BUILD_DIR" "$DEPS_DIR" "$INSTALL_DIR"

# Versions
GMP_VERSION="6.3.0"
PBC_VERSION="0.5.14"

# Check for emscripten
if ! command -v emcc &> /dev/null; then
    echo "❌ Emscripten not found. Please install emsdk first:"
    echo "   git clone https://github.com/emscripten-core/emsdk.git"
    echo "   cd emsdk && ./emsdk install latest && ./emsdk activate latest"
    echo "   source ./emsdk_env.sh"
    exit 1
fi

echo "✅ Emscripten found: $(emcc --version | head -n1)"

# Download GMP
echo "📦 Downloading GMP ${GMP_VERSION}..."
cd "$DEPS_DIR"
if [ ! -f "gmp-${GMP_VERSION}.tar.xz" ]; then
    curl -L "https://gmplib.org/download/gmp/gmp-${GMP_VERSION}.tar.xz" -o "gmp-${GMP_VERSION}.tar.xz"
fi

# Extract GMP
echo "📂 Extracting GMP..."
tar -xf "gmp-${GMP_VERSION}.tar.xz"

# Build GMP with Emscripten
echo "🔧 Building GMP with Emscripten..."
cd "gmp-${GMP_VERSION}"

# Configure for WASM
emconfigure ./configure \
    --prefix="$INSTALL_DIR" \
    --disable-assembly \
    --host=none \
    CFLAGS="-O3" \
    CXXFLAGS="-O3"

emmake make -j$(nproc 2>/dev/null || echo 4)
emmake make install

echo "✅ GMP built successfully"

# Download PBC
echo "📦 Downloading PBC ${PBC_VERSION}..."
cd "$DEPS_DIR"
if [ ! -f "pbc-${PBC_VERSION}.tar.gz" ]; then
    curl -L "https://crypto.stanford.edu/pbc/files/pbc-${PBC_VERSION}.tar.gz" -o "pbc-${PBC_VERSION}.tar.gz"
fi

# Extract PBC
echo "📂 Extracting PBC..."
tar -xzf "pbc-${PBC_VERSION}.tar.gz"

# Build PBC with Emscripten
echo "🔧 Building PBC with Emscripten..."
cd "pbc-${PBC_VERSION}"

# Configure for WASM
emconfigure ./configure \
    --prefix="$INSTALL_DIR" \
    --with-gmp="$INSTALL_DIR" \
    CFLAGS="-O3 -I${INSTALL_DIR}/include" \
    LDFLAGS="-L${INSTALL_DIR}/lib"

emmake make -j$(nproc 2>/dev/null || echo 4)
emmake make install

echo "✅ PBC built successfully"
echo ""
echo "✅ All dependencies built!"
echo "   Headers: ${INSTALL_DIR}/include"
echo "   Libraries: ${INSTALL_DIR}/lib"
