# WebGL atlas assets

`multilang.atlas` is the WebGL font atlas loaded by the WASM runtime (`src/web_atlas.rs`).

## Workflow

1. Extract required characters used by app assets.
2. Build or replace `multilang.atlas` with your custom atlas generator.
3. Compose missing symbols into the atlas using fallback glyph aliases.
4. Verify coverage against required characters.

Run:

```bash
bash scripts/download-noto-fonts.sh
bash scripts/atlas-workflow.sh
```

The helper currently writes the default beamterm atlas as a starting point. Replace
`assets/atlas/multilang.atlas` with your custom atlas when available, then rerun
verification.

If Noto font files are present under `assets/fonts/noto/`, the workflow runs
`atlas_builder build-noto` to generate real glyph bitmaps for required symbols.

If those fonts are missing, it falls back to `compose`, which aliases symbols
to `?` so glyph lookup remains deterministic.



