/**
 Unit tests for the live update run, using a Node script as a fake pnpm.

 @module
 */

import {
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  runUpdate,
  STRICT_REQUIRES_SAVE_CODE,
  UPDATE_ARGS,
} from './update.ts';

/**
 Fake pnpm script location that removes itself when its `await using` scope ends.
 */
type FakeScript = AsyncDisposable & {
  /**
   Absolute script path.
   */
  readonly path: string;
};

/**
 Writes a Node script standing in for pnpm.

 @param source - script body

 @returns disposable script

 @example
 ```ts
 await using script = await fakeScript('process.exit(0)');
 ```
 */
async function fakeScript(source: string,): Promise<FakeScript> {
  /**
   Directory holding the script.
   */
  const dir = await mkdtemp(join(scratchRoot(), 'deps-update-fake-pnpm-',),);
  /**
   Script path.
   */
  const path = join(dir, 'pnpm.mjs',);
  await writeFile(path, source,);
  return {
    path,
    [Symbol.asyncDispose]: async function removeScript(): Promise<void> {
      await rm(dir, { recursive: true, force: true, },);
    },
  };
}

await describe({
  name: runUpdate.name,
  children: [
    it({
      name: 'passes update arguments and reports success',
      fn: async ctx => {
        // Forwarded stderr lines go through console.error; keep test output quiet.
        ctx.sinon.stub(console, 'error',);
        await using script = await fakeScript(
          `if (process.argv.slice(2).join(' ') !== ${JSON.stringify(UPDATE_ARGS.join(' ',),)}) process.exit(3);\n`
            + "console.error('progress');\n",
        );
        expect(await runUpdate({ cwd: scratchRoot(), command: process.execPath, commandArgs: [script.path,], },),).toEqual({
          ok: true,
        },);
      },
    },),
    it({
      name: 'captures stderr and exit code on failure',
      fn: async ctx => {
        /**
         Stub recording forwarded lines.
         */
        const forwarded = ctx.sinon.stub(console, 'error',);
        await using script = await fakeScript(
          `console.error('Error: ${STRICT_REQUIRES_SAVE_CODE}');\nprocess.exit(1);\n`,
        );
        /**
         Failed outcome.
         */
        const outcome = await runUpdate({ cwd: scratchRoot(), command: process.execPath, commandArgs: [script.path,], },);
        expect(outcome,).toEqual({
          ok: false,
          stderr: `Error: ${STRICT_REQUIRES_SAVE_CODE}`,
          exitCode: 1,
        },);
        expect(forwarded,).toHaveBeenCalledWith(`Error: ${STRICT_REQUIRES_SAVE_CODE}`,);
      },
    },),
    it({
      name: 'omits the exit code when a signal ends pnpm',
      fn: async ctx => {
        ctx.sinon.stub(console, 'error',);
        await using script = await fakeScript("process.kill(process.pid, 'SIGTERM');\n",);
        /**
         Failed outcome.
         */
        const outcome = await runUpdate({ cwd: scratchRoot(), command: process.execPath, commandArgs: [script.path,], },);
        expect(outcome.ok,).toBe(false,);
        expect('exitCode' in outcome,).toBe(false,);
      },
    },),
  ],
},);
