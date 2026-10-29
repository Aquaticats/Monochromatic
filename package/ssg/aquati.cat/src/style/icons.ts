/**
 Material Symbols icon font `@font-face` and utility class.
 
 `public/materialSymbols.woff2` is a pre-subsetted woff2 (a few KB)
 containing only the icons referenced through `icon('...')` in
 components, preserving every variation axis (opsz, wght, FILL,
 GRAD) so icons can be tuned per context. The full upstream font
 lives in `fonts-source/` and is never shipped directly.
 
 Icons are rendered by **PUA codepoint**, not by ligature. Use the
 `icon(name)` helper from `src/lib/icon/icon.ts` as the text child
 of a `<span class="material-symbols-outlined">`. The helper looks
 the name up in the upstream codepoints data
 (`src/lib/icon/material-symbols-outlined.codepoints`) and emits
 the single-codepoint string. Rendering by codepoint, rather
 than by letting the browser shape a ligature from the icon name,
 is what makes tight subsetting possible: harfbuzz only retains
 the specific glyphs whose codepoints appear in the subset text
 and does not over-include everything spellable from the letters
 in the name.
 
 To add an icon: call `icon('home')` inside a component, then run
 `mise run format:fonts` to re-subset. The source scan picks up
 the literal name. An unknown name throws at `format:fonts` time.
 
 The full font in `fonts-source/materialSymbols.woff2` was fetched
 once from the Google Fonts CSS API without `icon_names`:
 
 ```
 https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:opsz,wght,FILL,GRAD@20..48,100..700,0..1,-50..200
 ```
 
 @example
 ```ts
 import { icon, } from '../lib/icon/icon.ts';
 h({
   tag: 'span',
   class: 'material-symbols-outlined',
   text: icon('invert_colors',),
 });
 ```
 */
import {
  cssRem,
  type CssValue,
  hCss as $,
} from '@monochromatic-dev/hyperscript-core/ts';

/**
 Generates the `@font-face` declaration and `.material-symbols-outlined`
 utility class for Material Symbols Outlined.
 
 Placed first in the CSS cascade so the font is declared before any
 rules that reference it.
 
 @returns CSS string with font-face and utility class
 
 @example
 ```ts
 const css = iconFontStyles();
 ```
 */
export function iconFontStyles(): string {
  return [
    $({
      at: 'font-face',
      decls: {
        'font-family': "'Material Symbols Outlined'",
        'font-style': 'normal',
        'font-weight': '100 700',
        'font-display': 'swap',
        src: "url('/materialSymbols.woff2') format('woff2')",
      },
    },),
    /* oxlint-disable no-unsafe-type-assertion -- icon utility class mixes standard keywords with values that lack branded constructors */
    $({
      rule: '.material-symbols-outlined',
      decls: {
        'font-family': "'Material Symbols Outlined'" as CssValue,
        'font-weight': 'normal' as CssValue,
        'font-style': 'normal' as CssValue,
        'font-size': cssRem(1 + ((1 / 2) / 2),),
        'line-height': '1' as CssValue,
        display: 'inline-block',
        'white-space': 'nowrap',
      },
    },),
    /* oxlint-enable no-unsafe-type-assertion */
  ]
    .join('\n',);
}
