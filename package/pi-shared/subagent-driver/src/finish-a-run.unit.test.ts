/**
 What the launcher prints for a start that ended without a run: the line the start left in its own folder, so a reader of the
 launcher's output learns the wait, its bound, how long it had waited and what to do next, without opening anything.

 Issue 6556's four starts of 2026-11-26 printed only their unit's status: the driver stopped before it could write a summary, the
 run folder held no standard error, and the launcher said nothing about the store read that had timed out.

 @module
 */

import {
  mkdir,
  mkdtemp,
  rm,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  START_WITHOUT_A_RUN_FILE_NAME,
  type StartThatEndedWithoutARun,
  reportAStartThatEndedWithoutARun,
  writeAStartThatEndedWithoutARun,
} from '../dist/final/node/index.mjs';

/**
 The start the four silent runs of 2026-11-26 met, in the words the driver now writes: the sops decrypt of the checkout's store
 reached the bound the reader of the store gives it.
 */
const STORE_READ_START: StartThatEndedWithoutARun = {
  waits: [{
    wait: 'the sops decrypt of the checkout\'s store',
    bound: '15000 ms',
    waitedMilliseconds: 15_001,
  },],
  what: 'nothing could be read from it, so no value of this machine\'s secret environment could be masked',
  next: 'start the run again when the host is quieter',
};

/**
 A folder one test writes in, removed when the test's scope ends.

 @returns the folder, and how to remove it
 */
async function makeFolder(): Promise<{
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>
}> {
  /**
   The folder, under the scratch root the harness gives this suite.
   */
  const path = await mkdtemp(join(scratchRoot(), 'launcher-start-line-',),);
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(
        path,
        { recursive: true, force: true, },
      );
    },
  };
}

await describe({
  name: reportAStartThatEndedWithoutARun.name,
  children: [
    it({
      name: 'prints the line the start left, so the launcher names the wait and its bound',
      fn: async () => {
        await using folder = await makeFolder();
        /**
         The line the driver left in the run folder, which is what the launcher is to print.
         */
        const written = await writeAStartThatEndedWithoutARun({
          runFolder: folder.path,
          start: STORE_READ_START,
        },);
        /**
         Every line the launcher printed.
         */
        const printed: string[] = [];
        await reportAStartThatEndedWithoutARun({
          runFolder: folder.path,
          report: function report(line,) {
            printed.push(line,);
          },
        },);
        expect(printed,).toEqual([written,],);
        expect(printed[0],).toContain('sops decrypt',);
        expect(printed[0],).toContain('15000 ms',);
        expect(printed[0],).toContain('start the run again when the host is quieter',);
      },
    },),
    it({
      name: 'prints nothing for a run folder a start that ended with a run left alone',
      fn: async () => {
        await using folder = await makeFolder();
        const printed: string[] = [];
        await reportAStartThatEndedWithoutARun({
          runFolder: folder.path,
          report: function report(line,) {
            printed.push(line,);
          },
        },);
        expect(printed,).toEqual([],);
      },
    },),
    it({
      name: 'names the file it could not read, rather than leaving a start that left one unmentioned',
      fn: async () => {
        await using folder = await makeFolder();
        // A path where the line belongs that is a folder: the launcher cannot read it, and saying so is what keeps the start
        // from reading as one that ended with a run.
        await mkdir(join(folder.path, START_WITHOUT_A_RUN_FILE_NAME,),);
        const printed: string[] = [];
        await reportAStartThatEndedWithoutARun({
          runFolder: folder.path,
          report: function report(line,) {
            printed.push(line,);
          },
        },);
        expect(printed,).toHaveLength(1,);
        expect(printed[0],).toContain(START_WITHOUT_A_RUN_FILE_NAME,);
        expect(printed[0],).toContain('could not be read',);
      },
    },),
  ],
},);
