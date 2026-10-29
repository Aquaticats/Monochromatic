# wc

Generates a single self-contained HTML file that computes text statistics
and word frequency live in the browser,
 with no server and no network
requests after the page loads.

## Design

- **Palette**:
   strictly five opaque grayscale stops,
   `oklch(L 0 0)` for L
  in 0,
   0.1,
   0.5,
   0.9,
   1.
   No other lightness values,
   no alpha;
   subtlety
  comes from adjacent stops (0.9 surfaces on a 1 background in light mode,
  0 surfaces on a 0.1 background in dark mode).
   A unit test scans the
  palette fragments and rejects any other `oklch()` value.
- **Contrast**:
   every text role meets WCAG AAA small-text contrast (7:1).
  For achromatic oklch the WCAG relative luminance is L cubed,
   so
  mid-gray (0.5) text peaks at 6.0:1 on white and 3.4:1 on the dark
  background;
   text therefore only ever pairs the 0/0.1 stops with the
  0.9/1 stops (15:1 and up),
   and secondary text (headings,
   description,
  placeholder) shares the body-text stop,
   differentiated by size and
  weight instead of color.
   The mid stop is reserved for non-text borders
  (textarea),
   which meet the 3:1 non-text minimum.
  A unit test computes the ratios for every rendered ink-on-paper pairing
  in both themes and rejects regressions.
- **Type**:
   everything renders in Inter (variable,
   weights 100 to 900),
  auto-subsetted (see the fonts section) and inlined as a woff2 data URI.
  Frequency numbers use tabular numerals;
   tile headline numbers keep
  Inter's default proportional figures.
- **Layout**:
   flexbox only;
   CSS grid is not allowed in this package.
   At
  wide viewports the input sits beside a sticky results column;
   below
  64rem they stack.
   The input box prefers 100% of its column,
   clamped
  between `min(60ch, 100%)` and `90ch`.
- **Stats**:
   six tiles.
   Chars,
   lines,
   words,
   sentences,
   and paragraphs
  pair their headline count with a "longest" sub-stat (widest grapheme
  cluster in bytes,
   max line/word length in chars,
   max sentence length
  in words,
   max paragraph length in sentences);
   bytes stands alone.
- **Frequency rows**:
   flex rows with ARIA table roles instead of a native
  `<table>`,
   because `content-visibility: auto` (used to keep unbounded
  row counts cheap) is ignored on internal table boxes like `tr`.
   Counts
  and percentages come first,
   padded with figure spaces (U+2007) so the
  columns align purely through tabular numerals,
   with no column-width
  CSS.
   The word column is pinned to the widest word's measured width
  (canvas `measureText` in one pass,
   no per-row layout reads,
   re-run
  when font loading settles),
   capped at 12rem so one pathological token
  (a chemical name,
   a URL) cannot crush the bars;
   words wider than the
  cell truncate with an ellipsis while the full word rides in the
  cell's `title` and `aria-label`.
   The bar track flex-grows into all
  remaining width;
   with every other column equal across rows,
   the grown
  tracks are identical,
   so the bars fill the free width while their
  lengths stay comparable.
   Bars are native `<progress>` elements kept
  on the grayscale palette:
   the fill is the strong foreground stop and
  the track is transparent.
   The transparent track is author styling,
  and any author background or border on `<progress>` switches both
  engines from the natively themed widget (which honors
  `accent-color`) to a fallback rendering that ignores it (Chromium:
  green fill on a gray track;
   Firefox:
   a UA-blue fill plus a
  blue-tinted border),
   so the fill is pinned on
  `::-webkit-progress-value` and `::-moz-progress-bar`,
   which style
  exactly that fallback,
   and the tinted Firefox border is removed
  (details in `doc/troubleshooting/progress-element-fill-styling.md`);
  all of it pixel-verified in both engines by
  `src/page.browser.test.ts` in the playwright container.
   The header
  row is visually hidden by
  design (the columns are self-explanatory to sighted users) but stays
  in the accessibility tree for screen-reader column context,
   using the
  [inclusively-hidden][] pattern;
   the decorative bar cell is
  `aria-hidden`,
   so assistive tech sees exactly the count,
   percent,
   and
  word columns the hidden header names.

[inclusively-hidden]: https://www.scottohara.me/blog/2017/04/14/inclusively-hidden.html

### Known limitations

- Hard linebreaks render at the same height as soft-wrapped lines.
   Giving
  true linebreaks taller spacing requires per-line styling,
   which a
  native `<textarea>` cannot do (that needs a contenteditable editor);
  keeping the textarea was the chosen trade.
- "Longest" sub-stats always pluralize their unit:
   a one-byte widest
  grapheme reads "longest 1 bytes".
   Singular/plural switching is a
  known gap.
- All analysis runs synchronously on the main thread.
   Measured in
  headless Chromium:
   a 400KB (60,000-word) paste recomputes in roughly
  800ms after the 150ms debounce,
   and a recompute producing 20,000
  frequency rows takes roughly 1.2s.
   Fine for paste-and-read use;
  live-editing very large documents would need a worker or incremental
  recomputation,
   a known gap.
- The counting-methodology explanations ride in `title` attributes,
   so
  they only surface on hover:
   unreachable on touch screens and not
  announced by default by screen readers.
   The methodology is documented
  here instead;
   an on-page affordance is a known gap.
- The textarea's growth is scripted (an `input` listener raises
  `min-block-size` to the scroll height) because `field-sizing: content`
  is missing from the Firefox ESR baseline.
   Without JavaScript the box
  stays at its viewport-filling flex size and scrolls internally.
