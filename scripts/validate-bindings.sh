#!/usr/bin/env bash
# Linux/WSL binding validation. Requires an active Python virtual environment.
set -euo pipefail
repo_root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
cd "$repo_root"
: "${VIRTUAL_ENV:?Activate a Python virtual environment containing maturin first}"
python_bin="$VIRTUAL_ENV/bin/python"
maturin_bin="$VIRTUAL_ENV/bin/maturin"
validation_dir=$(mktemp -d)
trap 'rm -rf "$validation_dir"' EXIT
cbindgen --config crates/gensolid-ffi/cbindgen.toml --crate gensolid-ffi \
    --output "$validation_dir/gensolid.h" crates/gensolid-ffi
cmp crates/gensolid-ffi/include/gensolid.h "$validation_dir/gensolid.h"
# C and Python both emit libgensolid.so; keep their build outputs independent.
# Cargo supplies the base target directory so an override remains supported.
target_dir=$(cargo metadata --no-deps --format-version 1 | "$python_bin" -c 'import json,sys; print(json.load(sys.stdin)["target_directory"])')
c_target_dir="$target_dir/binding-validation/c"
python_target_dir="$target_dir/binding-validation/python"
wasm_target_dir="$target_dir/binding-validation/wasm"
CARGO_TARGET_DIR="$c_target_dir" cargo build --locked -p gensolid-ffi
for sample in crates/gensolid-ffi/examples/block.c crates/gensolid-ffi/tests/through_hole.c; do
    "${CC:-cc}" -std=c11 -Wall -Wextra -Werror "$sample" \
        -I crates/gensolid-ffi/include -L "$c_target_dir/debug" \
        -Wl,-rpath,"$c_target_dir/debug" -lgensolid -o "$validation_dir/c-test"
    "$validation_dir/c-test"
done
CARGO_TARGET_DIR="$python_target_dir" "$maturin_bin" develop --manifest-path bindings/python/Cargo.toml
"$python_bin" -m unittest discover -s bindings/python/tests -v
CARGO_TARGET_DIR="$wasm_target_dir" cargo build --locked -p gensolid-wasm --target wasm32-unknown-unknown
wasm-bindgen --target nodejs --out-dir "$validation_dir/wasm" \
    "$wasm_target_dir/wasm32-unknown-unknown/debug/gensolid_wasm.wasm"
GENSOLID_WASM_MODULE="$validation_dir/wasm/gensolid_wasm.js" \
    node bindings/wasm/tests/through-hole.cjs
