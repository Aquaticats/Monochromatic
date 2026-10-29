/**
 ISO 9660 constants and binary format helpers.
 Provides sector sizes, field offsets, and functions for writing
 dual-endian integers and fixed-width strings into byte buffers.
 */

/**
 ISO9660 logical sector size in bytes.
 */
export const SECTOR_SIZE = 2_048;

//region ISO 9660 field offsets and sizes
// These constants correspond to byte offsets within ISO 9660 data structures.
// Names follow the pattern: <STRUCTURE>_<FIELD>_OFFSET or <FIELD>_SIZE.

/**
 ASCII space character used to pad fixed-width string fields.
 */
const ASCII_SPACE = 0x20;

/**
 Size of dual-endian 32-bit values (4 bytes LE + 4 bytes BE).
 */
const DUAL_32_SIZE = 4;

/**
 Length of the ISO 9660 standard identifier string `CD001`.
 */
export const CD001_LENGTH = 5;

/**
 Length of a fixed-width ISO 9660 string identifier (system id, volume id).
 */
export const ISO_STRING_FIELD_LENGTH = 32;

/**
 Length of a 17-byte descriptor timestamp (ISO 8.4.26.1) string portion.
 */
const TIMESTAMP_17_STR_LENGTH = 16;

//region Directory entry field offsets (relative to record start)
/**
 Offset of extent location (LBA) within a directory record.
 */
export const DIR_EXTENT_OFFSET = 2;
/**
 Offset of data length within a directory record.
 */
export const DIR_SIZE_OFFSET = 10;
/**
 Offset of recording timestamp within a directory record.
 */
export const DIR_TIMESTAMP_OFFSET = 18;
/**
 Offset of file flags byte within a directory record.
 */
export const DIR_FLAGS_OFFSET = 25;
/**
 Offset of volume sequence number within a directory record.
 */
export const DIR_VOL_SEQ_OFFSET = 28;
/**
 Offset of file identifier length within a directory record.
 */
export const DIR_NAME_LEN_OFFSET = 32;
/**
 Offset of file identifier string within a directory record.
 */
export const DIR_NAME_DATA_OFFSET = 33;
/**
 Fixed header size before the file identifier in a directory record.
 */
export const DIR_FIXED_HEADER_SIZE = 33;
//endregion Directory entry field offsets

/**
 Year value stored in 7-byte timestamps: 2026 minus 1900 base year.
 */
const TIMESTAMP_YEAR_SINCE_1900 = 126;

//region Primary Volume Descriptor field offsets (relative to PVD start)
/**
 PVD sector number within the ISO image.
 */
export const PVD_SECTOR = 16;
/**
 Offset of version byte within PVD.
 */
export const PVD_VERSION_OFFSET = 6;
/**
 Offset of system identifier within PVD.
 */
export const PVD_SYSTEM_ID_OFFSET = 8;
/**
 Offset of volume identifier within PVD.
 */
export const PVD_VOLUME_ID_OFFSET = 40;
/**
 Offset of volume space size (total sectors) within PVD.
 */
export const PVD_VOLUME_SPACE_OFFSET = 80;
/**
 Offset of volume set size within PVD.
 */
export const PVD_VOLUME_SET_SIZE_OFFSET = 120;
/**
 Offset of volume sequence number within PVD.
 */
export const PVD_VOLUME_SEQ_OFFSET = 124;
/**
 Offset of logical block size within PVD.
 */
export const PVD_BLOCK_SIZE_OFFSET = 128;
/**
 Offset of path table size within PVD.
 */
export const PVD_PATH_TABLE_SIZE_OFFSET = 132;
/**
 Offset of type L path table location within PVD.
 */
export const PVD_PATH_TABLE_L_OFFSET = 140;
/**
 Offset of type M path table location within PVD.
 */
export const PVD_PATH_TABLE_M_OFFSET = 148;
/**
 Offset of root directory record within PVD.
 */
export const PVD_ROOT_DIR_RECORD_OFFSET = 156;
/**
 Offset of volume creation timestamp within PVD.
 */
export const PVD_CREATION_TIMESTAMP_OFFSET = 813;
/**
 Offset of volume modification timestamp within PVD.
 */
export const PVD_MODIFICATION_TIMESTAMP_OFFSET = 830;
/**
 Offset of volume expiration timestamp within PVD.
 */
export const PVD_EXPIRATION_TIMESTAMP_OFFSET = 847;
/**
 Offset of volume effective timestamp within PVD.
 */
export const PVD_EFFECTIVE_TIMESTAMP_OFFSET = 864;
/**
 Offset of file structure version within PVD.
 */
