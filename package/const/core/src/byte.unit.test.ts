/**
 Tests for byte and bit unit constants.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  BITS_PER_BYTE,
  BYTES_PER_GB,
  BYTES_PER_GIB,
  BYTES_PER_KB,
  BYTES_PER_KIB,
  BYTES_PER_MB,
  BYTES_PER_MIB,
  BYTES_PER_TB,
  BYTES_PER_TIB,
} from '../dist/final/neutral/index.mjs';

await describe({
  name: 'byte',
  children: [
    it({
      name: 'BITS_PER_BYTE is 8',
      fn: async () => {
        expect(BITS_PER_BYTE,).toBe(8,);
      },
    },),
    it({
      name: 'IEC binary prefixes are powers of 1024',
      fn: async () => {
        expect(BYTES_PER_KIB,).toBe(1_024,);
        expect(BYTES_PER_MIB,).toBe(1_048_576,);
        expect(BYTES_PER_GIB,).toBe(1_073_741_824,);
        expect(BYTES_PER_TIB,).toBe(1_099_511_627_776,);
      },
    },),
    it({
      name: 'SI decimal prefixes are powers of 1000',
      fn: async () => {
        expect(BYTES_PER_KB,).toBe(1_000,);
        expect(BYTES_PER_MB,).toBe(1_000_000,);
        expect(BYTES_PER_GB,).toBe(1_000_000_000,);
        expect(BYTES_PER_TB,).toBe(1_000_000_000_000,);
      },
    },),
    it({
      name: 'IEC binary unit exceeds SI decimal counterpart',
      fn: async () => {
        expect(BYTES_PER_KIB > BYTES_PER_KB,).toBe(true,);
        expect(BYTES_PER_MIB > BYTES_PER_MB,).toBe(true,);
        expect(BYTES_PER_GIB > BYTES_PER_GB,).toBe(true,);
      },
    },),
  ],
},);
