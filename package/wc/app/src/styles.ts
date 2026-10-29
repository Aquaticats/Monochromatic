/**
 CSS stylesheet for the wc text-stats tool.
 
 Composes the five-stop grayscale palette (`./styles-colors.ts`), the
 embedded Inter `@font-face` (`./styles-font.ts`), the page scaffold
 and input panel (`./styles-layout.ts`), and the tiles and Frequency
 table (`./styles-results.ts`).
 */
import {
  renderDarkColors,
  renderRootColors,
} from './styles-colors.ts';
import { renderFontFace, } from './styles-font.ts';
import { renderLayoutStyles, } from './styles-layout.ts';
import { renderResultsStyles, } from './styles-results.ts';

export { WIDE_VIEWPORT_REM, } from './styles-layout.ts';

/**
 Generates the complete CSS stylesheet for the wc tool.
 
 @param fontWoff2Base64 - base64-encoded subsetted Inter woff2 bytes,
 inlined as a data URI by {@link renderFontFace}
 
 @returns minified CSS string
 
 @example
 ```ts
 const css = renderStyles({ fontWoff2Base64: 'd09GMg…' });
 ```
 */
export function renderStyles(
  { fontWoff2Base64, }: Readonly<{ fontWoff2Base64: string; }>,
): string {
  return [
    renderFontFace({ fontWoff2Base64, },),
    renderRootColors(),
    renderDarkColors(),
    renderLayoutStyles(),
    renderResultsStyles(),
  ]
    .join('',);
}
