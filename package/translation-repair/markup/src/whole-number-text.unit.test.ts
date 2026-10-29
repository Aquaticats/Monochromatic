/**
 Tests for the one rule every count reader in the package shares (ledger
 B73): ASCII digits that a double holds exactly, and nothing `Number` would
 also read.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  isDecimalText,
  isNegativeWholeNumberText,
  isUnsignedNumberText,
  isWholeNumberText,
  WHOLE_NUMBER_RULE,
} from '../dist/final/node/index.mjs';

/**
 Largest whole number a double holds exactly, as digits.
 */
const LARGEST = String(Number.MAX_SAFE_INTEGER,);

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: isWholeNumberText.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'TAKES digits, a zero, a leading zero and the largest whole number a double holds exactly',
          fn: async () => {
            expect(['0', '12', '04', LARGEST,].map(function taken(text,): boolean {
              return isWholeNumberText({ text, },);
            },),).toEqual([true, true, true, true,],);
          },
        },),
        it({
          name: 'REFUSES what `Number` would read as a number nobody wrote: empty, blank, a sign, a point, an '
            + 'exponent, a radix, a space either side, a full-width digit, and one past the exact range',
          fn: async () => {
            /**
             Texts `Number` reads, or reads as something else, that no count here is written as.
             */
            const refused = [
              '',
              ' ',
              '-1',
              '+1',
              '1.0',
              '1e3',
              '0x1F',
              '0b1',
              '0o7',
              ' 1',
              '1 ',
              '１',
              'Infinity',
              String(BigInt(Number.MAX_SAFE_INTEGER,) + 1n,),
            ];
            expect(refused.filter(function taken(text,): boolean {
              return isWholeNumberText({ text, },);
            },),).toEqual([],);
          },
        },),
        it({
          name: 'NAMES its rule with the limit spelled out, for every refusal that cites it',
          fn: async () => {
            expect(WHOLE_NUMBER_RULE,).toBe(`a whole number written in digits, at most ${LARGEST}`,);
          },
        },),
      ],
    },),

    describe({
      name: isNegativeWholeNumberText.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'TAKES a minus sign before digits above zero, a leading zero among them, up to the exact range',
          fn: async () => {
            expect(['-1', '-3', '-04', `-${LARGEST}`,].map(function taken(text,): boolean {
              return isNegativeWholeNumberText({ text, },);
            },),).toEqual([true, true, true, true,],);
          },
        },),
        it({
          name: 'REFUSES minus zero, which names no count below zero, a bare sign, a doubled or mixed sign, a space, '
            + 'a point, an exponent, a typographic minus, digits with no sign, and one past the exact range',
          fn: async () => {
            /**
             Texts that are no minus sign before a whole number above zero.
             */
            const refused = [
              '-0',
              '-00',
              '-',
              '',
              '--3',
              '-+3',
              '- 3',
              '-3 ',
              '-3.0',
              '-1e3',
              '−3',
              '3',
              `-${String(BigInt(Number.MAX_SAFE_INTEGER,) + 1n,)}`,
            ];
            expect(refused.filter(function taken(text,): boolean {
              return isNegativeWholeNumberText({ text, },);
            },),).toEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: isDecimalText.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'TAKES digits, and digits, a point and digits, leading and trailing zeros included, and a digit run '
            + 'past the largest double, which the rule leaves to its readers to check for finite',
          fn: async () => {
            expect(['0', '7', '7.5', '0.000003', '007.50', `1${'0'.repeat(400,)}`,].map(function taken(text,): boolean {
              return isDecimalText({ text, },);
            },),).toEqual([true, true, true, true, true, true,],);
          },
        },),
        it({
          name: 'REFUSES empty, blank, a sign, a point missing a digit on either side, a second point, an exponent, a '
            + 'radix, a space either side, a comma and a full-width digit',
          fn: async () => {
            /**
             Texts `Number` reads, or reads as something else, that no plain decimal is written as.
             */
            const refused = [
              '',
              ' ',
              '-1',
              '+1',
              '.5',
              '5.',
              '.',
              '1.2.3',
              '1e3',
              '0x1F',
              ' 1',
              '1 ',
              '1,5',
              '１',
              'Infinity',
            ];
            expect(refused.filter(function taken(text,): boolean {
              return isDecimalText({ text, },);
            },),).toEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: isUnsignedNumberText.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'TAKES a plain decimal, and one followed by an exponent mark in either case, a sign or none, and digits',
          fn: async () => {
            expect(['7', '7.5', '0.042e-6', '1.28E5', '1e+3', '15e0',].map(function taken(text,): boolean {
              return isUnsignedNumberText({ text, },);
            },),).toEqual([true, true, true, true, true, true,],);
          },
        },),
        it({
          name: 'REFUSES an exponent missing its digits or carrying anything else, a second mark, a mark with no decimal '
            + 'before it, a sign before the number, and what a plain decimal refuses',
          fn: async () => {
            /**
             Texts that are no unsigned number as JSON writes one.
             */
            const refused = [
              '1e',
              '1e+',
              '1e-',
              '1e5e5',
              '1e5.5',
              '1e 5',
              '1e+-5',
              'e5',
              '.5e3',
              '5.e3',
              '-1e3',
              '+1e3',
              '0x1e3',
              '',
              ' 1e3',
            ];
            expect(refused.filter(function taken(text,): boolean {
              return isUnsignedNumberText({ text, },);
            },),).toEqual([],);
          },
        },),
      ],
    },),
  ],
},);
