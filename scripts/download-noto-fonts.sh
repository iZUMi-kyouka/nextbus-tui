#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
FONT_DIR="$ROOT_DIR/assets/fonts/noto"

mkdir -p "$FONT_DIR"

curl -fL -o "$FONT_DIR/NotoSansMono-Regular.ttf" \
  "https://raw.githubusercontent.com/notofonts/noto-fonts/main/hinted/ttf/NotoSansMono/NotoSansMono-Regular.ttf"

curl -fL -o "$FONT_DIR/NotoSansMono-Bold.ttf" \
  "https://raw.githubusercontent.com/notofonts/noto-fonts/main/hinted/ttf/NotoSansMono/NotoSansMono-Bold.ttf"

curl -fL -o "$FONT_DIR/NotoSansSC-Regular.ttf" \
  "https://raw.githubusercontent.com/notofonts/noto-cjk/main/Sans/OTF/SimplifiedChinese/NotoSansCJKsc-Regular.otf"

curl -fL -o "$FONT_DIR/NotoSansTC-Regular.ttf" \
  "https://raw.githubusercontent.com/notofonts/noto-cjk/main/Sans/OTF/TraditionalChineseHK/NotoSansCJKhk-Regular.otf"

curl -fL -o "$FONT_DIR/NotoSansJP-Regular.ttf" \
  "https://raw.githubusercontent.com/notofonts/noto-cjk/main/Sans/OTF/Japanese/NotoSansCJKjp-Regular.otf"

curl -fL -o "$FONT_DIR/NotoSansTamil-Regular.ttf" \
  "https://raw.githubusercontent.com/notofonts/noto-fonts/main/hinted/ttf/NotoSansTamil/NotoSansTamil-Regular.ttf"

echo "Downloaded Noto fonts into $FONT_DIR"

