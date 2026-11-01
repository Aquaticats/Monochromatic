/**
 Reset, page scaffold, masthead, and input-panel styles for the wc
 text-stats tool.
 
 The page is a viewport-filling flex column (`min-block-size: 100dvh`
 on `.page`, not on the textarea) so an empty page shows no scrollbar:
 the layout row flexes to fill exactly the remaining height and the
 textarea stretches inside it. Growth beyond the viewport comes from
 the client script's auto-grow, which raises the textarea's
 `min-block-size` to its scroll height.
 
 Flexbox only; CSS grid is not allowed in this package.
 */
import {
  cssCh,
  cssCommaList,
  cssDvb,
  cssMin,
  cssNum,
  cssPercent,
  cssRem,
  cssVar,
  hCss as $,
} from '@monochromatic-dev/hyperscript-core/ts';

/**
 Viewport width, in rem, above which the input and results panels sit
 side by side instead of stacked: the 60ch input minimum plus a usable
 results column no longer fit below this.
 */
export const WIDE_VIEWPORT_REM: number = 64;

/**
 One half, composed from the exempt literal range.
 */
export const HALF: number = 1 / 2;

/**
 One quarter, composed from {@link HALF}.
 */
export const QUARTER: number = HALF / 2;

/**
 One eighth, composed from {@link QUARTER}.
 */
export const EIGHTH: number = QUARTER / 2;

/**
 Three quarters, composed from {@link HALF} and {@link QUARTER}.
 */
export const THREE_QUARTERS: number = HALF + QUARTER;

/**
 Base spacing unit (rem) the scaffold's paddings and gaps derive from.
 */
const SPACE = 1 + HALF;

/**
 Hairline border width in rem (one device pixel at default zoom),
 composed from {@link EIGHTH}.
 */
export const HAIRLINE: number = EIGHTH / 2;

/**
 Masthead title weight.
 */
const WEIGHT_TITLE = 700;

/**
 Input label weight.
 */
const WEIGHT_LABEL = 600;

/**
 Flex-grow factor that lets the input panel absorb all free inline
 space (up to its 90ch cap) before the results panel starts growing.
 */
const INPUT_GROW_FACTOR = 999;

/**
 Results panel preferred inline size in rem.
 */
const RESULTS_BASIS_REM = 24;

/**
 Results panel maximum inline size in rem.
 */
const RESULTS_MAX_REM = 40;

/**
 Empty-textarea minimum block size in rem, a floor for when the
 flex-stretch has little room (stacked narrow layout).
 */
const INPUT_MIN_BLOCK_REM = 16;

/**
 Input box minimum inline size in ch (user requirement: never narrower
 than 60ch, capped by the container on narrow viewports).
 */
const INPUT_MIN_CH = 60;

/**
 Input box maximum inline size in ch (user requirement: never wider
 than 90ch).
 */
const INPUT_MAX_CH = 90;

/**
 Full-length percentage.
 */
const FULL_PERCENT = 100;

/**
 Generates reset, body, page, masthead, and input-panel rules.
 
 @returns minified CSS string
 
 @example
 ```ts
 const css = renderLayoutStyles();
 ```
 */
