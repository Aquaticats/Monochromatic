/**
 Test of the container the cut-over starts to check a bind through the link: its command carries CPU, memory and process
 limits and the lowest disk weight, from the shared helper, so the command the owner is shown cannot start an unbounded
 container.

 @module
 */

import {
  mkdtemp,
  rm,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { diskWeightArguments, } from '@monochromatic-dev/container-limit-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import {
  createGuidedConfig,
  cutoverStages,
  type Stage,
} from '../dist/final/node/index.mjs';

/**
 Identifier of the stage that starts the container.
 */
const CONTAINER_BIND_STAGE_ID = 'c27-container-bind';

/**
 A disposable home folder, removed when the scope ends.
 */
type HomeFolder = {
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Makes a disposable home folder.

 @returns the folder, to hold with `await using`

 @example
 ```ts
 await using home = await makeHome();
 ```
 */
async function makeHome(): Promise<HomeFolder> {
  /**
   Path of the new folder.
   */
  const path = await mkdtemp(join(scratchRoot(), 'guided-container-bind-limits-',),);
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

/**
 Command of the container-bind stage, built from a disposable configuration.

 @returns the stage's command text

 @throws Error when the stage's command is not plain text

 @example
 ```ts
 const command = await containerBindCommand();
 ```
 */
async function containerBindCommand(): Promise<string> {
  await using home = await makeHome();
  /**
   The configuration the stages read.
   */
  const config = await createGuidedConfig({ env: { HOME: home.path, }, nodePath: process.execPath, },);
  /**
   The cut-over stages.
   */
  const stages = await cutoverStages({ c: config, stamp: 'S', },);
  /**
   The stage that starts the container.
   */
  const stage = stages.find((candidate: Stage,) => candidate.id === CONTAINER_BIND_STAGE_ID,) as Stage;
  if ((typeof stage.command) !== 'string')
    throw new Error(`stage ${CONTAINER_BIND_STAGE_ID} has no plain command text`,);
  return stage.command;
}

await describe({
  name: 'the container-bind stage of the cut-over',
  children: [
    it({
      name: 'starts its container at the lowest disk weight in both forms',
      fn: async () => {
        /**
         The stage's command.
         */
        const command = await containerBindCommand();
        for (const option of diskWeightArguments()) {
          expect(command,).toContain(option,);
        }
      },
    },),
    it({
      name: 'keeps the memory, CPU and process limits it had',
      fn: async () => {
        /**
         The stage's command.
         */
        const command = await containerBindCommand();
        expect(command,).toContain('--memory=256m',);
        expect(command,).toContain('--cpus=1',);
        expect(command,).toContain('--pids-limit=64',);
      },
    },),
    it({
      name: 'puts every limit before the bind mount and the image',
      fn: async () => {
        /**
         The stage's command.
         */
        const command = await containerBindCommand();
        /**
         Where the bind mount starts.
         */
        const mount = command.indexOf('--mount',);
        expect(mount,).toBeGreaterThan(-1,);
        for (const option of [
          '--memory=256m',
          '--blkio-weight=',
          '--cgroup-conf=',
        ]) {
          expect(command.indexOf(option,),).toBeLessThan(mount,);
        }
      },
    },),
  ],
},);
