//! Renders laid-out pages to SVG for previews.
//!
//! Text is emitted as `<text>` elements, so the viewer's font rendering applies.
//! Embedding glyph outlines for pixel-identical output is planned.

use std::fmt::Write;

use typemark_layout::{Item, Paged};
use typemark_text::FontBook;

/// Renders each page as an SVG document.
pub fn svg(paged: &Paged, fonts: &FontBook) -> Vec<String> {
    paged
        .pages
        .iter()
        .map(|page| {
            let mut out = format!(
                r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}pt" height="{h}pt" viewBox="0 0 {w} {h}"><rect width="100%" height="100%" fill="white"/>"#,
                w = page.width,
                h = page.height
            );
            for item in &page.items {
                match item {
                    Item::Text { x, y, font, size, text } => {
                        let weight = fonts.get(*font).weight;
                        let _ = write!(
                            out,
                            r#"<text x="{x:.2}" y="{y:.2}" font-size="{size}" font-weight="{weight}" xml:space="preserve">{}</text>"#,
                            escape(text)
                        );
                    }
                }
            }
            out.push_str("</svg>");
            out
        })
        .collect()
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}
