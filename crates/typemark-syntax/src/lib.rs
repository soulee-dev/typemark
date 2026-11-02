//! Parses Markdown (CommonMark + GFM + attribute syntax) into a document tree.

use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};

/// A parsed Markdown document.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Document {
    pub children: Vec<Block>,
}

/// A block-level node.
#[derive(Debug, Clone, PartialEq)]
pub enum Block {
    Heading { level: u8, attrs: Attrs, inlines: Vec<Inline> },
    Paragraph { inlines: Vec<Inline> },
    List { ordered: Option<u64>, items: Vec<Vec<Block>> },
    CodeBlock { code: String },
    PageBreak,
}

/// An inline node.
///
/// Emphasis, strong and inline code are flattened to text for now.
#[derive(Debug, Clone, PartialEq)]
pub enum Inline {
    Text(String),
    LineBreak,
}

/// Attributes attached with `{#id .class key=value}` syntax.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Attrs {
    pub id: Option<String>,
    pub classes: Vec<String>,
    pub pairs: Vec<(String, Option<String>)>,
}

/// The paragraph that forces a page break.
const PAGE_BREAK: &str = "\\pagebreak";

/// Options that change how Markdown is interpreted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParseOptions {
    /// Treat every newline inside a paragraph as a line break.
    ///
    /// On by default. Set to `false` for CommonMark behavior, where a single
    /// newline is joined with a space and only a trailing backslash or two
    /// trailing spaces break the line.
    pub newline_breaks: bool,
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self { newline_breaks: true }
    }
}

/// Parses Markdown source into a [`Document`] with default options.
pub fn parse(source: &str) -> Document {
    parse_with(source, ParseOptions::default())
}

/// Parses Markdown source into a [`Document`].
pub fn parse_with(source: &str, parse_options: ParseOptions) -> Document {
    let options = Options::ENABLE_HEADING_ATTRIBUTES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TABLES;
    let mut builder = Builder::new(parse_options);
    for event in Parser::new_ext(source, options) {
        builder.event(event);
    }
    Document { children: builder.finish() }
}

/// Containers that are still open while events stream in.
#[derive(Debug)]
enum Frame {
    Root(Vec<Block>),
    Heading { level: u8, attrs: Attrs, inlines: Vec<Inline> },
    Paragraph(Vec<Inline>),
    List { ordered: Option<u64>, items: Vec<Vec<Block>> },
    Item(Vec<Block>),
    Code(String),
    /// Elements that are not supported yet. Their text is kept as a paragraph.
    Other(Vec<Inline>),
}

#[derive(Debug)]
struct Builder {
    stack: Vec<Frame>,
    options: ParseOptions,
}

impl Builder {
    fn new(options: ParseOptions) -> Self {
        Self { stack: vec![Frame::Root(Vec::new())], options }
    }

    fn event(&mut self, event: Event<'_>) {
        match event {
            Event::Start(tag) => self.start(tag),
            Event::End(tag) => self.end(tag),
            Event::Text(text) | Event::Code(text) => self.text(&text),
            Event::SoftBreak if self.options.newline_breaks => self.inline(Inline::LineBreak),
            Event::SoftBreak => self.text(" "),
            Event::HardBreak => self.inline(Inline::LineBreak),
            _ => {}
        }
    }

    fn start(&mut self, tag: Tag<'_>) {
        let frame = match tag {
            Tag::Heading { level, id, classes, attrs } => Frame::Heading {
                level: heading_level(level),
                attrs: Attrs {
                    id: id.map(|s| s.to_string()),
                    classes: classes.iter().map(|s| s.to_string()).collect(),
                    pairs: attrs
                        .iter()
                        .map(|(k, v)| (k.to_string(), v.as_ref().map(|v| v.to_string())))
                        .collect(),
                },
                inlines: Vec::new(),
            },
            Tag::Paragraph => Frame::Paragraph(Vec::new()),
            Tag::List(ordered) => Frame::List { ordered, items: Vec::new() },
            Tag::Item => Frame::Item(Vec::new()),
            Tag::CodeBlock(_) => Frame::Code(String::new()),
            // Inline containers: their text flows into the enclosing frame.
            Tag::Emphasis | Tag::Strong | Tag::Strikethrough | Tag::Link { .. } => return,
            _ => Frame::Other(Vec::new()),
        };
        self.stack.push(frame);
    }

