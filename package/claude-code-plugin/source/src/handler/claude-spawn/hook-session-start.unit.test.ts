/**
 Tests for the `spawn-claude` executable probe the SessionStart hook uses.

 A negative answer decides whether the plugin rewrites the user's
 `~/.local/bin/spawn-claude` symlink and prints a PATH warning, so the probe may
 answer "not executable" only for failures that speak about the path itself.

 @module
 */

import {
  chmod,
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

import { isExecutablePath, } from './hook-session-start.ts';

//region Fixtures

/**
 Longest path component this test needs to exceed every filesystem's limit.
 */
const OVERLONG_COMPONENT_NAME_LENGTH = 300;

/**
 Disposable directory a test hangs its candidate paths under.

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
      name: isExecutablePath.name,
      children: [
        //region A runnable entry reads as executable

        it({
          name: 'answers true for a file the process may execute',
          fn: async () => {
            await using base = await scratchBase('claude-spawn-executable-',);
            /**
             Candidate CLI whose permission bits allow execution.
             */
            const path = join(
              base.path,
              'spawn-claude',
            );
            await writeFile(
              path,
              '',
            );
            await chmod(
              path,
              0o755,
            );
            expect(await isExecutablePath(path,),).toBe(true,);
          },
        },),

        //endregion A runnable entry reads as executable

        //region Failures that speak about the path answer false

        it({
          name: 'answers false when nothing stands at the path',
          fn: async () => {
            await using base = await scratchBase('claude-spawn-absent-',);
            /**
             Answer for a path nothing stands at.
             */
            const executable = await isExecutablePath(join(
              base.path,
              'not-there',
            ),);
            expect(executable,).toBe(false,);
          },
        },),

        it({
          name: 'answers false when a path component is a file rather than a directory',
          fn: async () => {
            await using base = await scratchBase('claude-spawn-notdir-',);
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
            const executable = await isExecutablePath(join(
              file,
              'spawn-claude',
            ),);
            expect(executable,).toBe(false,);
          },
        },),

        it({
          name: 'answers false for a file whose permission bits withhold execution',
          fn: async () => {
            await using base = await scratchBase('claude-spawn-unexecutable-',);
            /**
             Candidate CLI with no execute bit, so the kernel answers `EACCES`.
             */
            const path = join(
              base.path,
              'spawn-claude',
            );
            await writeFile(
              path,
              '',
            );
            await chmod(
              path,
              0o644,
            );
            expect(await isExecutablePath(path,),).toBe(false,);
          },
        },),

        //endregion Failures that speak about the path answer false

        //region A failure that says nothing about the path reaches the caller

        it({
          name: 'reports a probe the filesystem refused instead of answering false',
          fn: async () => {
            await using base = await scratchBase('claude-spawn-overlong-',);
            /**
             Candidate whose single component exceeds the filesystem's limit.
             */
            const path = join(
              base.path,
              'x'.repeat(OVERLONG_COMPONENT_NAME_LENGTH,),
            );

            let caught: unknown;
            try {
              await isExecutablePath(path,);
            }
            catch (error) {
              caught = error;
            }
            expect(codeOf(caught,),).toBe('ENAMETOOLONG',);
          },
        },),

        //endregion A failure that says nothing about the path reaches the caller
      ],
    },),
  ],
},);
