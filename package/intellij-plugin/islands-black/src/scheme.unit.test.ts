/**
 The shipped editor scheme's colour literals.

 Every colour the scheme declares is written as six hex digits, the form the
 platform's scheme reader parses as red, green and blue bytes, because a value
 of any other shape leaves the reader on a default the theme author never chose.

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

 The same section writes a font style and an effect kind as bare integers,
 which are not colour literals.
 */
const ATTRIBUTE_COLOR_OPTION_NAMES = [
  'FOREGROUND',
  'BACKGROUND',
  'EFFECT_COLOR',
  'ERROR_STRIPE_COLOR',
] as const;

/**
 Hex digits a colour literal holds: one byte each for red, green and blue.
 */
const SIX_DIGIT_HEX_LENGTH = 6;

/**
 Characters a hexadecimal literal may hold, in either case.
 */
const HEX_DIGIT_CHARACTERS = new Set('0123456789abcdefABCDEF',);

/**
 Text every option element starts with.
 */
const OPTION_ELEMENT_START = '<option';

/**
 Start of the attribute holding an option's name.
 */
const NAME_ATTRIBUTE_START = 'name=\'';

/**
 Start of the attribute holding an option's value.
 */
const VALUE_ATTRIBUTE_START = 'value=\'';

/**
 Attribute values in the scheme are single quoted.
 */
const ATTRIBUTE_QUOTE = '\'';

/**
 What `indexOf` returns when the searched text is absent.
 */
const NOT_FOUND = -1;

/**
 Reports whether `indexOf` found nothing.

 @param index - Offset `indexOf` returned

 @returns Whether the searched text is absent

 @example
 ```ts
 isAbsent({ index: NOT_FOUND, }); // true
 ```
 */
function isAbsent({ index, }: Readonly<{ index: number; }>,): boolean {
  return index === NOT_FOUND;
}

/**
 The two scheme sections that carry colour literals.
 */
const COLORS_SECTION_NAME = 'colors';

/**
 The attribute section, whose leaf options hold foregrounds and backgrounds.
 */
const ATTRIBUTES_SECTION_NAME = 'attributes';

type SchemeSections = Readonly<{
  colors: string;
  attributes: string;
}>;

type OptionValue = Readonly<{
  name: string;
  value: string;
}>;

/**
 Reads one attribute of a text span.

 A span without the attribute is a container element rather than a malformed
 file, so the caller probes with `includes` before asking for the value.

 @param text - Span holding an option element's attributes

 @param attributeStart - Attribute prefix, including the opening quote

 @returns The attribute value

 @throws When the span carries no such attribute, or opens the quote and never closes it

 @example
 ```ts
 const name = attributeValue({ text: span, attributeStart: NAME_ATTRIBUTE_START, });
 ```
 */
function attributeValue(
  {
    text,
    attributeStart,
  }: Readonly<{
    text: string;
    attributeStart: string;
  }>,
): string {
  const start = text.indexOf(attributeStart,);
  if (isAbsent({ index: start, },))
    throw new Error(`option span carries no ${attributeStart} attribute`,);
  const valueStart = start + attributeStart.length;
  const end = text.indexOf(ATTRIBUTE_QUOTE, valueStart,);
  if (isAbsent({ index: end, },))
    throw new Error(`option span opens ${attributeStart} and never closes it`,);
  return text.slice(
    valueStart,
    end,
  );
}

/**
 Takes one scheme section, from its opening tag through its closing tag.

 @param text - Whole scheme file

 @param name - Section name, without angle brackets

 @returns The section text

 @throws When the scheme no longer holds the section

 @example
 ```ts
 const colors = sectionText({ text, name: COLORS_SECTION_NAME, });
 ```
 */
function sectionText(
  {
    text,
    name,
  }: Readonly<{
    text: string;
    name: string;
  }>,
): string {
  const start = text.indexOf(`<${name}>`,);
  const end = text.indexOf(`</${name}>`,);
  if (isAbsent({ index: start, },) || isAbsent({ index: end, },))
    throw new Error(`scheme no longer holds a ${name} section, so this test checked nothing`,);
  return text.slice(
    start,
    end,
  );
}

/**
 Splits the scheme into the two sections that hold colour literals.

 @param text - Whole scheme file

 @returns The colors section and the attributes section

 @throws When the scheme no longer holds either section

 @example
 ```ts
 const sections = schemeSections({ text, });
 ```
 */
function schemeSections({ text, }: Readonly<{ text: string; }>,): SchemeSections {
  return {
    colors: sectionText({
      text,
      name: COLORS_SECTION_NAME,
    },),
    attributes: sectionText({
      text,
      name: ATTRIBUTES_SECTION_NAME,
    },),
  };
}

/**
 Reads every leaf option of a scheme section.

 A nested option's own span holds no value attribute, so a span with no value
 attribute is a container and is skipped rather than misread as its child.

 @param section - Section text taken from the scheme

 @returns Name and value of each leaf option, in file order

 @example
 ```ts
 const options = leafOptions({ section: sections.colors, });
 ```
 */
function leafOptions({ section, }: Readonly<{ section: string; }>,): readonly OptionValue[] {
  const spans = section
    .split(OPTION_ELEMENT_START,)
    .slice(1,);
  const options: OptionValue[] = [];
  for (const span of spans) {
    const carriesName = span.includes(NAME_ATTRIBUTE_START,);
    const carriesValue = span.includes(VALUE_ATTRIBUTE_START,);
    if (!carriesName)
      continue;
    if (!carriesValue)
      continue;
    options.push({
      name: attributeValue({
        text: span,
        attributeStart: NAME_ATTRIBUTE_START,
      },),
      value: attributeValue({
        text: span,
        attributeStart: VALUE_ATTRIBUTE_START,
      },),
    },);
  }
  return options;
}

/**
 Reports whether a literal is six hexadecimal digits.

 @param value - Attribute value read from the scheme

 @returns Whether the value is a colour literal of the expected shape

 @example
 ```ts
 isSixDigitHex({ value: '000000', }); // true
 ```
 */
function isSixDigitHex({ value, }: Readonly<{ value: string; }>,): boolean {
  if (value.length !== SIX_DIGIT_HEX_LENGTH)
    return false;
  for (const character of value) {
    if (!HEX_DIGIT_CHARACTERS.has(character,))
      return false;
  }
  return true;
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
      return !isSixDigitHex({ value: option.value, },);
    },)
    .map(function toOffender(option,) {
      return `${option.name}=${option.value}`;
    },);
}

/**
 Reads the shipped scheme once for both cases.

 @returns The scheme's two colour-holding sections

 @example
 ```ts
 const sections = await readSections();
 ```
 */
async function readSections(): Promise<SchemeSections> {
  return schemeSections({ text: await readFile(SCHEME_URL, 'utf8',), },);
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
            const sections = await readSections();
            const options = leafOptions({ section: sections.colors, },);
            expect(options.length,).toBeGreaterThan(0,);
            expect(nonHexColorValues({ options, },),).toEqual([],);
          },
        },),
        it({
          name: 'writes every attribute color as six hex digits',
          fn: async () => {
            const sections = await readSections();
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
