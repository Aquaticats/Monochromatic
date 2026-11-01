/**
 CRC-32 implementation using the standard ZIP polynomial.
 
 Reference: PKWARE APPNOTE.txt v6.3.10 section 4.4.7 (CRC-32 algorithm).
 
 @module
 */

import {
  BYTE_BITS,
  BYTE_MASK,
  BYTE_VALUES,
} from './constants.ts';

/**
 CRC-32 polynomial in reflected form (used by ZIP, gzip, PNG).
 */
const CRC32_POLYNOMIAL = 0xED_B8_83_20;

/**
 Initial CRC-32 register value.
 */
const CRC32_INIT = 0xFF_FF_FF_FF;

/**
 Build the precomputed CRC-32 lookup table using the standard reflected
 polynomial `0xEDB88320`. Result is 1 KiB (256 × 4 bytes).
 
 @returns Filled lookup table indexed by input byte value
 */
function buildCrc32Table(): Uint32Array {
  /**
   Lookup table populated in place and returned once filled.
   */
  const table = new Uint32Array(BYTE_VALUES,);
  for (let byte = 0; byte < BYTE_VALUES; byte += 1) {
    /**
     Per-byte CRC register cycled through eight rounds of polynomial mixing.
     */
    let c = byte;
    for (let bit = 0; bit < BYTE_BITS; bit += 1)
      c = (c & 1) === 1 ? CRC32_POLYNOMIAL ^ (c >>> 1) : c >>> 1;
    // `>>> 0` reinterprets the 32-bit result as unsigned. `Math.trunc` would
    // leave negative values from the signed-bitwise interpretation in place
    // and the table would store wrong values.
    // oxlint-disable-next-line eslint-plugin-unicorn/prefer-math-trunc
    table[byte] = c >>> 0;
  }
  return table;
}

/**
 Precomputed CRC-32 lookup table. Built once at module load.
 */
const CRC32_TABLE = buildCrc32Table();

/**
 Compute CRC-32 of a byte sequence using the standard ZIP polynomial.
 
 @param data - Bytes to checksum
 
 @returns Unsigned 32-bit CRC value
 
 @example
 ```ts
 crc32(new Uint8Array(),) === 0;
 crc32(new TextEncoder().encode('123456789',),) === 0xCBF43926;
 ```
 */
export function crc32(data: Uint8Array,): number {
  /**
   CRC register accumulating across the input bytes.
   */
  let c = CRC32_INIT;
  for (const byte of data) {
    /**
     Indexed lookup checked because strict index-access typing widens to `undefined`.
     */
    const tableEntry = CRC32_TABLE[(c ^ byte) & BYTE_MASK];
    if (tableEntry === undefined)
      throw new Error('zip-writer: CRC32 table corrupted',);
    c = tableEntry ^ (c >>> BYTE_BITS);
  }
  /* oxlint-disable eslint-plugin-unicorn/prefer-math-trunc -- See note on `>>> 0` in buildCrc32Table. */
  /**
   Final CRC value after the standard finalize XOR; reinterpreted as unsigned 32-bit.
   */
  const result = (c ^ CRC32_INIT) >>> 0;
  /* oxlint-enable eslint-plugin-unicorn/prefer-math-trunc */
  return result;
}
