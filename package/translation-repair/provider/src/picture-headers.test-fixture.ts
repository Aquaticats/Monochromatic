/**
 Hand-built image file headers for the tests of the picture size and the
 picture tokens: a WebP with a chosen first chunk, a lossy WebP of a chosen
 size, and the data URI a picture travels as. Cat-themed invention: headers
 and padding, no pixels.

 @module
 */

/**
 Bytes of the container header: the form size, the form name and the first chunk's name and size.
 */
const RIFF_HEADER_BYTES = 20;

/**
 Where the container's size is stored.
 */
const RIFF_SIZE_AT = 4;

/**
 Bytes the container's size counts beyond the chunk's payload.
 */
const RIFF_SIZE_OVERHEAD = 12;

/**
 Where the form name is stored.
 */
const FORM_NAME_AT = 8;

/**
 Where the first chunk's name is stored.
 */
const FOURCC_AT = 12;

/**
 Where the first chunk's size is stored.
 */
const CHUNK_SIZE_AT = 16;

/**
 Bytes of the lossy frame the size is read from.
 */
const FRAME_BYTES = 40;

/**
 Where the frame's start code is stored.
 */
const START_CODE_AT = 3;

/**
 Where the frame's width is stored.
 */
const FRAME_WIDTH_AT = 6;

/**
 Where the frame's height is stored.
 */
const FRAME_HEIGHT_AT = 8;

/**
 The three bytes of a lossy frame's start code.
 */
const START_CODE = Buffer.from(
  '9d012a',
  'hex',
);

/**
 A WebP file with the given first chunk.

 @param fourcc - chunk coding

 @param chunk - chunk payload

 @returns The file's bytes

 @example
 ```ts
 const bytes = webpOf({ fourcc: 'VP8L', chunk, },);
 ```
 */
export function webpOf({
  fourcc,
  chunk,
}: {
  readonly fourcc: string;
  readonly chunk: Buffer
},): Buffer {
  /**
   Container header.
   */
  const riff = Buffer.alloc(RIFF_HEADER_BYTES,);
  riff.write('RIFF',);
  riff.writeUInt32LE(
    RIFF_SIZE_OVERHEAD + chunk.length,
    RIFF_SIZE_AT,
  );
  riff.write(
    'WEBP',
    FORM_NAME_AT,
  );
  riff.write(
    fourcc,
    FOURCC_AT,
  );
  riff.writeUInt32LE(
    chunk.length,
    CHUNK_SIZE_AT,
  );
  return Buffer.concat([
    riff,
    chunk,
  ],);
}

/**
 A lossy WebP of the stated size.

 @param width - pixels across

 @param height - pixels down

 @returns The file's bytes

 @example
 ```ts
 const bytes = lossy({ width: 576, height: 1280, },);
 ```
 */
export function lossy({
  width,
  height,
}: {
  readonly width: number;
  readonly height: number
},): Buffer {
  /**
   Frame tag, start code and size.
   */
  const chunk = Buffer.alloc(FRAME_BYTES,);
  Buffer.from(START_CODE,)
    .copy(
    chunk,
    START_CODE_AT,
    );
  chunk.writeUInt16LE(
    width,
    FRAME_WIDTH_AT,
  );
  chunk.writeUInt16LE(
    height,
    FRAME_HEIGHT_AT,
  );
  return webpOf({
    fourcc: 'VP8 ',
    chunk,
  },);
}

/**
 A base64 data URI around bytes.

 @param bytes - the file

 @returns URI as the reading stage builds it

 @example
 ```ts
 const url = uriOf({ bytes, },);
 ```
 */
export function uriOf({ bytes, }: { readonly bytes: Buffer; },): string {
  return `data:image/webp;base64,${bytes.toString('base64',)}`;
}
