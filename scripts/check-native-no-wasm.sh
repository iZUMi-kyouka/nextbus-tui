#!/usr/bin/env bash
set -euo pipefail

# Verifies wasm-only crates are absent from native target dependency graphs.
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

check_target() {
  local target="$1"
  local crate="$2"

  if cargo tree --target "$target" -p "$crate" 2>&1 | grep -q "nothing to print"; then
    return 0
  fi

  echo "[FAIL] '$crate' is present for native target '$target'"
  cargo tree --target "$target" -p "$crate" || true
  return 1
}

TARGETS=(
  "x86_64-unknown-linux-gnu"
  "x86_64-pc-windows-msvc"
  "x86_64-apple-darwin"
)

WASM_CRATES=(
  "ratzilla"
  "wasm-bindgen"
  "wasm-bindgen-futures"
  "web-sys"
  "js-sys"
)

for target in "${TARGETS[@]}"; do
  echo "Checking target: $target"
  for crate in "${WASM_CRATES[@]}"; do
    check_target "$target" "$crate"
  done
  echo "[OK] No wasm-only crates in native dependency graph for $target"
done

echo "All native target checks passed."


