#!/bin/bash

set -e

cd engine

cargo test

wasm-pack build --target no-modules
cp pkg/deslop_engine.js ../extension/
{
  printf 'self.DESLOP_WASM_B64 = "'
  base64 < pkg/deslop_engine_bg.wasm | tr -d '\n'
  printf '";\n'
} > ../extension/deslop_wasm.js

cd ..
