import type { ImageInput, } from './types.ts';

/**
 Parse a positional image argument into an {@link ImageInput}.
 URLs (starting with http:// or https://) become URL inputs;
 everything else is treated as a file path.
 
 @param arg - CLI positional argument
 
 @returns parsed image input
 
 @example
 ```ts
 parseImageArg('photo.png') // { path: 'photo.png' }
 parseImageArg('https://example.com/a.jpg') // { url: 'https://...' }
 ```
 */
export function parseImageArg(arg: string,): ImageInput {
  if (arg.startsWith('http://',)
    || arg
      .startsWith('https://',))
    return { url: arg, };
  return { path: arg, };
}
