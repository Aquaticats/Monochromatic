import { readFile, } from 'node:fs/promises';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  bufferToDataUri,
  inferFormat,
  isImageBase64,
  isImageBuffer,
  isImagePath,
  isImageUrl,
} from './encoding.ts';
import type { ImageInput, } from './types.ts';

/**
 Logger root for image-diff after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l, },);
 ```
 */
const l = tagged({ tag: 'image-diff', },);

/**
 Convert any {@link ImageInput} variant into a URI string suitable for
 OpenAI-compatible vision APIs (regular URL or base64 data URI).
 
 - {@link ImageUrl} passes through as-is
 - {@link ImageBase64} passes through as-is (already a data URI)
 - {@link ImageBuffer} is base64-encoded into a data URI
 - {@link ImagePath} is read from disk and base64-encoded into a data URI
 
 @param input - image in any supported format
 
 @returns URL or data URI string
 
 @example
 ```ts
 const uri = await toImageUri({ path: 'photo.png' });
 // 'data:image/png;base64,iVBOR...'
 ```
 */
export async function toImageUri(input: ImageInput,): Promise<string> {
  /**
   Logger pre-tagged with this function's name so call-site context is preserved across debug lines.
   */
  const rl = tagged({
    tag: toImageUri.name,
    l,
  },);

  if (isImageUrl(input,)) {
    rl.debug(`passing through URL: ${input.url}`,);
    return input.url;
  }

  if (isImageBase64(input,)) {
    rl.debug('passing through base64 data URI',);
    return input.base64;
  }

  if (isImageBuffer(input,)) {
    rl.debug(
      `encoding buffer as data URI (${
        String(input
          .buffer
          .byteLength,)
      } bytes, format: ${input.format})`,
    );
    return bufferToDataUri({
      buffer: input.buffer,
      format: input.format,
    },);
  }

  if (isImagePath(input,)) {
    rl.debug(`reading file as data URI: ${input.path}`,);
    /**
     Image format inferred from the path's extension; drives the data-URI's media type.
     */
    const format = inferFormat(input.path,);
    /**
     Raw file bytes read from disk; re-encoded as a data URI below.
     */
    const fileBuffer = await readFile(input.path,);
    return bufferToDataUri({
      buffer: fileBuffer.buffer,
      format,
    },);
  }

  throw new Error(
    'Unrecognized ImageInput variant: expected one of buffer, path, url, or base64',
  );
}
