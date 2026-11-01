/**
 CSS styles for the syllable break demo.
 
 Generates a stylesheet with three side-by-side comparison columns:
 ZWS-based syllable breaking, CSS hyphens: auto, and plain overflow-wrap.
 */
import {
  cssCh,
  cssCommaList,
  cssNum,
  cssOklch,
  cssPercent,
  cssRem,
  hCss as $,
} from '@monochromatic-dev/hyperscript-core/ts';

/**
 Muted text for descriptions and secondary content.
 */
const COLOR_MUTED = cssOklch({
  l: 0.42,
  c: 0,
  h: 0,
},);

/**
 Lighter muted text for code previews.
 */
const COLOR_MUTED_LIGHT = cssOklch({
  l: 0.48,
  c: 0,
  h: 0,
},);

/**
 Faint text for notes and annotations.
 */
const COLOR_FAINT = cssOklch({
  l: 0.6,
  c: 0,
  h: 0,
},);

/**
 Outline color for the output container.
 */
const COLOR_OUTLINE = cssOklch({
  l: 0.63,
  c: 0.26,
  h: 25,
},);

/**
 Generates the complete CSS stylesheet for the syllable break demo, using
 {@link COLOR_MUTED}, {@link COLOR_MUTED_LIGHT}, {@link COLOR_FAINT}, and
 {@link COLOR_OUTLINE} for its palette.
 
 @returns minified CSS string
 
 @example
 ```ts
 const css = renderStyles();
 ```
 */
export function renderStyles(): string {
  return [
    $({
      rule: '*, *::before, *::after',
      decls: {
        'box-sizing': 'border-box',
        'margin-block': cssNum(0,),
        'margin-inline': cssNum(0,),
      },
    },),

    $({
      rule: 'body',
      decls: {
        'font-family': cssCommaList([
          'system-ui',
          'sans-serif',
        ],),
        'padding-block': cssRem(2,),
        'padding-inline': cssRem(2,),
        'max-inline-size': cssRem(10 * ((2 * 2) + 2),),
        'margin-inline': 'auto',
        'line-height': cssNum(1 + (1 / 2),),
      },
    },),

    $({
      rule: 'h1',
      decls: {
        'font-size': cssRem(1 + (1 / 2),),
        'margin-block-end': cssRem(1 / 2,),
      },
    },),

    $({
      rule: '.description',
      decls: {
        'margin-block-end': cssRem(1 + (1 / 2),),
        color: COLOR_MUTED,
      },
    },),

    $({
      rule: '.controls',
      decls: {
        'margin-block-end': cssRem(1 + (1 / 2),),
        display: 'flex',
        'flex-direction': 'column',
        gap: cssRem((1 / 2) + (1 / 2
          / 2),),
      },
    },),

    $({
      rule: 'textarea',
      decls: {
        'inline-size': cssPercent(100,),
        'min-block-size': cssRem(2 * 2,),
        'font-family': 'inherit',
        'font-size': 'inherit',
        'padding-block': cssRem(1 / 2,),
        'padding-inline': cssRem(1 / 2,),
      },
    },),

    $({
      rule: '.slider-row',
      decls: {
        display: 'flex',
        'align-items': 'center',
        gap: cssRem((1 / 2) + (1 / 2
          / 2),),
      },
    },),

    $({
      rule: '.slider-row input[type="range"]',
      decls: {
        'flex-grow': cssNum(1,),
      },
    },),

    $({
      rule: '.processed-preview',
      decls: {
        'font-size': cssRem(1 - (1 / 2
          / 2
          / 2),),
        color: COLOR_MUTED_LIGHT,
        'font-family': 'monospace',
        'word-break': 'break-all',
      },
    },),

    $({
      rule: '.columns',
      decls: {
        display: 'flex',
        gap: cssRem(2,),
        'flex-wrap': 'wrap',
      },
    },),

    $({
      rule: '.column',
      decls: {
        'flex-shrink': cssNum(0,),
      },
    },),

    $({
      rule: '.column h2',
      decls: {
        'font-size': cssRem(1,),
        'margin-block-end': cssRem(1 / 2,),
      },
    },),

    $({
      rule: '.column p.note',
      decls: {
        'font-size': cssRem((1 / 2) + (1 / 2
          / 2),),
        color: COLOR_FAINT,
        'margin-block-end': cssRem(1 / 2,),
      },
    },),

    $({
      rule: '.output-box',
      decls: {
        'outline-style': 'solid',
        'outline-color': COLOR_OUTLINE,
        'outline-width': cssRem(1 / 16,),
        'inline-size': cssCh(10,),
        'font-size': cssRem(1,),
        'padding-block': cssRem(1 / 2
          / 2,),
        'padding-inline': cssRem(1 / 2
          / 2,),
        'overflow-wrap': 'break-word',
      },
    },),

    $({
      rule: '.output-box.hyphens-auto',
      decls: {
        hyphens: 'auto',
        'hyphenate-character': '""',
      },
    },),

    $({
      rule: '.output-box.plain',
      decls: {
        'overflow-wrap': 'normal',
      },
    },),
  ]
    .join('',);
}
