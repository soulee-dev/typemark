//! Writes laid-out pages to PDF.

use std::collections::HashMap;

use krilla::Document;
use krilla::geom::Point;
use krilla::page::PageSettings;
use krilla::text::{Font, TextDirection};
use typemark_layout::{Item, Paged};
use typemark_text::{FontBook, FontId};

/// An error that occurred while writing a PDF.
#[derive(Debug)]
pub enum PdfError {
    InvalidFont(FontId),
    InvalidPageSize,
    Krilla(String),
}

impl std::fmt::Display for PdfError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidFont(id) => write!(f, "font {} could not be embedded", id.0),
            Self::InvalidPageSize => f.write_str("page width and height must be positive"),
            Self::Krilla(message) => write!(f, "failed to write PDF: {message}"),
        }
    }
}

impl std::error::Error for PdfError {}

/// Encodes pages as a PDF file.
pub fn pdf(paged: &Paged, fonts: &FontBook) -> Result<Vec<u8>, PdfError> {
    let mut document = Document::new();
    let mut cache: HashMap<FontId, Font> = HashMap::new();

    for frame in &paged.pages {
        let settings = PageSettings::from_wh(frame.width as f32, frame.height as f32)
            .ok_or(PdfError::InvalidPageSize)?;
        let mut page = document.start_page_with(settings);
        let mut surface = page.surface();
        for item in &frame.items {
            match item {
                Item::Text { x, y, font, size, text } => {
                    let pdf_font = match cache.get(font) {
                        Some(f) => f.clone(),
                        None => {
                            let face = fonts.get(*font);
                            let f = Font::new(face.data.clone().into(), face.index)
                                .ok_or(PdfError::InvalidFont(*font))?;
                            cache.insert(*font, f.clone());
                            f
                        }
                    };
                    surface.draw_text(
                        Point::from_xy(*x as f32, *y as f32),
                        pdf_font,
                        *size as f32,
                        text,
                        false,
                        TextDirection::Auto,
                    );
                }
            }
        }
        surface.finish();
        page.finish();
    }

    document.finish().map_err(|e| PdfError::Krilla(format!("{e:?}")))
}
