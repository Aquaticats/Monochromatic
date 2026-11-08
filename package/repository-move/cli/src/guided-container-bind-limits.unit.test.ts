/**
 Test of the container the cut-over starts to check a bind through the link: its command carries CPU, memory and process
 limits and the lowest disk weight, from the shared helper, so the command the owner is shown cannot start an unbounded
 container.

 @module
 */

import {
  chmod,
  mkdtemp,
  rm,
  writeFile,
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
  runShell,
  type Stage,
} from '../dist/final/node/index.mjs';

/**
 Identifier of the stage that starts the container.
 */
const CONTAINER_BIND_STAGE_ID = 'c27-container-bind';

/**
 Bytes a Linux pipe holds by default.
 A listing longer than this cannot all be written before a reader that stops early has gone.
 */
const PIPE_CAPACITY_BYTES = 65_536;

/**
 Full pipes the long listing fills, so its writer is still writing after a reader has taken its first buffer.
 */
const PIPES_OF_LISTING = 4;

/**
 The image the stage must choose: the first with a name.
 */
const FIRST_NAMED_IMAGE = 'registry.example/first:latest';

/**
 An image further down the listing.
 */
const LATER_IMAGE = 'registry.example/later:latest';

/**
 The listing line of an image with no name, which the stage skips.
 */
const UNNAMED_IMAGE = '<none>:<none>';

/**
 Permissions of the stand-in `podman`: owner may read and run it.
 */
const STAND_IN_MODE = 0o700;

/**
 Deadline of a shell run, in seconds; the stand-in only prints a file.
 */
const SHELL_DEADLINE_SECONDS = 30;

/**
 Seconds between terminate and kill for a shell run.
 */
const SHELL_KILL_GRACE_SECONDS = 5;

/**
 Exit status of a shell that succeeded.
 */
const SHELL_SUCCESS = 0;

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

/**
 Runs the stage's line that chooses an image against a stand-in `podman`, in the shell the runner gives a stage.

 @param listing - lines the stand-in prints

 @returns the shell's exit status and the image it chose

 @throws Error when the stage's command has no first line

 @example
 ```ts
 const { status, chosen, } = await chooseImageFrom(['registry.example/first:latest',],);
 ```
 */
async function chooseImageFrom(
  listing: readonly string[],
): Promise<{ readonly status: number; readonly chosen: string; }> {
  await using home = await makeHome();
  /**
   The line of the stage that picks the image.
   */
  const chooseImage = (await containerBindCommand()).split('\n',).at(0,);
  if (chooseImage === undefined)
    throw new Error(`stage ${CONTAINER_BIND_STAGE_ID} has no first line`,);
  /**
   File the stand-in prints.
   */
  const listingFile = join(home.path, 'listing',);
  /**
   The stand-in, found first on the shell's path.
   */
  const standIn = join(home.path, 'podman',);
  await writeFile(listingFile, `${listing.join('\n',)}\n`,);
  await writeFile(standIn, `#!/usr/bin/env bash\nexec cat '${listingFile}'\n`,);
  await chmod(standIn, STAND_IN_MODE,);
  /**
   What the shell printed.
   */
  const printed: string[] = [];
  /**
   The shell's exit status.
   */
  const status = await runShell({
    command: `PATH='${home.path}':"$PATH"\n${chooseImage}\nprintf '%s\\n' "$image"`,
    outputFile: join(home.path, 'output',),
    seconds: SHELL_DEADLINE_SECONDS,
    killGrace: SHELL_KILL_GRACE_SECONDS,
    outputFileEnv: join(home.path, 'output',),
    out: (text: string,) => {
      printed.push(text,);
    },
  },);
  return { status, chosen: printed.join('',).trim(), };
}

/**
 A listing longer than a pipe holds, whose first named image is the one the stage must choose.

 @returns the listing's lines

 @example
 ```ts
 const listing = longListing();
 ```
 */
function longListing(): readonly string[] {
  /**
   Lines needed to fill the pipes.
   */
  const lineCount = Math.ceil((PIPE_CAPACITY_BYTES * PIPES_OF_LISTING) / LATER_IMAGE.length,);
  return [UNNAMED_IMAGE, FIRST_NAMED_IMAGE, ...Array.from({ length: lineCount, }, () => LATER_IMAGE,),];
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
    it({
      name: 'chooses the first named image from a listing longer than a pipe holds',
      fn: async () => {
        /**
         The result of choosing from the long listing.
         */
        const result = await chooseImageFrom(longListing(),);
        expect(result.status,).toBe(SHELL_SUCCESS,);
        expect(result.chosen,).toBe(FIRST_NAMED_IMAGE,);
      },
    },),
    it({
      name: 'chooses the first named image from a short listing',
      fn: async () => {
        /**
         The result of choosing from a listing that fits in a pipe.
         */
        const result = await chooseImageFrom([UNNAMED_IMAGE, FIRST_NAMED_IMAGE, LATER_IMAGE,],);
        expect(result.status,).toBe(SHELL_SUCCESS,);
        expect(result.chosen,).toBe(FIRST_NAMED_IMAGE,);
      },
    },),
    it({
      name: 'fails when no image has a name',
      fn: async () => {
        /**
         The result of choosing from a listing of unnamed images.
         */
        const result = await chooseImageFrom([UNNAMED_IMAGE,],);
        expect(result.status,).toBeGreaterThan(SHELL_SUCCESS,);
      },
    },),
  ],
},);
