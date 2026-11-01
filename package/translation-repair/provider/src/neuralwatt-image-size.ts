//region NeuralWatt image size
// THE PIXEL SIZE OF A PICTURE, read off its header, for the energy estimate
// that holds a call against the day's free energy (`neuralwatt-picture-tokens.ts`).
//
// ONLY THE HEADERS, because the estimate needs two numbers and a decoder
// would cost a dependency for them: WebP in its three codings (lossy `VP8 `,
// lossless `VP8L`, extended `VP8X`), PNG, and JPEG. The corpus is 194 WebP and
// 4 JPEG files (the pictures investigation of XingZ6016, 2026-10-21), and the
// sizes read here for `image0.webp` (2400 by 2616) and `photo1.webp` (576 by
// 1280) match what `file` reports for them. A format or a damaged header this
// does not read is reported as unsized, and the estimate then holds the
// picture at a ceiling instead.
//
// THE OFFSETS ARE NAMED, each from its format's published layout: RIFF holds
// `RIFF`, a size and `WEBP` in 12 bytes, then the first chunk's four-character
// coding, its size and its payload; a lossy frame opens with a three-byte tag
// and a three-byte start code, then two little-endian 16-bit sizes with 14
// bits of size in each; a lossless one opens with a signature byte and one
// 32-bit word packing two 14-bit sizes less one; an extended one with four
// bytes of flags and two 24-bit canvas sizes less one; PNG's header chunk
// follows its 8-byte signature; a JPEG is a run of marker segments.

/**
 What a header said about a picture's size.

 @example
 ```ts
 const size: ImagePixelSize = { kind: 'sized', width: 576, height: 1280, };
 ```
 */
export type ImagePixelSize =
  | {
    /**
     Discriminator marking a size that was read.
     */
    readonly kind: 'sized';

    /**
     Pixels across.
     */
    readonly width: number;

    /**
     Pixels down.
     */
    readonly height: number;
  }
    | {
    /**
     Discriminator marking a picture whose size could not be read.
     */
      readonly kind: 'unsized';
    };

/**
 Result for a file no reader here understands or a header that is cut short.
 */
const UNSIZED: ImagePixelSize = { kind: 'unsized', };

//region WebP layout
/**
 Bytes before the first chunk's payload: `RIFF`, size, `WEBP`, coding, chunk size.
 */
const WEBP_PAYLOAD_OFFSET = 20;

/**
 Shortest file whose lossy and extended headers can be read in full.
 */
const WEBP_MIN_BYTES = 30;

/**
 Where `WEBP` stands in the container header.
 */
const WEBP_FORM_OFFSET = 8;

/**
 Where the first chunk's four-character coding stands.
 */
const WEBP_CODING_OFFSET = 12;

/**
 Width of every four-character code in these formats.
 */
const FOURCC_BYTES = 4;

/**
 Offset of a lossy frame's start code: the payload's three-byte tag, then the code.
 */
const VP8_START_CODE_OFFSET = 23;

/**
 First byte of the start code of a lossy frame.
 */
const VP8_START_CODE_FIRST = 0x9D;

/**
 Second byte of the start code of a lossy frame.
 */
const VP8_START_CODE_MIDDLE = 0x01;

/**
 Third byte of the start code of a lossy frame.
 */
const VP8_START_CODE_LAST = 0x2A;

/**
 Offset of a lossy frame's width, 16 bits little-endian, 14 of them size.
 */
const VP8_WIDTH_OFFSET = 26;

/**
 Offset of a lossy frame's height, laid out as the width is.
 */
const VP8_HEIGHT_OFFSET = 28;

/**
 Mask of the 14 bits of size in a field of a lossy or lossless header.
 */
const SIZE_MASK_14_BITS = 0x3F_FF;

/**
 Bits one packed size takes in a lossless header.
 */
const VP8L_SIZE_BITS = 14;

/**
 Offset of a lossless header's signature byte, the first byte of its payload.
 */
const VP8L_SIGNATURE_OFFSET = WEBP_PAYLOAD_OFFSET;

/**
 Value of that signature byte.
 */
const VP8L_SIGNATURE = 0x2F;

/**
 Offset of the 32-bit word packing a lossless header's two sizes.
 */
const VP8L_SIZES_OFFSET = WEBP_PAYLOAD_OFFSET + 1;

/**
 Bytes of flags and reserved fields that open an extended header's payload.
 */
const VP8X_FLAG_BYTES = 4;

/**
 Offset of an extended header's 24-bit canvas width, after the flags.
 */
const VP8X_WIDTH_OFFSET = WEBP_PAYLOAD_OFFSET + VP8X_FLAG_BYTES;

/**
 Bytes of one 24-bit canvas size.
 */
