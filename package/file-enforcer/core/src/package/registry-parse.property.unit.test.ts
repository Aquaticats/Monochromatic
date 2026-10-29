/**
 Property-based fuzz tests for `firstWhitespaceToken` in `./registry-parse.ts`.
 
 Properties: the returned token never contains an ASCII whitespace
 character; an empty or all-whitespace line yields the empty token; and a
 line built as `leadingWhitespace + token + separator + tail` recovers
 exactly `token`. The token alphabet deliberately includes Unicode
 whitespace-like code points (U+00A0) and an astral code point, which the
 function treats as ordinary token characters because it matches only the
 six ASCII `\s` characters.
 
 Run plan and seed policy: see `../fuzz-budget.ts`.
 
 @module
 */

import {
  assert,
  asyncProperty,
  constantFrom,
  record,
  string,
} from 'fast-check';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { fuzzRunPlan, } from '../fuzz-budget.ts';
import { firstWhitespaceToken, } from './registry-parse.ts';

//region Constants and arbitraries

/**
 Run plan resolved once for every property in this file.
 */
const RUN = fuzzRunPlan();

/**
 The six ASCII whitespace characters `firstWhitespaceToken` recognizes,
 matching regex `\s` for ASCII.
 */
const ASCII_WHITESPACE = [
  ' ',
  '\t',
  '\n',
  '\r',
  '\f',
  '\v',
] as const;

/**
 Arbitrary run of ASCII whitespace, possibly empty.
 */
const whitespaceArbitrary = string({ unit: constantFrom(...ASCII_WHITESPACE,), },);

/**
 Arbitrary single ASCII whitespace character used to terminate a token.
 */
const separatorArbitrary = constantFrom(...ASCII_WHITESPACE,);

/**
 Arbitrary non-empty token of characters that are not ASCII whitespace,
 including a Unicode no-break space and an astral code point the function
 keeps inside the token.
 */
const tokenArbitrary = string({
  minLength: 1,
  unit: constantFrom(
    'a',
    'b',
    '-',
    ':',
    '/',
    ' ',
    '\u{1F600}',
  ),
},);

//endregion Constants and arbitraries

await describe({
  name: firstWhitespaceToken.name,
  children: [
    it({
      name: 'never returns a token containing ASCII whitespace',
      timeout: RUN.timeout,
      fn: async () => {
        await assert(
          asyncProperty(
            string(),
            async function tokenHasNoWhitespace(line,) {
              /**
               Leading token extracted from the line.
               */
              const token = firstWhitespaceToken(line,);
              ASCII_WHITESPACE.forEach(function absent(whitespace,) {
                expect(token.includes(whitespace,),).toBe(false,);
              },);
            },
          ),
          RUN.params,
        );
      },
    },),

    it({
      name: 'returns the empty token for empty or all-whitespace lines',
      timeout: RUN.timeout,
      fn: async () => {
        await assert(
          asyncProperty(
            whitespaceArbitrary,
            async function emptyForBlank(line,) {
              expect(firstWhitespaceToken(line,),).toBe('',);
            },
          ),
          RUN.params,
        );
      },
    },),

    it({
      name: 'recovers the token from leading whitespace, token, and separated tail',
      timeout: RUN.timeout,
      fn: async () => {
        await assert(
          asyncProperty(
            record({
              lead: whitespaceArbitrary,
              token: tokenArbitrary,
              separator: separatorArbitrary,
              tail: string(),
            },),
            async function recoversToken({
              lead,
              token,
              separator,
              tail,
            },) {
              expect(firstWhitespaceToken(`${lead}${token}${separator}${tail}`,),).toBe(token,);
            },
          ),
          RUN.params,
        );
      },
    },),
  ],
},);
