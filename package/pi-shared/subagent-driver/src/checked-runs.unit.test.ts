/**
 What the watch remembers about the runs it reported, in a state file a test writes and throws away.

 @module
 */

import {
  mkdtemp,
  readFile,
  rm,
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
  defaultCheckedRunsFile,
  readCheckedRuns,
  writeCheckedRuns,
} from '../dist/final/node/index.mjs';

/**
 One moment a run was reported at.
 */
const REPORTED_AT = 1_800_000_000_000;

/**
 A folder this test throws away.

 @returns the folder and how to remove it

 @example
 ```ts
 await using folder = await makeFolder();
 ```
 */
async function makeFolder() {
  /**
   The folder.
   */
  const path = await mkdtemp(join(scratchRoot(), 'pi-overdue-state-',),);
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(
        path,
        {
          recursive: true,
          force: true,
        },
      );
    },
  };
}

await describe({
  name: readCheckedRuns.name,
  children: [
    it({
      name: 'reads back what was written, so a rerun stays quiet about the same run',
      fn: async () => {
        await using folder = await makeFolder();
        /**
         The state file.
         */
        const file = join(
          folder.path,
          'nested',
          'reported.json',
        );
        await writeCheckedRuns({
          file,
          checked: { 'pi-alpha.service': REPORTED_AT, },
        },);
        expect(await readCheckedRuns({ file, },),).toEqual({ 'pi-alpha.service': REPORTED_AT, },);
        expect(await readFile(
          file,
          { encoding: 'utf8', },
        ),).toBe('{"pi-alpha.service":1800000000000}',);
      },
    },),
    it({
      name: 'answers nothing at all for a state file the watch never wrote',
      fn: async () => {
        await using folder = await makeFolder();
        expect(await readCheckedRuns({ file: join(
          folder.path,
          'never-written.json',
        ), },),).toEqual({},);
      },
    },),
    it({
      name: 'answers nothing at all for a state file that is not readable as times',
      fn: async () => {
        await using folder = await makeFolder();
        /**
         A state file a watch killed mid-write could have left.
         */
        const file = join(
          folder.path,
          'cut.json',
        );
        await writeFile(
          file,
          '{"pi-alpha.service": 17',
          { encoding: 'utf8', },
        );
        expect(await readCheckedRuns({ file, },),).toEqual({},);
      },
    },),
    it({
      name: 'drops a value that is no moment, so an edited file cannot silence a finding',
      fn: async () => {
        await using folder = await makeFolder();
        /**
         A state file whose values are not all moments: one is infinite, which JSON writes as a number too large to hold.
         */
        const file = join(
          folder.path,
          'not-times.json',
        );
        await writeFile(
          file,
          '{"pi-alpha.service":1e999,"pi-beta.service":1800000000000}',
          { encoding: 'utf8', },
        );
        expect(await readCheckedRuns({ file, },),).toEqual({ 'pi-beta.service': REPORTED_AT, },);
      },
    },),
    it({
      name: 'keeps the times a hand-edited file holds and leaves everything else in it out',
      fn: async () => {
        await using folder = await makeFolder();
        /**
         A state file holding a list instead of an object, and an object holding values that are no times.
         */
        const file = join(
          folder.path,
          'edited.json',
        );
        await writeFile(
          file,
          JSON.stringify([
            REPORTED_AT,
          ],),
          { encoding: 'utf8', },
        );
        expect(await readCheckedRuns({ file, },),).toEqual({},);
        await writeFile(
          file,
          JSON.stringify({
            'pi-alpha.service': REPORTED_AT,
            'pi-beta.service': 'yesterday',
            'pi-gamma.service': null,
          },),
          { encoding: 'utf8', },
        );
        expect(await readCheckedRuns({ file, },),).toEqual({ 'pi-alpha.service': REPORTED_AT, },);
      },
    },),
  ],
},);

await describe({
  name: defaultCheckedRunsFile.name,
  children: [
    it({
      name: 'keeps its state in the user state folder, never in the runs folder and never under a named home',
      fn: async () => {
        /**
         Where the watch would keep what it reported.
         */
        const file = defaultCheckedRunsFile();
        expect(file.endsWith('/pi-overdue-watch/reported-runs.json',),).toBe(true,);
        expect(file.startsWith('/',),).toBe(true,);
      },
    },),
  ],
},);