const VP8X_SIZE_BYTES = 3;

/**
 Offset of an extended header's 24-bit canvas height.
 */
const VP8X_HEIGHT_OFFSET = VP8X_WIDTH_OFFSET + VP8X_SIZE_BYTES;
//endregion WebP layout

//region PNG layout
/**
 Offset of a PNG's header chunk name, after its 8-byte signature and 4-byte length.
 */
const PNG_CHUNK_NAME_OFFSET = 12;

/**
 Offset of a PNG's width, 32 bits big-endian, after the chunk name.
 */
const PNG_WIDTH_OFFSET = 16;

/**
 Offset of a PNG's height.
 */
const PNG_HEIGHT_OFFSET = 20;

/**
 Shortest file holding a PNG's width and height.
 */
const PNG_MIN_BYTES = 24;
//endregion PNG layout

//region JPEG layout
/**
 Where a JPEG's first segment starts, after the two-byte start of image.
 */
const JPEG_FIRST_SEGMENT_OFFSET = 2;

/**
 Bytes a segment marker takes.
 */
const JPEG_MARKER_BYTES = 2;

/**
 Bytes a segment needs before its marker and length can be read.
 */
const JPEG_SEGMENT_HEAD_BYTES = 4;

/**
 Bytes from a frame header's marker to the end of its width.
 */
const JPEG_FRAME_HEAD_BYTES = 9;

/**
 Offset, from a frame header's marker, of the height.
 */
const JPEG_FRAME_HEIGHT_OFFSET = 5;

/**
 Offset, from a frame header's marker, of the width.
 */
const JPEG_FRAME_WIDTH_OFFSET = 7;

/**
 First byte of every marker.
 */
const JPEG_MARKER_PREFIX = 0xFF;

/**
 Second byte of the start of image, and so of every JPEG file.
 */
const JPEG_START_OF_IMAGE = 0xD8;

/**
 Marker that ends an image; a frame header before it is missing.
 */
const JPEG_END_OF_IMAGE = 0xD9;

/**
 Marker that opens the entropy-coded data; a frame header before it is missing.
 */
const JPEG_START_OF_SCAN = 0xDA;

/**
 First of the restart markers, which stand alone, the run ending at the start
 of image.
 */
const JPEG_FIRST_STANDALONE = 0xD0;

/**
 First marker code of the start-of-frame family.
 */
const JPEG_FIRST_FRAME = 0xC0;

/**
 Last marker code of the start-of-frame family.
 */
const JPEG_LAST_FRAME = 0xCF;

/**
 Marker of the Huffman table definition, inside the start-of-frame range.
 */
const JPEG_HUFFMAN_TABLE = 0xC4;

/**
 Marker of the reserved JPEG extension, inside the start-of-frame range.
 */
const JPEG_EXTENSION = 0xC8;

/**
 Marker of the arithmetic conditioning, inside the start-of-frame range.
 */
const JPEG_ARITHMETIC_CONDITIONING = 0xCC;

/**
 Codes inside the start-of-frame range that are not frame headers.
 */
const JPEG_NOT_FRAME: ReadonlySet<number> = new Set([
  JPEG_HUFFMAN_TABLE,
  JPEG_EXTENSION,
  JPEG_ARITHMETIC_CONDITIONING,
],);

/**
 The zero stuffing byte, which follows a 0xFF inside entropy-coded data.
 */
const JPEG_STUFFING = 0x00;

/**
 The temporary marker, which carries no length.
 */
const JPEG_TEMPORARY = 0x01;

/**
 Marker codes that carry no length: the stuffing byte and the temporary marker.
 */
const JPEG_LENGTHLESS: ReadonlySet<number> = new Set([
  JPEG_STUFFING,
  JPEG_TEMPORARY,
],);
//endregion JPEG layout

/**
 A size, or that the header gave none, from the two numbers a reader found.

 @param width - pixels across as read

 @param height - pixels down as read

 @returns Sized when both are positive

 @example
 ```ts
 const size = sizeOf({ width: 576, height: 1280, },);
 ```
 */
function sizeOf(
  {
    width,
    height,
  }: {
    readonly width: number;
    readonly height: number;
  },
): ImagePixelSize {
  if ((width < 1) || (height < 1))
    return UNSIZED;
  return {
    kind: 'sized',
    width,
    height,
  };
}

/**
 Four characters of a header, read as text.

 @param bytes - the file

 @param offset - where the code starts

 @returns The code, shorter when the file ends first

 @example
 ```ts
 fourccAt({ bytes, offset: 8, },); // 'WEBP'
 ```
 */
function fourccAt(
  {
    bytes,
    offset,
  }: {
    readonly bytes: Buffer;
    readonly offset: number;
  },
): string {
  return bytes.toString(
    'latin1',
    offset,
    offset + FOURCC_BYTES,
  );
}

