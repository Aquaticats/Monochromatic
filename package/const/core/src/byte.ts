/**
 Byte and bit unit constants.
 
 IEC binary prefixes (KiB, MiB, GiB, TiB) are powers of 1024 and describe
 memory, page-cache, and most file-system block sizes. SI decimal prefixes
 (KB, MB, GB, TB) are powers of 1000 and describe storage marketing
 capacity, network throughput, and any other context where SI is the
 convention. The two are not interchangeable; mixing them produces
 approximately 7% error per binary order of magnitude.
 
 Reference: IEC 80000-13:2008 / NIST SP 811.
 
 @example
 ```ts
 import {
   BYTES_PER_KIB,
   BYTES_PER_MIB,
 } from '@monochromatic-dev/const-core';
 ```
 
 @module
 */

//region Bit primitive

/**
 Bits in one byte.
 
 @example
 ```ts
 const bits = byteCount * BITS_PER_BYTE;
 ```
 */
export const BITS_PER_BYTE = 8;

//endregion Bit primitive

//region IEC binary prefixes (powers of 1024)

/**
 Bytes in one kibibyte (KiB), the binary IEC unit. Equal to 2^10 = 1024.
 Use for memory, page-cache thresholds, and file-system blocks.
 
 @example
 ```ts
 const bufferSize = 64 * BYTES_PER_KIB;
 ```
 */
export const BYTES_PER_KIB = 1_024;

/**
 Bytes in one mebibyte (MiB), the binary IEC unit. Equal to 2^20 = 1_048_576.
 Derived from {@link BYTES_PER_KIB} squared.
 
 @example
 ```ts
 const memoryCap = 256 * BYTES_PER_MIB;
 ```
 */
export const BYTES_PER_MIB: number = BYTES_PER_KIB * BYTES_PER_KIB;

/**
 Bytes in one gibibyte (GiB), the binary IEC unit. Equal to 2^30.
 Derived from {@link BYTES_PER_MIB} times {@link BYTES_PER_KIB}.
 
 @example
 ```ts
 const diskCap = 4 * BYTES_PER_GIB;
 ```
 */
export const BYTES_PER_GIB: number = BYTES_PER_MIB * BYTES_PER_KIB;

/**
 Bytes in one tebibyte (TiB), the binary IEC unit. Equal to 2^40.
 Derived from {@link BYTES_PER_GIB} times {@link BYTES_PER_KIB}.
 
 @example
 ```ts
 const archiveBudget = BYTES_PER_TIB;
 ```
 */
export const BYTES_PER_TIB: number = BYTES_PER_GIB * BYTES_PER_KIB;

//endregion IEC binary prefixes (powers of 1024)

//region SI decimal prefixes (powers of 1000)

/**
 Bytes in one kilobyte (KB), the decimal SI unit. Equal to 10^3 = 1000.
 Use for storage capacity marketing, network throughput, or any context
 where SI is the spec. Distinct from {@link BYTES_PER_KIB} = 1024.
 
 @example
 ```ts
 const advertisedCapacity = 500 * BYTES_PER_KB;
 ```
 */
export const BYTES_PER_KB = 1_000;

/**
 Bytes in one megabyte (MB), the decimal SI unit. Equal to 10^6.
 Derived from {@link BYTES_PER_KB} squared.
 
 @example
 ```ts
 const downloadBudget = 50 * BYTES_PER_MB;
 ```
 */
export const BYTES_PER_MB: number = BYTES_PER_KB * BYTES_PER_KB;

/**
 Bytes in one gigabyte (GB), the decimal SI unit. Equal to 10^9.
 Derived from {@link BYTES_PER_MB} times {@link BYTES_PER_KB}.
 
 @example
 ```ts
 const monthlyTransfer = 100 * BYTES_PER_GB;
 ```
 */
export const BYTES_PER_GB: number = BYTES_PER_MB * BYTES_PER_KB;

/**
 Bytes in one terabyte (TB), the decimal SI unit. Equal to 10^12.
 Derived from {@link BYTES_PER_GB} times {@link BYTES_PER_KB}.
 
 @example
 ```ts
 const advertisedDriveSize = 2 * BYTES_PER_TB;
 ```
 */
export const BYTES_PER_TB: number = BYTES_PER_GB * BYTES_PER_KB;

//endregion SI decimal prefixes (powers of 1000)
