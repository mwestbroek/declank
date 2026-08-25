#!/bin/bash

set -e

cd engine

cargo test

wasm-pack build --target no-modules
cp pkg/declank_engine.js ../extension/
{
  printf 'self.DECLANK_WASM_B64 = "'
  base64 < pkg/declank_engine_bg.wasm | tr -d '\n'
  printf '";\n'
} > ../extension/declank_wasm.js

cd ..
