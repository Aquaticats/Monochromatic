/**
 String-returning hyperscript factory for CSS generation.
 
 Same composable pattern as `h-xml` and `h-html` but produces CSS strings.
 Each call constructs a single node in the CSS tree (a style rule or an
 at-rule) and returns it as a plain string. Nesting is explicit via
 `children`, mirroring native CSS nesting.
 
 Replaces `@mixin`/`@apply` with plain functions that return declaration
 records, composed via object spread.
 
 Types from `csstype` provide editor intellisense for:
 - CSS property names (kebab-case), with disallowed properties omitted
 - CSS property values (keyword autocomplete, named colors excluded)
 - At-rule names (`media`, `layer`, `keyframes`, ...), with disallowed at-rules excluded
 - At-rule descriptors per type (`@font-face`, `@property`, `@counter-style`, ...)
 
 Branded value constructors ({@link cssRem}, {@link cssVar}, {@link cssOklch}, etc.) replace raw
 string values, preventing invalid units, disallowed color functions, and named
 colors at the type level, eliminating the need for runtime lint checks.
 
 @param options - Named parameters describing the CSS construct
 @returns CSS string
 
 @example Style rule with strict values
 ```ts
 const css = $({
   rule: '.card',
   decls: { display: 'flex', gap: cssRem(1), 'background-color': cssVar('bg') },
 });
 ```
 
 @example At-rule with children
 ```ts
 const css = $({
   at: 'media',
   params: '(prefers-color-scheme: dark)',
   children: [
     $({ rule: ':root', decls: { '--color-fg': cssOklch({ l: 0.9, c: 0, h: 0 }) } }),
   ],
 });
 ```
 
 @example At-rule with typed descriptors (`@property`)
 ```ts
 const css = $({
   at: 'property',
   params: '--color-fg',
   decls: { syntax: '"<color>"', inherits: 'true', 'initial-value': 'black' },
 });
 ```
 
 @example Statement at-rule (no block)
 ```ts
 const css = $({ at: 'layer', params: 'tokens, base, components' });
 ```
 */

import {
  buildAtRule,
  buildRule,
  type CssOptions,
} from './index.builders.ts';

//region Re-exports

export type { StrictCssDeclarations as CssDeclarations, } from './properties.ts';
export type { CssValue, } from './values.ts';
export {
  cssAnchor,
  cssCalc,
  cssCh,
  cssClamp,
  cssColorFn,
  cssCommaList,
  cssCompounded,
  cssCqb,
  cssCqi,
  cssCubicBezier,
  cssDvb,
  cssDvi,
  cssEm,
  cssFr,
  cssInt,
  cssLh,
  cssMax,
  cssMin,
  cssNum,
  cssOklch,
  cssOklchFrom,
  cssPercent,
  cssRandom,
  cssRem,
  cssRotate,
  cssS,
  cssScale,
  cssTranslateX,
  cssTranslateY,
  cssTurn,
  cssVar,
  cssVb,
  cssVi,
} from './values.ts';

//endregion

//region Main export

/**
 Creates a CSS string from declarative options.
 
 Accepts either a style rule (`rule` key, built via {@link buildRule}) or an
 at-rule (`at` key, built via {@link buildAtRule}).
 Returns a minified CSS string with no trailing newlines.
 
 @param options - style rule or at-rule configuration
 
 @returns CSS string
 
 @example
 ```ts
 $({ rule: '.card', decls: { display: 'flex' } });
 // '.card{display:flex}'
 ```
 */
/* @__NO_SIDE_EFFECTS__ */ export function $(
  options: CssOptions,
): string {
  if ('rule' in options)
    return buildRule(options,);

  return buildAtRule(options,);
}

//endregion
