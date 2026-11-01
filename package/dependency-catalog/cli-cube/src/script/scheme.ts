/**
 Colour-scheme detection for the deck.gl scene chrome.
 
 The HTML page respects `prefers-color-scheme: dark` for its CSS
 variables, but the deck.gl scene's text/axis/origin colours are
 passed to layer factories as raw `[r, g, b, a]` tuples; there's no
 CSS variable resolution inside the canvas. This module picks the
 right palette once at session start by branching on
 `globalThis.matchMedia('(prefers-color-scheme: dark)')`.
 
 Threading the result through {@link ../deck-config.ts#buildLayers}
 keeps the layer factories pure: they accept colours, they don't
 call `matchMedia` themselves. This makes them testable in node and
 lets future iterations swap palettes (high-contrast mode, theme
 pickers) without touching the factories.
 
 @example
 ```ts
 import { detectScheme } from './scheme.ts';
 
 const chrome = detectScheme();
 const layers = buildLayers({ probes, state, visibleIndices, bounds, chrome });
 ```
 
 @module
 */

//region Types

/**
 Discrete RGBA colour tuple; matches the shape every deck.gl
 accessor (`getColor`, `getFillColor`, `getLineColor`) expects when
 fed a constant per-layer colour.
 */
export type RgbaColor = readonly [
  number,
  number,
  number,
  number,
];

/**
 Palette of colours used to render the scene chrome. The layer
 factories pick one field each:
 
 - `axis`: axis shaft `PathLayer` and cone `SimpleMeshLayer` colour.
 - `axisTick`: tick-mark `PathLayer` colour. Slightly muted vs the shafts.
 - `axisLabel`: `X` / `Y` / `Z` capitals and dim-name subtitles.
 - `originLabel`: the `O` at the min corner.
 - `nameLabel`: per-glyph package-name labels.
 */
export type ChromeColors = {
  readonly axis: RgbaColor;
  readonly axisTick: RgbaColor;
  readonly axisLabel: RgbaColor;
  readonly originLabel: RgbaColor;
  readonly nameLabel: RgbaColor;
};

//endregion Types

//region Palettes

/* oxlint-disable eslint/no-magic-numbers, stylistic/array-element-per-line -- 8-bit RGBA components within fixed 4-element tuples; vertical splits per channel make the palette table unreadable. */
/**
 Dark-mode palette; light tones so chrome reads against the
 dark page background (`--bg-page: #0f0f0f`).
 */
const DARK_CHROME: ChromeColors = {
  axis: [210, 210, 210, 255,],
  axisTick: [180, 180, 180, 255,],
  axisLabel: [235, 235, 235, 255,],
  originLabel: [235, 235, 235, 255,],
  nameLabel: [210, 210, 210, 255,],
};

/**
 Light-mode palette; dark tones so chrome reads against the
 light page background (`--bg-page: #fafafa`).
 */
const LIGHT_CHROME: ChromeColors = {
  axis: [40, 40, 40, 255,],
  axisTick: [60, 60, 60, 255,],
  axisLabel: [30, 30, 30, 255,],
  originLabel: [80, 80, 80, 255,],
  nameLabel: [50, 50, 50, 255,],
};
/* oxlint-enable eslint/no-magic-numbers, stylistic/array-element-per-line */

//endregion Palettes

//region Public API

/**
 Returns the active {@link ChromeColors} palette based on the
 browser's preferred colour scheme.
 
 Reads `globalThis.matchMedia('(prefers-color-scheme: dark)').matches`
 exactly once; the result is captured in the controller and reused
 for every `setProps` cycle. We don't listen for scheme changes
 mid-session; re-detecting on every render would require teardown
 of the colour-baked vertex buffers anyway, and the user can reload
 the page if they flip OS theme.
 
 @returns Dark palette when `prefers-color-scheme: dark` matches; light palette otherwise.
 
 @example
 ```ts
 const { axisLabel } = detectScheme();
 // [235, 235, 235, 255] under dark mode; [30, 30, 30, 255] under light.
 ```
 */
export function detectScheme(): ChromeColors {
  /**
   True when the OS or browser is currently in dark mode; captured once per session, no live listener.
   */
  const isDark = globalThis.matchMedia('(prefers-color-scheme: dark)',)
    .matches;
  return isDark ? DARK_CHROME : LIGHT_CHROME;
}

//endregion Public API
