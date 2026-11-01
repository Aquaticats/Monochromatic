/**
 CSS styles for the doodle widget.
 
 Uses h-css for type-safe CSS generation with strict property validation.
 */
import {
  cssCalc,
  cssCqb,
  cssCqi,
  cssMin,
  cssNum,
  cssOklch,
  hCss as $,
} from '@monochromatic-dev/hyperscript-core/ts';
import {
  BG_VIEWPORT,
  BORDER_WIDTH,
  FULL_DVB,
  FULL_PERCENT,
  INSET_ZERO_DECLS,
  LETTER_ASPECT_RATIO,
  LETTER_HEIGHT_IN,
  LETTER_WIDTH_IN,
  PAGE_FRAME_COLOR,
  VIEWPORT_PADDING,
} from './style-tokens.ts';
import { renderAttributionStyles, } from './styles-attribution.ts';
import { renderTextStyles, } from './styles-text.ts';
import { renderToastStyles, } from './styles-toast.ts';
import { renderToolbarStyles, } from './styles-toolbar.ts';

/**
 Generates the complete CSS stylesheet for the doodle widget.
 
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
      rule: '#app',
      decls: {
        display: 'flex',
        'flex-direction': 'column',
        'block-size': FULL_DVB,
      },
    },),

    ...renderToolbarStyles(),

    $({
      rule: '#canvas-container',
      decls: {
        'flex-grow': cssNum(1,),
        display: 'flex',
        'container-type': 'size',
        'overflow-x': 'clip',
        'overflow-y': 'clip',
        'padding-block': VIEWPORT_PADDING,
        'padding-inline': VIEWPORT_PADDING,
        'background-color': BG_VIEWPORT,
      },
    },),

    $({
      rule: '#page',
      decls: {
        position: 'relative',
        'margin-block': 'auto',
        'margin-inline': 'auto',
        'inline-size': cssMin([
          cssCqi(100,),
          cssCalc(
            `${cssCqb(100,)} * ${String(LETTER_WIDTH_IN,)} / ${
              String(LETTER_HEIGHT_IN,)
            }`,
          ),
        ],),
        'aspect-ratio': LETTER_ASPECT_RATIO,
        'background-color': cssOklch({
          l: 1,
          c: 0,
          h: 0,
        },),
        'border-block-style': 'solid',
        'border-block-color': PAGE_FRAME_COLOR,
        'border-block-width': BORDER_WIDTH,
        'border-inline-style': 'solid',
        'border-inline-color': PAGE_FRAME_COLOR,
        'border-inline-width': BORDER_WIDTH,
      },
    },),

    $({
      rule: '#zoom-layer',
      decls: { ...INSET_ZERO_DECLS, },
    },),

    $({
      rule: '#draw-canvas',
      decls: {
        ...INSET_ZERO_DECLS,
        display: 'block',
        cursor: 'crosshair',
        'touch-action': 'none',
        '-webkit-touch-callout': 'none',
      },
    },),

    $({
      rule: '#svg-overlay',
      decls: {
        ...INSET_ZERO_DECLS,
        display: 'flex',
        'align-items': 'center',
        'justify-content': 'center',
        'pointer-events': 'none',
        'mix-blend-mode': 'multiply',
      },
    },),

    $({
      rule: '#svg-overlay > svg',
      decls: {
        'max-inline-size': FULL_PERCENT,
        'max-block-size': FULL_PERCENT,
      },
    },),

    ...renderTextStyles(),

    ...renderToastStyles(),

    ...renderAttributionStyles(),
  ]
    .join('',);
}
