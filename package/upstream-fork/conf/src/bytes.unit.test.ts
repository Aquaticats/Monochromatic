/**
 Round-trip and concatenation tests for the UTF-8 byte helpers shipped as
 `@internal` exports of the built artifact.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  bytesToString,
  concatBytes,
  stringToBytes,
} from '../dist/final/neutral/index.mjs';

await describe({
  name: 'byte conversions',
  children: [
    it({
      name: 'round-trips ASCII text through its exact UTF-8 bytes',
      fn: async () => {
        /**
         ASCII fixture decoded back below.
         */
        const ascii = 'hello config';
        /**
         UTF-8 encoding of the two-letter sample.
         */
        const hiBytes = stringToBytes('hi',);

        expect(hiBytes,).toEqual(new Uint8Array([
          104,
          105,
        ],),);
        /**
         Round-trip result of the ASCII fixture.
         */
        const decoded = bytesToString(stringToBytes(ascii,),);
        expect(decoded,).toBe(ascii,);
      },
    },),

    it({
      name: 'round-trips multi-byte UTF-8 and emoji text through their exact bytes',
      fn: async () => {
        /**
         Mixed-width fixture carrying two-byte,
         three-byte,
         and four-byte code points.
         */
        const mixed = 'héllo 世界 🎉';
        /**
         UTF-8 encoding of the accented two-byte sample.
         */
        const accentBytes = stringToBytes('é',);
        /**
         UTF-8 encoding of the four-byte emoji sample.
         */
        const emojiBytes = stringToBytes('🎉',);

        expect(accentBytes,).toEqual(new Uint8Array([
          195,
          169,
        ],),);
        expect(emojiBytes,).toEqual(new Uint8Array([
          240,
          159,
          142,
          137,
        ],),);
        /**
         Round-trip result of the mixed-width fixture.
         */
        const decoded = bytesToString(stringToBytes(mixed,),);
        expect(decoded,).toBe(mixed,);
      },
    },),

    it({
      name: 'encodes the empty string as zero bytes and decodes empty bytes back',
      fn: async () => {
        /**
         Empty byte buffer decoded back below.
         */
        const emptyBytes = new Uint8Array(0,);

        expect(stringToBytes('',),).toHaveLength(0,);
        expect(bytesToString(emptyBytes,),).toBe('',);
      },
    },),

    it({
      name: 'concatenates multiple chunks into one buffer in order',
      fn: async () => {
        /**
         Joined result of three consecutive chunks.
         */
        const joined = concatBytes([
          new Uint8Array([
            1,
            2,
          ],),
          new Uint8Array([
            3,
          ],),
          new Uint8Array([
            4,
            5,
          ],),
        ],);

        expect(joined,).toEqual(new Uint8Array([
          1,
          2,
          3,
          4,
          5,
        ],),);
      },
    },),

    it({
      name: 'copies a single chunk into one buffer of its own',
      fn: async () => {
        /**
         Lone chunk handed to `concatBytes`.
         */
        const chunk = new Uint8Array([
          7,
          8,
          9,
        ],);
        /**
         Result buffer built from the lone chunk.
         */
        const joined = concatBytes([chunk],);

        expect(joined === chunk,).toBe(false,);
        expect(joined,).toEqual(chunk,);
      },
    },),

    it({
      name: 'returns a zero-length buffer for no chunks',
      fn: async () => {
        expect(concatBytes([],),).toHaveLength(0,);
      },
    },),

    it({
      name: 'agrees with the runtime TextEncoder and TextDecoder on mixed text',
      fn: async () => {
        /**
         Fixture exercising every conversion width at once.
         */
        const mixed = 'abc é 世 🎉';
        /**
         Reference encoding from the runtime itself.
         */
        const expectedBytes = new TextEncoder()
          .encode(mixed,);

        expect(stringToBytes(mixed,),).toEqual(expectedBytes,);
        expect(bytesToString(expectedBytes,),).toBe(new TextDecoder().decode(expectedBytes,),);
      },
    },),
  ],
},);
