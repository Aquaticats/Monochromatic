/**
 Numeric constants shared across the ZIP writer modules.
 
 Values come from the PKZIP/APPNOTE.txt v6.3.10 specification. Section
 references are noted on each declaration.
 
 @module
 */

// region Header signatures

/**
 Local file header signature `PK\x03\x04`. APPNOTE 4.3.7.
 */
export const LFH_SIGNATURE = 0x04_03_4B_50;

/**
 Central directory file header signature `PK\x01\x02`. APPNOTE 4.4.1.1.
 */
export const CDH_SIGNATURE = 0x02_01_4B_50;

/**
 End of central directory record signature `PK\x05\x06`. APPNOTE 4.5.
 */
export const EOCD_SIGNATURE = 0x06_05_4B_50;

// endregion

// region Header sizes

/**
 Local file header fixed size in bytes (excluding name and extra).
 */
export const LFH_FIXED_SIZE = 30;

/**
 Central directory header fixed size in bytes (excluding name, extra, comment).
 */
export const CDH_FIXED_SIZE = 46;

/**
 End of central directory record fixed size in bytes (excluding comment).
 */
export const EOCD_FIXED_SIZE = 22;

// endregion

// region Version

/**
 ZIP version needed to extract: 2.0. APPNOTE 4.4.3.
 
 2.0 is the lowest version that supports the directory-style file paths
 commonly used by consumers. STORE compression is supported from 1.0 but
 every reader accepts 2.0.
 */
export const VERSION_NEEDED = 20;

/**
 ZIP `version made by` host id 3 = Unix. APPNOTE 4.4.2.
 */
const ZIP_HOST_UNIX = 3;

/**
 ZIP `version made by` spec version 30 = ZIP 3.0. APPNOTE 4.4.2.
 */
const ZIP_SPEC_VERSION_30 = 30;

/**
 Bits per byte.
 */
export const BYTE_BITS = 8;

/**
 ZIP version made by: 3.0 on Unix host. APPNOTE 4.4.2.
 
 High byte 3 = Unix (so external file attributes are interpreted as
 Unix mode bits); low byte 30 = ZIP 3.0.
 */
export const VERSION_MADE_BY: number = (ZIP_HOST_UNIX << BYTE_BITS) | ZIP_SPEC_VERSION_30;

// endregion

// region Compression and flags

/**
 Compression method 0 = STORE (no compression). APPNOTE 4.4.5.
 */
export const COMPRESSION_STORE = 0;

/**
 General purpose bit flag: bit 11 set, indicating filename and comment
 are UTF-8 encoded. APPNOTE 4.4.4 / Appendix D.
 */
export const FLAG_UTF8_FILENAME = 0x08_00;

// endregion

// region External file attributes

/**
 Default Unix file mode (owner rw, group r, other r).
 */
const DEFAULT_FILE_MODE = 0o644;

/**
 Bit position where Unix mode bits live in the external file attributes field. APPNOTE 4.4.15.
 */
const EXTERNAL_ATTRS_MODE_SHIFT = 16;

/**
 Default Unix file mode shifted into the upper 16 bits of the external
 file attributes field. APPNOTE 4.4.15.
 */
export const UNIX_FILE_MODE_DEFAULT: number = DEFAULT_FILE_MODE
  << EXTERNAL_ATTRS_MODE_SHIFT;

// endregion

// region Field-size limits

/**
 Maximum value representable in 16 unsigned bits.
 */
export const MAX_UINT16 = 0xFF_FF;

/**
 Maximum value representable in 32 unsigned bits.
 */
export const MAX_UINT32 = 0xFF_FF_FF_FF;

// endregion

// region Byte-level helpers

/**
 Number of distinct byte values.
 */
export const BYTE_VALUES = 256;

/**
 Lowest 8 bits mask.
 */
export const BYTE_MASK = 0xFF;

/**
 Bytes occupied by a uint32 field.
 */
export const BYTES_UINT32 = 4;

/**
 Bytes occupied by a uint16 field.
 */
export const BYTES_UINT16 = 2;

// endregion
