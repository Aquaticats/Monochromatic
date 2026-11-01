/**
 Type-safe hyperscript factories for declarative HTML, CSS, DOM, and XML generation.
 
 Each factory function builds strings (or DOM elements) from a named-parameter
 options object, replacing manual template literals with composable, type-checked calls.
 
 @example HTML generation
 ```ts
 import { hHtml, hCss, cssRem } from '@monochromatic-dev/hyperscript-core';
 
 const card = hHtml({ tag: 'div', class: 'card', text: 'hello' });
 const styles = hCss({ rule: '.card', decls: { gap: cssRem(1) } });
 ```
 */

//region h-css

export { $ as hCss, } from './css/index.ts';
export type { CssDeclarations, } from './css/index.ts';
export type { CssValue, } from './css/index.ts';
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
} from './css/index.ts';

//endregion

//region h-dom

export { $ as hDom, } from './dom/index.ts';

//endregion

//region h-html

export {
  $ as hHtml,
  escapeHtml,
  VOID_ELEMENTS,
} from './html/index.ts';

//endregion

//region h-xml

export { $ as hXml, } from './xml/index.ts';

//endregion
