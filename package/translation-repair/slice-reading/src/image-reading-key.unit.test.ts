/**
 Tests for the cross-run cache key one picture's paired reading is stored
 under.

 WHAT THIS PINS is the one claim the whole module exists to make true: a key
 is built from what a reading was ASKED, never from what came back. A
 reading is not deterministic, so a key built from its wording would miss on
 every resume and re-buy a whole document's picture-bearing slices. Calling
 this function twice with the same picture and the same roster stands in
 for two runs that asked the same question and would have gotten different
 words back, and both runs still have to land on the same key.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import { imageReadingKey, } from '../dist/final/node/index.mjs';

/**
 Length a lowercase SHA-256 hex digest always carries, which is what
 `hashContent` documents itself as returning and what `imageReadingKey`
 builds its key from.
 */
const SHA256_HEX_LENGTH = 64;

/**
 Characters a lowercase hex digest may carry.
 */
const HEX_ALPHABET = '0123456789abcdef';

/**
 Bytes standing in for one picture, distinguished by seed so two calls can
 stand for two different pictures.

 @param seed - byte every position of returned buffer carries

 @returns Small buffer filled with that byte

 @example
 ```ts
 const bytes = bytesOf({ seed: 7, },);
 ```
 */
function bytesOf({ seed, }: { readonly seed: number; },): Uint8Array {
  return new Uint8Array(32,).fill(seed,);
}

/**
 Casts a cat-themed stand-in identifier to roster's closed union type,
 since this module only folds a model id into JSON and never validates it
 against real roster.

 @param id - cat-themed stand-in for production model id

 @returns Same string, typed as roster's closed union

 @example
 ```ts
 const modelId = catModelId({ id: 'hf:cat/Whiskers', },);
 ```
 */
function catModelId({ id, }: { readonly id: string; },): RosterModelId {
  return id as unknown as RosterModelId;
}

/**
 Whether every character of a string is a lowercase hex digit.

 SCANS BY INDEX RATHER THAN SPREADING, since spreading a string produces
 Unicode code points that break multi-unit characters apart; a digest is
 always single-byte-per-character, but `charAt` sidesteps the question
 rather than resting on that assumption.

 @param value - string to check

 @returns True when every character sits in `HEX_ALPHABET`

 @example
 ```ts
 const isHex = isHexDigest({ value: 'a3', },);
 ```
 */
function isHexDigest({ value, }: { readonly value: string; },): boolean {
  for (let index = 0; index < value.length; index += 1) {
    if (!HEX_ALPHABET.includes(value.charAt(index,),))
      return false;
  }
  return true;
}

/**
 Vision sub-roster asked about one picture, cat-themed since a reading key
 never validates a model id against the real roster.
 */
const ROSTER: readonly RosterModelId[] = [
  catModelId({ id: 'hf:cat/Whiskers', },),
  catModelId({ id: 'hf:cat/Marmalade', },),
];

/**
 Same two readers as `ROSTER`, asked in the opposite order.
 */
const REORDERED_ROSTER: readonly RosterModelId[] = [
  catModelId({ id: 'hf:cat/Marmalade', },),
  catModelId({ id: 'hf:cat/Whiskers', },),
];

/**
 Different roster membership from `ROSTER`, same length.
 */
const OTHER_ROSTER: readonly RosterModelId[] = [
  catModelId({ id: 'hf:cat/Nutmeg', },),
  catModelId({ id: 'hf:cat/Biscuit', },),
];

