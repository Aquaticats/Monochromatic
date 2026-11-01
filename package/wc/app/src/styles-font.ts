/**
 Inter `@font-face` for the wc text-stats tool.
 
 The subsetted variable woff2 (`public/inter.woff2`, produced by
 `src/subset-fonts.ts` via `mise run format:fonts`) is inlined as a
 base64 data URI so the final HTML stays a single self-contained file
 with zero network requests. Variable axes are preserved, so one rule
 covers weights 100 through 900.
 */
import { hCss as $, } from '@monochromatic-dev/hyperscript-core/ts';

/**
 Generates the `@font-face` rule embedding the subsetted Inter variable
 font.
 
 User text in scripts outside the subset (the charset is page chrome
 plus printable ASCII) falls back to `system-ui` via the body font
 stack; only page chrome is guaranteed to render in Inter.
 
 @param fontWoff2Base64 - base64-encoded subsetted Inter woff2 bytes
 
 @returns CSS string for the `@font-face` rule
 
 @example
 ```ts
 const fontFace = renderFontFace({ fontWoff2Base64: 'd09GMg…' });
 ```
 */
export function renderFontFace(
  { fontWoff2Base64, }: Readonly<{ fontWoff2Base64: string; }>,
): string {
  return $(
    {
      at: 'font-face',
      decls: {
        'font-family': "'Inter'",
        'font-style': 'normal',
        'font-weight': '100 900',
        'font-display': 'swap',
        src: `url('data:font/woff2;base64,${fontWoff2Base64}') format('woff2')`,
      },
    },
  );
}
