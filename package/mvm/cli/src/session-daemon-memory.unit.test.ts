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
  MEMINFO_PATH,
  memoryMaxArgument,
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
 */
const FAKE_TOTAL_KIB = 64_668_048;

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
 Cap half of `memTotalKib` must give, in mebibytes with systemd's suffix.

 @param memTotalKib - Total memory to halve

 @returns Expected `MemoryMax` value

 @example
 ```ts
 expectedCap({ memTotalKib: 4096 }); // => '2M'
 ```
 */
function expectedCap({ memTotalKib, }: { readonly memTotalKib: number; },): string {
  return `${String(Math.floor(memTotalKib / 2 / KIB_PER_MIB,),)}M`;
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

await describe({
  name: 'the session daemon unit\'s memory cap',
  children: [
    it({
      name: 'reads the total from a meminfo file',
      fn: async () => {
        expect(parseMemTotalKib({
          path: MEMINFO_PATH,
          text: meminfoText({ memTotalKib: FAKE_TOTAL_KIB, },),
        },),).toBe(FAKE_TOTAL_KIB,);
      },
    },),

    it({
      name: 'caps at half of the total, in whole mebibytes',
      fn: async () => {
        expect(memoryMaxArgument({ memTotalKib: FAKE_TOTAL_KIB, },),)
          .toBe(expectedCap({ memTotalKib: FAKE_TOTAL_KIB, },),);
      },
    },),

    it({
      name: 'rounds an odd total down to whole mebibytes',
      fn: async () => {
        /**
         Total that halves to a fraction of a mebibyte.
         */
        const oddKib = (KIB_PER_MIB * 3) + 1;
        expect(memoryMaxArgument({ memTotalKib: oddKib, },),)
          .toBe(expectedCap({ memTotalKib: oddKib, },),);
        expect(memoryMaxArgument({ memTotalKib: oddKib, },),).toBe('1M',);
      },
    },),

    it({
      name: 'gives the smallest cap a machine under two mebibytes can have',
      fn: async () => {
        expect(memoryMaxArgument({ memTotalKib: 1, },),).toBe('1M',);
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
