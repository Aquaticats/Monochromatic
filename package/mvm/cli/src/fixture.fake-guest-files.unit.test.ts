/**
 Tests for the stand-in guest agent's file commands around the existence probe.

 A guest path this process may not reach is not a missing guest path: the fake
 must report the failure the filesystem raised, not answer "No such file or
 directory", because mvm tests would otherwise accept a wrong diagnosis from
 the agent.

 @module
 */

import { chmod, mkdir, mkdtemp, rm, writeFile } from 'node:fs/promises';
import { join, } from 'node:path';

import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { guestFileCommand, } from './fixture.fake-guest-files.ts';

//region Fixtures

/**
 Disposable state directory the fake agent works in.

 @param prefix - name prefix inside the scratch root

 @returns the absolute path and the disposal that removes it
 */
async function scratchState(prefix: string,): Promise<{
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>;
}> {
  /**
   Absolute path of the disposable state directory.
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
      name: guestFileCommand.name,
      children: [
        //region A missing guest file is the agent's not-found answer

        it({
          name: 'answers a guest file that is not there with the not-found refusal',
          fn: async () => {
            await using state = await scratchState('mvm-fake-guest-absent-',);
            /**
             Outcome the fake reports for a read-mode open of an absent path.
             */
            const outcome = await guestFileCommand({
              directory: state.path,
              execute: 'guest-file-open',
              request: { path: '/not-there', mode: 'r', },
              scenario: {},
            },);
            expect(outcome,).toHaveProperty('failure',);
            expect(JSON.stringify(outcome,),).toContain('No such file or directory',);
          },
        },),

        //endregion A missing guest file is the agent's not-found answer

        //region An unreachable guest file is a failure, not an absence

        it({
          name: 'reports a guest path behind a directory nobody may enter instead of answering not found',
          skip: (process.getuid?.() === 0)
            ? 'the process is root, and no mode refuses root a directory'
            : false,
          fn: async () => {
            await using state = await scratchState('mvm-fake-guest-sealed-',);
            /**
             Directory inside the simulated guest filesystem with no permission bits.
             */
            const sealed = join(
              state.path,
              'guest',
              'sealed',
            );
            await mkdir(
              sealed,
              { recursive: true, },
            );
            await writeFile(
              join(
                sealed,
                'a.txt',
              ),
              '',
            );
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
              await guestFileCommand({
                directory: state.path,
                execute: 'guest-file-open',
                request: { path: '/sealed/a.txt', mode: 'r', },
                scenario: {},
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(codeOf(caught,),).toBe('EACCES',);
          },
        },),

        //endregion An unreachable guest file is a failure, not an absence
      ],
    },),
  ],
},);
