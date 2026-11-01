/**
 Composed CSS mixin patterns built on the primitives in `mixins.ts`.
 
 Separated to keep `mixins.ts` under the line budget.
 */
import {
  cssCalc,
  type CssDeclarations,
  cssInt,
  cssRem,
  cssVar,
} from '@monochromatic-dev/hyperscript-core/ts';
import {
  flexCenter,
  flexRow,
  minTouchTarget,
} from './mixins.ts';

/**
 Button gap and padding in rem (1/2).
 */
const BTN_PADDING = 1 / 2;

/**
 Sticky bar height in rem.
 */
const BAR_HEIGHT = 3;

/**
 Z-index for sticky navigation bars.
 */
const Z_INDEX_STICKY = 10;

/**
 Outlined interactive button with token-based colors.
 
 @returns Declarations for outlined button
 
 @example
 ```ts
 css({ rule: '.btn', decls: buttonOutlined() })
 ```
 */
export function buttonOutlined(): CssDeclarations {
  return {
    ...flexCenter(),
    ...minTouchTarget(),
    gap: cssRem(BTN_PADDING,),
    'border-width': cssCalc(`${cssRem(1,)} / 16`,),
    'border-style': 'solid',
    'border-color': cssVar('fg',),
    'padding-block': cssRem(BTN_PADDING,),
    'padding-inline': cssRem(BTN_PADDING,),
    'background-color': 'transparent',
    color: cssVar('fg',),
    'font-family': 'inherit',
    'font-size': 'inherit',
    'font-style': 'inherit',
    'font-weight': 'inherit',
    'line-height': 'inherit',
    cursor: 'pointer',
  };
}

/**
 Top-anchored sticky navigation bar.
 
 @returns Declarations for sticky bar
 
 @example
 ```ts
 css({ rule: ':host', decls: { ...stickyBar(), 'justify-content': 'center' } })
 ```
 */
export function stickyBar(): CssDeclarations {
  return {
    ...flexRow(),
    gap: cssVar('min-gap',),
    'block-size': cssRem(BAR_HEIGHT,),
    'padding-block': 0,
    'padding-inline': cssVar('min-padding',),
    'background-color': cssVar('bg',),
    position: 'sticky',
    'inset-block-start': 0,
    'z-index': cssInt(Z_INDEX_STICKY,),
  };
}
