/**
 Tests that the cache-invalidation glob names the sources the pipeline reads.
 
 @module
 */

import { existsSync, } from 'node:fs';
import { join, relative, resolve, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import readdir from 'tiny-readdir-glob';

import { PIPELINE_GLOB, } from '../../dist/final/node/index.mjs';

/**
 Package root, two levels up from this file's `src/lib` directory.
 */
const PACKAGE_ROOT = resolve(
  import.meta.dirname,
  '../..',
);

/**
 Source directory the pipeline modules live under.
 */
const SOURCE_DIRECTORY = 'src';

/**
 Pairs each brace-delimited glob alternative with the directory it must name.
 
 @param glob - glob whose first brace group lists source directories
 
 @returns directory names taken from the brace group
 
 @example
 ```ts
 directorialAlternatives('src/{lib,client}/**\/*.ts');
 // ['lib', 'client']
 ```
 */
function directorialAlternatives(glob: string,): string[] {
  /**
   Offset of the opening brace, or `-1` when the glob has no alternatives.
   */
  const openAt = glob.indexOf('{',);
  /**
   Offset of the closing brace, or `-1` when the glob has no alternatives.
   */
  const closeAt = glob.indexOf('}',);
  if ((openAt === (-1)) || (closeAt === (-1)))
    return [];

  /**
   Brace group of the glob, naming its alternatives.
   */
  const group = glob.slice(
    openAt + 1,
    closeAt,
  );

  return group.split(',',);
}

await describe({
  name: 'PIPELINE_GLOB',
  children: [
    it({
      name: 'names only directories that exist in the package',
      fn: async function namesExistingDirectories(): Promise<void> {
        /**
         Directory names listed in the glob's brace group.
         */
        const alternatives = directorialAlternatives(PIPELINE_GLOB,);

        expect(alternatives.length,).toBeGreaterThan(0,);
        for (const directory of alternatives) {
          expect(
            existsSync(join(
              PACKAGE_ROOT,
              SOURCE_DIRECTORY,
              directory,
            ),),
          ).toBe(true,);
        }
      },
    },),
    it({
      name: 'matches the MDX component sources the renderer imports',
      fn: async function matchesMdxComponentSources(): Promise<void> {
        /**
         Files and directories the glob expands to, relative to the package root.
         */
        const result = await readdir(PIPELINE_GLOB, { cwd: PACKAGE_ROOT, },);
        /**
         Package-relative paths of every matched file.
         */
        const matched = result.files.map(function toRelative(file,) {
          return relative(
            PACKAGE_ROOT,
            file,
          );
        },);

        expect(matched,).toContain('src/component/index.ts',);
      },
    },),
  ],
},);