- User text in scripts outside the embedded subset (the charset is page
  chrome plus printable ASCII) falls back to `system-ui`;
   only page
  chrome is guaranteed to render in Inter.

## Fonts

The full upstream Inter variable woff2 lives in `fonts-source/`
(committed,
 never shipped).
 `mise run format:fonts` runs
`src/subset-fonts.ts`,
 which collects the charset from every `src/**/*.ts`
file plus a printable-ASCII floor and an explicit figure space (U+2007),
subsets via `hb-subset-wasm` (decode `wawoff2`,
 re-encode
`woff2-encode-wasm`),
 and writes `public/inter.woff2`.
 The STAT table is
dropped during subsetting:
 hb-subset-wasm's HarfBuzz build
(`HB_NO_STYLE`) prunes the name records STAT's axis-value entries
reference while passing STAT through,
 Firefox's font sanitizer logged
the dangling nameIDs as console errors before discarding the table
anyway,
 and no browser renders from STAT (fvar/gvar carry the variable
axes);
 the full trace lives in
`doc/troubleshooting/hb-subset-stat-dangling-nameids.md`.
 That subset
is committed,
 and `src/build.ts` inlines it into the final HTML as a
base64 data URI.
 Re-run `format:fonts` after adding non-ASCII page text (the
scan picks up literal characters,
 not escape sequences).
 Inter is
licensed under the SIL OFL 1.1 (`LICENSES/OFL-1.1.txt`).

## What is computed

Paste or type text into the box;
 a debounced (150ms) recompute updates two
sections.

### Stats

- **Bytes**:
   UTF-8 encoded byte length.
- **Chars**:
   grapheme cluster count (`Intl.Segmenter`,
  `granularity: 'grapheme'`),
   so combining accents and multi-codepoint
  emoji count as one character each,
   matching what a person would count by
  eye.
- **Widest char**:
   UTF-8 encoded byte length of the widest grapheme
  cluster (a family emoji is one char but many bytes).
- **Lines**:
   editor-style line count.
   A single trailing newline does not
  add a phantom empty line (`'a\n'` is 1 line),
   but a blank line that
  exists before end-of-text still counts (`'a\n\n'` is 2 lines).
- **Max line length**:
   grapheme cluster length of the longest line.
- **Words**:
   word-like segments (`Intl.Segmenter`,
   `granularity: 'word'`,
  `isWordLike` segments only),
   so CJK text with no spaces between words
  tokenizes correctly and punctuation-only segments are dropped.
- **Max word length**:
   grapheme cluster length of the longest word.
- **Sentences**:
   sentence segments (`Intl.Segmenter`,
  `granularity: 'sentence'`),
   trimmed of surrounding whitespace.
- **Max sentence length**:
   word count of the longest sentence (the
  standard readability-analysis unit).
- **Paragraphs**:
   blocks of text separated by one or more blank
  (whitespace-only) lines.
- **Max paragraph length**:
   sentence count of the longest paragraph.

### Frequency

A table of every word that occurs 2 or more times,
 case-folded to
lowercase before counting (`The` and `the` count together,
 displayed as
`the`).
 Sorted by count descending,
 ties broken alphabetically.
 The
percentage is each word's count divided by the total word count in the
text (not just the words shown in the table),
 to one decimal place.
 Words
that occur only once are omitted;
 an input with no repeated words shows a
placeholder row instead of an empty table.

## How it works

The build script reads pre-bundled client JS from `dist/client/main.js`
and the subsetted font from `public/inter.woff2`,
 then inlines HTML (via
`h-html`),
 CSS (via `h-css`),
 JS,
 the font,
 and the favicon into a single
`dist/final/index.html` file.
 The favicon is a `w<` wordmark drawn from
first principles at build time (`src/favicon.ts`:
 no font;
 every stick
shares one length and width,
 the w's turns close at 30 degrees and the
chevron at 60,
 near-white ink on the near-black palette stop) as an SVG
document,
 which sharp rasterizes to a PNG so the raster can never drift
from the vector;
 both ship as data-URI `<link rel="icon">` entries,
 the
SVG (`sizes="any"`) for engines that take vector icons and the PNG as
the fallback.
 `src/stat/` holds the pure,
 framework-free
tokenization and analysis logic (unit tested independently of the DOM);
`src/client/main.ts` wires a debounced `input` listener on the textarea to
that logic,
 writes results into the page,
 and auto-grows the textarea.
 It
also recomputes from the textarea's live value at startup and on
`pageshow`,
 because browsers restore textarea values across reloads (F5)
and back/forward navigations without firing `input`,
 at timings that vary
by browser;
 without the re-sync,
 restored text would sit in the box while
every stat reads zero.

Colors are CSS custom properties with light defaults,
 overridden inside a
`prefers-color-scheme: dark` media query,
 so the page follows the OS theme
automatically with no client-side toggle;
 `color-scheme: light dark` on
`:root` also switches native form-control chrome (the textarea's resize
handle,
 scrollbars).

## Build

```sh
mise run //package/wc/app:build
```

This runs `build:js:client` first to bundle `src/client/main.ts` into
`dist/client/main.js`,
 then runs `src/build.ts` to assemble the final HTML.

Output:
 `dist/final/index.html`.

## Open

The output is a single self-contained HTML file with no external
dependencies,
 so any of these works:

```sh
# native browser
xdg-open package/wc/app/dist/final/index.html

# agent-browser (for AI agents or headless inspection)
agent-browser open file://$PWD/packages/webapp-productivity/wc/dist/final/index.html
```

You can also paste a `file://` URL directly into any browser's address bar.

## Interact

- Type or paste text into the textarea;
   Stats and Frequency update about
  150ms after you stop typing.
- The Frequency table lists repeated words only,
   most frequent first.
