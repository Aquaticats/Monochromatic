/**
 Test file discovery for matrix test files.
 
 When the consumer does not provide explicit `files`, this module
 discovers `*.unit.matrix.test.ts` files by walking the directory tree.
 */

import type { Dirent, } from 'node:fs';
import { readdir, } from 'node:fs/promises';
import { resolve, } from 'node:path';

/**
 Glob suffix for matrix test files.
 */
const MATRIX_TEST_SUFFIX = '.unit.matrix.test.ts';

/**
 Discovers matrix test files in a directory tree.
 
 Searches recursively for files ending in `.unit.matrix.test.ts`,
 skipping `node_modules` and `dist` directories.
 
 @param cwd - Directory to search in
 
 @returns absolute paths to discovered test files, sorted lexicographically
 
 @throws Error when no test files are found
 
 @example
 ```ts
 const files = await discoverTestFiles(
   '/var/home/user/Monochromatic/package/logger/core',
 );
 // ['/var/home/user/.../ensure-package.unit.matrix.test.ts']
 ```
 */
export async function discoverTestFiles(cwd: string,): Promise<readonly string[]> {
  /**
   Raw recursive directory walk; filtered and mapped below into the public result.
   */
  const entries = await readdir(
    cwd,
    {
      recursive: true,
      withFileTypes: true,
    },
  );

  /**
   Filtered, sorted absolute paths; built as a chain to keep each step inspectable.
   */
  const files = entries
    .filter(function isMatrixTest(entry: Readonly<Dirent>,) {
      if (!entry.isFile())
        return false;
      if (!entry.name
        .endsWith(MATRIX_TEST_SUFFIX,))
        return false;
      /**
       Skip node_modules and dist directories.
       */
      const { parentPath, } = entry;
      if (parentPath.includes('node_modules',)
        || parentPath
          .includes('/dist/',))
        return false;
      return true;
    },)
    .map(function toAbsolutePath(entry: Readonly<Dirent>,) {
      return resolve(
        entry.parentPath,
        entry.name,
      );
    },)
    .toSorted();

  if (files.length
    === 0) {
    throw new Error(
      `No matrix test files (*${MATRIX_TEST_SUFFIX}) found in ${cwd}`,
    );
  }

  return files;
}
