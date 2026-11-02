# Markdown extensions

Typemark accepts CommonMark and GitHub Flavored Markdown, plus the extensions below.

## Attributes

Attach an id, classes, or key-value pairs with `{...}` at the end of a block.

```markdown
# Title {#intro .cover .text-center}
```

## Fenced divs

```markdown
::: {.callout .warning}
Content
:::
```

## Page breaks

```markdown
\pagebreak
```

## Line breaks

Every newline inside a paragraph is a line break. You do not need a trailing
backslash or two trailing spaces. Use a blank line to start a new paragraph.

```markdown
첫 번째 줄
두 번째 줄
```

This differs from CommonMark, which joins single newlines with a space.
Pass `--commonmark-breaks` to the CLI, or set `ParseOptions::newline_breaks`
to `false` in the library, to get the CommonMark behavior.

## Planned

- Mermaid code blocks
- Footnotes
- Cross-references
