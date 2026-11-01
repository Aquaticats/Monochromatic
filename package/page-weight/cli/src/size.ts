/**
 Resolves the on-wire transfer size for an asset.
 
 When a `.zst` companion exists alongside the requested file,
 its size is returned instead; this mirrors file servers that
 serve pre-compressed variants when the client advertises zstd support.
 Otherwise the raw file size is returned.
 */
import { stat, } from 'node:fs/promises';

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';

/**
 Sentinel returned by {@link wireSize} when neither the requested asset nor
 its `.zst` companion exists. A `unique symbol` rather than a falsy default
 because `0` is a valid wire size (an empty file); callers narrow with
 `=== WIRE_SIZE_UNAVAILABLE`.
 */
export const WIRE_SIZE_UNAVAILABLE: unique symbol = Symbol('page-weight wire size cannot be resolved',);

/**
 Returns `true` if the path exists and is a regular file, `false` otherwise.
 
 Any error accessing the path (permissions, I/O) also yields `false`.
 
 @param absolutePath - absolute filesystem path to probe
 
 @returns whether the path resolves to a readable file
 
 @example
 ```ts
 if (await fileReadable('/srv/www/index.html')) { ... }
 ```
 */
async function fileReadable(
  absolutePath: string,
): Promise<boolean> {
  try {
    /**
     Stat record used to distinguish files from directories or symlinks.
     */
    const info = await stat(absolutePath,);
    return info.isFile();
  }
  catch (error) {
    console.warn(
      `[page-weight] file readability probe failed for ${absolutePath}: ${caughtValueText(error,)}`,
    );
    return false;
  }
}

/**
 Resolves the wire size of a file, preferring a `.zst` companion when present.
 
 If `path.zst` exists as a regular file, its size is returned.
 Otherwise the raw size of `path` is returned.
 Returns {@link WIRE_SIZE_UNAVAILABLE} when neither form exists, letting
 callers decide how to handle dead references instead of silently
 zero-counting them.
 
 @param absolutePath - absolute path to the originally-requested asset
 
 @returns wire size in bytes, or {@link WIRE_SIZE_UNAVAILABLE} if the file is
   missing
 
 @example
 ```ts
 const bytes = await wireSize('/srv/www/styles.css');
 ```
 */
export async function wireSize(
  absolutePath: string,
): Promise<number | typeof WIRE_SIZE_UNAVAILABLE> {
  /**
   Candidate companion path; pre-compressed asset served when the client supports zstd.
   */
  const zstPath = `${absolutePath}.zst`;
  if (await fileReadable(zstPath,)) {
    /**
     Stat of the `.zst` companion; only `size` is needed downstream.
     */
    const info = await stat(zstPath,);
    return info.size;
  }
  if (await fileReadable(absolutePath,)) {
    /**
     Stat of the uncompressed asset; fallback when no `.zst` companion exists.
     */
    const info = await stat(absolutePath,);
    return info.size;
  }
  return WIRE_SIZE_UNAVAILABLE;
}
