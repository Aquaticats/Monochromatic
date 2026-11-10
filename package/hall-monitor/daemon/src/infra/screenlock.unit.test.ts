/**
 Failure tests for the screen-lock probe.

 Skipping a cycle on a locked session is an optimisation, so a probe that cannot
 answer must cost one cycle rather than the daemon. These cases prove the three
 ways the probe ends: the session bus says locked, says unlocked, and cannot be
 asked at all, including when it rejects with something that is not an `Error`.

 The probe is injected, so no case talks to a session bus.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { isScreenLocked, } from '../../dist/final/node/testing.mjs';

//region Fixtures

/**
 Standard output `gdbus` prints for a locked session.
 */
const LOCKED_OUTPUT = '(true,)';

/**
 Standard output `gdbus` prints for an unlocked session.
 */
const UNLOCKED_OUTPUT = '(false,)';

//endregion Fixtures

await describe({
  name: '',
  children: [
    describe({
      name: isScreenLocked.name,
      children: [
        it({
          name: 'reports a locked session',
          fn: async () => {
            expect(await isScreenLocked({
              probe: async () => ({ stdout: LOCKED_OUTPUT, }),
            },),).toBe(true,);
          },
        },),

        it({
          name: 'reports an unlocked session',
          fn: async () => {
            expect(await isScreenLocked({
              probe: async () => ({ stdout: UNLOCKED_OUTPUT, }),
            },),).toBe(false,);
          },
        },),

        it({
          name: 'treats a failing probe as unlocked instead of throwing',
          fn: async () => {
            expect(await isScreenLocked({
              probe: async () => {
                throw new Error('gdbus: command not found',);
              },
            },),).toBe(false,);
          },
        },),

        it({
          name: 'treats a probe that throws a non-Error as unlocked too',
          fn: async () => {
            expect(await isScreenLocked({
              probe: async () => {
                // oxlint-disable-next-line eslint/no-throw-literal, typescript/only-throw-error -- the case under test is a probe that fails with a value which is not an Error, so the daemon still has to survive it
                throw 'the session bus refused the call';
              },
            },),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
