use std::collections::{BTreeSet, HashSet};
use std::env;
use std::fs;
use std::io;
use std::path::Path;

use beamterm_data::{FontAtlasData, FontStyle, Glyph};
use fontdue::{Font, FontSettings, Metrics};

const DEFAULT_FONT_REGULAR: &str = "assets/fonts/noto/NotoSansMono-Regular.ttf";
const DEFAULT_FONT_BOLD: &str = "assets/fonts/noto/NotoSansMono-Bold.ttf";
const DEFAULT_FONT_ITALIC: &str = "assets/fonts/noto/NotoSansMono-Regular.ttf";
const DEFAULT_FONT_BOLD_ITALIC: &str = "assets/fonts/noto/NotoSansMono-Bold.ttf";
const DEFAULT_FONT_SC: &str = "assets/fonts/noto/NotoSansSC-Regular.ttf";
const DEFAULT_FONT_TC: &str = "assets/fonts/noto/NotoSansTC-Regular.ttf";
const DEFAULT_FONT_JP: &str = "assets/fonts/noto/NotoSansJP-Regular.ttf";
const DEFAULT_FONT_TAMIL: &str = "assets/fonts/noto/NotoSansTamil-Regular.ttf";

fn main() {
    if let Err(e) = run() {
        eprintln!("atlas_builder: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let Some(cmd) = args.next() else {
        print_usage();
        return Err("missing subcommand".into());
    };

    match cmd.as_str() {
        "extract-required" => {
            let output = parse_flag_value(args.collect(), "--output")?;
            extract_required_chars(Path::new(&output)).map_err(|e| e.to_string())
        }
        "verify" => {
            let all = args.collect::<Vec<_>>();
            let atlas = parse_flag_value(all.clone(), "--atlas")?;
            let required = parse_flag_value(all, "--required")?;
            verify_required(Path::new(&atlas), Path::new(&required)).map_err(|e| e.to_string())
        }
        "compose" => {
            let all = args.collect::<Vec<_>>();
            let base = parse_flag_value(all.clone(), "--base")?;
            let required = parse_flag_value(all.clone(), "--required")?;
            let output = parse_flag_value(all.clone(), "--output")?;
            let fallback_symbol = parse_flag_value_or_default(all, "--fallback-symbol", "?");
            compose_atlas(
                Path::new(&base),
                Path::new(&required),
                Path::new(&output),
                &fallback_symbol,
            )
            .map_err(|e| e.to_string())
        }
        "build-noto" => {
            let all = args.collect::<Vec<_>>();
            let base = parse_flag_value(all.clone(), "--base")?;
            let required = parse_flag_value(all.clone(), "--required")?;
            let output = parse_flag_value(all.clone(), "--output")?;
            let regular =
                parse_flag_value_or_default(all.clone(), "--font-regular", DEFAULT_FONT_REGULAR);
            let bold = parse_flag_value_or_default(all.clone(), "--font-bold", DEFAULT_FONT_BOLD);
            let italic =
                parse_flag_value_or_default(all.clone(), "--font-italic", DEFAULT_FONT_ITALIC);
            let bold_italic = parse_flag_value_or_default(
                all.clone(),
                "--font-bold-italic",
                DEFAULT_FONT_BOLD_ITALIC,
            );
            let fallback_sc =
                parse_flag_value_or_default(all.clone(), "--font-sc", DEFAULT_FONT_SC);
            let fallback_tc =
                parse_flag_value_or_default(all.clone(), "--font-tc", DEFAULT_FONT_TC);
            let fallback_jp =
                parse_flag_value_or_default(all.clone(), "--font-jp", DEFAULT_FONT_JP);
            let fallback_ta = parse_flag_value_or_default(all, "--font-ta", DEFAULT_FONT_TAMIL);

            let fonts = FontSet::load(
                &regular,
                &bold,
                &italic,
                &bold_italic,
                &[&fallback_sc, &fallback_tc, &fallback_jp, &fallback_ta],
            )?;
            build_noto_atlas(
                Path::new(&base),
                Path::new(&required),
                Path::new(&output),
                &fonts,
            )
            .map_err(|e| e.to_string())
        }
        "write-default" => {
            let output = parse_flag_value(args.collect(), "--output")?;
            write_default_atlas(Path::new(&output)).map_err(|e| e.to_string())
        }
        _ => {
            print_usage();
            Err(format!("unknown subcommand: {cmd}"))
        }
    }
}

fn parse_flag_value(args: Vec<String>, flag: &str) -> Result<String, String> {
    let mut i = 0usize;
    while i + 1 < args.len() {
        if args[i] == flag {
            return Ok(args[i + 1].clone());
        }
        i += 1;
    }
    Err(format!("missing required flag {flag}"))
}

fn parse_flag_value_or_default(args: Vec<String>, flag: &str, default: &str) -> String {
    parse_flag_value(args, flag).unwrap_or_else(|_| default.to_string())
}

fn print_usage() {
    eprintln!(
        "Usage:\n  atlas_builder extract-required --output <path>\n  atlas_builder write-default --output <path>\n  atlas_builder build-noto --base <path> --required <path> --output <path> [--font-regular <ttf>] [--font-bold <ttf>] [--font-italic <ttf>] [--font-bold-italic <ttf>] [--font-sc <otf/ttf>] [--font-tc <otf/ttf>] [--font-jp <otf/ttf>] [--font-ta <ttf>]\n  atlas_builder compose --base <path> --required <path> --output <path> [--fallback-symbol ?]\n  atlas_builder verify --atlas <path> --required <path>"
    );
}

struct FontSet {
    regular: Font,
    bold: Font,
    italic: Font,
    bold_italic: Font,
    fallbacks: Vec<Font>,
}

impl FontSet {
    fn load(
        regular: &str,
        bold: &str,
        italic: &str,
        bold_italic: &str,
        fallback_paths: &[&str],
    ) -> Result<Self, String> {
        let mut fallbacks = Vec::new();
        for path in fallback_paths {
            if Path::new(path).exists() {
                fallbacks.push(load_font_file(path)?);
            }
        }

        Ok(Self {
            regular: load_font_file(regular)?,
            bold: load_font_file(bold)?,
            italic: load_font_file(italic)?,
            bold_italic: load_font_file(bold_italic)?,
            fallbacks,
        })
    }

    fn for_style_char(&self, style: FontStyle, c: char) -> &Font {
        let primary = match style {
            FontStyle::Normal => &self.regular,
            FontStyle::Bold => &self.bold,
            FontStyle::Italic => &self.italic,
            FontStyle::BoldItalic => &self.bold_italic,
        };

        if primary.lookup_glyph_index(c) > 0 {
            return primary;
        }
        for fallback in &self.fallbacks {
            if fallback.lookup_glyph_index(c) > 0 {
                return fallback;
            }
        }
        primary
    }
}

fn load_font_file(path: &str) -> Result<Font, String> {
    let bytes = fs::read(path).map_err(|e| format!("failed to read font '{path}': {e}"))?;
    Font::from_bytes(bytes, FontSettings::default())
        .map_err(|e| format!("failed to parse font '{path}': {e}"))
}

fn load_required_chars(path: &Path) -> io::Result<BTreeSet<char>> {
    let text = fs::read_to_string(path)?;
    Ok(text.chars().filter(|c| is_required_char(*c)).collect())
}

fn build_noto_atlas(
    base_atlas_path: &Path,
    required_path: &Path,
    output_path: &Path,
    fonts: &FontSet,
) -> io::Result<()> {
    let bytes = fs::read(base_atlas_path)?;
    let mut atlas = FontAtlasData::from_binary(&bytes)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.message))?;

    let required = load_required_chars(required_path)?;
    let non_ascii_required: Vec<char> = required.into_iter().filter(|c| !c.is_ascii()).collect();

    atlas.glyphs.retain(|g| {
        if g.is_emoji {
            return true;
        }
        let Some(c) = g.symbol.chars().next() else {
            return false;
        };
        c.is_ascii()
    });

    let mut used_base_ids: HashSet<u16> = atlas
        .glyphs
        .iter()
        .filter(|g| !g.is_emoji)
        .map(|g| g.id & Glyph::GLYPH_ID_MASK)
        .collect();

    let mut next_id = 0u16;
    let mut generated = 0usize;
    for c in non_ascii_required {
        let Some(base_id) = next_free_base_id(&mut next_id, &used_base_ids) else {
            return Err(io::Error::other(
                "no free base glyph IDs remain (limit reached: 1024) while building custom atlas",
            ));
        };
        used_base_ids.insert(base_id);

        for style in FontStyle::ALL {
            let full_id = with_style(base_id, style);
            let glyph = Glyph {
                id: full_id,
                style,
                symbol: c.to_string().into(),
                pixel_coords: glyph_pixel_coords(full_id, atlas.cell_size),
                is_emoji: false,
            };
            render_glyph_into_atlas(&mut atlas, fonts.for_style_char(style, c), c, full_id)?;
            atlas.glyphs.push(glyph);
        }

        generated += 1;
    }

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(output_path, atlas.to_binary())?;

    println!(
        "built noto atlas at {} with {} generated non-ASCII glyph symbols",
        output_path.display(),
        generated
    );
    Ok(())
}

