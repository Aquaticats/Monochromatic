/**
 ICO binary container encoding.
 
 Wraps a single PNG image in the ICO format for use as a browser favicon.
 */

//region ICO binary format constants
/**
 ICO file header size in bytes (reserved + type + image count fields).
 */
const ICO_HEADER_SIZE = 6;

/**
 ICO directory entry size in bytes per the ICO specification.
 */
const ICO_ENTRY_SIZE = 16;

/**
 ICO favicon dimensions in pixels (width and height).
 */
const ICO_DIMENSION = 32;

/**
 Bits per pixel for RGBA color depth in ICO directory entries.
 */
const ICO_BITS_PER_PIXEL = 32;

/**
 Byte offset of the image type within the ICO header.
 */
const ICO_HEADER_TYPE_OFFSET = 2;

/**
 Byte offset of image count within the ICO header.
 */
const ICO_HEADER_COUNT_OFFSET = 4;

/**
 Byte offset of the image width within an ICO directory entry.
 */
const ICO_ENTRY_WIDTH_OFFSET = 0;

/**
 Byte offset of the image height within an ICO directory entry.
 */
const ICO_ENTRY_HEIGHT_OFFSET = 1;

/**
 Byte offset of color planes within an ICO directory entry.
 */
const ICO_ENTRY_PLANES_OFFSET = 4;

/**
 Byte offset of bits-per-pixel within an ICO directory entry.
 */
const ICO_ENTRY_BPP_OFFSET = 6;

/**
 Byte offset of image data size within an ICO directory entry.
 */
const ICO_ENTRY_DATASIZE_OFFSET = 8;

/**
 Byte offset of data pointer within an ICO directory entry.
 */
const ICO_ENTRY_DATAPTR_OFFSET = 12;

/**
 Absolute byte offset where image data begins (header + one directory entry).
 */
const ICO_DATA_OFFSET = 22;
//endregion

/**
 Creates an ICO container wrapping a single 32x32 PNG image.
 
 @param pngData - PNG buffer to embed in the ICO container
 
 @returns ICO file buffer
 
 @example
 ```ts
 const ico = createIco({ pngData: png32Buffer });
 ```
 */
export function createIco({ pngData, }: { readonly pngData: Uint8Array; },): Buffer {
  /**
   ICONDIR header preceding the image directory entries per the ICO spec.
   */
  const header = Buffer.alloc(ICO_HEADER_SIZE,);
  /* Reserved field is zero; the type field declares this container an ICO. */
  header.writeUInt16LE(
    1,
    ICO_HEADER_TYPE_OFFSET,
  );
  /* One directory entry follows, for the single embedded image. */
  header.writeUInt16LE(
    1,
    ICO_HEADER_COUNT_OFFSET,
  );

  /**
   ICONDIRENTRY describing the single embedded PNG image.
   */
  const entry = Buffer.alloc(ICO_ENTRY_SIZE,);
  /* Width in pixels; 32 fits the single-byte field the ICO spec uses. */
  entry.writeUInt8(
    ICO_DIMENSION,
    ICO_ENTRY_WIDTH_OFFSET,
  );
  /* Height in pixels, matching the width for this square icon. */
  entry.writeUInt8(
    ICO_DIMENSION,
    ICO_ENTRY_HEIGHT_OFFSET,
  );
  /* Color planes; 1 for the single-plane PNG payload. */
  entry.writeUInt16LE(
    1,
    ICO_ENTRY_PLANES_OFFSET,
  );
  /* Color depth of the embedded PNG. */
  entry.writeUInt16LE(
    ICO_BITS_PER_PIXEL,
    ICO_ENTRY_BPP_OFFSET,
  );
  /* Byte size of the embedded PNG payload. */
  entry.writeUInt32LE(
    pngData.length,
    ICO_ENTRY_DATASIZE_OFFSET,
  );
  /* Absolute offset where the embedded PNG payload starts. */
  entry.writeUInt32LE(
    ICO_DATA_OFFSET,
    ICO_ENTRY_DATAPTR_OFFSET,
  );

  return Buffer.concat([
    header,
    entry,
    pngData,
  ],);
}
