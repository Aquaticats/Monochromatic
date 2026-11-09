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
  tokenStyles,
} from '../style/tokens.ts';
import { headFragment, } from './head.ts';

/**
 Head metadata values shared by the fixtures.
 */
const FIXTURE_HEAD = {
  title: 'Home',
  description: 'Welcome to the site',
  canonicalUrl: 'https://aquati.cat/en/',
} as const;

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

  return afterMarker
    .slice(
      0,
      ends.length > 0 ? Math.min(...ends,) : afterMarker.length,
    )
    .trim();
}

await describe({
  name: 'brand colour tokens',
  children: [
    it({
      name: 'declares the light brand colour once',
      fn: async function declaresLightBrandColour(): Promise<void> {
        expect(PRIMARY_LIGHT,).toMatch(/^#[0-9a-f]{6}$/u,);
        expect(declarationValueIn(tokenStyles(), '--primary-light',),).toBe(PRIMARY_LIGHT,);
      },
    },),
    it({
      name: 'declares the dark brand colour once',
      fn: async function declaresDarkBrandColour(): Promise<void> {
        expect(PRIMARY_DARK,).toMatch(/^#[0-9a-f]{6}$/u,);
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
