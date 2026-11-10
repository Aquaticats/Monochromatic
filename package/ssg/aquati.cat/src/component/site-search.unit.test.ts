/**
 Tests for the site-search component styles.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  siteSearchStyles,
  tokenStyles,
} from '../../dist/final/node/index.mjs';

/**
 Marker prefixing every emitted outline declaration.
 */
const OUTLINE_MARKER = 'outline-';

/**
 Fragments that spell a colour literal instead of referencing a token.
 */
const COLOUR_LITERALS = [
  'rgb(',
  'hsl(',
  '#',
] as const;

/**
 Stylesheet that declares the shared colour tokens.
 */
const TOKEN_STYLES = tokenStyles();

/**
 Token name the dropdown shadow reads its colour from.
 */
const SHADOW_TOKEN = '--color-shadow';

await describe({
  name: 'site-search styles',
  children: [
    it({
      name: 'leaves the focus ring to the site-wide :focus-visible rule',
      fn: async function leavesFocusRingToGlobalRule(): Promise<void> {
        /**
         Generated search stylesheet.
         */
        const styles = siteSearchStyles();

        expect(styles,).not.toContain(OUTLINE_MARKER,);
      },
    },),
    it({
      name: 'takes the dropdown shadow colour from a token',
      fn: async function takesShadowColourFromToken(): Promise<void> {
        /**
         Generated search stylesheet.
         */
        const styles = siteSearchStyles();

        expect(styles,).toContain(`var(${SHADOW_TOKEN})`,);
        expect(TOKEN_STYLES,).toContain(`${SHADOW_TOKEN}:`,);
      },
    },),
    it({
      name: 'spells no raw colour literal',
      fn: async function spellsNoRawColour(): Promise<void> {
        /**
         Generated search stylesheet.
         */
        const styles = siteSearchStyles();

        for (const literal of COLOUR_LITERALS)
          expect(styles,).not.toContain(literal,);
      },
    },),
  ],
},);
