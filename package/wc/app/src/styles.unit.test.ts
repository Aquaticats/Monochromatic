/**
 Tests for the complete stylesheet assembly.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { renderStyles, } from './styles.ts';

/**
 Fixture base64 payload standing in for the subsetted font bytes.
 */
const FONT_FIXTURE_BASE64 = 'AAEC';

await describe({
  name: renderStyles.name,
  children: [
    it({
      name: 'uses flexbox for the layout, tiles, and frequency rows, never CSS grid',
      fn: async function usesFlexboxNeverGrid(): Promise<void> {
        /**
         Complete stylesheet string.
         */
        const css = renderStyles({ fontWoff2Base64: FONT_FIXTURE_BASE64, },);

        expect(css,).toContain('.layout{display:flex',);
        expect(css,).toContain('.tiles{display:flex',);
        expect(css,).toContain('.frequency-row{display:flex',);
        expect(css,).not
          .toContain('display:grid',);
      },
    },),
    it({
      name: 'embeds the font bytes as a woff2 data URI in an Inter font-face',
      fn: async function embedsFontDataUri(): Promise<void> {
        /**
         Complete stylesheet string.
         */
        const css = renderStyles({ fontWoff2Base64: FONT_FIXTURE_BASE64, },);

        expect(css,).toContain('@font-face',);
        expect(css,).toContain(
          `url('data:font/woff2;base64,${FONT_FIXTURE_BASE64}')`,
        );
        expect(css,).toContain("font-family:'Inter'",);
      },
    },),
    it({
      name: 'includes the wide-viewport media query and the dark palette override',
      fn: async function includesMediaQueryAndPalette(): Promise<void> {
        /**
         Complete stylesheet string.
         */
        const css = renderStyles({ fontWoff2Base64: FONT_FIXTURE_BASE64, },);

        expect(css,).toContain('@media (min-width: 64rem)',);
        expect(css,).toContain('--color-fg:',);
        expect(css,).toContain('@media (prefers-color-scheme: dark)',);
      },
    },),
    it({
      name: 'contains per-row rendering containment and tabular numerals for frequency',
      fn: async function containsFrequencyRendering(): Promise<void> {
        /**
         Complete stylesheet string.
         */
        const css = renderStyles({ fontWoff2Base64: FONT_FIXTURE_BASE64, },);

        expect(css,).toContain('content-visibility:auto',);
        expect(css,).toContain('contain-intrinsic-block-size:auto',);
        expect(css,).toContain('font-variant-numeric:tabular-nums',);
      },
    },),
    it({
      name: 'keeps the frequency bars grayscale with a transparent track in both engines',
      fn: async function keepsBarsGrayscale(): Promise<void> {
        /**
         Complete stylesheet string.
         */
        const css = renderStyles({ fontWoff2Base64: FONT_FIXTURE_BASE64, },);

        // The transparent track is author styling, which drops both
        // engines into their unthemed fallback rendering where
        // accent-color is inert (kept anyway for engines that honor
        // it there); the vendor pseudos below style that fallback.
        // See doc/troubleshooting/progress-element-fill-styling.md.
        expect(css,).toContain('accent-color:var(--color-fg-strong)',);
        expect(css,).toContain('background-color:transparent',);
        // Chromium's fallback is a green fill on a gray track, so
        // both webkit pseudos are pinned.
        expect(css,).toContain(
          '.freq-bar::-webkit-progress-bar{background-color:transparent}',
        );
        expect(css,).toContain(
          '.freq-bar::-webkit-progress-value{background-color:var(--color-fg-strong)}',
        );
        // Firefox's fallback is a UA-blue fill, so the moz fill
        // pseudo is pinned too.
        expect(css,).toContain(
          '.freq-bar::-moz-progress-bar{background-color:var(--color-fg-strong)}',
        );
      },
    },),
    it({
      name: 'ships the inclusively-hidden utility for the frequency header row',
      fn: async function shipsVisuallyHidden(): Promise<void> {
        /**
         Complete stylesheet string.
         */
        const css = renderStyles({ fontWoff2Base64: FONT_FIXTURE_BASE64, },);

        expect(css,).toContain('.visually-hidden:not(:focus):not(:active){',);
        expect(css,).toContain('clip-path:inset(50%)',);
        expect(css,).toContain('position:absolute',);
      },
    },),
  ],
},);
