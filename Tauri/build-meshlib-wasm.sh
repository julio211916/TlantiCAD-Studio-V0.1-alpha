#!/bin/bash
# Build MeshLib for WebAssembly (Emscripten)
# Usage: ./build-meshlib-wasm.sh
# Requires: emscripten, cmake, ninja

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MESHLIB_DIR="${SCRIPT_DIR}/MeshLib"
BUILD_DIR="${MESHLIB_DIR}/build-wasm"
OUTPUT_DIR="${SCRIPT_DIR}/../packages/meshlib-wasm"

# Verify emscripten
if ! command -v emcc &>/dev/null; then
  echo "ERROR: emscripten (emcc) not found. Install via: brew install emscripten"
  exit 1
fi

echo "MeshLib source: ${MESHLIB_DIR}"

# Step 1: Install macOS requirements
echo "=== Step 1: Checking macOS dependencies ==="
for pkg in boost cmake eigen fmt glfw jsoncpp tbb spdlog tl-expected ninja binaryen; do
  if ! brew list "$pkg" &>/dev/null; then
    echo "Installing ${pkg}..."
    brew install "$pkg" 2>/dev/null || true
  fi
done

# Step 2: Build thirdparty (required before source build)
echo "=== Step 2: Building thirdparty ==="
cd "${MESHLIB_DIR}"
if [ ! -d "lib" ] && [ -f "scripts/build_thirdparty.sh" ]; then
  export MR_EMSCRIPTEN="ON"
  export MR_EMSCRIPTEN_SINGLETHREAD=1
  bash scripts/build_thirdparty.sh 2>&1 | tee "${MESHLIB_DIR}/thirdparty-build.log"
fi

# Step 3: Configure with CMake for Emscripten
echo "=== Step 3: Configuring CMake for WASM ==="
mkdir -p "${BUILD_DIR}"
cd "${BUILD_DIR}"

emcmake cmake "${MESHLIB_DIR}" \
  -G "Unix Makefiles" \
  -DCMAKE_BUILD_TYPE=Release \
  -DMR_EMSCRIPTEN=1 \
  -DMR_EMSCRIPTEN_SINGLETHREAD=1 \
  2>&1 | tee "${BUILD_DIR}/cmake-config.log"

# Step 4: Build
echo "=== Step 4: Building MeshLib WASM ==="
cmake --build . --parallel "$(sysctl -n hw.ncpu)" 2>&1 | tee "${BUILD_DIR}/build.log"

# Step 5: Copy output to packages
echo "=== Step 5: Copying output ==="
mkdir -p "${OUTPUT_DIR}/dist"
find "${BUILD_DIR}/bin" -name "*.wasm" -o -name "*.js" | while read -r f; do
  cp "$f" "${OUTPUT_DIR}/dist/"
done

echo "=== MeshLib WASM build complete! ==="
echo "Output: ${OUTPUT_DIR}/dist/"
