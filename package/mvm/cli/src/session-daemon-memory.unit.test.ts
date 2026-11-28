/**
 Tests for reading the memory cap of the session daemon's unit from a `meminfo`
 file, all from the built package.

 Each test that needs a file writes one in its own sandbox, so no test reads this
 machine's real `/proc/meminfo` and the tests run side by side.

 @module
 */

import { writeFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  CAP_DENOMINATOR,
  CAP_NUMERATOR,
  MEMINFO_PATH,
  memoryMaxArgument,
  MINIMUM_MEMORY_MAX_MIB,
  parseMemTotalKib,
  SessionDaemonMemoryError,
  sessionDaemonMemoryMax,
} from '../dist/final/node/index.mjs';
import {
  createSandbox,
  type Sandbox,
} from './fixture.cli-harness.ts';

//region The fake machine

/**
 Kibibytes in one mebibyte, used to derive expected caps from the memory the fake reports.
 */
const KIB_PER_MIB = 1_024;

/**
 Total memory this machine reports, so each expectation is derived from the input.

 `/proc/meminfo` read `MemTotal: 64668036 kB` when this test was written.
 */
const MACHINE_TOTAL_KIB = 64_668_036;

/**
 Text of a `meminfo` file that reports `memTotalKib` and nothing else mvm reads.

 @param memTotalKib - Total memory the file reports

 @returns File text in the layout of `/proc/meminfo`

 @example
 ```ts
 meminfoText({ memTotalKib: 2048 }); // => 'MemTotal:       2048 kB\n...'
 ```
 */
function meminfoText({ memTotalKib, }: { readonly memTotalKib: number; },): string {
  return [
    `MemTotal:       ${String(memTotalKib,)} kB`,
    'MemFree:        1234 kB',
    'MemAvailable:   5678 kB',
    '',
  ].join('\n',);
}

/**
 Cap the decided fraction of `memTotalKib` must give, in mebibytes with systemd's
 suffix.

 @param memTotalKib - Total memory to take that fraction of

 @returns Expected `MemoryMax` value

 @example
 ```ts
 expectedCap({ memTotalKib: 4_096 }); // => '3M'
 ```
 */
function expectedCap({ memTotalKib, }: { readonly memTotalKib: number; },): string {
  /**
   The fraction of that total the cap covers, in kibibytes.
   */
  const capKib = (memTotalKib * CAP_NUMERATOR) / CAP_DENOMINATOR;
  return `${String(Math.floor(capKib / KIB_PER_MIB,),)}M`;
}

/**
 Writes a `meminfo` file in a sandbox and returns its path.

 @param memTotalKib - Total memory the file reports

 @param sandbox - Sandbox whose state directory holds it

 @returns Path of the written file

 @example
 ```ts
 await writeMeminfo({ memTotalKib: 4096, sandbox });
 ```
 */
async function writeMeminfo({
  memTotalKib,
  sandbox,
}: {
  readonly memTotalKib: number;
  readonly sandbox: Sandbox;
},): Promise<string> {
  /**
   Path of the fake `meminfo` file inside the sandbox.
   */
  const path = join(
    sandbox.state,
    'meminfo',
  );
  await writeFile(
    path,
    meminfoText({ memTotalKib, },),
  );
  return path;
}

//endregion The fake machine

//region The decided fraction

/**
 `MemoryMax` the decision record names for this machine, the one place these tests
 pin the decided fraction.

 The owner's answer recorded in
 `doc/decision/owner-answers-of-2026-11-28-guests-swap-drives-removal-and-releases.md`,
 "The unit's cap", set the running unit to `MemoryMax=47364M`, and three quarters
 of this machine's `MemTotal` of 64,668,036 kB is 47,364.28 MiB, which rounds down
 to the same number. Every other expectation here follows {@link CAP_NUMERATOR}
 and {@link CAP_DENOMINATOR}, so a deliberate change to the fraction breaks
 nothing else.
 */
