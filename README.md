# Typemark

A fast, browser-free typesetting engine that turns Markdown and CSS into pixel-identical PDFs and previews.

> Typemark is an independent engine. It is not built on Typst or a headless browser.

## Why

See [docs/vision.md](docs/vision.md) for the full motivation and roadmap.

- **No new syntax.** Write Markdown, style it with CSS.
- **No browser.** No Chromium or Gotenberg in the PDF pipeline.
- **Identical output.** PDFs and previews come from the same layout, so they always match.
- **Korean typesetting.** Word-level line breaking (`word-break: keep-all`) and font fallback are first-class.

## Example

```markdown
# Quarterly Report {.cover .text-center}

First paragraph.

\pagebreak

::: {.callout .warning}
Something to watch out for.
:::
```

```css
@page { size: A4; margin: 20mm; }
body  { font-family: "Pretendard"; word-break: keep-all; }
h1    { break-before: page; }
```

```sh
typemark build report.md --css report.css -o report.pdf
```

## Status

Pre-alpha. Markdown renders to PDF end to end with a built-in default theme.

Works today: headings (with `{#id .class}` attributes), paragraphs, newline line breaks,
ordered and unordered lists, code blocks, `\pagebreak`, automatic pagination,
and Korean word-level line breaking.

Not yet: CSS parsing (`--css` is ignored), bold and italic inline styles, tables,
images, a monospace font for code, font fallback, and kerning-aware justification.

## Repository layout

| Path | Purpose |
|---|---|
| `crates/typemark-syntax` | Markdown + attribute syntax → document tree |
| `crates/typemark-style` | CSS parsing, selector matching, computed styles |
| `crates/typemark-text` | Fonts, shaping, line breaking |
| `crates/typemark-layout` | Box layout and pagination → page frames |
| `crates/typemark-pdf` | Page frames → PDF |
| `crates/typemark-svg` | Page frames → SVG |
| `crates/typemark` | Public API |
| `crates/typemark-cli` | `typemark` command |
| `crates/typemark-wasm` | Browser bindings |
| `packages/` | npm packages (preview component, playground) |
| `editors/vscode` | VS Code extension |
| `themes/` | Built-in CSS themes |
| `docs/` | Syntax and CSS support specs |
| `tests/` | Fixtures and visual regression tests |
| `benches/` | Benchmarks |

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.