fn next_free_base_id(cursor: &mut u16, used: &HashSet<u16>) -> Option<u16> {
    while *cursor <= Glyph::GLYPH_ID_MASK {
        let id = *cursor;
        *cursor = cursor.saturating_add(1);
        if used.contains(&id) {
            continue;
        }
        if (0x20..0x80).contains(&id) {
            continue;
        }
        return Some(id);
    }
    None
}

fn with_style(base_id: u16, style: FontStyle) -> u16 {
    base_id | style.style_mask()
}

fn glyph_pixel_coords(id: u16, cell_size: (i32, i32)) -> (i32, i32) {
    let col = (id as i32) % FontAtlasData::CELLS_PER_SLICE;
    (
        col * cell_size.0 + FontAtlasData::PADDING,
        FontAtlasData::PADDING,
    )
}

fn render_glyph_into_atlas(
    atlas: &mut FontAtlasData,
    font: &Font,
    c: char,
    glyph_id: u16,
) -> io::Result<()> {
    let (cell_w, cell_h) = atlas.cell_size;
    let inner_w = (cell_w - 2 * FontAtlasData::PADDING).max(1) as usize;
    let inner_h = (cell_h - 2 * FontAtlasData::PADDING).max(1) as usize;

    let (metrics, bitmap) = rasterize_fit(font, c, inner_w, inner_h);

    clear_cell_rgba(atlas, glyph_id)?;
    if metrics.width == 0 || metrics.height == 0 {
        return Ok(());
    }

    let col = (glyph_id as usize) % FontAtlasData::CELLS_PER_SLICE as usize;
    let layer = (glyph_id as usize) / FontAtlasData::CELLS_PER_SLICE as usize;

    let tex_w = atlas.texture_dimensions.0 as usize;
    let tex_h = atlas.texture_dimensions.1 as usize;
    let tex_d = atlas.texture_dimensions.2 as usize;

    if layer >= tex_d {
        return Err(io::Error::other(format!(
            "atlas texture has {tex_d} layers, cannot place glyph id {glyph_id}"
        )));
    }

    let cell_x = col * cell_w as usize;
    let cell_y = 0usize;
    let pad = FontAtlasData::PADDING as usize;

    let draw_x = cell_x + pad + ((inner_w.saturating_sub(metrics.width)) / 2);
    let draw_y = cell_y + pad + ((inner_h.saturating_sub(metrics.height)) / 2);

    for y in 0..metrics.height {
        for x in 0..metrics.width {
            let alpha = bitmap[y * metrics.width + x];
            if alpha == 0 {
                continue;
            }
            let px = draw_x + x;
            let py = draw_y + y;
            if px >= tex_w || py >= tex_h {
                continue;
            }
            let idx = (((layer * tex_h + py) * tex_w + px) * 4) as usize;
            atlas.texture_data[idx] = 255;
            atlas.texture_data[idx + 1] = 255;
            atlas.texture_data[idx + 2] = 255;
            atlas.texture_data[idx + 3] = alpha;
        }
    }
    Ok(())
}

