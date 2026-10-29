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
import type { VoyageContentItem, } from './types.voyage-api.ts';

/**
 Logger root for image-diff after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l, },);
 ```
 */
const l = tagged({ tag: 'image-diff', },);

/**
 Normalize any {@link ImageInput} variant into a {@link VoyageContentItem}
 suitable for the Voyage AI multimodal embeddings API.
 
 - {@link ImageUrl} maps to `image_url` content type
 - {@link ImageBase64} maps to `image_base64` content type
 - {@link ImageBuffer} is base64-encoded and maps to `image_base64`
 - {@link ImagePath} is read from disk, base64-encoded, and maps to `image_base64`
 
 @param input - image in any supported format
 
 @returns content item ready for the API request
 
 @example
 ```ts
 const item = await toVoyageContentItem({ path: 'photo.png' });
 // { type: 'image_base64', image_base64: 'data:image/png;base64,...' }
 ```
 */
export async function toVoyageContentItem(
  input: ImageInput,
): Promise<VoyageContentItem> {
  /**
   Logger pre-tagged with this function's name so call-site context is preserved across debug lines.
   */
  const rl = tagged({
    tag: toVoyageContentItem.name,
    l,
  },);

  if (isImageUrl(input,)) {
    rl.debug(`using URL input: ${input.url}`,);
    return {
      type: 'image_url',
      image_url: input.url,
    };
  }

  if (isImageBase64(input,)) {
    rl.debug('using pre-encoded base64 input',);
    return {
      type: 'image_base64',
      image_base64: input.base64,
    };
  }

  if (isImageBuffer(input,)) {
    rl.debug(
      `encoding buffer (${
        String(input
          .buffer
          .byteLength,)
      } bytes, format: ${input.format})`,
    );
    /**
     Buffer re-encoded as a `data:` URI; Voyage's `image_base64` content type expects the full URI form.
     */
    const dataUri = bufferToDataUri({
      buffer: input.buffer,
      format: input.format,
    },);
    return {
      type: 'image_base64',
      image_base64: dataUri,
    };
  }

  if (isImagePath(input,)) {
    rl.debug(`reading file: ${input.path}`,);
    /**
     Image format inferred from the path's extension; drives the data-URI's media type.
     */
    const format = inferFormat(input.path,);
    /**
     Raw file bytes read from disk; re-encoded as a data URI below.
     */
    const fileBuffer = await readFile(input.path,);
    /**
     File contents re-encoded as a `data:` URI; Voyage's `image_base64` content type expects the full URI form.
     */
    const dataUri = bufferToDataUri({
      buffer: fileBuffer.buffer,
      format,
    },);
    return {
      type: 'image_base64',
      image_base64: dataUri,
    };
  }

  throw new Error(
    'Unrecognized ImageInput variant: expected one of buffer, path, url, or base64',
  );
}
