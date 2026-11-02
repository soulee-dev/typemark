//! Lays out styled documents into pages.
//!
//! The [`Frame`]s produced here are the shared input for every backend,
//! which is what keeps previews and PDFs identical.

use typemark_style::{Stylesheet, TextStyle};
use typemark_syntax::{Block, Document, Inline};
use typemark_text::{FontBook, FontId, break_lines, break_preformatted};

/// A laid-out document.
#[derive(Debug, Default, Clone)]
pub struct Paged {
    pub pages: Vec<Frame>,
}

/// A single page with absolutely positioned items.
///
/// Units are points, with the origin at the top-left corner and y pointing down.
#[derive(Debug, Default, Clone)]
pub struct Frame {
    pub width: f64,
    pub height: f64,
    pub items: Vec<Item>,
}

/// Something drawn on a page.
#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    /// A single line of text. `y` is the baseline.
    Text { x: f64, y: f64, font: FontId, size: f64, text: String },
}

/// An error that prevents layout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayoutError {
    NoFonts,
}

impl std::fmt::Display for LayoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoFonts => f.write_str("no fonts were provided"),
        }
    }
}

impl std::error::Error for LayoutError {}

/// Lays out a document with the given stylesheet and fonts.
pub fn layout(doc: &Document, styles: &Stylesheet, fonts: &FontBook) -> Result<Paged, LayoutError> {
    if fonts.is_empty() {
        return Err(LayoutError::NoFonts);
    }
    let mut engine = Engine::new(styles, fonts);
    for block in &doc.children {
        engine.block(block, 0.0);
    }
    Ok(engine.finish())
}

struct Engine<'a> {
    styles: &'a Stylesheet,
    fonts: &'a FontBook,
    pages: Vec<Frame>,
    current: Frame,
    /// Top of the next line on the current page.
    y: f64,
    /// Bottom margin of the previous block, collapsed with the next block's top margin.
    pending_margin: f64,
}

impl<'a> Engine<'a> {
    fn new(styles: &'a Stylesheet, fonts: &'a FontBook) -> Self {
        let mut engine = Self {
            styles,
            fonts,
            pages: Vec::new(),
            current: Frame::default(),
            y: 0.0,
            pending_margin: 0.0,
        };
        engine.current = engine.blank_page();
        engine.y = styles.page.margin;
        engine
    }

    fn blank_page(&self) -> Frame {
        Frame { width: self.styles.page.width, height: self.styles.page.height, items: Vec::new() }
    }

    fn top(&self) -> f64 {
        self.styles.page.margin
    }

    fn bottom(&self) -> f64 {
        self.styles.page.height - self.styles.page.margin
    }

    fn content_width(&self) -> f64 {
        self.styles.page.width - 2.0 * self.styles.page.margin
    }

    fn at_page_top(&self) -> bool {
        self.current.items.is_empty()
    }

    fn new_page(&mut self) {
        let blank = self.blank_page();
        let page = std::mem::replace(&mut self.current, blank);
        self.pages.push(page);
        self.y = self.top();
        self.pending_margin = 0.0;
    }

    fn block(&mut self, block: &Block, indent: f64) {
        match block {
            Block::Heading { level, inlines, .. } => {
                let style = &self.styles.headings[(*level as usize).clamp(1, 6) - 1];
                self.paragraph(inlines, style, indent, None);
            }
            Block::Paragraph { inlines } => {
                self.paragraph(inlines, &self.styles.body, indent, None);
            }
            Block::List { ordered, items } => {
                let list_indent = indent + self.styles.list_indent;
                for (i, item) in items.iter().enumerate() {
                    let marker = match ordered {
                        Some(start) => format!("{}.", start + i as u64),
                        None => "•".to_string(),
                    };
                    for (j, child) in item.iter().enumerate() {
                        match child {
                            Block::Paragraph { inlines } if j == 0 => {
                                let style = TextStyle { margin_bottom: 3.0, ..self.styles.body.clone() };
                                self.paragraph(inlines, &style, list_indent, Some(&marker));
                            }
                            other => self.block(other, list_indent),
                        }
                    }
                }
                self.pending_margin = self.pending_margin.max(self.styles.body.margin_bottom);
            }
            Block::CodeBlock { code } => {
                let style = self.styles.code.clone();
                let font = self.font(style.font_weight);
                self.space_before(style.margin_top);
                let max_width = self.content_width() - indent;
                for source_line in code.trim_end_matches('\n').split('\n') {
                    let lines = break_preformatted(self.fonts, font, style.font_size, source_line, max_width);
                    for line in lines {
                        self.line(font, &style, indent, line);
                    }
                }
                self.pending_margin = style.margin_bottom;
            }
            Block::PageBreak => {
                if !self.at_page_top() {
                    self.new_page();
                }
            }
        }
    }

