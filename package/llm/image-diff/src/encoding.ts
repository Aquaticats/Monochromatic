import { extname, } from 'node:path';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import type {
  ImageBase64,
  ImageBuffer,
  ImageFormat,
  ImageInput,
  ImagePath,
  ImageUrl,
} from './types.ts';

/**
 Logger root for image-diff after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l, },);
 ```
 */
const l = tagged({ tag: 'image-diff', },);

/**
 Extension-to-format mapping for inferring image format from file paths.
 */
const EXTENSION_FORMAT_MAP: Record<string, ImageFormat> = {
  '.png': 'png',
  '.jpg': 'jpeg',
  '.jpeg': 'jpeg',
  '.webp': 'webp',
  '.gif': 'gif',
};

/**
 Type guard for {@link ImageBuffer} inputs.
 
 @param input - image input to test
 
 @returns whether the input contains a `buffer` property
 
 @example
 ```ts
 if (isImageBuffer(input)) {
   console.log(input.buffer.byteLength);
 }
 ```
 */
export function isImageBuffer(input: ImageInput,): input is ImageBuffer {
  return 'buffer' in input;
}

/**
 Type guard for {@link ImagePath} inputs.
 
 @param input - image input to test
 
 @returns whether the input contains a `path` property
 
 @example
 ```ts
 if (isImagePath(input)) {
   console.log(input.path);
 }
 ```
 */
export function isImagePath(input: ImageInput,): input is ImagePath {
  return 'path' in input;
}

/**
 Type guard for {@link ImageUrl} inputs.
 
 @param input - image input to test
 
 @returns whether the input contains a `url` property
 
 @example
 ```ts
 if (isImageUrl(input)) {
   console.log(input.url);
 }
 ```
 */
export function isImageUrl(input: ImageInput,): input is ImageUrl {
  return 'url' in input;
}

/**
 Type guard for {@link ImageBase64} inputs.
 
 @param input - image input to test
 
 @returns whether the input contains a `base64` property
 
 @example
 ```ts
 if (isImageBase64(input)) {
   console.log(input.base64.slice(0, 30));
 }
 ```
 */
export function isImageBase64(input: ImageInput,): input is ImageBase64 {
  return 'base64' in input;
}

/**
 Infer the {@link ImageFormat} from a file extension.
 
 @param filePath - path whose extension determines the format
 
 @returns inferred format
 
 @throws when the extension is not a recognized image format
 
 @example
 ```ts
 const fmt = inferFormat('/tmp/photo.png'); // 'png'
 ```
 */
export function inferFormat(filePath: string,): ImageFormat {
  /**
   Logger pre-tagged with this function's name so call-site context is preserved across debug lines.
   */
  const rl = tagged({
    tag: inferFormat.name,
    l,
  },);
  /**
   Path extension normalised to lower case for case-insensitive lookup in {@link EXTENSION_FORMAT_MAP}.
   */
  const ext = extname(filePath,)
    .toLowerCase();
  /**
   Format resolved from the extension; `undefined` for unsupported extensions triggers the explicit error below.
   */
  const format = EXTENSION_FORMAT_MAP[ext];
  if (format === undefined) {
    throw new Error(
      `Unsupported image extension "${ext}" for path "${filePath}". Supported: ${
        Object
          .keys(EXTENSION_FORMAT_MAP,)
          .join(', ',)
      }`,
    );
  }
  rl.debug(`inferred format "${format}" from extension "${ext}"`,);
  return format;
}

/**
 Convert an {@link ArrayBuffer} to a raw base64 string (no data URI prefix).
 
 @param buffer - raw image bytes
 
 @returns base64-encoded string
 
 @example
 ```ts
 const b64 = bufferToBase64(bytes);
 ```
 */
export function bufferToBase64(buffer: ArrayBuffer,): string {
  /**
   Byte-level view over the buffer; `btoa` needs a string of single-byte chars, not the raw buffer.
   */
  const bytes = new Uint8Array(buffer,);
  /**
   Binary string assembled byte by byte; one code point per byte is what `btoa` expects.
   */
  const binary = Array
    .from(
    bytes,
    function byteToChar(byte,) {
      return String.fromCodePoint(byte,);
    },
    )
    .join('',);
  return btoa(binary,);
}

/**
 Convert an {@link ArrayBuffer} to a base64-encoded data URI string.
 
 @param buffer - raw image bytes
 
 @param format - image format for the media type prefix
 
 @returns data URI like `data:image/png;base64,...`
 
 @example
 ```ts
 const uri = bufferToDataUri({ buffer: bytes, format: 'png' });
 // 'data:image/png;base64,iVBOR...'
 ```
 */
export function bufferToDataUri({
  buffer,
  format,
}: {
  readonly buffer: ArrayBuffer;
  readonly format: ImageFormat;
},): string {
  /**
   Logger pre-tagged with this function's name so call-site context is preserved across debug lines.
   */
  const rl = tagged({
    tag: bufferToDataUri.name,
    l,
  },);
  /**
   Raw base64 payload; prefixed below with the media-type header to produce the final data URI.
   */
  const base64 = bufferToBase64(buffer,);
  rl.debug(
    `encoded ${String(new Uint8Array(buffer,).length,)} bytes as base64 data URI`,
  );
  return `data:image/${format};base64,${base64}`;
}

/**
 Parse a data URI string into its raw base64 data and MIME type.
 
 @param dataUri - full data URI (e.g. `data:image/png;base64,iVBOR...`)
 
 @returns parsed mime type and raw base64 data
 
 @throws when the data URI format is not recognized
 
 @example
 ```ts
 const { mimeType, data } = parseDataUri('data:image/png;base64,iVBOR...');
 // mimeType === 'image/png', data === 'iVBOR...'
 ```
 */
export function parseDataUri(dataUri: string,): {
  mimeType: string;
  data: string;
} {
  /**
   Literal scheme prefix; everything before the MIME type.
   */
  const SCHEME = 'data:';
  /**
   Literal payload separator joining the MIME type and the encoded body.
   */
  const SEPARATOR = ';base64,';
  /**
   Maximum chars of `dataUri` echoed back in error messages so they stay readable.
   */
  const ERROR_PREVIEW_LENGTH = 50;

  /**
   Throws a normalised `Invalid data URI format` error with the truncated input.
   
   @throws Error describing the expected shape and the truncated input
   */
  function reject(): never {
    throw new Error(
      `Invalid data URI format: expected "data:<mime>;base64,<data>", got "${
        dataUri.slice(
          0,
          ERROR_PREVIEW_LENGTH,
        )
      }..."`,
    );
  }

  if (!dataUri.startsWith(SCHEME,))
    reject();
  /**
   Cursor at the first MIME-type character.
   */
  const mimeStart = SCHEME.length;
  /**
   Position of `;base64,` separator; `-1` means the URI lacks the base64 marker.
   */
  const sepIdx = dataUri.indexOf(
    SEPARATOR,
    mimeStart,
  );
  if ((sepIdx === (-1)) || (sepIdx === mimeStart))
    reject();
  /**
   Cursor at the first payload byte after the separator.
   */
  const dataStart = sepIdx + SEPARATOR
    .length;
  if (dataStart >= dataUri
    .length)
    reject();
  return {
    mimeType: dataUri.slice(
      mimeStart,
      sepIdx,
    ),
    data: dataUri.slice(dataStart,),
  };
}