export const PVD_FILE_STRUCTURE_VERSION_OFFSET = 881;
//endregion Primary Volume Descriptor field offsets

/**
 VDST sector number (Volume Descriptor Set Terminator).
 */
export const VDST_SECTOR = 17;

/**
 Path table LE sector number.
 */
export const PATH_TABLE_LE_SECTOR = 18;
/**
 Path table BE sector number.
 */
export const PATH_TABLE_BE_SECTOR = 19;
/**
 Offset of parent directory number within a path table entry.
 */
export const PATH_TABLE_PARENT_DIR_OFFSET = 6;

/**
 Root directory sector number.
 */
export const ROOT_DIRECTORY_SECTOR = 20;
/**
 First sector available for file data.
 */
export const FIRST_FILE_DATA_SECTOR = 21;
/**
 Path table size in bytes (single root entry).
 */
export const PATH_TABLE_SIZE = 10;

//endregion ISO 9660 field offsets and sizes

//region Binary format helpers

/**
 Writes a space-padded ASCII string at the given offset.
 
 @param buf - Target byte array
 
 @param offset - Byte offset to start writing at
 
 @param str - ASCII string to write
 
 @param len - Fixed-width field length to fill (padded with spaces)
 
 @example
 ```ts
 const buf = new Uint8Array(32);
 writeStr({ buf, offset: 0, str: 'CD001', len: 5 });
 ```
 */
export function writeStr({
  buf,
  len,
  offset,
  str,
}: {
  readonly buf: Uint8Array;
  readonly len: number;
  readonly offset: number;
  readonly str: string;
},): void {
  for (let cursorIndex = 0; cursorIndex < len; cursorIndex++) {
    buf[offset + cursorIndex] = str.codePointAt(cursorIndex,) ?? ASCII_SPACE;
  }
}

/**
 Writes a 16-bit value in both little-endian and big-endian (ISO 7.3.3).
 
 @param view - DataView for the target buffer
 
 @param offset - Byte offset for the LE value (BE follows at offset+2)
 
 @param value - 16-bit unsigned integer to write
 
 @example
 ```ts
 const view = new DataView(new ArrayBuffer(4));
 writeBoth16({ view, offset: 0, value: 2048 });
 ```
 */
export function writeBoth16({
  offset,
  value,
  view,
}: {
  readonly offset: number;
  readonly value: number;
  readonly view: DataView;
},): void {
  view.setUint16(
    offset,
    value,
    true,
  );
  view.setUint16(
    offset + 2,
    value,
    false,
  );
}

/**
 Writes a 32-bit value in both little-endian and big-endian (ISO 7.3.3).
 
 @param view - DataView for the target buffer
 
 @param offset - Byte offset for the LE value (BE follows at offset+4)
 
 @param value - 32-bit unsigned integer to write
 
 @example
 ```ts
 const view = new DataView(new ArrayBuffer(8));
 writeBoth32({ view, offset: 0, value: 65_536 });
 ```
 */
export function writeBoth32({
  offset,
  value,
  view,
}: {
  readonly offset: number;
  readonly value: number;
  readonly view: DataView;
},): void {
  view.setUint32(
    offset,
    value,
    true,
  );
  view.setUint32(
    offset + DUAL_32_SIZE,
    value,
    false,
  );
}

/**
 Writes a 7-byte recording timestamp (ISO 9.1.5).
 
 @param buf - Target byte array
 
 @param offset - Byte offset to start writing at
 
 @example
 ```ts
 const buf = new Uint8Array(7);
 writeTimestamp7({ buf, offset: 0 });
 ```
 */
export function writeTimestamp7({
  buf,
  offset,
}: {
  readonly buf: Uint8Array;
  readonly offset: number;
},): void {
  buf[offset] = TIMESTAMP_YEAR_SINCE_1900;
  buf[offset + 1] = 1; // month
  buf[offset + 2] = 1; // day
}

/**
 Writes a 17-byte descriptor timestamp (ISO 8.4.26.1).
 
 @param buf - Target byte array
 
 @param offset - Byte offset to start writing at
 
 @example
 ```ts
 const buf = new Uint8Array(17);
 writeTimestamp17({ buf, offset: 0 });
 ```
 */
export function writeTimestamp17({
  buf,
  offset,
}: {
  readonly buf: Uint8Array;
  readonly offset: number;
},): void {
  writeStr({
    buf,
    len: TIMESTAMP_17_STR_LENGTH,
    offset,
    str: '2026010100000000',
  },);
}

//endregion Binary format helpers
