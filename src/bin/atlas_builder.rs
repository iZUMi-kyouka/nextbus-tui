use std::collections::BTreeSet;
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
const DEFAULT_FONT_SC: &str = "assets/fonts/noto/NotoSansSC-VF.ttf";
const DEFAULT_FONT_TC: &str = "assets/fonts/noto/NotoSansTC-VF.ttf";
const DEFAULT_FONT_JP: &str = "assets/fonts/noto/NotoSansJP-VF.ttf";
const DEFAULT_FONT_TAMIL: &str = "assets/fonts/noto/NotoSansTamil-Regular.ttf";
const DEFAULT_CELL_WIDTH: i32 = 16;
const DEFAULT_CELL_HEIGHT: i32 = 28;
const DEFAULT_ATLAS_LAYERS: i32 = 128;
const ALWAYS_INCLUDE_SYMBOLS: &str = "─│┌┐└┘├┤┬┴┼↑↓←→↵⌫";

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
        "extract-required-lang" => {
            let all = args.collect::<Vec<_>>();
            let lang = parse_flag_value(all.clone(), "--lang")?;
            let output = parse_flag_value(all, "--output")?;
            extract_required_chars_for_lang(&lang, Path::new(&output)).map_err(|e| e.to_string())
        }
        "verify" => {
            let all = args.collect::<Vec<_>>();
            let atlas = parse_flag_value(all.clone(), "--atlas")?;
            let required = parse_flag_value(all.clone(), "--required")?;
            let use_pua_mapping = has_flag(&all, "--use-pua-mapping");
            verify_required(Path::new(&atlas), Path::new(&required), use_pua_mapping)
                .map_err(|e| e.to_string())
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
            let force_fullwidth_ascii = has_flag(&all, "--force-fullwidth-ascii");
            let use_pua_mapping = has_flag(&all, "--use-pua-mapping");
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
            let fallback_ta =
                parse_flag_value_or_default(all.clone(), "--font-ta", DEFAULT_FONT_TAMIL);
            let cell_width =
                parse_flag_i32_or_default(all.clone(), "--cell-width", DEFAULT_CELL_WIDTH);
            let cell_height = parse_flag_i32_or_default(all, "--cell-height", DEFAULT_CELL_HEIGHT);

            let fonts = FontSet::load(
                &regular,
                &bold,
                &italic,
                &bold_italic,
                &fallback_sc,
                &fallback_tc,
                &fallback_jp,
                &fallback_ta,
            )?;
            build_noto_atlas(
                Path::new(&base),
                Path::new(&required),
                Path::new(&output),
                &fonts,
                (cell_width, cell_height),
                force_fullwidth_ascii,
                use_pua_mapping,
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

fn parse_flag_i32_or_default(args: Vec<String>, flag: &str, default: i32) -> i32 {
    parse_flag_value(args, flag)
        .ok()
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(default)
}

fn print_usage() {
    eprintln!(
        "Usage:\n  atlas_builder extract-required --output <path>\n  atlas_builder extract-required-lang --lang <code> --output <path>\n  atlas_builder write-default --output <path>\n  atlas_builder build-noto --base <path> --required <path> --output <path> [--font-regular <ttf>] [--font-bold <ttf>] [--font-italic <ttf>] [--font-bold-italic <ttf>] [--font-sc <otf/ttf>] [--font-tc <otf/ttf>] [--font-jp <otf/ttf>] [--font-ta <ttf>] [--cell-width <px>] [--cell-height <px>] [--force-fullwidth-ascii] [--use-pua-mapping]\n  atlas_builder compose --base <path> --required <path> --output <path> [--fallback-symbol ?]\n  atlas_builder verify --atlas <path> --required <path> [--use-pua-mapping]"
    );
}

struct FontSet {
    regular: Font,
    bold: Font,
    italic: Font,
    bold_italic: Font,
    sc: Option<Font>,
    tc: Option<Font>,
    jp: Option<Font>,
    ta: Option<Font>,
}

impl FontSet {
    #[allow(clippy::too_many_arguments)]
    fn load(
        regular: &str,
        bold: &str,
        italic: &str,
        bold_italic: &str,
        sc: &str,
        tc: &str,
        jp: &str,
        ta: &str,
    ) -> Result<Self, String> {
        Ok(Self {
            regular: load_font_file(regular)?,
            bold: load_font_file(bold)?,
            italic: load_font_file(italic)?,
            bold_italic: load_font_file(bold_italic)?,
            sc: load_font_optional(sc)?,
            tc: load_font_optional(tc)?,
            jp: load_font_optional(jp)?,
            ta: load_font_optional(ta)?,
        })
    }

    fn for_style_char(&self, style: FontStyle, c: char) -> &Font {
        let primary = match style {
            FontStyle::Normal => &self.regular,
            FontStyle::Bold => &self.bold,
            FontStyle::Italic => &self.italic,
            FontStyle::BoldItalic => &self.bold_italic,
        };

        if c.is_ascii() {
            return primary;
        }

        if is_tamil(c) {
            if let Some(font) = self.ta.as_ref().filter(|f| has_glyph(f, c)) {
                return font;
            }
        }

        if is_japanese_script(c) {
            if let Some(font) = self.jp.as_ref().filter(|f| has_glyph(f, c)) {
                return font;
            }
        }

        if is_cjk_ideograph(c) {
            if let Some(font) = self.jp.as_ref().filter(|f| has_glyph(f, c)) {
                return font;
            }
            if let Some(font) = self.sc.as_ref().filter(|f| has_glyph(f, c)) {
                return font;
            }
            if let Some(font) = self.tc.as_ref().filter(|f| has_glyph(f, c)) {
                return font;
            }
        }

        if has_glyph(primary, c) {
            return primary;
        }

        for fallback in [&self.jp, &self.sc, &self.tc, &self.ta] {
            if let Some(font) = fallback.as_ref().filter(|f| has_glyph(f, c)) {
                return font;
            }
        }

        primary
    }
}

fn load_font_optional(path: &str) -> Result<Option<Font>, String> {
    if !Path::new(path).exists() {
        return Ok(None);
    }
    Ok(Some(load_font_file(path)?))
}

fn has_glyph(font: &Font, c: char) -> bool {
    font.lookup_glyph_index(c) != 0
}

fn is_tamil(c: char) -> bool {
    matches!(c as u32, 0x0B80..=0x0BFF)
}

fn is_japanese_script(c: char) -> bool {
    matches!(
        c as u32,
        0x3040..=0x309F // Hiragana
        | 0x30A0..=0x30FF // Katakana
        | 0x31F0..=0x31FF // Katakana Phonetic Extensions
        | 0xFF65..=0xFF9F // Halfwidth Katakana
    )
}

fn is_cjk_ideograph(c: char) -> bool {
    matches!(
        c as u32,
        0x3400..=0x4DBF // CJK Extension A
        | 0x4E00..=0x9FFF // CJK Unified Ideographs
        | 0xF900..=0xFAFF // CJK Compatibility Ideographs
    )
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
    cell_size: (i32, i32),
    force_fullwidth_ascii: bool,
    use_pua_mapping: bool,
) -> io::Result<()> {
    let bytes = fs::read(base_atlas_path)?;
    let base = FontAtlasData::from_binary(&bytes)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.message))?;

    let required = load_required_chars(required_path)?;

    let mut all_symbols: Vec<char> = (0x20u8..0x7Fu8).map(|b| b as char).collect();
    for c in required {
        if !all_symbols.contains(&c) {
            all_symbols.push(c);
        }
    }
    for c in ALWAYS_INCLUDE_SYMBOLS.chars() {
        if !all_symbols.contains(&c) {
            all_symbols.push(c);
        }
    }
    all_symbols.sort_unstable();

    let non_ascii_count = all_symbols.iter().filter(|c| !c.is_ascii()).count();
    let available_custom_slots = (Glyph::GLYPH_ID_MASK + 1) as usize - 0x80usize;
    if non_ascii_count > available_custom_slots {
        return Err(io::Error::other(format!(
            "too many non-ASCII symbols ({non_ascii_count}) for 1024-glyph atlas capacity"
        )));
    }

    let (cell_w, cell_h) = cell_size;
    let tex_w = cell_w * FontAtlasData::CELLS_PER_SLICE;
    let tex_h = cell_h;
    let tex_d = DEFAULT_ATLAS_LAYERS;
    let tex_len = (tex_w as usize) * (tex_h as usize) * (tex_d as usize) * 4;

    let mut atlas = FontAtlasData {
        font_name: "Noto Custom".into(),
        font_size: cell_h as f32,
        texture_dimensions: (tex_w, tex_h, tex_d),
        cell_size,
        underline: base.underline,
        strikethrough: base.strikethrough,
        glyphs: Vec::new(),
        texture_data: vec![0u8; tex_len],
    };

    let mut next_custom_id = 0x80u16;
    for c in all_symbols {
        let base_id = if c.is_ascii() {
            c as u16
        } else {
            let id = next_custom_id;
            next_custom_id = next_custom_id.saturating_add(1);
            id
        };

        for style in FontStyle::ALL {
            let full_id = with_style(base_id, style);
            if glyph_layer(full_id) >= tex_d as usize {
                return Err(io::Error::other(format!(
                    "glyph id {full_id} exceeds configured atlas depth ({tex_d} layers)"
                )));
            }

            let glyph = Glyph {
                id: full_id,
                style,
                symbol: mapped_symbol_char(c, use_pua_mapping).to_string().into(),
                pixel_coords: glyph_pixel_coords(full_id, atlas.cell_size),
                is_emoji: false,
            };
            render_glyph_into_atlas(
                &mut atlas,
                fonts.for_style_char(style, c),
                c,
                full_id,
                force_fullwidth_ascii,
            )?;
            atlas.glyphs.push(glyph);
        }
    }

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(output_path, atlas.to_binary())?;

    println!(
        "built noto atlas at {} with {} symbols (cell {}x{}, layers {})",
        output_path.display(),
        atlas.glyphs.len() / FontStyle::ALL.len(),
        cell_w,
        cell_h,
        tex_d
    );
    Ok(())
}

