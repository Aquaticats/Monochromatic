# syllable-break-demo

Generates a single self-contained HTML file that demonstrates syllable-aware
word breaking without visible hyphens.

## What is demonstrated

Long words (especially scientific terminology like
`Ribulose-1,5-bisphosphate carboxylase/oxygenase`) often overflow narrow
containers because the browser's built-in `hyphens: auto` dictionary does not
cover them.
This demo compares three side-by-side approaches to the same input text in the
same narrow container:

- **JS zero-width space**:
   TeX hyphenation patterns (via the `hyphen` library)
  find syllable boundaries;
   the demo inserts `U+200B` (zero-width space)
  instead of a soft hyphen,
   so the browser breaks at the boundary without
  rendering a visible hyphen character.
- **CSS `hyphens: auto` with `hyphenate-character: ""`**:
   the browser's own
  hyphenation dictionary,
   with the hyphen glyph suppressed.
  Works for common words;
   fails on terms outside the dictionary.
- **Plain `overflow-wrap: normal`**:
   no word breaking;
   long words overflow the
  container.

A textarea lets you enter arbitrary text,
 and a slider adjusts the container
width (in `ch` units) so you can watch each strategy succeed or fail at
different sizes.
A "Break points" line shows the JS-inserted zero-width spaces rendered as
visible middle dots (`U+00B7`) so you can inspect where the hyphenation library
placed the boundaries.

## How it works

The build script reads pre-bundled client JS from `dist/client/main.js`,
 then
inlines HTML (via `h-html`),
 CSS (via `h-css`),
 and JS into a single
`dist/final/index.html` file.
The client script runs TeX patterns against the input text on every keystroke
and updates all three output columns;
 the slider rewrites
`element.style.inlineSize` on each column.

## Build

```sh
mise run //package/syllable-break-demo/app:build
```

This runs `build:js:client` first to bundle `src/client/main.ts` into
`dist/client/main.js`,
 then runs `src/build.ts` to assemble the final HTML.

Output:
 `dist/final/index.html`.

## Open

The output is a single self-contained HTML file with no external dependencies,
so any of these works:

```sh
# native browser
xdg-open package/syllable-break-demo/app/dist/final/index.html

# agent-browser (for AI agents or headless inspection)
agent-browser open file://$PWD/packages/webapp-productivity/syllable-break-demo/dist/final/index.html
```

You can also paste a `file://` URL directly into any browser's address bar.

## Interact

- Type or paste text into the textarea to see all three columns update live.
- Drag the slider to change the container width from `5ch` to `40ch`.
- Watch the "Break points" line to see exactly where the hyphenation library
  inserts zero-width spaces.
