# WebGL atlas assets

`atlas.<lang>.atlas` files are WebGL custom atlases loaded by the WASM runtime (`src/web_atlas.rs`).
English/Malay use ratzilla's built-in default atlas for stable Latin metrics.

## Workflow

1. Extract required characters used by app assets.
2. Build or replace `atlas.<lang>.atlas` with your custom atlas generator.
3. Build glyph bitmaps for each language atlas.
4. Verify per-language coverage against required characters.

Run:

```bash
bash scripts/download-noto-fonts.sh
bash scripts/atlas-workflow.sh
```

The helper writes custom atlases for CJK locales:

- `assets/atlas/atlas.ja.atlas`
- `assets/atlas/atlas.zh-CN.atlas`
- `assets/atlas/atlas.zh-TW.atlas`

and matching required-character files `required_chars.<lang>.txt`.

If Noto font files are present under `assets/fonts/noto/`, the workflow runs
`atlas_builder build-noto` to generate real glyph bitmaps for required symbols.

If those fonts are missing, it falls back to `compose`, which aliases symbols
to `?` so glyph lookup remains deterministic.



