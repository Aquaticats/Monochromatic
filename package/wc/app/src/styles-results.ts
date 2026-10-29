/**
 Results-panel styles for the wc text-stats tool: the six stat tiles
 and the Frequency ARIA table.
 
 Tiles are flex-wrapped (no CSS grid). Frequency rows are flex rows so
 per-row `content-visibility: auto` takes effect (it is ignored on
 internal table boxes), keeping unbounded row counts cheap to render.
 Number columns align purely through Inter's tabular numerals plus
 figure-space padding done by the client script; no column widths are
 managed in CSS. The word cell is pinned to `--word-col`, the widest
 word's measured width (set by the client script on `.frequency`,
 capped so one pathological token cannot crush the bars), and the bar
 track flex-grows into all remaining width; with the number cells
 equal through figure-space padding and the word cell fixed, the
 grown track is identical in every row, so the bars fill the free
 width while their lengths stay comparable. Words wider than their
 cell truncate with an ellipsis on a single line (full word in
 `title` and `aria-label`), keeping every row at the intrinsic block
 size containment promises. Bars keep the five-stop grayscale
 palette: the fill is the strong foreground stop and the track is
 transparent. The transparent track requires an author background,
 and any author background or border on `<progress>` switches both
 engines from the natively themed widget (which honors
 `accent-color`) to a fallback rendering that ignores it (Chromium
 paints green-on-gray, Firefox a UA blue plus a blue-tinted
 border), so the fill stop is pinned on both engines' fill
 pseudo-elements, `::-webkit-progress-value` and
 `::-moz-progress-bar`, which style exactly that fallback, and
 Firefox's tinted fallback border is removed. Catalogued in
 doc/troubleshooting/progress-element-fill-styling.md; verified in
 containerized Firefox and Chromium by ./page.browser.test.ts.
 */
import {
  cssCompounded,
  cssNum,
  cssPercent,
  cssRem,
  cssVar,
  hCss as $,
} from '@monochromatic-dev/hyperscript-core/ts';

import {
  EIGHTH,
  HAIRLINE,
  HALF,
  QUARTER,
  THREE_QUARTERS,
} from './styles-layout.ts';

/**
 Tile headline-number weight, heavier than body but below bold so the
 variable font renders it crisply at display size.
 */
const WEIGHT_VALUE = 650;

/**
 Section-heading weight.
 */
const WEIGHT_HEADING = 600;

/**
 Tile label and tile sub-stat text size in rem, floored at the browser's
 default 1rem so no on-page text renders smaller than the user's base
 font size.
 */
const LABEL_TEXT_REM = 1;

/**
 Tile headline-number size in rem.
 */
const VALUE_SIZE_REM = 2;

/**
 Estimated frequency-row block size in rem for
 `contain-intrinsic-block-size`, so skipped rows keep the scrollbar
 stable.
 */
const ROW_INTRINSIC_REM = 2 + QUARTER;

/**
 Tile minimum inline size in rem, a floor so a short-content tile (e.g.
 bytes, with no sub-stat) doesn't shrink to an unreadably narrow card;
 tiles otherwise size to their own content instead of stretching to
 match row siblings.
 */
const TILE_MIN_REM = ((2 * 2) * 2)
  + HALF;

/**
 Full-length percentage.
 */
const FULL_PERCENT = 100;

/**
 Bar track minimum inline size in rem, a floor so bars stay legible
 when a long word column squeezes a narrow row; below it the word
 cell shrinks and wraps instead.
 */
const BAR_TRACK_MIN_REM = 2 + 2;

/**
 Generates results-panel, tiles, and frequency rules.
 
 @returns minified CSS string
 
 @example
 ```ts
 const css = renderResultsStyles();
 ```
 */