fn clear_cell_rgba(atlas: &mut FontAtlasData, glyph_id: u16) -> io::Result<()> {
    let tex_w = atlas.texture_dimensions.0 as usize;
    let tex_h = atlas.texture_dimensions.1 as usize;
    let tex_d = atlas.texture_dimensions.2 as usize;
    let cell_w = atlas.cell_size.0 as usize;
    let cell_h = atlas.cell_size.1 as usize;

    let col = (glyph_id as usize) % FontAtlasData::CELLS_PER_SLICE as usize;
    let layer = (glyph_id as usize) / FontAtlasData::CELLS_PER_SLICE as usize;
    if layer >= tex_d {
        return Err(io::Error::other(format!(
            "atlas texture has {tex_d} layers, cannot clear glyph id {glyph_id}"
        )));
    }

    let x0 = col * cell_w;
    let y0 = 0usize;

    for y in 0..cell_h {
        for x in 0..cell_w {
            let px = x0 + x;
            let py = y0 + y;
            if px >= tex_w || py >= tex_h {
                continue;
            }
            let idx = (((layer * tex_h + py) * tex_w + px) * 4) as usize;
            atlas.texture_data[idx] = 0;
            atlas.texture_data[idx + 1] = 0;
            atlas.texture_data[idx + 2] = 0;
            atlas.texture_data[idx + 3] = 0;
        }
    }
    Ok(())
}

