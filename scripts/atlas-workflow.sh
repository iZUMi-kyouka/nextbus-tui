#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
ATLAS_PATH="$ROOT_DIR/assets/atlas/multilang.atlas"
REQUIRED_PATH="$ROOT_DIR/assets/atlas/required_chars.txt"
FONT_REGULAR="$ROOT_DIR/assets/fonts/noto/NotoSansMono-Regular.ttf"
FONT_BOLD="$ROOT_DIR/assets/fonts/noto/NotoSansMono-Bold.ttf"
FONT_ITALIC="$ROOT_DIR/assets/fonts/noto/NotoSansMono-Regular.ttf"
FONT_BOLD_ITALIC="$ROOT_DIR/assets/fonts/noto/NotoSansMono-Bold.ttf"
FONT_SC="$ROOT_DIR/assets/fonts/noto/NotoSansSC-Regular.ttf"
FONT_TC="$ROOT_DIR/assets/fonts/noto/NotoSansTC-Regular.ttf"
FONT_JP="$ROOT_DIR/assets/fonts/noto/NotoSansJP-Regular.ttf"
FONT_TAMIL="$ROOT_DIR/assets/fonts/noto/NotoSansTamil-Regular.ttf"

echo "[1/4] Extracting required characters from assets..."
cargo run --bin atlas_builder -- extract-required --output "$REQUIRED_PATH"

if [[ ! -s "$ATLAS_PATH" ]]; then
  echo "[2/4] Writing current default beamterm atlas as a starting file..."
  cargo run --bin atlas_builder -- write-default --output "$ATLAS_PATH"
else
  echo "[2/4] Keeping existing atlas: $ATLAS_PATH"
fi

echo "[3/4] Composing required symbols into atlas via fallback aliases..."
if [[ -f "$FONT_REGULAR" && -f "$FONT_BOLD" ]]; then
  echo "[3/4] Building real glyph atlas from Noto fonts..."
  cargo run --bin atlas_builder -- build-noto \
    --base "$ATLAS_PATH" \
    --required "$REQUIRED_PATH" \
    --output "$ATLAS_PATH" \
    --font-regular "$FONT_REGULAR" \
    --font-bold "$FONT_BOLD" \
    --font-italic "$FONT_ITALIC" \
    --font-bold-italic "$FONT_BOLD_ITALIC" \
    --font-sc "$FONT_SC" \
    --font-tc "$FONT_TC" \
    --font-jp "$FONT_JP" \
    --font-ta "$FONT_TAMIL"
else
  echo "[3/4] Noto font files not found; using fallback alias compose mode."
  cargo run --bin atlas_builder -- compose --base "$ATLAS_PATH" --required "$REQUIRED_PATH" --output "$ATLAS_PATH" --fallback-symbol "?"
fi

echo "[4/4] Verifying atlas coverage against required characters..."
if cargo run --bin atlas_builder -- verify --atlas "$ATLAS_PATH" --required "$REQUIRED_PATH"; then
  echo "Atlas covers current required non-ASCII characters."
else
  echo "Atlas does not fully cover required non-ASCII characters."
  echo "Replace $ATLAS_PATH with a custom generated atlas, then rerun this script."
  exit 1
fi