fn glyph_layer(glyph_id: u16) -> usize {
    (glyph_id as usize) / FontAtlasData::CELLS_PER_SLICE as usize
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
    force_fullwidth_ascii: bool,
) -> io::Result<()> {
    clear_cell_rgba(atlas, glyph_id)?;

    if render_box_drawing_glyph(atlas, glyph_id, c)? {
        return Ok(());
    }

    let (cell_w, cell_h) = atlas.cell_size;
    let inner_w = (cell_w - 2 * FontAtlasData::PADDING).max(1) as usize;
    let inner_h = (cell_h - 2 * FontAtlasData::PADDING).max(1) as usize;

    let raster_char = if force_fullwidth_ascii {
        to_fullwidth_ascii(c)
    } else {
        c
    };
    let (metrics, bitmap) = rasterize_fit(font, raster_char, inner_w, inner_h);

    if metrics.width == 0 || metrics.height == 0 {
        return Ok(());
    }

    let col = (glyph_id as usize) % FontAtlasData::CELLS_PER_SLICE as usize;
    let layer = glyph_layer(glyph_id);

    let tex_w = atlas.texture_dimensions.0 as usize;
    let tex_h = atlas.texture_dimensions.1 as usize;
    let tex_d = atlas.texture_dimensions.2 as usize;

    if layer >= tex_d {
        return Err(io::Error::other(format!(
            "atlas texture has {tex_d} layers, cannot place glyph id {glyph_id}"
        )));
    }

    let cell_x = col * cell_w as usize;
    let pad = FontAtlasData::PADDING as usize;
    let draw_x = cell_x + pad + ((inner_w.saturating_sub(metrics.width)) / 2);
    let draw_y = pad + ((inner_h.saturating_sub(metrics.height)) / 2);

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
            let idx = ((layer * tex_h + py) * tex_w + px) * 4;
            atlas.texture_data[idx] = 255;
            atlas.texture_data[idx + 1] = 255;
            atlas.texture_data[idx + 2] = 255;
            atlas.texture_data[idx + 3] = alpha;
        }
    }
    Ok(())
}

