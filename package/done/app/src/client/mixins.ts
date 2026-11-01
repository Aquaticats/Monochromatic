/**
 Shared CSS mixin functions for web component Shadow DOM styles.
 
 Replaces the PostCSS `@mixin`/`@apply` pipeline with plain TypeScript
 functions that return declaration records. Composed via object spread.
 
 Composed patterns (buttonOutlined, stickyBar) are in `mixins-composed.ts`.
 */
import {
  type CssDeclarations,
  cssRem,
  type CssValue,
  cssVar,
} from '@monochromatic-dev/hyperscript-core/ts';
import { $ as css, } from './css.ts';

/**
 Large border-radius for pill/circle shape in rem.
 */
const PILL_RADIUS = 62.5;

/**
 Minimum touch target size in rem (48px).
 */
const TOUCH_TARGET = 3;

/**
 Disabled button opacity.
 */
const DISABLED_OPACITY = 0.45;

/**
 Focus outline width in rem (1/8).
 */
const OUTLINE_WIDTH = 1 / 2
  / 2
  / 2;

//region Layout primitives

/**
 Flexbox centering on both axes.
 
 @returns Declarations for centered flex container
 
 @example
 ```ts
 $({ rule: '.icon', decls: { ...flexCenter(), 'font-size': '2rem' } })
 ```
 */
export function flexCenter(): CssDeclarations {
  return {
    display: 'flex',
    'align-items': 'center',
    'justify-content': 'center',
  };
}

/**
 Horizontal flex row with vertically centered items.
 
 @returns Declarations for row flex container
 
 @example
 ```ts
 $({ rule: '.toolbar', decls: { ...flexRow(), gap: cssRem(1) } })
 ```
 */
export function flexRow(): CssDeclarations {
  return {
    display: 'flex',
    'align-items': 'center',
  };
}

/**
 Vertical flex column.
 
 @returns Declarations for column flex container
 
 @example
 ```ts
 $({ rule: '.sidebar', decls: { ...flexColumn(), gap: cssRem(1) } })
 ```
 */
export function flexColumn(): CssDeclarations {
  return {
    display: 'flex',
    'flex-direction': 'column',
  };
}

//endregion Layout primitives

//region Visual primitives

/**
 Fully-rounded corners (pill / circle shape).
 
 @returns Border-radius declaration
 
 @example
 ```ts
 $({ rule: '.pill', decls: { ...borderRadiusFull() } })
 ```
 */
export function borderRadiusFull(): CssDeclarations {
  return {
    'border-radius': cssRem(PILL_RADIUS,),
  };
}

/**
 Prevents text wrapping.
 
 @returns White-space declaration
 
 @example
 ```ts
 $({ rule: '.label', decls: { ...whitespaceNowrap() } })
 ```
 */
export function whitespaceNowrap(): CssDeclarations {
  return {
    'white-space': 'nowrap',
  };
}

/**
 Hides the scrollbar while preserving scroll behavior.
 
 @returns Scrollbar-width declaration
 
 @example
 ```ts
 $({ rule: '.scroll-container', decls: { ...scrollbarHidden() } })
 ```
 */
export function scrollbarHidden(): CssDeclarations {
  return {
    'scrollbar-width': 'none',
  };
}

//endregion Visual primitives

//region Interactive primitives

/**
 Strips all browser-default button/anchor chrome.
 
 @returns Declarations for unstyled interactive element
 
 @example
 ```ts
 $({ rule: '.icon-button', decls: { ...appearanceNone() } })
 ```
 */
export function appearanceNone(): CssDeclarations {
  return {
    'background-color': 'transparent',
    'border-style': 'none',
    cursor: 'pointer',
    'padding-block': 0,
    'padding-inline': 0,
    'font-family': 'inherit',
    'font-size': 'inherit',
    'font-style': 'inherit',
    'font-weight': 'inherit',
    'line-height': 'inherit',
    color: 'inherit',
  };
}

/**
 Ensures the element meets the 48px (3rem) minimum touch target
 (Material Design) while allowing it to grow if content demands.
 
 @returns Min size declarations
 
 @example
 ```ts
 $({ rule: 'button', decls: { ...minTouchTarget() } })
 ```
 */
export function minTouchTarget(): CssDeclarations {
  return {
    'min-inline-size': cssRem(TOUCH_TARGET,),
    'min-block-size': cssRem(TOUCH_TARGET,),
  };
}

/**
 Light-DOM global resets that shadow DOM cannot inherit.
 
 @returns Array of CSS rule strings for shadow DOM globals
 
 @example
 ```ts
 const styles = [...shadowDomGlobals(), $({ rule: ':host', decls: { display: 'block' } })];
 ```
 */
export function shadowDomGlobals(): string[] {
  return [
    css({
      rule: 'button:disabled',
      decls: {
        opacity: DISABLED_OPACITY,
        cursor: 'not-allowed',
      },
    },),
    css({
      rule: 'input::placeholder,textarea::placeholder',
      decls: { color: cssVar('medium',), },
    },),
  ];
}

//endregion Interactive primitives

//region Composed patterns

/**
 Horizontally scrollable row of inline items.
 
 @returns Declarations for scroll row
 
 @example
 ```ts
 $({ rule: '.chips', decls: { ...scrollRow() } })
 ```
 */
export function scrollRow(): CssDeclarations {
  return {
    display: 'flex',
    gap: cssVar('min-gap',),
    'align-items': 'flex-start',
    'overflow-x': 'auto',
    'overflow-y': 'clip',
    ...scrollbarHidden(),
  };
}

//endregion Composed patterns

//region Focus outline

/**
 Standard focus-visible outline declarations.
 
 @param offset - Outline offset value (default `cssRem(0.125)`)
 
 @returns Declarations for focus outline
 
 @example
 ```ts
 $({ rule: ':focus-visible', decls: { ...focusOutline() } })
 ```
 */
export function focusOutline(
  { offset = cssRem(OUTLINE_WIDTH,), }: { readonly offset?: CssValue; } = {},
): CssDeclarations {
  return {
    'outline-width': cssRem(OUTLINE_WIDTH,),
    'outline-style': 'solid',
    'outline-color': cssVar('fg',),
    'outline-offset': offset,
  };
}

//endregion Focus outline