export function renderResultsStyles(): string {
  return [
    $(
      {
        rule: '.results-panel h2',
        decls: {
          'font-size': cssRem(1 + EIGHTH,),
          'font-weight': cssNum(WEIGHT_HEADING,),
          color: cssVar('color-muted',),
          'margin-block-end': cssRem(THREE_QUARTERS,),
        },
      },
    ),

    $(
      {
        rule: '.stats-section',
        decls: { 'margin-block-end': cssRem(2,), },
      },
    ),

    $(
      {
        rule: '.tiles',
        decls: {
          display: 'flex',
          'flex-wrap': 'wrap',
          gap: cssRem(THREE_QUARTERS,),
        },
      },
    ),

    $(
      {
        // No `flex-grow`: tiles are not forced to equal widths within a
        // row. Each sizes to its own content (`flex-basis: auto`) above
        // `min-inline-size`, so a tile with a longer "longest N unit"
        // phrase gets the extra width it needs instead of wrapping at
        // the same point as its narrower row siblings.
        rule: '.tile',
        decls: {
          'flex-grow': cssNum(0,),
          'flex-shrink': cssNum(1,),
          'flex-basis': 'auto',
          'min-inline-size': cssRem(TILE_MIN_REM,),
          display: 'flex',
          'flex-direction': 'column',
          gap: cssRem(QUARTER,),
          'padding-block': cssRem(1 - EIGHTH,),
          'padding-inline': cssRem(1,),
          'border-radius': cssRem(HALF,),
          'background-color': cssVar('color-surface',),
        },
      },
    ),

    $(
      {
        rule: '.tile-label',
        decls: { 'font-size': cssRem(LABEL_TEXT_REM,), },
      },
    ),

    $(
      {
        rule: '.tile-value',
        decls: {
          'font-size': cssRem(VALUE_SIZE_REM,),
          'font-weight': cssNum(WEIGHT_VALUE,),
          'line-height': cssNum(1 + EIGHTH,),
          color: cssVar('color-fg-strong',),
        },
      },
    ),

    $(
      {
        rule: '.tile-sub',
        decls: { 'font-size': cssRem(LABEL_TEXT_REM,), },
      },
    ),

    $(
      {
        // Keeps the value and its unit on one line: the wrap between
        // "longest" and this span stays the only breakable space, so a
        // narrow tile never splits e.g. "23" from "chars".
        rule: '.tile-sub-amount',
        decls: { 'white-space': 'nowrap', },
      },
    ),

    $(
      {
        rule: '.frequency',
        decls: { 'font-variant-numeric': 'tabular-nums', },
      },
    ),

    $(
      {
        rule: '.frequency-row',
        decls: {
          display: 'flex',
          gap: cssRem(THREE_QUARTERS,),
          'align-items': 'baseline',
          'padding-block': cssRem(HALF,),
          'border-block-end-width': cssRem(HAIRLINE,),
          'border-block-end-style': 'solid',
          'border-block-end-color': cssVar('color-border-subtle',),
        },
      },
    ),

    $(
      {
        rule: '.frequency-body .frequency-row',
        decls: {
          'content-visibility': 'auto',
          'contain-intrinsic-block-size': cssCompounded(
            [
              'auto',
              cssRem(ROW_INTRINSIC_REM,),
            ],
          ),
        },
      },
    ),

    $(
      {
        rule: '.freq-count, .freq-pct',
        decls: { 'flex-shrink': cssNum(0,), },
      },
    ),

    $(
      {
        rule: '.freq-word',
        decls: {
          'flex-grow': cssNum(0,),
          'flex-shrink': cssNum(1,),
          'flex-basis': cssVar('word-col',),
          'min-inline-size': cssNum(0,),
          'white-space': 'nowrap',
          overflow: 'hidden',
          'text-overflow': 'ellipsis',
        },
      },
    ),

    $(
      {
        rule: '.freq-bar-track',
        decls: {
          'flex-grow': cssNum(1,),
          'flex-shrink': cssNum(0,),
          'flex-basis': cssNum(0,),
          'min-inline-size': cssRem(BAR_TRACK_MIN_REM,),
          'align-self': 'center',
          'block-size': cssRem(HALF + EIGHTH,),
        },
      },
    ),

    $(
      {
        // A native `<progress value max>` replaces the old two-span,
        // custom-property-driven bar: `value`/`max` are ordinary content
        // attributes (not `style`, not `id`), so the fill ratio needs no
        // per-row inline style, and screen readers get progressbar
        // semantics for free (kept out of the accessibility tree, since
        // the count/percentage cells already carry the real data).
        // Palette discipline: the transparent background clears the
        // track, and, as author styling, it also drops both engines
        // into their unthemed fallback rendering, where `accent-color`
        // is inert and only the vendor fill pseudos below recolor the
        // fill. `accent-color` is kept for engines that honor it on
        // author-styled progress (neither tested engine does today);
        // no `appearance` reset is needed because the author
        // background already disables native theming. See
        // doc/troubleshooting/progress-element-fill-styling.md.
        rule: '.freq-bar',
        decls: {
          display: 'block',
          'inline-size': cssPercent(FULL_PERCENT,),
          'block-size': cssPercent(FULL_PERCENT,),
          'background-color': 'transparent',
          // Firefox's fallback rendering otherwise keeps a border in
          // a slightly blue-tinted gray (rgb(143 143 157) measured in
          // the playwright container) that violates the grayscale
          // palette.
          'border-style': 'none',
          'accent-color': cssVar('color-fg-strong',),
        },
      },
    ),

    $(
      {
        // Chromium's fallback track: gray (rgb(128 128 128)) once the
        // element carries an author background; this pseudo clears it.
        rule: '.freq-bar::-webkit-progress-bar',
        decls: { 'background-color': 'transparent', },
      },
    ),

    $(
      {
        // Chromium's fallback fill: green (rgb(0 128 0)) once the
        // element carries an author background; this pseudo pins the
        // fill stop.
        rule: '.freq-bar::-webkit-progress-value',
        decls: { 'background-color': cssVar('color-fg-strong',), },
      },
    ),

    $(
      {
        // Firefox's fallback fill: a UA blue (rgb(0 100 180)) once the
        // element carries an author background; this pseudo pins the
        // fill stop.
        rule: '.freq-bar::-moz-progress-bar',
        decls: { 'background-color': cssVar('color-fg-strong',), },
      },
    ),

    $(
      {
        rule: '.frequency-empty',
        decls: { color: cssVar('color-muted',), },
      },
    ),
  ]
    .join('',);
}
