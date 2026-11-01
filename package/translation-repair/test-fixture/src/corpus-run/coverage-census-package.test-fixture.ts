import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { scratchDirWith, } from '../scratch-dir.test-fixture.ts';
import type { ReadonlyEntry, } from '../readonly-entry.ts';

//region Coverage census package fixture
// A DISPOSABLE PACKAGE DIRECTORY with a build directory holding the files a
// case names, for the cases of the census's checks of its build. Names and
// text are cat-themed invention.

/**
 A bundle's text as the coverage build writes it, with a module region
 comment, which the minified build omits.
 */
export const UNMINIFIED_BUNDLE: string = '//#region src/nap.ts\nfunction nap() {}\n//#endregion\n';

/**
 A bundle's text as the normal build writes it: no module region comment.
 */
export const MINIFIED_BUNDLE: string = 'function nap(){}\n';

/**
 A package directory with the build directory's files written.

 @param files - text of each file by its name in the build directory, none
 for an empty build directory

 @returns The package directory and its build directory, removed on dispose

 @example
 ```ts
 await using built = await makeBuiltPackage({ files: { 'nap.mjs': MINIFIED_BUNDLE, }, },);
 ```
 */
export async function makeBuiltPackage(
  { files, }: { readonly files: Readonly<Record<string, string>>; },
): Promise<AsyncDisposable & {
  readonly path: string;
  readonly distDirectory: string;
}> {
  // Fresh temp directory holding the package.
  return await scratchDirWith({
    prefix: 'mochi-coverage-census-',
    setup: async function seeded({ path, },): Promise<{ readonly distDirectory: string; }> {
      /**
       Where the build's files go.
       */
      const distDirectory = join(
        path,
        'dist',
        'final',
        'node',
      );
      await mkdir(
        distDirectory,
        { recursive: true, },
      );
      await Promise.all(Object.entries(files,)
        .map(async function written([name, text,]: ReadonlyEntry<string, string>,): Promise<void> {
          await writeFile(
            join(
              distDirectory,
              name,
            ),
            text,
            'utf8',
          );
        },),);
      return { distDirectory, };
    },
  },);
}

//endregion Coverage census package fixture
