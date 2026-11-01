/**
 Tests for the decoding of a link target as far as it reads.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { decodedTarget, } from '../../dist/final/node/index.mjs';

/**
 Pieces a target is built from in the equivalence case: valid escapes, text, escapes `decodeURI` keeps, and the malformed.
 */
const PIECES: readonly string[] = [
  'a',
  '虎',
  '/',
  '%41',
  '%e4%bd%a0',
  '%E4%BD%A0',
  '%C3%A9',
  '%F0%9F%90%88',
  '%2F',
  '%23',
  '%3b',
  '%25',
  '%7E',
  '%EF%BB%BF',
  '%E0%A4',
  '%ED%A0%80',
  '%C0%80',
  '%zz',
  '%4',
  '%',
];

/**
 The targets the equivalence case reads: every pair and every triple of the pieces.

 @returns Targets, each of two or three pieces joined

 @example
 ```ts
 const targets = pieceTargets();
 ```
 */
function pieceTargets(): readonly string[] {
  /**
   Targets found.
   */
  const targets: string[] = [];
  for (const first of PIECES) {
    for (const second of PIECES) {
      targets.push(`${first}${second}`,);
      for (const third of PIECES)
        targets.push(`${first}${second}${third}`,);
    }
  }
  return targets;
}

/**
 What `decodeURI` answers for a target it refuses.
 */
const REFUSED: unique symbol = Symbol('decodeURI refuses the target',);

/**
 What `decodeURI` makes of a target.

 @param target - text of a link target

 @returns Decoded target; the sentinel where `decodeURI` refuses it

 @throws Whatever `decodeURI` raises other than a refusal of the target

 @example
 ```ts
 const decoded = engineDecoding({ target: '%E0%A4', },); // REFUSED
 ```
 */
function engineDecoding({ target, }: { readonly target: string; },): string | typeof REFUSED {
  try {
    return decodeURI(target,);
  }
  catch (error) {
    if (error instanceof URIError)
      return REFUSED;
    throw error;
  }
}

await describe({
  name: decodedTarget.name,
  children: [
    it({
      name: 'LEAVES A TARGET WITHOUT AN ESCAPE AS IT STANDS',
      fn: async () => {
        expect(decodedTarget({ target: 'https://example.net/a?b=c#d', },),).toBe('https://example.net/a?b=c#d',);
        expect(decodedTarget({ target: '', },),).toBe('',);
        expect(decodedTarget({ target: '猫.png', },),).toBe('猫.png',);
      },
    },),
    it({
      name: 'DECODES A RUN OF ESCAPES AS UTF-8, a character that stands in several of them included',
      fn: async () => {
        expect(decodedTarget({ target: 'a%20b%E4%BD%A0%f0%9f%90%88', },),).toBe('a b你\u{1F408}',);
      },
    },),
    it({
      name: 'KEEPS THE ESCAPES OF THE CHARACTERS THAT DELIMIT AN ADDRESS, and a byte order mark',
      fn: async () => {
        expect(decodedTarget({ target: 'a%2Fb%23c%3Bd%3fe', },),).toBe('a%2Fb%23c%3Bd%3fe',);
        expect(decodedTarget({ target: '%EF%BB%BFa', },),).toBe('\uFEFFa',);
      },
    },),
    it({
      name: 'DECODES BYTES THAT READ AS NO CHARACTER TO REPLACEMENT CHARACTERS, and the text around them as it stands',
      fn: async () => {
        expect(decodedTarget({ target: '%E0%A4', },),).toBe('�',);
        expect(decodedTarget({ target: 'a%C3%A9%E0%A4b', },),).toBe('aé�b',);
        expect(decodedTarget({ target: '%ED%A0%80', },),).toBe('���',);
      },
    },),
    it({
      name: 'LEAVES TEXT THAT IS NO ESCAPE AS IT STANDS, a percent sign with fewer than two digits after it among them',
      fn: async () => {
        expect(decodedTarget({ target: 'a%zzb', },),).toBe('a%zzb',);
        expect(decodedTarget({ target: '100%', },),).toBe('100%',);
        expect(decodedTarget({ target: '100%4', },),).toBe('100%4',);
        expect(decodedTarget({ target: '%%41', },),).toBe('%A',);
      },
    },),
    it({
      name: 'ANSWERS WHAT decodeURI ANSWERS FOR EVERY TARGET IT READS, over every pair and triple of pieces',
      fn: async () => {
        /**
         How many targets `decodeURI` read and how many it refused.
         */
        const counted = {
          read: 0,
          refused: 0,
        };
        for (const target of pieceTargets()) {
          /**
           What `decodeURI` makes of the target, absent where it throws.
           */
          const expected = engineDecoding({ target, },);
          if (expected === REFUSED) {
            counted.refused += 1;
            expect(function decodes(): string {
              return decodedTarget({ target, },);
            },).not.toThrow();
            continue;
          }
          counted.read += 1;
          expect(decodedTarget({ target, },),).toBe(expected,);
        }
        expect(counted.read,).toBeGreaterThan(1_000,);
        expect(counted.refused,).toBeGreaterThan(1_000,);
      },
    },),
  ],
},);