fn to_fullwidth_ascii(c: char) -> char {
    match c {
        '!'..='~' => {
            let mapped = c as u32 + 0xFEE0;
            char::from_u32(mapped).unwrap_or(c)
        }
        _ => c,
    }
}

fn render_box_drawing_glyph(atlas: &mut FontAtlasData, glyph_id: u16, c: char) -> io::Result<bool> {
    let kind = match c {
        '─' => (true, true, false, false),
        '│' => (false, false, true, true),
        '┌' => (false, true, false, true),
        '┐' => (true, false, false, true),
        '└' => (false, true, true, false),
        '┘' => (true, false, true, false),
        '├' => (false, true, true, true),
        '┤' => (true, false, true, true),
        '┬' => (true, true, false, true),
        '┴' => (true, true, true, false),
        '┼' => (true, true, true, true),
        _ => return Ok(false),
    };

    let (left, right, up, down) = kind;
    let (cell_w, cell_h) = atlas.cell_size;
    let pad = FontAtlasData::PADDING.max(0) as usize;
    let half_x = (cell_w.max(1) as usize) / 2;
    let half_y = (cell_h.max(1) as usize) / 2;
    let thick = if (glyph_id & Glyph::BOLD_FLAG) != 0 {
        2usize
    } else {
        1usize
    };

    let col = (glyph_id as usize) % FontAtlasData::CELLS_PER_SLICE as usize;
    let layer = glyph_layer(glyph_id);
    let tex_w = atlas.texture_dimensions.0 as usize;
    let tex_h = atlas.texture_dimensions.1 as usize;
    let tex_d = atlas.texture_dimensions.2 as usize;
    if layer >= tex_d {
        return Err(io::Error::other(format!(
            "atlas texture has {tex_d} layers, cannot place glyph id {glyph_id}"
        )));
    }

    let x0 = col * (cell_w as usize);
    let y0 = 0usize;
    let x_center = x0 + half_x;
    let y_center = y0 + half_y;

    let mut put = |x: usize, y: usize| {
        if x >= tex_w || y >= tex_h {
            return;
        }
        let idx = ((layer * tex_h + y) * tex_w + x) * 4;
        atlas.texture_data[idx] = 255;
        atlas.texture_data[idx + 1] = 255;
        atlas.texture_data[idx + 2] = 255;
        atlas.texture_data[idx + 3] = 255;
    };

    if left {
        for x in x0 + pad..=x_center {
            for t in 0..thick {
                put(x, y_center.saturating_add(t));
            }
        }
    }
    if right {
        let right_end = x0 + (cell_w as usize).saturating_sub(1 + pad);
        for x in x_center..=right_end {
            for t in 0..thick {
                put(x, y_center.saturating_add(t));
            }
        }
    }
    if up {
        for y in y0 + pad..=y_center {
            for t in 0..thick {
                put(x_center.saturating_add(t), y);
            }
        }
    }
    if down {
        let down_end = y0 + (cell_h as usize).saturating_sub(1 + pad);
        for y in y_center..=down_end {
            for t in 0..thick {
                put(x_center.saturating_add(t), y);
            }
        }
    }

    Ok(true)
}

