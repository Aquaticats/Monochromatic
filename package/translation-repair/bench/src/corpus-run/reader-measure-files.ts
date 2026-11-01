import { join, } from 'node:path';

import {
  allInInputOrder,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import { namesOfKind, } from '@monochromatic-dev/translation-repair-glossary/ts';
import { textsInCodePointOrder, } from '@monochromatic-dev/translation-repair-markup/ts';

//region Reader measure files
// THE FILES UNDER A DIRECTORY, AT ANY DEPTH, through the package's one directory
// lister (`namesOfKind`), which takes a regular file or a directory and never a
// link, so a link to a record is not a second record and a link out of the
// directory is not followed.

/**
 Lists the regular files under a directory, at any depth.

 @param dir - directory to search

 @returns Each file's path under `dir`, in code point order

 @throws What the lister raises for a directory it cannot read, an absent one
 included

 @example
 ```ts
 const paths = await filesUnder({ dir: '/people', },);
 ```
 */
export async function filesUnder({ dir, }: { readonly dir: string; },): Promise<readonly string[]> {
  /**
   Files directly in the directory.
   */
  const files = await namesOfKind({
    dir,
    kind: 'file',
  },);

  /**
   Directories directly in it.
   */
  const directories = await namesOfKind({
    dir,
    kind: 'directory',
  },);

  /**
   Files under each directory, each path under the directory they sit in.
   */
  const nested = await allInInputOrder({
    members: directories.map(async function nestedIn(directory,): Promise<readonly string[]> {
      return (await filesUnder({
        dir: join(
          dir,
          directory,
        ),
      },)).map(function underDirectory(path,): string {
        return join(
          directory,
          path,
        );
      },);
    },),
  },);
  return textsInCodePointOrder({
    texts: [
      ...files,
      ...nested.flat(),
    ],
  },);
}

//endregion Reader measure files
