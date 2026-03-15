#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
FONT_REGULAR="$ROOT_DIR/assets/fonts/noto/NotoSansMono-Regular.ttf"
FONT_BOLD="$ROOT_DIR/assets/fonts/noto/NotoSansMono-Bold.ttf"
FONT_ITALIC="$ROOT_DIR/assets/fonts/noto/NotoSansMono-Regular.ttf"
FONT_BOLD_ITALIC="$ROOT_DIR/assets/fonts/noto/NotoSansMono-Bold.ttf"
FONT_SC="$ROOT_DIR/assets/fonts/noto/NotoSansSC-VF.ttf"
FONT_TC="$ROOT_DIR/assets/fonts/noto/NotoSansTC-VF.ttf"
FONT_JP="$ROOT_DIR/assets/fonts/noto/NotoSansJP-VF.ttf"
FONT_TAMIL="$ROOT_DIR/assets/fonts/noto/NotoSansTamil-Regular.ttf"
ATLAS_DIR="$ROOT_DIR/assets/atlas"

LANGS=(ja zh-CN zh-TW vi)

for lang in "${LANGS[@]}"; do
  REQUIRED_PATH="$ATLAS_DIR/required_chars.$lang.txt"
  ATLAS_PATH="$ATLAS_DIR/atlas.$lang.atlas"

  echo "[lang=$lang] [1/4] Extracting required characters..."
  cargo run --bin atlas_builder -- extract-required-lang --lang "$lang" --output "$REQUIRED_PATH"

  if [[ ! -s "$ATLAS_PATH" ]]; then
    echo "[lang=$lang] [2/4] Writing default atlas seed..."
    cargo run --bin atlas_builder -- write-default --output "$ATLAS_PATH"
  else
    echo "[lang=$lang] [2/4] Keeping existing atlas: $ATLAS_PATH"
  fi

  echo "[lang=$lang] [3/4] Building atlas glyph bitmaps..."
  if [[ -f "$FONT_REGULAR" && -f "$FONT_BOLD" ]]; then
    CELL_W=12
    CELL_H=18
    LANG_REG="$FONT_REGULAR"
    LANG_BOLD="$FONT_BOLD"
    LANG_ITALIC="$FONT_ITALIC"
    LANG_BOLD_ITALIC="$FONT_BOLD_ITALIC"
    EXTRA_FLAGS=()

    case "$lang" in
      ja)
        CELL_W=18
        CELL_H=18
        LANG_REG="$FONT_JP"
        LANG_BOLD="$FONT_JP"
        LANG_ITALIC="$FONT_JP"
        LANG_BOLD_ITALIC="$FONT_JP"
        EXTRA_FLAGS+=(--force-fullwidth-ascii)
        EXTRA_FLAGS+=(--use-pua-mapping)
        ;;
      zh-CN)
        CELL_W=18
        CELL_H=18
        LANG_REG="$FONT_SC"
        LANG_BOLD="$FONT_SC"
        LANG_ITALIC="$FONT_SC"
        LANG_BOLD_ITALIC="$FONT_SC"
        EXTRA_FLAGS+=(--force-fullwidth-ascii)
        EXTRA_FLAGS+=(--use-pua-mapping)
        ;;
      zh-TW)
        CELL_W=18
        CELL_H=18
        LANG_REG="$FONT_TC"
        LANG_BOLD="$FONT_TC"
        LANG_ITALIC="$FONT_TC"
        LANG_BOLD_ITALIC="$FONT_TC"
        EXTRA_FLAGS+=(--force-fullwidth-ascii)
        EXTRA_FLAGS+=(--use-pua-mapping)
        ;;
      ta)
        CELL_W=18
        CELL_H=18
        LANG_REG="$FONT_TAMIL"
        LANG_BOLD="$FONT_TAMIL"
        LANG_ITALIC="$FONT_TAMIL"
        LANG_BOLD_ITALIC="$FONT_TAMIL"
        EXTRA_FLAGS+=(--force-fullwidth-ascii)
        ;;
    esac

    cargo run --bin atlas_builder -- build-noto \
      --base "$ATLAS_PATH" \
      --required "$REQUIRED_PATH" \
      --output "$ATLAS_PATH" \
      --font-regular "$LANG_REG" \
      --font-bold "$LANG_BOLD" \
      --font-italic "$LANG_ITALIC" \
      --font-bold-italic "$LANG_BOLD_ITALIC" \
      --font-sc "$FONT_SC" \
      --font-tc "$FONT_TC" \
      --font-jp "$FONT_JP" \
      --font-ta "$FONT_TAMIL" \
      --cell-width "$CELL_W" \
      --cell-height "$CELL_H" \
      "${EXTRA_FLAGS[@]}"
  else
    echo "[lang=$lang] Noto font files not found; using fallback alias compose mode."
    cargo run --bin atlas_builder -- compose --base "$ATLAS_PATH" --required "$REQUIRED_PATH" --output "$ATLAS_PATH" --fallback-symbol "?"
  fi

  echo "[lang=$lang] [4/4] Verifying atlas coverage..."
  cargo run --bin atlas_builder -- verify --atlas "$ATLAS_PATH" --required "$REQUIRED_PATH" "${EXTRA_FLAGS[@]}"
done
echo "All per-language atlases generated successfully."
