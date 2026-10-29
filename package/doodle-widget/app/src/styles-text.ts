/**
 Text overlay layer CSS styles for the doodle widget.
 
 Renders styles for the text layer container and individual
 text input elements (active and finalized).
 */
import {
  cssCommaList,
  cssNum,
  cssOklch,
  cssRem,
  hCss as $,
} from '@monochromatic-dev/hyperscript-core/ts';
import { INSET_ZERO_DECLS, } from './style-tokens.ts';

/**
 Text input minimum inline size in rem
 */
const TEXT_INPUT_MIN_INLINE_SIZE = 2 * 2
  * 2;

/**
 Text input line height (wider than CSS normal ~1.2 for readability)
 */
const TEXT_INPUT_LINE_HEIGHT = 1 + (2 / (2 + 2
  + 1));

/**
 Generates CSS rules for the text overlay layer and text inputs.
 
 @returns array of CSS rule strings
 
 @example
 ```ts
 const rules = renderTextStyles();
 ```
 */
export function renderTextStyles(): string[] {
  return [
    $({
      rule: '#text-layer',
      decls: {
        ...INSET_ZERO_DECLS,
        'pointer-events': 'none',
      },
    },),

    $({
      rule: '.text-input',
      decls: {
        position: 'absolute',
        'background-color': cssOklch({
          l: 1,
          c: 0,
          h: 0,
          a: 0.85,
        },),
        color: cssOklch({
          l: 0.3,
          c: 0,
          h: 0,
        },),
        'pointer-events': 'auto',
        'border-block-style': 'none',
        'border-inline-style': 'none',
        'outline-style': 'none',
        'font-family': cssCommaList([
          'system-ui',
          'sans-serif',
        ],),
        'font-size': cssRem(1 + ((1 / 2) / 2),),
        'line-height': cssNum(TEXT_INPUT_LINE_HEIGHT,),
        'padding-block': cssNum(0,),
        'padding-inline': cssNum(0,),
        'min-inline-size': cssRem(TEXT_INPUT_MIN_INLINE_SIZE,),
      },
    },),

    /**
     Finalized inputs look like plain text
     */
    $({
      rule: '.text-input:read-only',
      decls: {
        'background-color': 'transparent',
        'pointer-events': 'none',
        'min-inline-size': cssNum(0,),
        cursor: 'default',
      },
    },),
  ];
}
