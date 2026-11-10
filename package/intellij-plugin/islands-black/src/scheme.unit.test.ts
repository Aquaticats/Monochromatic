/**
 The shipped editor scheme's colour literals.

 Every colour the scheme declares is written as six hex digits, the form the
 platform's scheme reader accepts, because a reader that sees any other length
 falls back to a default the theme author never chose.

 @module
 */

import { readFile, } from 'node:fs/promises';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

/**
 Shipped editor colour scheme this package installs.
 */
const SCHEME_URL = new URL('../scheme/Islands_Black.xml', import.meta.url,);

/**
 Option names whose value is a colour inside the attributes section.

 The same options elsewhere in that section hold a font style or an effect
 kind, as bare integers.
 */
const ATTRIBUTE_COLOR_OPTION_NAMES = [
  'FOREGROUND',
  'BACKGROUND',
  'EFFECT_COLOR',
  'ERROR_STRIPE_COLOR',
] as const;

/**
 A colour literal the scheme reader parses as red, green and blue bytes.

 A single hex digit does not match, which is the whole point: the reader pads
 nothing and silently keeps its own default instead.
 */
const SIX_DIGIT_HEX = /^[\dA-Fa-f]{6}$/u;

/**
 A leaf option carrying a name and a value, as the scheme writes both.
 */
const LEAF_OPTION = /<option\s+name='([^']+)'\s+value='([^']*)'\s*\/>/gu;

type SchemeSections = Readonly<{
  colors: string;
  attributes: string;
}>;

type OptionValue = Readonly<{
  name: string;
  value: string;
}>;

/**
 Splits the scheme into the two sections that hold colour literals.

 @param text - Whole scheme file

 @returns The colors section and the attributes section, each with its tags

 @throws When the scheme no longer holds either section

 @example
 ```ts
 const sections = schemeSections({ text, });
 ```
 */
function schemeSections({ text, }: Readonly<{ text: string; }>,): SchemeSections {
  const colorsStart = text.indexOf('<colors>',);
  const colorsEnd = text.indexOf('</colors>',);
  const attributesStart = text.indexOf('<attributes>',);
  const attributesEnd = text.indexOf('</attributes>',);
  if (colorsStart < 0 || colorsEnd < 0 || attributesStart < 0 || attributesEnd < 0) {
    throw new Error(
      'scheme no longer holds a colors and an attributes section, so this test checked nothing',
    );
  }
  return {
    colors: text.slice(
      colorsStart,
      colorsEnd,
    ),
    attributes: text.slice(
      attributesStart,
      attributesEnd,
    ),
  };
}

/**
 Reads every leaf option of a scheme section.

 @param section - Section text taken from the scheme

 @returns Name and value of each leaf option, in file order

 @example
 ```ts
 const options = leafOptions({ section: sections.colors, });
 ```
 */
function leafOptions({ section, }: Readonly<{ section: string; }>,): readonly OptionValue[] {
  return [...section.matchAll(LEAF_OPTION,),].map(function toOptionValue(match,) {
    const [, name = '', value = '',] = match;
    return {
      name,
      value,
    };
  },);
}

/**
 Narrows colour options to the ones whose value is not six hex digits.

 An empty value is not a literal: the scheme uses it to say "unset", which
 leaves the parent scheme's colour in place.

 @param options - Leaf options taken from a scheme section

 @returns `NAME=value` for each option with a value of any other shape

 @example
 ```ts
 const offenders = nonHexColorValues({ options, });
 ```
 */
function nonHexColorValues({ options, }: Readonly<{ options: readonly OptionValue[]; }>,): readonly string[] {
  return options
    .filter(function isSetColor(option,) {
      return option.value !== '';
    },)
    .filter(function isNotSixDigitHex(option,) {
      return !SIX_DIGIT_HEX.test(option.value,);
    },)
    .map(function toOffender(option,) {
      return `${option.name}=${option.value}`;
    },);
}

await describe({
  name: '',
  children: [
    describe({
      name: 'Islands_Black.xml',
      children: [
        it({
          name: 'writes every color-section value as six hex digits',
          fn: async () => {
            const sections = schemeSections({ text: await readFile(SCHEME_URL, 'utf8',), },);
            const options = leafOptions({ section: sections.colors, },);
            expect(options.length,).toBeGreaterThan(0,);
            expect(nonHexColorValues({ options, },),).toEqual([],);
          },
        },),
        it({
          name: 'writes every attribute color as six hex digits',
          fn: async () => {
            const sections = schemeSections({ text: await readFile(SCHEME_URL, 'utf8',), },);
            const options = leafOptions({ section: sections.attributes, },)
              .filter(function isColorOption(option,) {
                return (ATTRIBUTE_COLOR_OPTION_NAMES as readonly string[]).includes(option.name,);
              },);
            expect(options.length,).toBeGreaterThan(0,);
            expect(nonHexColorValues({ options, },),).toEqual([],);
          },
        },),
      ],
    },),
  ],
},);
