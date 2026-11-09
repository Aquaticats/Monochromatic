/**
 Tests for the `pathExists` probe.

 The probe answers "is something there" only when the kernel says the entry is
 missing. Any other failure (a directory the process may not search, an I/O
 error) is a real failure and must reach the caller instead of being reported
 as absence, so a caller never overwrites a cached image it merely could not
 inspect.

 @module
 */

import {
  chmod,
  mkdir,
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { pathExists, } from '../dist/final/node/index.mjs';

//region Fixtures

/**
 Disposable directory a test points the probe at.

 @param prefix - name prefix inside the scratch root

 @returns the absolute path and the disposal that removes it
 */
async function scratchBase(prefix: string,): Promise<{
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>;
}> {
  /**
   Absolute path of the disposable directory.
   */
  const path = await mkdtemp(join(
    scratchRoot(),
    prefix,
  ),);
  return {
    path,
    [Symbol.asyncDispose]: async function remove(): Promise<void> {
      await rm(
        path,
        { recursive: true, force: true, },
      );
    },
  };
}

/**
 Reads the `code` of a caught system error.

 @param error - caught value

 @returns the errno code, or the empty text for a value without one
 */
function codeOf(error: unknown,): string {
  if (!(Error.isError(error,)))
    return '';
  if (!('code' in error))
    return '';
  /**
   The code a system error carries as text.
   */
  const { code, } = error;
  return (typeof code) === 'string' ? code : '';
}

//endregion Fixtures

await describe({
  name: '',
  children: [
    describe({
      name: pathExists.name,
      children: [
        //region A present entry reads as present

        it({
          name: 'answers true for an entry that exists',
          fn: async () => {
            await using base = await scratchBase('mvm-path-exists-present-',);
            expect(await pathExists(base.path,),).toBe(true,);
          },
        },),

        //endregion A present entry reads as present

        //region A missing entry reads as absent

        it({
          name: 'answers false for a path that is not there',
          fn: async () => {
            await using base = await scratchBase('mvm-path-exists-missing-',);
            /**
             Answer for a path nothing stands at.
             */
            const exists = await pathExists(join(
              base.path,
              'not-there',
            ),);
            expect(exists,).toBe(false,);
          },
        },),

        it({
          name: 'answers false when a path component is a file rather than a directory',
          fn: async () => {
            await using base = await scratchBase('mvm-path-exists-notdir-',);
            /**
             Regular file used as a path component, which the kernel refuses with `ENOTDIR`.
             */
            const file = join(
              base.path,
              'plain-file',
            );
            await writeFile(
              file,
              '',
            );
            /**
             Answer for a path whose parent component is a regular file.
             */
            const exists = await pathExists(join(
              file,
              'below',
            ),);
            expect(exists,).toBe(false,);
          },
        },),

        //endregion A missing entry reads as absent

        //region An unreadable entry is a failure, not an absence

        it({
          name: 'reports a directory the process may not search instead of answering false',
          skip: (process.getuid?.() === 0)
            ? 'the process is root, and no mode refuses root a directory'
            : false,
          fn: async () => {
            await using base = await scratchBase('mvm-path-exists-sealed-',);
            /**
             Directory with no permission bits, and a path below it.
             */
            const sealed = join(
              base.path,
              'sealed',
            );
            await mkdir(sealed,);
            await chmod(
              sealed,
              0,
            );
            await using _reopened = {
              [Symbol.asyncDispose]: async function reopen(): Promise<void> {
                await chmod(
                  sealed,
                  0o700,
                );
              },
            };

            let caught: unknown;
            try {
              await pathExists(join(
                sealed,
                'below',
              ),);
            }
            catch (error) {
              caught = error;
            }
            expect(codeOf(caught,),).toBe('EACCES',);
          },
        },),

        //endregion An unreadable entry is a failure, not an absence
      ],
    },),
  ],
},);
