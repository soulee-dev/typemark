//! Typemark: a fast, browser-free typesetting engine that turns
//! Markdown and CSS into pixel-identical PDFs and previews.

pub use typemark_layout::{Frame, Item, LayoutError, Paged};
pub use typemark_pdf::PdfError;
pub use typemark_syntax::ParseOptions;
pub use typemark_text::{FontBook, FontId};

/// Compiles Markdown and CSS into laid-out pages with default options.
pub fn compile(markdown: &str, css: &str, fonts: &FontBook) -> Result<Paged, LayoutError> {
    compile_with(markdown, css, fonts, ParseOptions::default())
}

/// Compiles Markdown and CSS into laid-out pages.
pub fn compile_with(
    markdown: &str,
    css: &str,
    fonts: &FontBook,
    options: ParseOptions,
) -> Result<Paged, LayoutError> {
    let doc = typemark_syntax::parse_with(markdown, options);
    let styles = typemark_style::parse(css);
    typemark_layout::layout(&doc, &styles, fonts)
}

/// Encodes pages as PDF.
pub fn pdf(paged: &Paged, fonts: &FontBook) -> Result<Vec<u8>, PdfError> {
    typemark_pdf::pdf(paged, fonts)
}

/// Renders pages as SVG.
pub fn svg(paged: &Paged, fonts: &FontBook) -> Vec<String> {
    typemark_svg::svg(paged, fonts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requires_fonts() {
        assert_eq!(compile("# hi", "", &FontBook::new()).unwrap_err(), LayoutError::NoFonts);
    }
}
