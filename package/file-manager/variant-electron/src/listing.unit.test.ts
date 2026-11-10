/**
 Unit tests for confined directory listing.
 
 One fixture tree is built under the module-test scratch root: a root
 directory holding a plain subdirectory, a symbolic link to a directory
 inside the root, and a symbolic link to a directory outside it. Tests import
 the main-process modules from built `dist/main` so they verify what the
 Electron main bundle inlines.
 
 @module
 */

import {
  mkdir,
  mkdtemp,
  rm,
  symlink,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import {
  confinePath,
  ListingOutsideRootError,
} from '../dist/main/confinement.js';
import { listDirectory, } from '../dist/main/listing.js';

/**
 Name of the regular file the inside directory holds, so a listing has a
 non-directory entry to report.
 */
const MARKER_FILE_NAME = 'note.txt';

/**
 Fixture tree the confinement tests read. Disposing it removes the scratch
 directory the tree lives in.
 
 @example
 ```ts
 await using fixture = await createConfinementFixture();
 ```
 */
type ConfinementFixture = AsyncDisposable & {
  /**
   Symbolic link inside the root whose target lies outside it.
   */
  readonly escapingLink: string;

  /**
   Directory inside the root, reached without a symbolic link.
   */
  readonly insideDirectory: string;

  /**
   Symbolic link inside the root whose target lies inside it.
   */
  readonly insideLink: string;

  /**
   Directory outside the root the escaping link points at.
   */
  readonly outsideDirectory: string;

  /**
   Configured root directory.
   */
  readonly rootDirectory: string;
};

/**
 Creates the fixture tree for one test file.
 
 @returns Paths of the root, its plain child, and both symbolic links, plus the disposal that removes them.
 
 @example
 ```ts
 await using fixture = await createConfinementFixture();
 ```
 */
async function createConfinementFixture(): Promise<ConfinementFixture> {
  /**
   Fresh scratch parent holding every fixture path below.
   */
  const scratchDirectory = await mkdtemp(join(scratchRoot(), 'file-manager-confinement-',),);
  const rootDirectory = join(scratchDirectory, 'root',);
  const insideDirectory = join(rootDirectory, 'inside',);
  const outsideDirectory = join(scratchDirectory, 'outside',);

  await mkdir(insideDirectory, { recursive: true, },);
  await mkdir(outsideDirectory, { recursive: true, },);
  await writeFile(join(insideDirectory, MARKER_FILE_NAME,), 'note\n', 'utf8',);
  await symlink(outsideDirectory, join(rootDirectory, 'escape',), 'dir',);
  await symlink(insideDirectory, join(rootDirectory, 'inside-link',), 'dir',);

  return {
    async [Symbol.asyncDispose](): Promise<void> {
      await rm(scratchDirectory, { force: true, recursive: true, },);
    },
    escapingLink: join(rootDirectory, 'escape',),
    insideDirectory,
    insideLink: join(rootDirectory, 'inside-link',),
    outsideDirectory,
    rootDirectory,
  };
}

await using fixture = await createConfinementFixture();

await describe({
  name: '',
  children: [
    describe({
      name: confinePath.name,
      children: [
        it({
          name: 'allows the configured root itself',
          fn: async (): Promise<void> => {
            const confined = await confinePath({
              path: fixture.rootDirectory,
              rootDirectory: fixture.rootDirectory,
            },);

            expect(confined,).toBe(fixture.rootDirectory,);
          },
        },),
        it({
          name: 'allows a plain subdirectory under the root',
          fn: async (): Promise<void> => {
            const confined = await confinePath({
              path: fixture.insideDirectory,
              rootDirectory: fixture.rootDirectory,
            },);

            expect(confined,).toBe(fixture.insideDirectory,);
          },
        },),
        it({
          name: 'folds dot-dot segments before comparing against the root',
          fn: async (): Promise<void> => {
            /**
             Request path that leaves the root through `..` segments.
             */
            const escaped = join(fixture.rootDirectory, 'inside', '..', '..', 'outside',);

            await expect(confinePath({
              path: escaped,
              rootDirectory: fixture.rootDirectory,
            },),).rejects.toThrow(ListingOutsideRootError,);
          },
        },),
        it({
          name: 'rejects a directory beside the root',
          fn: async (): Promise<void> => {
            await expect(confinePath({
              path: fixture.outsideDirectory,
              rootDirectory: fixture.rootDirectory,
            },),).rejects.toThrow(ListingOutsideRootError,);
          },
        },),
        it({
          name: 'rejects a symbolic link whose target lies outside the root',
          fn: async (): Promise<void> => {
            await expect(confinePath({
              path: fixture.escapingLink,
              rootDirectory: fixture.rootDirectory,
            },),).rejects.toThrow(ListingOutsideRootError,);
          },
        },),
        it({
          name: 'allows a symbolic link whose target lies inside the root',
          fn: async (): Promise<void> => {
            const confined = await confinePath({
              path: fixture.insideLink,
              rootDirectory: fixture.rootDirectory,
            },);

            expect(confined,).toBe(fixture.insideLink,);
          },
        },),
      ],
    },),
    describe({
      name: listDirectory.name,
      children: [
        it({
          name: 'lists a plain subdirectory under the root',
          fn: async (): Promise<void> => {
            const entries = await listDirectory({
              path: fixture.insideDirectory,
              rootDirectory: fixture.rootDirectory,
            },);

            expect(entries.map(function nameOf(entry,): string {
              return entry.name;
            },),).toEqual([MARKER_FILE_NAME,],);
          },
        },),
        it({
          name: 'refuses a symbolic link whose target lies outside the root',
          fn: async (): Promise<void> => {
            await expect(listDirectory({
              path: fixture.escapingLink,
              rootDirectory: fixture.rootDirectory,
            },),).rejects.toThrow(ListingOutsideRootError,);
          },
        },),
      ],
    },),
  ],
},);