export function renderLayoutStyles(): string {
  return [
    $(
      {
        rule: '*, *::before, *::after',
        decls: {
          'box-sizing': 'border-box',
          'margin-block': cssNum(0,),
          'margin-inline': cssNum(0,),
        },
      },
    ),

    $(
      {
        // Inclusively-hidden pattern, verbatim from
        // https://www.scottohara.me/blog/2017/04/14/inclusively-hidden.html:
        // removes content from the visual rendering while keeping it in
        // the accessibility tree (the Frequency header row uses it).
        // The declarations go through `raw` because the strict `decls`
        // typing disallows the deprecated `clip` and the physical
        // `height`/`width` the published pattern spells out, and the
        // pattern is kept exactly as attributed.
        rule: '.visually-hidden:not(:focus):not(:active)',
        raw: 'clip:rect(0 0 0 0);'
          + 'clip-path:inset(50%);'
          + 'height:1px;'
          + 'overflow:hidden;'
          + 'position:absolute;'
          + 'white-space:nowrap;'
          + 'width:1px',
      },
    ),

    $(
      {
        rule: 'body',
        decls: {
          'font-family': cssCommaList(
            [
              "'Inter'",
              'system-ui',
              'sans-serif',
            ],
          ),
          color: cssVar('color-fg',),
          'background-color': cssVar('color-bg',),
          'line-height': cssNum(1 + HALF,),
          'font-size': cssRem(1,),
        },
      },
    ),

    $(
      {
        rule: '.page',
        decls: {
          'min-block-size': cssDvb(FULL_PERCENT,),
          display: 'flex',
          'flex-direction': 'column',
          gap: cssRem(SPACE,),
          'padding-block': cssRem(SPACE,),
          'padding-inline': cssRem(SPACE,),
        },
      },
    ),

    $(
      {
        rule: 'h1',
        decls: {
          'font-size': cssRem(1 + THREE_QUARTERS,),
          'font-weight': cssNum(WEIGHT_TITLE,),
          'line-height': cssNum(1 + QUARTER,),
        },
      },
    ),

    $(
      {
        rule: '.description',
        decls: {
          color: cssVar('color-muted',),
          'max-inline-size': cssCh(INPUT_MIN_CH,),
          'margin-block-start': cssRem(QUARTER,),
        },
      },
    ),

    $(
      {
        // Rendered only when scripting is off (inside `<noscript>`),
        // explaining why every count stays at 0.
        rule: '.noscript-note',
        decls: {
          color: cssVar('color-muted',),
          'max-inline-size': cssCh(INPUT_MIN_CH,),
          'margin-block-start': cssRem(QUARTER,),
        },
      },
    ),

    $(
      {
        rule: '.layout',
        decls: {
          display: 'flex',
          'flex-direction': 'column',
          gap: cssRem(SPACE,),
          'flex-grow': cssNum(1,),
        },
      },
    ),

    $(
      {
        rule: '.input-panel',
        decls: {
          display: 'flex',
          'flex-direction': 'column',
          'min-inline-size': cssMin(
            [
              cssCh(INPUT_MIN_CH,),
              cssPercent(FULL_PERCENT,),
            ],
          ),
          'max-inline-size': cssCh(INPUT_MAX_CH,),
        },
      },
    ),

    $(
      {
        // `.input-label` wraps the textarea (implicit label/control
        // association, no `id`/`for` pair needed), so it takes over the
        // flex-column layout `.input-panel` used to run directly between
        // its label and textarea children. It sets no font-size of its
        // own so the nested `.wc-input`'s `inherit` reaches all the way
        // to `body`, not to a label-specific size; the caption's own
        // size/weight live on `.input-label-text` instead.
        rule: '.input-label',
        decls: {
          display: 'flex',
          'flex-direction': 'column',
          gap: cssRem(HALF,),
          'flex-grow': cssNum(1,),
        },
      },
    ),

    $(
      {
        rule: '.input-label-text',
        decls: {
          'font-size': cssRem(1 + EIGHTH,),
          'font-weight': cssNum(WEIGHT_LABEL,),
        },
      },
    ),

    $(
      {
        rule: '.wc-input',
        decls: {
          'inline-size': cssPercent(FULL_PERCENT,),
          'flex-grow': cssNum(1,),
          'min-block-size': cssRem(INPUT_MIN_BLOCK_REM,),
          'font-family': 'inherit',
          'font-size': 'inherit',
          'line-height': 'inherit',
          color: 'inherit',
          'background-color': cssVar('color-bg',),
          'border-width': cssRem(HAIRLINE,),
          'border-style': 'solid',
          'border-color': cssVar('color-border-strong',),
          'border-radius': cssRem(HALF,),
          'padding-block': cssRem(THREE_QUARTERS,),
          'padding-inline': cssRem(1 - EIGHTH,),
          resize: 'none',
        },
      },
    ),

    $(
      {
        rule: '.wc-input::placeholder',
        decls: { color: cssVar('color-placeholder',), },
      },
    ),

    $(
      {
        rule: '.wc-input:focus-visible',
        decls: {
          'outline-width': cssRem(EIGHTH,),
          'outline-style': 'solid',
          'outline-color': cssVar('color-fg-strong',),
          'outline-offset': cssRem(EIGHTH,),
        },
      },
    ),

    $(
      {
        // Growth tracks content once the client script runs (see
        // `client/main.ts`), so the inner scrollbar never has anything
        // to scroll; the script adds this class rather than reaching
        // for an inline style, and only once it has taken over sizing,
        // so content stays reachable via the native scrollbar if
        // scripting is unavailable.
        rule: '.wc-input.scripted',
        decls: { 'overflow-y': 'hidden', },
      },
    ),

    $(
      {
        at: 'media',
        params: `(min-width: ${cssRem(WIDE_VIEWPORT_REM,)})`,
        children: [
          $(
            {
              rule: '.layout',
              decls: {
                'flex-direction': 'row',
                'justify-content': 'center',
                gap: cssRem(SPACE + HALF,),
              },
            },
          ),
          $(
            {
              rule: '.input-panel',
              decls: {
                'flex-grow': cssNum(INPUT_GROW_FACTOR,),
                'flex-shrink': cssNum(1,),
                'flex-basis': cssNum(0,),
              },
            },
          ),
          $(
            {
              rule: '.results-panel',
              decls: {
                'flex-grow': cssNum(1,),
                'flex-shrink': cssNum(1,),
                'flex-basis': cssRem(RESULTS_BASIS_REM,),
                'min-inline-size': cssNum(0,),
                'max-inline-size': cssRem(RESULTS_MAX_REM,),
                position: 'sticky',
                'inset-block-start': cssRem(SPACE,),
                'align-self': 'flex-start',
              },
            },
          ),
        ],
      },
    ),
  ]
    .join('',);
}
