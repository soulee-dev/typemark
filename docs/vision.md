# Vision

Typemark is a typesetting engine that turns Markdown and CSS into print-quality PDFs, without a browser.

This document explains why Typemark exists, what it is trying to be, and how it plans to get there.

## The problem

Writing a document and producing a well-typeset PDF from it are still two different worlds.

**Markdown is the most widely used writing format, but it was never designed for print.**
It has no page breaks, no page size or margins, and no running headers or page numbers.
Even line breaks are surprising: a single newline is silently joined with the previous line,
so writers have to remember trailing backslashes or invisible trailing spaces.

**Modern typesetting systems produce excellent output, but ask writers to leave Markdown behind.**
Typst, for example, is fast and beautiful, but it has its own markup and scripting language.
Existing Markdown cannot be reused as is, popular tools such as Mermaid are not supported,
and its collaborative web editor is a hosted service that cannot be run inside an organization.

**Tools that bridge the two are heavy.**
Pandoc-based pipelines such as Quarto work well, but they chain several large tools together,
report errors against generated intermediate files rather than the source,
and re-render the whole document on every change.

**Producing PDFs from HTML means running a browser.**
Many teams generate documents such as reports, invoices, and statements by rendering HTML
in headless Chromium, often through services like Gotenberg.
This works, but it is slow and resource-hungry, it needs a browser in production,
and the browser's print output never quite matches what the same HTML looks like on screen.

None of these tools are bad. They were each built for a different job.
What is missing is a single engine built for this one:
**write in Markdown, style with CSS, and get the same pages everywhere, fast.**

## What Typemark is

Typemark is one engine, written in Rust, that powers every place a document is seen.

```
Markdown + CSS ──► parse ──► style ──► layout ──► page frames ──┬──► PDF
                                                                ├──► SVG preview
                                                                └──► WebAssembly (browser)
```

Every output is drawn from the same laid-out page frames,
so the preview in an editor, the preview in a browser, and the final PDF are the same pages.

### Principles

**No new language to learn.**
Content is Markdown. Styling is CSS. Typemark adds only small, familiar extensions to Markdown:

- Attributes on blocks: `# Title {#intro .cover}`
- Fenced divs: `::: {.callout .warning}`
- Explicit page breaks: `\pagebreak`
- Newlines that break lines, the way people expect them to

**No browser.**
Typemark lays out text and pages itself. There is no Chromium, no headless service, and no network call
between a Markdown file and its PDF.

**Identical output everywhere.**
Layout happens once. Every backend renders the same result, so what you review is exactly what you ship.

**Fast enough to feel instant.**
Short documents should compile in milliseconds, so that previews update as you type
and servers can generate documents on request.

**First-class Korean and CJK typesetting.**
Word-level line breaking (`word-break: keep-all`), mixed-script spacing, and font fallback
are part of the core, not afterthoughts.

**Embeddable.**
The engine never touches the file system; fonts and images are passed in.
The same library runs in a CLI, inside a backend service, and in the browser through WebAssembly.

### CSS, not a new style language

Typemark styles documents with a subset of CSS, including the parts of CSS Paged Media that browsers
largely ignore: `@page` sizes and margins, page breaks, margin boxes for headers and footers,
and counters for page numbers.

CSS is something most developers and designers already know, and existing HTML templates can be
carried over. Utility-class workflows such as Tailwind sit naturally on top, because Markdown attributes
can carry classes.

Typemark does not aim to implement all of CSS. Features that only make sense for interactive screens,
such as hover states, animation, and absolute positioning, are out of scope.
The supported subset is tracked in [css-support.md](css-support.md).

## Non-goals

- **Being a browser.** Typemark targets documents, not web applications.
- **Executing code.** Computational notebooks are well served by tools like Quarto and Jupyter.
- **Replacing Typst or LaTeX for every use.** Highly programmable layouts and advanced math typesetting
  are their strength. Typemark focuses on documents that are naturally written in Markdown.

## Roadmap

### v0: Markdown and CSS to PDF

- [x] Markdown parsing with attributes, page breaks, and newline line breaks
- [x] Line breaking, pagination, and PDF output with embedded fonts
- [x] Korean word-level line breaking
- [ ] CSS parsing: `@page`, type, class, and id selectors, font, spacing, and color properties
- [ ] Inline styles: bold, italic, inline code
- [ ] Tables and images
- [ ] Monospace fonts and font fallback
- [ ] Benchmarks against browser-based PDF generation

### v1: Previews everywhere

- [ ] Pixel-identical SVG pages with embedded glyph outlines
- [ ] WebAssembly build, a web preview component, and an online playground
- [ ] VS Code extension with live preview
- [ ] Server-side PDF generation as a drop-in alternative to headless browsers

### Later

- [ ] Mermaid diagrams
- [ ] Headers, footers, counters, and running heads
- [ ] Flexbox and grid layout
- [ ] Built-in utility classes
- [ ] Math

## Open questions

- **Line breaks.** Treating every newline as a line break matches what writers expect,
  but differs from CommonMark. Typemark makes it the default and offers a CommonMark mode
  (`--commonmark-breaks`). Whether this default holds up for long-form prose is worth revisiting.
- **Mermaid without a browser.** Mermaid depends on a JavaScript and DOM environment.
  Browser previews can run it directly, but server-side rendering needs another approach.
- **How much CSS is enough.** The subset must be small enough to implement well and large enough
  that real templates work. Real-world templates will decide where the line is.
