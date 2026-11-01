/**
 Tests for the constants of the repeat-group rule: the thresholds the design's
 census fixed, and the character sets the wording reader and the clause cuts
 are made on. The module holds one value built by a call, the break character,
 so the own-test ratchet asks for this file; the relations pinned here are the
 ones the reader and the finders lean on. Fixtures are cat-themed invention only.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { compareCodePoints, } from '@monochromatic-dev/translation-repair-markup';
import {
  BREAK,
  CLAUSE_BOUNDARY,
  MAXIMAL_REPEAT_MINIMUM,
  QUOTE_CLOSERS,
  QUOTE_OPENERS,
  REFRAIN_CLAUSE_MINIMUM,
  REFRAIN_PREFIX_MINIMUM,
  REFRAIN_SLOT_MAXIMUM,
  REFRAIN_SLOT_MINIMUM,
  REFRAIN_SUFFIX_MINIMUM,
  STRAIGHT_QUOTE,
  TAG_LETTERS,
  URL_SCHEMES,
} from '../dist/final/node/index.mjs';

await describe({
  name: 'repeat group bounds',
  concurrency: 1,
  children: [
    it({
      name: 'Holds the thresholds the design\'s census fixed',
      fn: async () => {
        expect({
          maximalRepeatMinimum: MAXIMAL_REPEAT_MINIMUM,
          refrainPrefixMinimum: REFRAIN_PREFIX_MINIMUM,
          refrainSuffixMinimum: REFRAIN_SUFFIX_MINIMUM,
          refrainSlotMinimum: REFRAIN_SLOT_MINIMUM,
          refrainSlotMaximum: REFRAIN_SLOT_MAXIMUM,
          refrainClauseMinimum: REFRAIN_CLAUSE_MINIMUM,
        },).toEqual({
          maximalRepeatMinimum: 12,
          refrainPrefixMinimum: 4,
          refrainSuffixMinimum: 3,
          refrainSlotMinimum: 1,
          refrainSlotMaximum: 8,
          refrainClauseMinimum: 6,
        },);
      },
    },),

    it({
      name: 'Makes the break the NUL character, one UTF-16 unit long',
      fn: async () => {
        expect({
          codePoint: BREAK.codePointAt(0,),
          units: BREAK.length,
        },).toEqual({
          codePoint: 0,
          units: 1,
        },);
      },
    },),

    it({
      name: 'Keeps the break out of every character set the reader scans, so that a break is never read as wording',
      fn: async () => {
        expect([
          CLAUSE_BOUNDARY,
          QUOTE_OPENERS,
          QUOTE_CLOSERS,
          STRAIGHT_QUOTE,
          TAG_LETTERS,
          ...URL_SCHEMES,
        ].map((characters,) => characters.includes(BREAK,),),).toEqual([false, false, false, false, false, false, false,],);
      },
    },),

    it({
      name: 'Keeps the opening and closing quotation marks apart, in equal number, each one UTF-16 unit',
      fn: async () => {
        expect({
          sharedMarks: Array.from(QUOTE_OPENERS).filter((mark,) => QUOTE_CLOSERS.includes(mark,),),
          openers: QUOTE_OPENERS.length,
          closers: QUOTE_CLOSERS.length,
          openerPoints: Array.from(QUOTE_OPENERS).length,
          closerPoints: Array.from(QUOTE_CLOSERS).length,
        },).toEqual({
          sharedMarks: [],
          openers: 3,
          closers: 3,
          openerPoints: 3,
          closerPoints: 3,
        },);
      },
    },),

    it({
      name: 'Keeps the straight double quotation mark out of the paired quotation marks',
      fn: async () => {
        expect({
          straight: STRAIGHT_QUOTE,
          inOpeners: QUOTE_OPENERS.includes(STRAIGHT_QUOTE,),
          inClosers: QUOTE_CLOSERS.includes(STRAIGHT_QUOTE,),
        },).toEqual({
          straight: '"',
          inOpeners: false,
          inClosers: false,
        },);
      },
    },),

    it({
      name: 'Makes every quotation mark a clause boundary, so that a clause never runs into quoted speech',
      fn: async () => {
        expect([...Array.from(QUOTE_OPENERS), ...Array.from(QUOTE_CLOSERS), STRAIGHT_QUOTE,].filter((mark,) => !CLAUSE_BOUNDARY.includes(mark,),),).toEqual([],);
      },
    },),

    it({
      name: 'Lists the Latin letters of a tag name, both cases and nothing else',
      fn: async () => {
        /** The twenty-six letters of the alphabet in each case. */
        const letters = Array.from({ length: 26, }, (_, index,) => String.fromCodePoint(65 + index,),);

        expect(Array.from(TAG_LETTERS).toSorted((left, right,) => compareCodePoints({ left, right, },),),).toEqual([
          ...letters,
          ...letters.map((letter,) => letter.toLowerCase(),),
        ].toSorted((left, right,) => compareCodePoints({ left, right, },),),);
      },
    },),

    it({
      name: 'Lists the two schemes a bare URL begins with',
      fn: async () => {
        expect(URL_SCHEMES.toSorted((left, right,) => compareCodePoints({ left, right, },),),).toEqual(['http://', 'https://',],);
      },
    },),
  ],
},);
