//! Font discovery and fallback, text shaping, and line breaking
//! (including Korean `word-break: keep-all`).

use std::sync::Arc;

pub use rustybuzz::ttf_parser;

/// Identifies a font face inside a [`FontBook`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FontId(pub usize);

/// A single font face.
#[derive(Debug, Clone)]
pub struct FontFace {
    pub data: Arc<Vec<u8>>,
    /// Face index inside a font collection (`.ttc`). `0` for single fonts.
    pub index: u32,
    pub weight: u16,
    pub italic: bool,
    units_per_em: f32,
    ascender: f32,
    descender: f32,
}

/// Vertical metrics of a face, scaled to a font size.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Metrics {
    pub ascent: f64,
    pub descent: f64,
}

/// A collection of fonts available to the layout engine.
///
/// Fonts are passed in as bytes so the engine never touches the file system,
/// which keeps it usable from WebAssembly.
#[derive(Debug, Default, Clone)]
pub struct FontBook {
    faces: Vec<FontFace>,
}

impl FontBook {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds every face in a font file or collection. Returns the number of faces added.
    pub fn add(&mut self, data: Vec<u8>) -> usize {
        let data = Arc::new(data);
        let count = ttf_parser::fonts_in_collection(&data).unwrap_or(1);
        let mut added = 0;
        for index in 0..count {
            let Ok(face) = ttf_parser::Face::parse(&data, index) else { continue };
            self.faces.push(FontFace {
                data: data.clone(),
                index,
                weight: face.weight().to_number(),
                italic: face.is_italic(),
                units_per_em: face.units_per_em() as f32,
                ascender: face.ascender() as f32,
                descender: face.descender() as f32,
            });
            added += 1;
        }
        added
    }

    pub fn len(&self) -> usize {
        self.faces.len()
    }

    pub fn is_empty(&self) -> bool {
        self.faces.is_empty()
    }

    pub fn get(&self, id: FontId) -> &FontFace {
        &self.faces[id.0]
    }

    /// Picks the upright face whose weight is closest to `weight`.
    pub fn select(&self, weight: u16) -> Option<FontId> {
        self.faces
            .iter()
            .enumerate()
            .min_by_key(|(_, f)| (f.italic, f.weight.abs_diff(weight)))
            .map(|(i, _)| FontId(i))
    }

    /// Scaled vertical metrics of a face.
    pub fn metrics(&self, id: FontId, size: f64) -> Metrics {
        let f = self.get(id);
        let scale = size / f.units_per_em as f64;
        Metrics { ascent: f.ascender as f64 * scale, descent: -f.descender as f64 * scale }
    }

    /// Measures the advance width of `text` when shaped with the given face.
    ///
    /// This uses the same shaper as the PDF backend, so measured and drawn widths match.
    pub fn measure(&self, id: FontId, size: f64, text: &str) -> f64 {
        let f = self.get(id);
        let Some(face) = rustybuzz::Face::from_slice(&f.data, f.index) else { return 0.0 };
        let mut buffer = rustybuzz::UnicodeBuffer::new();
        buffer.push_str(text);
        let glyphs = rustybuzz::shape(&face, &[], buffer);
        let units: i32 = glyphs.glyph_positions().iter().map(|p| p.x_advance).sum();
        units as f64 * size / f.units_per_em as f64
    }
}

/// Breaks `text` into lines no wider than `max_width`.
///
/// Lines break at spaces only, which matches `word-break: keep-all` and is the
/// natural behavior for Korean. A word wider than a whole line is split by character.
pub fn break_lines(
    fonts: &FontBook,
    font: FontId,
    size: f64,
    text: &str,
    max_width: f64,
) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        let candidate = if line.is_empty() { word.to_string() } else { format!("{line} {word}") };
        if fonts.measure(font, size, &candidate) <= max_width {
            line = candidate;
            continue;
        }
        if !line.is_empty() {
            lines.push(std::mem::take(&mut line));
        }
        if fonts.measure(font, size, word) <= max_width {
            line = word.to_string();
        } else {
            for ch in word.chars() {
                let mut next = line.clone();
                next.push(ch);
                if !line.is_empty() && fonts.measure(font, size, &next) > max_width {
                    lines.push(std::mem::take(&mut line));
                    line.push(ch);
                } else {
                    line = next;
                }
            }
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

/// Breaks preformatted text (code) into lines no wider than `max_width`.
///
/// Whitespace is preserved and lines wrap at any character.
pub fn break_preformatted(
    fonts: &FontBook,
    font: FontId,
    size: f64,
    text: &str,
    max_width: f64,
) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for ch in text.chars() {
        line.push(ch);
        if line.chars().count() > 1 && fonts.measure(font, size, &line) > max_width {
            line.pop();
            lines.push(std::mem::take(&mut line));
            line.push(ch);
        }
    }
    lines.push(line);
    lines
}
