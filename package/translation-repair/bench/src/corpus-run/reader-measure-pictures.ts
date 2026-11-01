import { readFile, } from 'node:fs/promises';
import {
  basename,
  extname,
  join,
} from 'node:path';

import { allInInputOrder, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { filesUnder, } from './reader-measure-files.ts';
import type { MeasuredPicture, } from './reader-measure-run.ts';

//region Reader measure pictures
// THE PICTURES A MEASUREMENT TRIES: every picture file under a directory at
// any depth, in path order, named by its path under that directory so two
// people's `photo1.webp` stay apart in the report.

/**
 Extensions of the picture files the corpus holds, the ones the reader stage
 can name a media type for.
 */
const PICTURE_EXTENSIONS: ReadonlySet<string> = new Set([
  '.webp',
  '.png',
  '.jpg',
  '.jpeg',
  '.gif',
],);

/**
 Reads the picture files under a directory.

 @param dir - directory holding the pictures, at any depth

 @returns Each picture's path under `dir`, as its asset name, and its bytes,
 in path order; hidden files are skipped

 @example
 ```ts
 const pictures = await loadPictures({ dir: '/people', },);
 ```
 */
export async function loadPictures({ dir, }: { readonly dir: string; },): Promise<readonly MeasuredPicture[]> {
  /**
   Paths of the picture files under the directory, in code point order.
   */
  const paths = (await filesUnder({ dir, },)).filter(function isPicture(path,): boolean {
    return PICTURE_EXTENSIONS.has(extname(path,)
      .toLowerCase(),) && (!basename(path,)
        .startsWith('.',));
  },);
  return await allInInputOrder({
    members: paths.map(async function load(path,): Promise<MeasuredPicture> {
      return {
        assetName: path,
        bytes: await readFile(join(
          dir,
          path,
        ),),
      };
    },),
  },);
}

//endregion Reader measure pictures