    /// Lays out a run of inlines. `marker` is drawn in the indent of the first line (list bullets).
    fn paragraph(&mut self, inlines: &[Inline], style: &TextStyle, indent: f64, marker: Option<&str>) {
        let font = self.font(style.font_weight);
        let max_width = self.content_width() - indent;
        self.space_before(style.margin_top);

        let mut first = true;
        for segment in split_hard_breaks(inlines) {
            let mut lines = break_lines(self.fonts, font, style.font_size, &segment, max_width);
            if lines.is_empty() {
                lines.push(String::new());
            }
            for line in lines {
                let baseline = self.line(font, style, indent, line);
                if first {
                    if let Some(marker) = marker {
                        let width = self.fonts.measure(font, style.font_size, marker);
                        let x = self.styles.page.margin + indent - width - 4.0;
                        self.current.items.push(Item::Text {
                            x,
                            y: baseline,
                            font,
                            size: style.font_size,
                            text: marker.to_string(),
                        });
                    }
                    first = false;
                }
            }
        }
        self.pending_margin = style.margin_bottom;
    }

    /// Places one line and returns its baseline.
    fn line(&mut self, font: FontId, style: &TextStyle, indent: f64, text: String) -> f64 {
        let height = style.font_size * style.line_height;
        if self.y + height > self.bottom() && !self.at_page_top() {
            self.new_page();
        }
        let metrics = self.fonts.metrics(font, style.font_size);
        let half_leading = (height - metrics.ascent - metrics.descent) / 2.0;
        let baseline = self.y + half_leading + metrics.ascent;
        if !text.is_empty() {
            self.current.items.push(Item::Text {
                x: self.styles.page.margin + indent,
                y: baseline,
                font,
                size: style.font_size,
                text,
            });
        }
        self.y += height;
        baseline
    }

    /// Applies the collapsed margin between the previous block and the next one.
    fn space_before(&mut self, margin_top: f64) {
        if !self.at_page_top() {
            self.y += self.pending_margin.max(margin_top);
        }
        self.pending_margin = 0.0;
    }

    fn font(&self, weight: u16) -> FontId {
        self.fonts.select(weight).expect("font book is not empty")
    }

    fn finish(mut self) -> Paged {
        self.pages.push(self.current);
        Paged { pages: self.pages }
    }
}

/// Joins inline text, splitting at hard line breaks.
fn split_hard_breaks(inlines: &[Inline]) -> Vec<String> {
    let mut segments = vec![String::new()];
    for inline in inlines {
        match inline {
            Inline::Text(text) => segments.last_mut().unwrap().push_str(text),
            Inline::LineBreak => segments.push(String::new()),
        }
    }
    segments
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fonts() -> FontBook {
        let mut book = FontBook::new();
        let data = std::fs::read("/System/Library/Fonts/AppleSDGothicNeo.ttc")
            .expect("these tests need a system Korean font");
        book.add(data);
        book
    }

    fn texts(frame: &Frame) -> Vec<&str> {
        frame.items.iter().map(|Item::Text { text, .. }| text.as_str()).collect()
    }

    fn run(markdown: &str) -> Paged {
        layout(&typemark_syntax::parse(markdown), &Stylesheet::default(), &fonts()).unwrap()
    }

    #[test]
    fn page_break_starts_new_page() {
        let paged = run("앞\n\n\\pagebreak\n\n뒤\n");
        assert_eq!(paged.pages.len(), 2);
        assert_eq!(texts(&paged.pages[0]), ["앞"]);
        assert_eq!(texts(&paged.pages[1]), ["뒤"]);
    }

    #[test]
    fn long_documents_flow_onto_new_pages() {
        let markdown = "문단입니다.\n\n".repeat(80);
        let paged = run(&markdown);
        assert!(paged.pages.len() > 1);
        let styles = Stylesheet::default();
        for page in &paged.pages {
            for Item::Text { y, .. } in &page.items {
                assert!(*y < styles.page.height - styles.page.margin);
            }
        }
    }

    #[test]
    fn korean_wraps_between_words() {
        let paragraph = "가나다라마바사 ".repeat(40);
        let paged = run(&paragraph);
        let lines = texts(&paged.pages[0]);
        assert!(lines.len() > 1);
        for line in lines {
            assert!(line.split(' ').all(|word| word == "가나다라마바사"), "split inside a word: {line}");
        }
    }

    #[test]
    fn code_keeps_indentation() {
        let paged = run("```\nfn main() {\n    body();\n}\n```\n");
        assert_eq!(texts(&paged.pages[0]), ["fn main() {", "    body();", "}"]);
    }
}
