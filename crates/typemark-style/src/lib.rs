//! Parses a CSS subset (including Paged Media) and computes styles for document nodes.
//!
//! CSS parsing is not implemented yet: [`parse`] always returns the built-in
//! default theme, which mirrors `themes/default.css`.

/// Points per millimeter.
const MM: f64 = 72.0 / 25.4;

/// Resolved styles used by the layout engine. Lengths are in points.
#[derive(Debug, Clone, PartialEq)]
pub struct Stylesheet {
    pub page: PageStyle,
    pub body: TextStyle,
    /// Styles for heading levels 1 to 6.
    pub headings: [TextStyle; 6],
    pub code: TextStyle,
    /// Left indent of list items.
    pub list_indent: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageStyle {
    pub width: f64,
    pub height: f64,
    pub margin: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextStyle {
    pub font_size: f64,
    pub font_weight: u16,
    /// Line height as a multiple of the font size.
    pub line_height: f64,
    pub margin_top: f64,
    pub margin_bottom: f64,
}

impl Default for Stylesheet {
    fn default() -> Self {
        let heading = |font_size, margin_top, margin_bottom| TextStyle {
            font_size,
            font_weight: 700,
            line_height: 1.3,
            margin_top,
            margin_bottom,
        };
        Self {
            page: PageStyle { width: 210.0 * MM, height: 297.0 * MM, margin: 20.0 * MM },
            body: TextStyle {
                font_size: 11.0,
                font_weight: 400,
                line_height: 1.6,
                margin_top: 0.0,
                margin_bottom: 8.0,
            },
            headings: [
                heading(20.0, 0.0, 12.0),
                heading(15.0, 18.0, 8.0),
                heading(12.0, 14.0, 6.0),
                heading(11.0, 12.0, 6.0),
                heading(11.0, 12.0, 6.0),
                heading(11.0, 12.0, 6.0),
            ],
            code: TextStyle {
                font_size: 9.5,
                font_weight: 400,
                line_height: 1.4,
                margin_top: 0.0,
                margin_bottom: 8.0,
            },
            list_indent: 14.0,
        }
    }
}

/// Parses CSS source into a [`Stylesheet`].
pub fn parse(_css: &str) -> Stylesheet {
    // TODO: implement the CSS parser.
    Stylesheet::default()
}