await describe({
  name: imageReadingKey.name,
  children: [
    it({
      name: 'RETURNS THE SAME KEY FOR TWO CALLS WITH THE SAME BYTES AND ROSTER, which is what '
        + 'lets a cache key outlive a reading whose WORDING is never the same twice: this '
        + 'function never sees what came back, only the picture and who was asked',
      fn: async () => {
        /**
         Picture bytes both calls are asked about.
         */
        const bytes = bytesOf({ seed: 3, },);

        /**
         Key from a first call, standing in for one run's cache write.
         */
        const firstKey = imageReadingKey({
          bytes,
          readerModelIds: ROSTER,
        },);

        /**
         Key from a second call with the same inputs, standing in for a
         resumed run asking the same question and getting different
         wording back.
         */
        const secondKey = imageReadingKey({
          bytes,
          readerModelIds: ROSTER,
        },);

        expect(secondKey,).toBe(firstKey,);
      },
    },),

    it({
      name: 'RETURNS A DIFFERENT KEY FOR A DIFFERENT PICTURE, since bytes sent are one of the '
        + 'inputs a reading answers a question about, not wording that came back',
      fn: async () => {
        /**
         Key for one picture.
         */
        const keyForOne = imageReadingKey({
          bytes: bytesOf({ seed: 1, },),
          readerModelIds: ROSTER,
        },);

        /**
         Key for a different picture, same roster.
         */
        const keyForOther = imageReadingKey({
          bytes: bytesOf({ seed: 2, },),
          readerModelIds: ROSTER,
        },);

        expect(keyForOther,).not.toBe(keyForOne,);
      },
    },),

    it({
      name: 'RETURNS A DIFFERENT KEY FOR A DIFFERENT ROSTER, since who was asked is as much an '
        + 'input as the picture itself',
      fn: async () => {
        /**
         Picture bytes shared by both calls, so membership is the only
         thing that differs between them.
         */
        const bytes = bytesOf({ seed: 5, },);

        /**
         Key asking `ROSTER` about this picture.
         */
        const keyForRoster = imageReadingKey({
          bytes,
          readerModelIds: ROSTER,
        },);

        /**
         Key asking a differently-membered roster about same picture.
         */
        const keyForOtherRoster = imageReadingKey({
          bytes,
          readerModelIds: OTHER_ROSTER,
        },);

        expect(keyForOtherRoster,).not.toBe(keyForRoster,);
      },
    },),

    it({
      name: 'RETURNS A DIFFERENT KEY WHEN THE SAME TWO READERS ARE ASKED IN A DIFFERENT ORDER, '
        + 'since the roster is a list and readings come back in that order, so a resumed run '
        + 'has to ask in the same order to land on the same key',
      fn: async () => {
        /**
         Picture bytes shared by both calls, so order is the only thing
         that differs between them.
         */
        const bytes = bytesOf({ seed: 6, },);

        /**
         Key asking `ROSTER` in its stated order.
         */
        const keyForRoster = imageReadingKey({
          bytes,
          readerModelIds: ROSTER,
        },);

        /**
         Key asking the same two readers in the opposite order.
         */
        const keyForReordered = imageReadingKey({
          bytes,
          readerModelIds: REORDERED_ROSTER,
        },);

        expect(keyForReordered,).not.toBe(keyForRoster,);
      },
    },),

    it({
      name: 'RETURNS A LOWERCASE HEX DIGEST OF ONE STABLE LENGTH ACROSS DIFFERENT INPUTS, since '
        + 'the key is a cache lookup rather than the reading itself and every picture has to '
        + 'fit the same slot',
      fn: async () => {
        /**
         Keys from several distinct inputs, so length and alphabet are
         checked across more than one call rather than pinned to a single
         lucky digest.
         */
        const keys = [
          imageReadingKey({
            bytes: bytesOf({ seed: 1, },),
            readerModelIds: ROSTER,
          },),
          imageReadingKey({
            bytes: bytesOf({ seed: 2, },),
            readerModelIds: OTHER_ROSTER,
          },),
          imageReadingKey({
            bytes: bytesOf({ seed: 3, },),
            readerModelIds: REORDERED_ROSTER,
          },),
        ];

        for (const key of keys) {
          expect(key.length,).toBe(SHA256_HEX_LENGTH,);
          expect(isHexDigest({ value: key, },),).toBe(true,);
        }
      },
    },),
  ],
},);