const RECORDED_MEMORY_MAX = '47364M';

//endregion The decided fraction

await describe({
  name: 'the session daemon unit\'s memory cap',
  children: [
    it({
      name: 'reads the total from a meminfo file',
      fn: async () => {
        expect(parseMemTotalKib({
          path: MEMINFO_PATH,
          text: meminfoText({ memTotalKib: MACHINE_TOTAL_KIB, },),
        },),).toBe(MACHINE_TOTAL_KIB,);
      },
    },),

    it({
      name: 'caps at the decided fraction of the total, in whole mebibytes',
      fn: async () => {
        expect(memoryMaxArgument({ memTotalKib: MACHINE_TOTAL_KIB, },),)
          .toBe(expectedCap({ memTotalKib: MACHINE_TOTAL_KIB, },),);
      },
    },),

    it({
      name: 'gives the cap the decision record names for this machine',
      fn: async () => {
        expect(memoryMaxArgument({ memTotalKib: MACHINE_TOTAL_KIB, },),)
          .toBe(RECORDED_MEMORY_MAX,);
      },
    },),

    it({
      name: 'rounds a total whose cap is not a whole number of mebibytes down',
      fn: async () => {
        /**
         Total whose cap leaves a fraction of a mebibyte: one kibibyte over a whole number of mebibytes of total.
         */
        const oddKib = (KIB_PER_MIB * CAP_DENOMINATOR) + 1;
        /**
         Cap in mebibytes before rounding, which has to be fractional for this test to mean anything.
         */
        const exactMib = ((oddKib * CAP_NUMERATOR) / CAP_DENOMINATOR) / KIB_PER_MIB;
        expect(Number.isInteger(exactMib,),).toBe(false,);
        expect(memoryMaxArgument({ memTotalKib: oddKib, },),)
          .toBe(expectedCap({ memTotalKib: oddKib, },),);
        // A cap rounded up instead of down would be one mebibyte more.
        expect(memoryMaxArgument({ memTotalKib: oddKib, },),)
          .not.toBe(`${String(Math.ceil(exactMib,),)}M`,);
      },
    },),

    it({
      name: 'gives the smallest cap when the fraction rounds down to nothing',
      fn: async () => {
        expect(memoryMaxArgument({ memTotalKib: 1, },),)
          .toBe(`${String(MINIMUM_MEMORY_MAX_MIB,)}M`,);
      },
    },),

    it({
      name: 'reads the cap from a meminfo file it is given',
      fn: async () => {
        await using sandbox = await createSandbox({},);
        /**
         Total the fake machine reports.
         */
        const memTotalKib = KIB_PER_MIB * 4_096;
        /**
         Path of the fake `meminfo` file.
         */
        const path = await writeMeminfo({
          memTotalKib,
          sandbox,
        },);
        expect(await sessionDaemonMemoryMax({ path, },),)
          .toBe(expectedCap({ memTotalKib, },),);
      },
    },),

    it({
      name: 'names the file when it holds no total',
      fn: async () => {
        await using sandbox = await createSandbox({},);
        /**
         Path of a `meminfo` file without the line mvm reads.
         */
        const path = join(
          sandbox.state,
          'meminfo',
        );
        await writeFile(
          path,
          'MemFree:  1234 kB\n',
        );
        await expect(sessionDaemonMemoryMax({ path, },),)
          .rejects
          .toBeInstanceOf(SessionDaemonMemoryError,);
        await expect(sessionDaemonMemoryMax({ path, },),)
          .rejects
          .toThrow(path,);
      },
    },),

    it({
      name: 'names the file when it cannot be read',
      fn: async () => {
        await using sandbox = await createSandbox({},);
        /**
         Path no file exists at.
         */
        const path = join(
          sandbox.state,
          'absent-meminfo',
        );
        await expect(sessionDaemonMemoryMax({ path, },),)
          .rejects
          .toThrow('the file could not be read',);
      },
    },),
  ],
},);
