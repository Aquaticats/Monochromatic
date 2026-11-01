/**
 Light/dark color-scheme custom properties for the wc text-stats tool.
 
 The palette is strictly five opaque grayscale stops,
 `oklch(L 0 0)` for L in 0, 0.1, 0.5, 0.9, 1: no other lightness values,
 no alpha. Subtlety comes from adjacent stops (0.9 on 1 in light mode,
 0 on 0.1 in dark mode) instead of translucency.
 
 Contrast discipline: every text role must reach WCAG AAA small-text
 contrast (7:1). For achromatic oklch the WCAG relative luminance is
 L cubed, so mid-gray text peaks at 6:1 on white and 3.4:1 on the dark
 background; only near-black/black ink on near-white/white paper (and
 the dark-mode mirror) clears 7:1, all at 15:1 or higher. The mid stop
 is therefore reserved for non-text borders.
 
 Colors are CSS custom properties with light defaults, overridden inside
 a `prefers-color-scheme: dark` block, so the page follows the OS theme
 automatically with no client-side toggle.
 */
import {
  cssCompounded,
  cssOklch,
  hCss as $,
} from '@monochromatic-dev/hyperscript-core/ts';

/**
 Lightness of the black stop.
 */
const L_BLACK = 0;

/**
 Lightness of the near-black stop.
 */
const L_NEAR_BLACK = 0.1;

/**
 Lightness of the mid-gray stop.
 */
const L_MID = 0.5;

/**
 Lightness of the near-white stop.
 */
const L_NEAR_WHITE = 0.9;

/**
 Lightness of the white stop.
 */
const L_WHITE = 1;

/**
 Black stop: light-mode strong foreground, dark-mode surface.
 */
const STOP_BLACK = cssOklch(
  {
    l: L_BLACK,
    c: 0,
    h: 0,
  },
);

/**
 Near-black stop: every light-mode text role (body, muted, placeholder);
 dark-mode background.
 */
const STOP_NEAR_BLACK = cssOklch(
  {
    l: L_NEAR_BLACK,
    c: 0,
    h: 0,
  },
);

/**
 Mid-gray stop: non-text borders only (textarea). As text it peaks at
 6:1 on white, under the AAA small-text floor of 7:1, so no text role
 may use it.
 */
const STOP_MID = cssOklch(
  {
    l: L_MID,
    c: 0,
    h: 0,
  },
);

/**
 Near-white stop: light-mode surface; every dark-mode text role.
 */
const STOP_NEAR_WHITE = cssOklch(
  {
    l: L_NEAR_WHITE,
    c: 0,
    h: 0,
  },
);

/**
 White stop: light-mode background, dark-mode strong foreground.
 */
const STOP_WHITE = cssOklch(
  {
    l: L_WHITE,
    c: 0,
    h: 0,
  },
);

/**
 Declares `:root`'s light-theme custom properties (the defaults) plus
 `color-scheme: light dark` so native form controls and scrollbars also
 follow the OS theme.
 
 Semantic roles: `bg` page background; `surface` tile fill one stop off
 `bg`; `fg` body text; `fg-strong` headline numbers, focus rings, and
 the frequency-bar fill (via the vendor fill pseudos in
 `./styles-results.ts`); `muted` and `placeholder` secondary
 text, sharing `fg`'s stop because the mid stop cannot reach AAA
 small-text contrast on any allowed background (secondary text
 differentiates by size and weight instead of color); `border-subtle`
 hairlines one stop off `bg`; `border-strong` mid-stop non-text
 borders (textarea).
 
 @returns CSS rule string for the `:root` custom-property declarations
 
 @example
 ```ts
 const rootColors = renderRootColors();
 ```
 */
export function renderRootColors(): string {
  return $(
    {
      rule: ':root',
      decls: {
        'color-scheme': cssCompounded(
          [
            'light',
            'dark',
          ],
        ),
        '--color-bg': STOP_WHITE,
        '--color-surface': STOP_NEAR_WHITE,
        '--color-fg': STOP_NEAR_BLACK,
        '--color-fg-strong': STOP_BLACK,
        '--color-muted': STOP_NEAR_BLACK,
        '--color-border-subtle': STOP_NEAR_WHITE,
        '--color-border-strong': STOP_MID,
        '--color-placeholder': STOP_NEAR_BLACK,
      },
    },
  );
}

/**
 Declares the `prefers-color-scheme: dark` override for every custom
 property {@link renderRootColors} declares: the same five stops with
 roles mirrored (background near-black, surfaces black, text near-white).
 
 @returns CSS at-rule string for the dark-theme override
 
 @example
 ```ts
 const darkColors = renderDarkColors();
 ```
 */
export function renderDarkColors(): string {
  return $(
    {
      at: 'media',
      params: '(prefers-color-scheme: dark)',
      children: [
        $(
          {
            rule: ':root',
            decls: {
              '--color-bg': STOP_NEAR_BLACK,
              '--color-surface': STOP_BLACK,
              '--color-fg': STOP_NEAR_WHITE,
              '--color-fg-strong': STOP_WHITE,
              '--color-muted': STOP_NEAR_WHITE,
              '--color-border-subtle': STOP_BLACK,
              '--color-border-strong': STOP_MID,
              '--color-placeholder': STOP_NEAR_WHITE,
            },
          },
        ),
      ],
    },
  );
}