    fn end(&mut self, tag: TagEnd) {
        if matches!(
            tag,
            TagEnd::Emphasis | TagEnd::Strong | TagEnd::Strikethrough | TagEnd::Link
        ) {
            return;
        }
        let frame = self.stack.pop().expect("unbalanced Markdown events");
        let block = match frame {
            Frame::Heading { level, attrs, inlines } => Block::Heading { level, attrs, inlines },
            Frame::Paragraph(inlines) if is_page_break(&inlines) => Block::PageBreak,
            Frame::Paragraph(inlines) => Block::Paragraph { inlines },
            Frame::List { ordered, items } => Block::List { ordered, items },
            Frame::Item(blocks) => {
                if let Some(Frame::List { items, .. }) = self.stack.last_mut() {
                    items.push(blocks);
                }
                return;
            }
            Frame::Code(code) => Block::CodeBlock { code },
            Frame::Other(inlines) => {
                // Nested unsupported frames (e.g. table cells inside rows) bubble their text up.
                if let Some(Frame::Other(parent)) = self.stack.last_mut() {
                    if !parent.is_empty() && !inlines.is_empty() {
                        parent.push(Inline::Text(" ".into()));
                    }
                    parent.extend(inlines);
                    return;
                }
                if inlines.is_empty() {
                    return;
                }
                Block::Paragraph { inlines }
            }
            Frame::Root(_) => unreachable!("root frame is never popped"),
        };
        self.block(block);
    }

    fn text(&mut self, text: &str) {
        match self.stack.last_mut() {
            Some(Frame::Code(code)) => code.push_str(text),
            Some(Frame::Item(blocks)) => {
                // Tight list items have no paragraph around their text.
                match blocks.last_mut() {
                    Some(Block::Paragraph { inlines }) => push_text(inlines, text),
                    _ => blocks.push(Block::Paragraph { inlines: vec![Inline::Text(text.into())] }),
                }
            }
            _ => {
                if let Some(inlines) = self.inlines() {
                    push_text(inlines, text);
                }
            }
        }
    }

    fn inline(&mut self, inline: Inline) {
        if let Some(Frame::Item(blocks)) = self.stack.last_mut() {
            if let Some(Block::Paragraph { inlines }) = blocks.last_mut() {
                inlines.push(inline);
            }
            return;
        }
        if let Some(inlines) = self.inlines() {
            inlines.push(inline);
        }
    }

    fn inlines(&mut self) -> Option<&mut Vec<Inline>> {
        match self.stack.last_mut()? {
            Frame::Heading { inlines, .. } | Frame::Paragraph(inlines) | Frame::Other(inlines) => {
                Some(inlines)
            }
            _ => None,
        }
    }

    fn block(&mut self, block: Block) {
        if let Some(Frame::Root(blocks) | Frame::Item(blocks)) = self.stack.last_mut() {
            blocks.push(block);
        }
    }

    fn finish(mut self) -> Vec<Block> {
        match self.stack.swap_remove(0) {
            Frame::Root(blocks) => blocks,
            _ => unreachable!("root frame is always at the bottom"),
        }
    }
}

fn push_text(inlines: &mut Vec<Inline>, text: &str) {
    if let Some(Inline::Text(last)) = inlines.last_mut() {
        last.push_str(text);
    } else {
        inlines.push(Inline::Text(text.into()));
    }
}

fn is_page_break(inlines: &[Inline]) -> bool {
    matches!(inlines, [Inline::Text(t)] if t.trim() == PAGE_BREAK)
}

fn heading_level(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(s: &str) -> Inline {
        Inline::Text(s.into())
    }

    #[test]
    fn parses_heading_with_attributes() {
        let doc = parse("# 제목 {#intro .cover}\n");
        assert_eq!(
            doc.children,
            vec![Block::Heading {
                level: 1,
                attrs: Attrs { id: Some("intro".into()), classes: vec!["cover".into()], pairs: vec![] },
                inlines: vec![text("제목")],
            }]
        );
    }

    #[test]
    fn parses_page_break() {
        let doc = parse("앞\n\n\\pagebreak\n\n뒤\n");
        assert_eq!(
            doc.children,
            vec![
                Block::Paragraph { inlines: vec![text("앞")] },
                Block::PageBreak,
                Block::Paragraph { inlines: vec![text("뒤")] },
            ]
        );
    }

    #[test]
    fn newlines_break_lines_by_default() {
        let doc = parse("가 *나*\n다\n라\n");
        assert_eq!(
            doc.children,
            vec![Block::Paragraph {
                inlines: vec![text("가 나"), Inline::LineBreak, text("다"), Inline::LineBreak, text("라")]
            }]
        );
    }

    #[test]
    fn commonmark_mode_joins_newlines_and_keeps_hard_breaks() {
        let options = ParseOptions { newline_breaks: false };
        let doc = parse_with("가 *나*\n다\\\n라\n", options);
        assert_eq!(
            doc.children,
            vec![Block::Paragraph {
                inlines: vec![text("가 나 다"), Inline::LineBreak, text("라")]
            }]
        );
    }

    #[test]
    fn parses_tight_list() {
        let doc = parse("- 하나\n- 둘\n");
        assert_eq!(
            doc.children,
            vec![Block::List {
                ordered: None,
                items: vec![
                    vec![Block::Paragraph { inlines: vec![text("하나")] }],
                    vec![Block::Paragraph { inlines: vec![text("둘")] }],
                ],
            }]
        );
    }
}