/**
 One 24-bit canvas size of an extended WebP header, which stores it less one.

 @param bytes - the file

 @param offset - where the 24 bits start

 @returns The size in pixels

 @example
 ```ts
 const width = canvasSizeAt({ bytes, offset: 24, },);
 ```
 */
function canvasSizeAt(
  {
    bytes,
    offset,
  }: {
    readonly bytes: Buffer;
    readonly offset: number;
  },
): number {
  return bytes.readUIntLE(
    offset,
    VP8X_SIZE_BYTES,
  ) + 1;
}

/**
 Whether a file opens with a PNG signature: a high byte, then `PNG`.

 @param bytes - the file

 @returns True for a PNG

 @example
 ```ts
 const png = pngSignatureOf({ bytes, },);
 ```
 */
function pngSignatureOf({ bytes, }: { readonly bytes: Buffer; },): boolean {
  /**
   The three characters after the first byte.
   */
  const name = fourccAt({
    bytes,
    offset: 1,
  },);
  return name.startsWith('PNG',);
}

/**
 Size of a lossy WebP frame.

 @param bytes - the file

 @returns Size read, or unsized when the frame's start code is absent

 @example
 ```ts
 const size = lossySizeOf({ bytes, },);
 ```
 */
function lossySizeOf({ bytes, }: { readonly bytes: Buffer; },): ImagePixelSize {
  if (bytes[VP8_START_CODE_OFFSET] !== VP8_START_CODE_FIRST)
    return UNSIZED;
  if (bytes[VP8_START_CODE_OFFSET + 1] !== VP8_START_CODE_MIDDLE)
    return UNSIZED;
  if (bytes[VP8_START_CODE_OFFSET + 2] !== VP8_START_CODE_LAST)
    return UNSIZED;
  return sizeOf({
    width: bytes.readUInt16LE(VP8_WIDTH_OFFSET,) & SIZE_MASK_14_BITS,
    height: bytes.readUInt16LE(VP8_HEIGHT_OFFSET,) & SIZE_MASK_14_BITS,
  },);
}

/**
 Size of a lossless WebP header.

 @param bytes - the file

 @returns Size read, or unsized when the signature byte is absent

 @example
 ```ts
 const size = losslessSizeOf({ bytes, },);
 ```
 */
function losslessSizeOf({ bytes, }: { readonly bytes: Buffer; },): ImagePixelSize {
  if (bytes[VP8L_SIGNATURE_OFFSET] !== VP8L_SIGNATURE)
    return UNSIZED;

  /**
   Packed fields: 14 bits of width less one, then 14 of height less one.
   */
  const packed = bytes.readUInt32LE(VP8L_SIZES_OFFSET,);
  return sizeOf({
    width: (packed & SIZE_MASK_14_BITS) + 1,
    height: ((packed >>> VP8L_SIZE_BITS) & SIZE_MASK_14_BITS) + 1,
  },);
}

/**
 Size of a WebP file, in any of its three codings.

 @param bytes - the file

 @returns Size read off the first chunk, or unsized

 @example
 ```ts
 const size = webpSizeOf({ bytes, },);
 ```
 */
function webpSizeOf({ bytes, }: { readonly bytes: Buffer; },): ImagePixelSize {
  if (bytes.length < WEBP_MIN_BYTES)
    return UNSIZED;
  if (fourccAt({
    bytes,
    offset: WEBP_FORM_OFFSET,
  },) !== 'WEBP')
    return UNSIZED;

  /**
   Coding of the first chunk.
   */
  const coding = fourccAt({
    bytes,
    offset: WEBP_CODING_OFFSET,
  },);
  if (coding === 'VP8 ')
    return lossySizeOf({ bytes, },);
  if (coding === 'VP8L')
    return losslessSizeOf({ bytes, },);
  if (coding === 'VP8X') {
    return sizeOf({
      width: canvasSizeAt({
        bytes,
        offset: VP8X_WIDTH_OFFSET,
      },),
      height: canvasSizeAt({
        bytes,
        offset: VP8X_HEIGHT_OFFSET,
      },),
    },);
  }
  return UNSIZED;
}

/**
 Size of a PNG file, off its header chunk.

 @param bytes - the file

 @returns Size read, or unsized

 @example
 ```ts
 const size = pngSizeOf({ bytes, },);
 ```
 */
function pngSizeOf({ bytes, }: { readonly bytes: Buffer; },): ImagePixelSize {
  if (bytes.length < PNG_MIN_BYTES)
    return UNSIZED;
  if (fourccAt({
    bytes,
    offset: PNG_CHUNK_NAME_OFFSET,
  },) !== 'IHDR')
    return UNSIZED;
  return sizeOf({
    width: bytes.readUInt32BE(PNG_WIDTH_OFFSET,),
    height: bytes.readUInt32BE(PNG_HEIGHT_OFFSET,),
  },);
}

