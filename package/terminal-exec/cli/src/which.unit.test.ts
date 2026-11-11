/**
 Tests for the one PATH lookup both terminal resolvers share.

 The Linux resolver and the Windows resolver used to carry a copy each, and the
 copies differed only in how they imported `access`. The injected `access` and
 `pathEnv` are what let this file drive every branch of the walk without
 touching this process's `PATH` or this machine's filesystem.

 @module
 */

import { delimiter, join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  EXECUTABLE_NOT_ON_PATH,
  which,
} from '../dist/final/node/launch.mjs';

//region Fixtures

/**
 First directory of the two-directory search path every case searches.
 */
const FIRST_DIRECTORY = join('/', 'first-bin',);

/**
 Second directory of the two-directory search path every case searches.
 */
const SECOND_DIRECTORY = join('/', 'second-bin',);

/**
 Search path holding {@link FIRST_DIRECTORY} then {@link SECOND_DIRECTORY}.
 */
const TWO_DIRECTORY_PATH = [
  FIRST_DIRECTORY,
  SECOND_DIRECTORY,
].join(delimiter,);

/**
 Executable name every case looks for.
 */
const TOOL_NAME = 'tool';

/**
 Candidate the fixture search path builds for {@link TOOL_NAME}.
 */
const FIRST_CANDIDATE = join(FIRST_DIRECTORY, TOOL_NAME,);

/**
 Candidate under the second directory.
 */
const SECOND_CANDIDATE = join(SECOND_DIRECTORY, TOOL_NAME,);

/**
 Builds an `access` that accepts exactly the candidates it is given.

 @param accepted - Candidate paths the fixture treats as executable

 @returns access stand-in rejecting every other candidate with an `Error`

 @example
 ```ts
 const access = accessAccepting([SECOND_CANDIDATE]);
 ```
 */
function accessAccepting(accepted: readonly string[],): (candidate: string,) => Promise<void> {
  return async function access(candidate: string,): Promise<void> {
    if (accepted.includes(candidate,))
      return;
    throw Object.assign(new Error(`not executable: ${candidate}`,), { code: 'ENOENT', },);
  };
}

//endregion Fixtures

await describe({
  name: '',
  children: [
    describe({
      name: which.name,
      children: [
        //region A candidate the search accepts answers the path

        it({
          name: 'answers the first candidate `access` accepts',
          fn: async () => {
            expect(
              await which({
                name: TOOL_NAME,
                access: accessAccepting([FIRST_CANDIDATE,],),
                pathEnv: TWO_DIRECTORY_PATH,
              },),
            ).toBe(FIRST_CANDIDATE,);
          },
        },),

        it({
          name: 'walks past a candidate `access` rejects with an `Error`',
          fn: async () => {
            expect(
              await which({
                name: TOOL_NAME,
                access: accessAccepting([SECOND_CANDIDATE,],),
                pathEnv: TWO_DIRECTORY_PATH,
              },),
            ).toBe(SECOND_CANDIDATE,);
          },
        },),

        //endregion

        //region No candidate is usable

        it({
          name: 'answers the sentinel when no candidate is usable',
          fn: async () => {
            expect(
              await which({
                name: TOOL_NAME,
                access: accessAccepting([],),
                pathEnv: TWO_DIRECTORY_PATH,
              },),
            ).toBe(EXECUTABLE_NOT_ON_PATH,);
          },
        },),

        it({
          name: 'answers the sentinel for an empty search path',
          fn: async () => {
            expect(
              await which({
                name: TOOL_NAME,
                access: accessAccepting([FIRST_CANDIDATE,],),
                pathEnv: '',
              },),
            ).toBe(EXECUTABLE_NOT_ON_PATH,);
          },
        },),

        //endregion

        //region A rejection that is not an `Error` leaves the walk

        it({
          name: 'rethrows a rejection that is not an `Error`',
          fn: async () => {
            /**
             Rejected value the walk must not swallow.
             */
            const rejected = 'access refused with a string';
            /**
             Handles of the rejection the probe answers with, so this case can
             reject with that string without a `throw` of a non-`Error` value.
             */
            const refusal = Promise.withResolvers<void>();
            refusal.reject(rejected,);

            await expect(
              which({
                name: TOOL_NAME,
                async access(candidate: string,): Promise<void> {
                  if (candidate === FIRST_CANDIDATE)
                    await refusal.promise;
                  throw new Error(`unexpected candidate: ${candidate}`,);
                },
                pathEnv: TWO_DIRECTORY_PATH,
              },),
            ).rejects.toBe(rejected,);
          },
        },),

        //endregion
      ],
    },),
  ],
},);
