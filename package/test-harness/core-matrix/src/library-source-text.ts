/**
 Every library source file of this package with its comments blanked, for the tests that check a convention across the
 package's own sources.

 Comment text is blanked rather than removed: offsets and line numbers stay those of the original file, so a scan of
 the blanked text reports the file's own lines, and a callback shape a TSDoc example shows is not mistaken for code.

 @module
 */

import { stripSourceComments, } from '@monochromatic-dev/container-limit-core/ts/source-text-without-comments.ts';
import type { Dirent, } from 'node:fs';
import {
  readdir,
  readFile,
} from 'node:fs/promises';
import {
  join,
  resolve,
} from 'node:path';

/**
 Suffix of a source file a source scan reads.
 */
const LIBRARY_SOURCE_SUFFIX = '.ts';

/**
 Directory holding this package's library sources.

 A package test imports this module from the built artifact in `dist/final/node`, so the sources sit three levels
 above the running module; a test that reached this module from source would find no sources at all and fail its own
 non-empty check rather than pass on an empty scan.
 */
const SOURCE_DIRECTORY = resolve(
  import.meta.dirname,
  '../../../src',
);

/**
 Suffixes that mark a test, benchmark or declaration file rather than library code; `oxlint` turns several rules off
 for the first two through its own override, so a source scan leaves them out as that override does.
 */
const NON_LIBRARY_SOURCE_SUFFIXES: readonly string[] = [
  '.bench.ts',
  '.d.ts',
  '.test.ts',
];

/**
 One library source file of this package, with the text a source scan reads.
 */
export type LibrarySource = {
  /**
   Absolute path of the file
   */
  readonly path: string;
  /**
   The file's text with its comments blanked
   */
  readonly text: string;
};

/**
 Tells whether one file name carries a suffix that marks it as something other than library code.

 @param fileName - name of the file

 @returns whether a non-library suffix ends the name
 */
function isNonLibraryName(fileName: string,): boolean {
  return NON_LIBRARY_SOURCE_SUFFIXES.some(function isNonLibrarySuffix(suffix: string,): boolean {
    return fileName.endsWith(suffix,);
  },);
}

/**
 Lists this package's library source files, each with the text a source scan reads.

 @returns file paths and their comment-blanked text

 @example
 ```ts
 const sources = await librarySourceTexts();
 sources.map(({ path }) => path); // ['/repo/package/test-harness/core-matrix/src/container.ts', ...]
 ```
 */
export async function librarySourceTexts(): Promise<readonly LibrarySource[]> {
  /**
   Every file below the sources directory, tests and declarations included so the filter below can drop them.
   */
  const entries = await readdir(
    SOURCE_DIRECTORY,
    {
      recursive: true,
      withFileTypes: true,
    },
  );
  /**
   Paths of the library sources, in directory order.
   */
  const paths = entries
    .filter(function isLibrarySource(entry: Dirent,): boolean {
      if (!entry.isFile())
        return false;
      if (!entry.name
        .endsWith(LIBRARY_SOURCE_SUFFIX,))
        return false;
      return !isNonLibraryName(entry.name,);
    },)
    .map(function absolutePath(entry: Dirent,): string {
      return join(
        entry.parentPath,
        entry.name,
      );
    },);

  return await Promise.all(paths.map(async function readLibrarySource(path: string,): Promise<LibrarySource> {
    return {
      path,
      text: stripSourceComments({
        text: await readFile(
          path,
          'utf8',
        ),
        language: 'typescript',
      },),
    };
  },),);
}