fn clear_cell_rgba(atlas: &mut FontAtlasData, glyph_id: u16) -> io::Result<()> {
    let tex_w = atlas.texture_dimensions.0 as usize;
    let tex_h = atlas.texture_dimensions.1 as usize;
    let tex_d = atlas.texture_dimensions.2 as usize;
    let cell_w = atlas.cell_size.0 as usize;
    let cell_h = atlas.cell_size.1 as usize;

    let col = (glyph_id as usize) % FontAtlasData::CELLS_PER_SLICE as usize;
    let layer = glyph_layer(glyph_id);
    if layer >= tex_d {
        return Err(io::Error::other(format!(
            "atlas texture has {tex_d} layers, cannot clear glyph id {glyph_id}"
        )));
    }

    let x0 = col * cell_w;
    for y in 0..cell_h {
        for x in 0..cell_w {
            let px = x0 + x;
            let py = y;
            if px >= tex_w || py >= tex_h {
                continue;
            }
            let idx = ((layer * tex_h + py) * tex_w + px) * 4;
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

fn extract_required_chars_for_lang(lang: &str, output: &Path) -> io::Result<()> {
    let mut chars = BTreeSet::new();

    // Shared data shown in every language (e.g. bus stop names).
    collect_chars_from_file(Path::new("assets/stops.toml"), &mut chars)?;
    collect_chars_from_file(Path::new("assets/routes.toml"), &mut chars)?;
    collect_chars_from_file(Path::new("assets/i18n/config.toml"), &mut chars)?;

    // Language-specific UI strings.
    let lang_ftl = format!("assets/i18n/{lang}/main.ftl");
    collect_chars_from_file(Path::new(&lang_ftl), &mut chars)?;

    // Keep common symbol set available in all atlases.
    for c in ALWAYS_INCLUDE_SYMBOLS.chars() {
        if is_required_char(c) {
            chars.insert(c);
        }
    }

    let required: String = chars.into_iter().collect();
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(output, required.as_bytes())?;

    println!(
        "wrote language-scoped required character set ({lang}) to {}",
        output.display()
    );
    Ok(())
}

fn collect_chars_from_file(path: &Path, out: &mut BTreeSet<char>) -> io::Result<()> {
    if !path.exists() {
        return Ok(());
    }
    let text = fs::read_to_string(path)?;
    for c in text.chars() {
        if is_required_char(c) {
            out.insert(c);
        }
    }
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

fn verify_required(
    atlas_path: &Path,
    required_path: &Path,
    use_pua_mapping: bool,
) -> io::Result<()> {
    let atlas_bytes = fs::read(atlas_path)?;
    let atlas = FontAtlasData::from_binary(&atlas_bytes)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.message))?;

    let required = fs::read_to_string(required_path)?;
    let glyph_set: BTreeSet<char> = atlas.glyphs.iter().flat_map(|g| g.symbol.chars()).collect();

    let missing: Vec<char> = required
        .chars()
        .filter(|c| {
            let mapped = mapped_symbol_char(*c, use_pua_mapping);
            !c.is_ascii() && !glyph_set.contains(&mapped)
        })
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

fn has_flag(args: &[String], flag: &str) -> bool {
    args.iter().any(|a| a == flag)
}

fn mapped_symbol_char(c: char, use_pua_mapping: bool) -> char {
    if !use_pua_mapping || c.is_ascii() || is_structural_symbol(c) {
        return c;
    }
    let mapped = 0xF0000 + c as u32;
    char::from_u32(mapped).unwrap_or(c)
}

fn is_structural_symbol(c: char) -> bool {
    matches!(
        c,
        '─' | '│'
            | '┌'
            | '┐'
            | '└'
            | '┘'
            | '├'
            | '┤'
            | '┬'
            | '┴'
            | '┼'
            | '↑'
            | '↓'
            | '←'
            | '→'
            | '↵'
            | '⌫'
    )
}
