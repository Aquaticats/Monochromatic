/**
 Tests that the brand colours have one owner and reach every consumer.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  PRIMARY_DARK,
  PRIMARY_LIGHT,
  headFragment,
  tokenStyles,
} from '../../dist/final/node/index.mjs';

/**
 Head metadata values shared by the fixtures.
 */
const FIXTURE_HEAD = {
  title: 'Home',
  description: 'Welcome to the site',
  canonicalUrl: 'https://aquati.cat/en/',
} as const;

/**
 Length of a `#rrggbb` colour literal.
 */
const HEX_COLOUR_LENGTH = 7;

/**
 Digits a hexadecimal colour literal may use, in both cases.
 */
const HEX_DIGITS = '0123456789abcdefABCDEF';

/**
 Checks a value is a `#rrggbb` colour literal.
 
 @param value - candidate colour string
 
 @returns `true` when the value is a six-digit hexadecimal colour
 
 @example
 ```ts
 isHexColour('#bf97e3');
 // true
 ```
 */
function isHexColour(value: string,): boolean {
  if ((value.length !== HEX_COLOUR_LENGTH) || (!value.startsWith('#',)))
    return false;
  /* Split, not spread: these are ASCII hex digits, so no grapheme cluster can straddle a divider. */
  return value.slice(1,)
    .split('',)
    .every(function isHexDigit(digit,) {
      return HEX_DIGITS.includes(digit,);
    },);
}

/**
 Reads one custom-property value out of a generated CSS block without a regex.
 
 @param styles - CSS text containing the declaration
 
 @param property - custom property name, including the leading `--`
 
 @returns the declared value, or the empty string when the property is absent
 
 @example
 ```ts
 declarationValueIn(':root{--primary-light:#fff;}', '--primary-light');
 // '#fff'
 ```
 */
function declarationValueIn(
  styles: string,
  property: string,
): string {
  /**
   Offset of the declaration, or `-1` when the property is absent.
   */
  const markerAt = styles.indexOf(`${property}:`,);
  if (markerAt === (-1))
    return '';
  /**
   Text after the property name up to the end of its declaration.
   */
  const afterMarker = styles.slice(markerAt + property.length + 1,);
  /**
   Offsets of the delimiters that can end a declaration; `-1` means absent.
   */
  const ends = [
    afterMarker.indexOf(';',),
    afterMarker.indexOf('}',),
  ]
    .filter(function isPresent(index,) {
      return index !== (-1);
    },);

  /**
   Value text between the property name and the end of its declaration.
   */
  const valueText = afterMarker.slice(
    0,
    ends.length > 0 ? Math.min(...ends,) : afterMarker.length,
  );

  return valueText.trim();
}

await describe({
  name: 'brand colour tokens',
  children: [
    it({
      name: 'declares the light brand colour once',
      fn: async function declaresLightBrandColour(): Promise<void> {
        expect(isHexColour(PRIMARY_LIGHT,),).toBe(true,);
        expect(declarationValueIn(tokenStyles(), '--primary-light',),).toBe(PRIMARY_LIGHT,);
      },
    },),
    it({
      name: 'declares the dark brand colour once',
      fn: async function declaresDarkBrandColour(): Promise<void> {
        expect(isHexColour(PRIMARY_DARK,),).toBe(true,);
        expect(declarationValueIn(tokenStyles(), '--primary-dark',),).toBe(PRIMARY_DARK,);
      },
    },),
    it({
      name: 'feeds the theme-color metas from the same tokens',
      fn: async function feedsThemeColorMetas(): Promise<void> {
        /**
         Rendered head element for the fixture page.
         */
        const head = headFragment({
          ...FIXTURE_HEAD,
          lang: 'en',
        },);

        expect(head,).toContain(`name="theme-color" content="${PRIMARY_LIGHT}"`,);
        expect(head,).toContain(`name="theme-color" content="${PRIMARY_DARK}"`,);
      },
    },),
  ],
},);