/**
 Whether a marker opens a frame header in a JPEG file: the start-of-frame
 family, less the codes in that range that mean something else.

 @param marker - code after the 0xFF

 @returns True for a frame header

 @example
 ```ts
 isFrameMarker({ marker: 0xC0, },); // true
 ```
 */
function isFrameMarker({ marker, }: { readonly marker: number; },): boolean {
  if (JPEG_NOT_FRAME.has(marker,))
    return false;
  return (marker >= JPEG_FIRST_FRAME) && (marker <= JPEG_LAST_FRAME);
}

/**
 Offset of the segment after the one at an offset, in a JPEG file.

 @param bytes - the file

 @param offset - start of the segment under inspection

 @returns Where the next segment starts; always past `offset`

 @example
 ```ts
 const next = jpegSegmentAfter({ bytes, offset: 2, },);
 ```
 */
function jpegSegmentAfter(
  {
    bytes,
    offset,
  }: {
    readonly bytes: Buffer;
    readonly offset: number;
  },
): number {
  if (bytes[offset] !== JPEG_MARKER_PREFIX)
    return offset + 1;

  /**
   Marker byte after the 0xFF.
   */
  const marker = bytes[offset + 1] ?? 0;
  // A fill byte repeats the prefix.
  if (marker === JPEG_MARKER_PREFIX)
    return offset + 1;
  // Markers that stand alone carry no length: the stuffing byte, the
  // temporary marker, the restarts and the start of image.
  if (JPEG_LENGTHLESS.has(marker,) || ((marker >= JPEG_FIRST_STANDALONE) && (marker <= JPEG_START_OF_IMAGE)))
    return offset + JPEG_MARKER_BYTES;
  /**
   Length the segment states, which counts its own two bytes.
   */
  const stated = bytes.readUInt16BE(offset + JPEG_MARKER_BYTES,);

  /**
   Bytes of the segment's body, at least the two its length field takes.
   */
  const body = Math.max(
    JPEG_MARKER_BYTES,
    stated,
  );

  /**
   Where the body starts, after the marker.
   */
  const bodyStart = offset + JPEG_MARKER_BYTES;
  return bodyStart + body;
}

/**
 Size of a JPEG file, off the first frame header.

 @param bytes - the file

 @returns Size read, or unsized

 @example
 ```ts
 const size = jpegSizeOf({ bytes, },);
 ```
 */
function jpegSizeOf({ bytes, }: { readonly bytes: Buffer; },): ImagePixelSize {
  for (
    let offset = JPEG_FIRST_SEGMENT_OFFSET;
    (offset + JPEG_SEGMENT_HEAD_BYTES) <= bytes.length;
    offset = jpegSegmentAfter({
      bytes,
      offset,
    },)
  ) {
    if (bytes[offset] !== JPEG_MARKER_PREFIX)
      continue;

    /**
     Marker byte after the 0xFF.
     */
    const marker = bytes[offset + 1] ?? 0;
    if (isFrameMarker({ marker, },)) {
      if ((offset + JPEG_FRAME_HEAD_BYTES) > bytes.length)
        return UNSIZED;
      return sizeOf({
        width: bytes.readUInt16BE(offset + JPEG_FRAME_WIDTH_OFFSET,),
        height: bytes.readUInt16BE(offset + JPEG_FRAME_HEIGHT_OFFSET,),
      },);
    }
    // The scan or the end of the image before any frame header is a damaged file.
    if ((marker === JPEG_START_OF_SCAN) || (marker === JPEG_END_OF_IMAGE))
      return UNSIZED;
  }
  return UNSIZED;
}

/**
 Pixel size of a picture, by the format its first bytes name.

 @param bytes - the file as it sits in the corpus

 @returns Size read off the header, or unsized for a format or a header this
 does not read

 @example
 ```ts
 const size = imagePixelSizeOf({ bytes, },);
 ```
 */
export function imagePixelSizeOf({ bytes, }: { readonly bytes: Buffer; },): ImagePixelSize {
  if (bytes.length < FOURCC_BYTES)
    return UNSIZED;
  if (fourccAt({
    bytes,
    offset: 0,
  },) === 'RIFF')
    return webpSizeOf({ bytes, },);
  if (pngSignatureOf({ bytes, },))
    return pngSizeOf({ bytes, },);
  if ((bytes[0] === JPEG_MARKER_PREFIX) && (bytes[1] === JPEG_START_OF_IMAGE))
    return jpegSizeOf({ bytes, },);
  return UNSIZED;
}

//endregion NeuralWatt image size