fn rasterize_fit(font: &Font, c: char, max_w: usize, max_h: usize) -> (Metrics, Vec<u8>) {
    let mut px = max_h as f32;
    while px >= 8.0 {
        let (metrics, bitmap) = font.rasterize(c, px);
        if metrics.width <= max_w && metrics.height <= max_h {
            return (metrics, bitmap);
        }
        px -= 1.0;
    }
    font.rasterize(c, 8.0)
}

fn compose_atlas(
    base_atlas_path: &Path,
    required_path: &Path,
    output_path: &Path,
    fallback_symbol: &str,
) -> io::Result<()> {
    let bytes = fs::read(base_atlas_path)?;
    let mut atlas = FontAtlasData::from_binary(&bytes)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.message))?;

    let required = load_required_chars(required_path)?;
    let mut existing_symbols: BTreeSet<char> =
        atlas.glyphs.iter().flat_map(|g| g.symbol.chars()).collect();

    let fallback_glyphs = fallback_glyphs_by_style(&atlas, fallback_symbol)?;

    let mut inserted = 0usize;
    for c in required {
        if existing_symbols.contains(&c) {
            continue;
        }
        if c.is_ascii() {
            continue;
        }

        for style in FontStyle::ALL {
            let Some(base) = fallback_glyphs.iter().find(|g| g.style == style) else {
                continue;
            };
            atlas.glyphs.push(Glyph {
                id: base.id,
                style,
                symbol: c.to_string().into(),
                pixel_coords: base.pixel_coords,
                is_emoji: false,
            });
        }
        existing_symbols.insert(c);
        inserted += 1;
    }

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(output_path, atlas.to_binary())?;

    println!(
        "composed atlas written to {} (added {} non-ASCII symbols via fallback '{}')",
        output_path.display(),
        inserted,
        fallback_symbol
    );
    Ok(())
}

fn fallback_glyphs_by_style(atlas: &FontAtlasData, symbol: &str) -> io::Result<Vec<Glyph>> {
    let glyphs: Vec<Glyph> = atlas
        .glyphs
        .iter()
        .filter(|g| g.symbol.as_str() == symbol)
        .map(|g| Glyph {
            id: g.id,
            style: g.style,
            symbol: g.symbol.clone(),
            pixel_coords: g.pixel_coords,
            is_emoji: g.is_emoji,
        })
        .collect();

    if glyphs.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "fallback symbol '{}' does not exist in the base atlas",
                symbol
            ),
        ));
    }
    Ok(glyphs)
}

fn extract_required_chars(output: &Path) -> io::Result<()> {
    let mut chars = BTreeSet::new();

    collect_chars_from_tree(Path::new("assets/i18n"), &["ftl"], &mut chars)?;
    collect_chars_from_tree(Path::new("assets"), &["toml"], &mut chars)?;

    let required: String = chars.into_iter().collect();
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(output, required.as_bytes())?;

    println!("wrote required character set to {}", output.display());
    Ok(())
}

fn collect_chars_from_tree(root: &Path, exts: &[&str], out: &mut BTreeSet<char>) -> io::Result<()> {
    if !root.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_chars_from_tree(&path, exts, out)?;
            continue;
        }
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or_default();
        if !exts.contains(&ext) {
            continue;
        }
        let text = fs::read_to_string(&path)?;
        for c in text.chars() {
            if is_required_char(c) {
                out.insert(c);
            }
        }
    }
    Ok(())
}

fn is_required_char(c: char) -> bool {
    !c.is_control() && !c.is_whitespace()
}

fn verify_required(atlas_path: &Path, required_path: &Path) -> io::Result<()> {
    let atlas_bytes = fs::read(atlas_path)?;
    let atlas = FontAtlasData::from_binary(&atlas_bytes)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.message))?;

    let required = fs::read_to_string(required_path)?;
    let glyph_set: BTreeSet<char> = atlas.glyphs.iter().flat_map(|g| g.symbol.chars()).collect();

    let missing: Vec<char> = required
        .chars()
        .filter(|c| !c.is_ascii() && !glyph_set.contains(c))
        .collect();

    println!("atlas font: {}", atlas.font_name);
    println!("atlas glyphs: {}", atlas.glyphs.len());
    println!("required chars: {}", required.chars().count());

    if missing.is_empty() {
        println!("coverage check passed: no missing non-ASCII chars");
        return Ok(());
    }

    println!("missing non-ASCII chars: {}", missing.len());
    for c in missing.into_iter().take(200) {
        println!("  U+{:04X} {}", c as u32, c);
    }

    Err(io::Error::other("coverage check failed"))
}

fn write_default_atlas(output: &Path) -> io::Result<()> {
    let atlas = FontAtlasData::default();
    let bytes = atlas.to_binary();

    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(output, bytes)?;

    println!("wrote default beamterm atlas to {}", output.display());
    Ok(())
}
