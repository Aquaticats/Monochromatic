/**
 Property-based fuzz tests for the XML attribute coder in `./xml-coding.ts`.
 
 This is a cross-syntax transformer (raw text to double-quoted XML
 attribute value), so the boundary cases matter: the round-trip
 `unescapeXmlAttribute(escapeXmlAttribute(s)) === s` must hold for every
 string, including active delimiters, entity-shaped text, control
 characters, and astral code points; escaped output must never contain a
 raw attribute-breaking character; and `isDigitCodePoint` must agree with
 the ASCII `0`..`9` range.
 
 Only the unescape-after-escape direction round-trips: escaping is not
 surjective onto its own output (for example `&apos;` decodes to `'`,
 which escaping leaves unchanged), so the reverse direction is not a
 property.
 
 Run plan and seed policy: see `../fuzz-budget.ts`.
 
 @module
 */

import {
  assert,
  asyncProperty,
  constantFrom,
  integer,
  oneof,
  string,
} from 'fast-check';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { fuzzRunPlan, } from '../fuzz-budget.ts';
import {
  escapeXmlAttribute,
  isDigitCodePoint,
  unescapeXmlAttribute,
} from './xml-coding.ts';

//region Constants and arbitraries

/**
 Run plan resolved once for every property in this file.
 */
const RUN = fuzzRunPlan();

/**
 Characters escaping must never emit raw in a double-quoted attribute.
 */
const ATTRIBUTE_BREAKING_CHARS = [
  '"',
  '<',
  '>',
  '\n',
  '\r',
  '\t',
] as const;

/**
 ASCII code point of the digit `0`.
 */
const DIGIT_ZERO_CODE_POINT = 48;

/**
 ASCII code point of the digit `9`.
 */
const DIGIT_NINE_CODE_POINT = 57;

/**
 Highest Unicode code point (`U+10FFFF`); upper bound for the digit-range
 property so the whole code space is sampled.
 */
const MAX_UNICODE_CODE_POINT = 1_114_111;

/**
 Arbitrary text rich in characters that exercise the coder's branches:
 the escaped delimiters, entity punctuation, an apostrophe, control
 characters, a no-break space, and an astral code point. Unioned with the
 default string arbitrary for broader coverage.
 */
const attributeTextArbitrary = oneof(
  string(),
  string({
    unit: constantFrom(
      '&',
      '"',
      '<',
      '>',
      '\n',
      '\r',
      '\t',
      '\'',
      'a',
      '#',
      ';',
      'x',
      '0',
      ' ',
      '\u{1F600}',
    ),
  },),
);

//endregion Constants and arbitraries

await describe({
  name: '',
  children: [
    //region Round-trip and delimiter safety

    describe({
      name: escapeXmlAttribute.name,
      children: [
        it({
          name: 'unescape after escape recovers the original string',
          timeout: RUN.timeout,
          fn: async () => {
            await assert(
              asyncProperty(
                attributeTextArbitrary,
                async function roundTrips(value,) {
                  /**
                   Escaped attribute text.
                   */
                  const escaped = escapeXmlAttribute({ value, },);
                  expect(unescapeXmlAttribute({ value: escaped, },),).toBe(value,);
                },
              ),
              RUN.params,
            );
          },
        },),

        it({
          name: 'escaped output contains no raw attribute-breaking character',
          timeout: RUN.timeout,
          fn: async () => {
            await assert(
              asyncProperty(
                attributeTextArbitrary,
                async function noRawDelimiters(value,) {
                  /**
                   Escaped attribute text.
                   */
                  const escaped = escapeXmlAttribute({ value, },);
                  ATTRIBUTE_BREAKING_CHARS.forEach(function absent(delimiter,) {
                    expect(escaped.includes(delimiter,),).toBe(false,);
                  },);
                },
              ),
              RUN.params,
            );
          },
        },),
      ],
    },),

    //endregion Round-trip and delimiter safety

    //region isDigitCodePoint

    describe({
      name: isDigitCodePoint.name,
      children: [
        it({
          name: 'agrees with the ASCII 0..9 range across code points',
          timeout: RUN.timeout,
          fn: async () => {
            await assert(
              asyncProperty(
                integer({
                  min: -10,
                  max: MAX_UNICODE_CODE_POINT,
                },),
                async function matchesRange(codePoint,) {
                  /**
                   Whether the code point falls in the ASCII digit range.
                   */
                  const inRange = (codePoint >= DIGIT_ZERO_CODE_POINT) && (codePoint <= DIGIT_NINE_CODE_POINT);
                  expect(isDigitCodePoint({ codePoint, },),).toBe(inRange,);
                },
              ),
              RUN.params,
            );
          },
        },),
      ],
    },),

    //endregion isDigitCodePoint
  ],
},);
