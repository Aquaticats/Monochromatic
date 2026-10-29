import { readFile, } from 'node:fs/promises';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  bufferToBase64,
  inferFormat,
  isImageBase64,
  isImageBuffer,
  isImagePath,
  isImageUrl,
  parseDataUri,
} from './encoding.ts';
import type { GeminiInlineData, } from './types.gemini-api.ts';
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
 Normalize any {@link ImageInput} variant into a {@link GeminiInlineData}
 suitable for the Gemini embedContent API.
 
 Gemini requires raw base64 data (no data URI prefix) and an explicit MIME type.
 URL inputs are fetched and converted to inline data.
 
 @param input - image in any supported format
 
 @returns inline data ready for the Gemini API request
 
 @example
 ```ts
 const item = await toGeminiInlineData({ path: 'photo.png' });
 // { mime_type: 'image/png', data: 'iVBOR...' }
 ```
 */
export async function toGeminiInlineData(input: ImageInput,): Promise<GeminiInlineData> {
  /**
   Logger pre-tagged with this function's name so call-site context is preserved across debug lines.
   */
  const rl = tagged({
    tag: toGeminiInlineData.name,
    l,
  },);

  if (isImageUrl(input,)) {
    rl.debug(`fetching URL for Gemini inline data: ${input.url}`,);
    /**
     Raw `fetch` response; status checked before reading the body so transport errors surface clearly.
     */
    const response = await fetch(input.url,);
    if (!response.ok) {
      throw new Error(
        `Failed to fetch image URL "${input.url}": ${String(response.status,)}`,
      );
    }
    /**
     MIME type pulled from the response, defaulted to `image/png` when the server omits the header.
     */
    const contentType = response.headers
      .get('content-type',)
      ?? 'image/png';
    /**
     Image bytes pulled into an `ArrayBuffer` before being re-encoded as base64 below.
     */
    const arrayBuffer = await response.arrayBuffer();
    /**
     Base64-encoded image payload; Gemini requires the raw form, without a data-URI prefix.
     */
    const data = bufferToBase64(arrayBuffer,);
    return {
      mime_type: contentType,
      data,
    };
  }

  if (isImageBase64(input,)) {
    rl.debug('parsing data URI for Gemini inline data',);
    /**
     MIME type and base64 body extracted from the caller's `data:` URI for direct Gemini consumption.
     */
    const {
      mimeType,
      data,
    } = parseDataUri(input.base64,);
    return {
      mime_type: mimeType,
      data,
    };
  }

  if (isImageBuffer(input,)) {
    rl.debug(
      `encoding buffer for Gemini (${
        String(input
          .buffer
          .byteLength,)
      } bytes, format: ${input.format})`,
    );
    /**
     Base64-encoded buffer payload; Gemini requires raw base64 without a data-URI prefix.
     */
    const data = bufferToBase64(input.buffer,);
    return {
      mime_type: `image/${input.format}`,
      data,
    };
  }

  if (isImagePath(input,)) {
    rl.debug(`reading file for Gemini: ${input.path}`,);
    /**
     Image format inferred from the path's extension; drives the `image/<format>` MIME type.
     */
    const format = inferFormat(input.path,);
    /**
     Raw file bytes read from disk; re-encoded as base64 below.
     */
    const fileBuffer = await readFile(input.path,);
    /**
     Base64-encoded file contents; Gemini requires raw base64 without a data-URI prefix.
     */
    const data = bufferToBase64(fileBuffer.buffer,);
    return {
      mime_type: `image/${format}`,
      data,
    };
  }

  throw new Error(
    'Unrecognized ImageInput variant: expected one of buffer, path, url, or base64',
  );
}
