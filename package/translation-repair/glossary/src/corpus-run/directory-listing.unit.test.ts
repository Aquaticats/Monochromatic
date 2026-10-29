/**
 Tests for listing the entries of one kind a directory holds.

 EACH KIND IS READ OFF THE ENTRY ITSELF (ledger B65). A reader that listed
 bare names took a directory named like a record as one, read a record twice
 through a symlink, or stopped on EISDIR; so every fixture here holds a file,
 a directory, a symlink to each, and asks for one kind at a time, the other
 kind's entries and both links being what must not come back.

 DISPOSABLE FIXTURES ONLY: every case writes into its own `scratchDir`
 directory, removed when the case ends, and nothing here reads a real run.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  mkdir,
  symlink,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  namesOfKind,
  presentNamesOfKind,
  readingOf,
} from '../../dist/final/node/index.mjs';
import { scratchDirWith, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

//region Directory listing tests

/**
 Writes one disposable directory holding a file `Mittens.json`, a directory
 `Tabby`, a symlink `Siamese.json` to the file and a symlink `Calico` to the
 directory.

 @returns Directory holding them, removed on dispose

 @example
 ```ts
 await using fixture = await mixedDirectory();
 ```
 */
async function mixedDirectory(): Promise<AsyncDisposable & { readonly dir: string; }> {
  // Disposable root for this case.
  return await scratchDirWith({
    prefix: 'directory-listing-',
    setup: async function seeded({ path: dir, },): Promise<{ readonly dir: string; }> {
      await writeFile(
        join(
          dir,
          'Mittens.json',
        ),
        '{}',
        'utf8',
      );
      await mkdir(join(
        dir,
        'Tabby',
      ),);
      await symlink(
        'Mittens.json',
        join(
          dir,
          'Siamese.json',
        ),
      );
      await symlink(
        'Tabby',
        join(
          dir,
          'Calico',
        ),
      );

      return { dir, };
    },
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: namesOfKind.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'LISTS REGULAR FILES ONLY when asked for files: neither the directory nor either symlink',
          fn: async () => {
            await using fixture = await mixedDirectory();
            expect(await namesOfKind({
              dir: fixture.dir,
              kind: 'file',
            },),).toEqual(['Mittens.json',],);
          },
        },),

        it({
          name: 'LISTS DIRECTORIES ONLY when asked for directories: neither the file nor either symlink',
          fn: async () => {
            await using fixture = await mixedDirectory();
            expect(await namesOfKind({
              dir: fixture.dir,
              kind: 'directory',
            },),).toEqual(['Tabby',],);
          },
        },),

        it({
          name: 'RAISES ENOENT for a directory that is not there, which only the caller can read as an answer',
          fn: async () => {
            await using fixture = await mixedDirectory();
            await expect(namesOfKind({
              dir: join(
                fixture.dir,
                'nowhere',
              ),
              kind: 'file',
            },),).rejects.toHaveProperty(
              'code',
              'ENOENT',
            );
          },
        },),
      ],
    },),

    describe({
      name: presentNamesOfKind.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'LISTS THE KIND ASKED FOR, the control the other cases depart from',
          fn: async () => {
            await using fixture = await mixedDirectory();
            expect(await presentNamesOfKind({
              dir: fixture.dir,
              kind: 'file',
            },),).toEqual(['Mittens.json',],);
          },
        },),

        it({
          name: 'READS A DIRECTORY THAT IS NOT THERE AS HOLDING NONE, the state before its writer first writes',
          fn: async () => {
            await using fixture = await mixedDirectory();
            expect(await presentNamesOfKind({
              dir: join(
                fixture.dir,
                'nowhere',
              ),
              kind: 'file',
            },),).toEqual([],);
          },
        },),

        it({
          name: 'RAISES ANY OTHER FAILURE: a path that is a file is ENOTDIR, never an empty directory',
          fn: async () => {
            await using fixture = await mixedDirectory();
            await expect(presentNamesOfKind({
              dir: join(
                fixture.dir,
                'Mittens.json',
              ),
              kind: 'file',
            },),).rejects.toHaveProperty(
              'code',
              'ENOTDIR',
            );
          },
        },),
      ],
    },),

    describe({
      name: readingOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS WHAT THE LISTING RETURNS, the control the refusals depart from',
          fn: async () => {
            expect(await readingOf({
              list: async function listed(): Promise<readonly string[]> {
                return ['Mittens.json',];
              },
            },),).toEqual({
              kind: 'read',
              names: ['Mittens.json',],
            },);
          },
        },),

        it({
          name: 'REPORTS A FILESYSTEM FAILURE BY ITS CODE rather than raising, so an operator reads the remedy',
          fn: async () => {
            expect(await readingOf({
              list: async function refused(): Promise<readonly string[]> {
                throw Object.assign(
                  new Error('the cat sat on the permissions',),
                  { code: 'EACCES', },
                );
              },
            },),).toEqual({
              kind: 'unreadable',
              reason: 'EACCES',
            },);
          },
        },),

        it({
          name: 'NAMES THE CLASS of a failure that carries no code, never its message',
          fn: async () => {
            expect(await readingOf({
              list: async function refused(): Promise<readonly string[]> {
                throw new TypeError('Mittens knocked the listing off the table',);
              },
            },),).toEqual({
              kind: 'unreadable',
              reason: 'TypeError',
            },);
          },
        },),
      ],
    },),
  ],
},);

//endregion Directory listing tests
