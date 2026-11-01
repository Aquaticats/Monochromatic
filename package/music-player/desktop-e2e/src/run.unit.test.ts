/**
 Tests for the preconditions of a run. The rest of the run needs a display, PipeWire and systemd; the end-to-end task
 exercises it.
 
 @module
 */

import {
  chmod,
  mkdir,
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import {
  describe,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { loadRunConfig, } from './config.ts';
import { checkPreconditions, } from './run.ts';

/**
 Runs the preconditions against a fake compositor that prints the given text for `--version`.
 
 @param versionOutput - what the fake compositor prints
 
 @returns the fake compositor's path and the message of the precondition failure, empty when all preconditions held
 */
async function preconditionMessage(
  { versionOutput, }: { readonly versionOutput: string; },
): Promise<{ readonly compositor: string; readonly message: string; }> {
  const appDir = await mkdtemp(join(scratchRoot(), 'e2e-pre-',),);
  await using _scratch = {
    async [Symbol.asyncDispose](): Promise<void> {
      await rm(appDir, { recursive: true, force: true, },);
    },
  };
  await mkdir(join(appDir, 'target', 'release',), { recursive: true, },);
  await writeFile(join(appDir, 'target', 'release', 'music-player',), '',);
  const compositor = join(appDir, 'fake-compositor',);
  await writeFile(compositor, `#!/usr/bin/env node\nconsole.log(${JSON.stringify(versionOutput,)});\n`,);
  await chmod(compositor, 0o700,);
  try {
    await checkPreconditions(loadRunConfig({
      appDir,
      home: scratchRoot(),
      overrides: { compositorPath: compositor, },
      defaultCompositorPath: '/no/such/compositor',
    },),);
  }
  catch (error: unknown) {
    return { compositor, message: caughtValueText(error,), };
  }
  return { compositor, message: '', };
}

await describe({
  name: checkPreconditions.name,
  children: [
    it({
      name: 'refuses to start without the release binary, before touching anything else',
      fn: async ({ expect, }) => {
        const appDir = join(scratchRoot(), 'e2e-no-such-app',);
        let caught: unknown;
        try {
          await checkPreconditions(loadRunConfig({
            appDir,
            home: scratchRoot(),
            overrides: {},
            defaultCompositorPath: '/no/such/compositor',
          },),);
        }
        catch (error: unknown) {
          caught = error;
        }
        expect((caught as Error).message,).toBe(
          `the release binary ${join(appDir, 'target', 'release', 'music-player',)} does not exist; the task builds it first`,
        );
      },
    },),
    it({
      name: 'refuses a compositor that does not exist, before asking PipeWire',
      fn: async ({ expect, }) => {
        const appDir = await mkdtemp(join(scratchRoot(), 'e2e-pre-',),);
        await mkdir(join(appDir, 'target', 'release',), { recursive: true, },);
        await writeFile(join(appDir, 'target', 'release', 'music-player',), '',);
        const compositor = join(appDir, 'no-such-compositor',);
        let caught: unknown;
        try {
          await checkPreconditions(loadRunConfig({
            appDir,
            home: scratchRoot(),
            overrides: { compositorPath: compositor, },
            defaultCompositorPath: '/no/such/compositor',
          },),);
        }
        catch (error: unknown) {
          caught = error;
        }
        await rm(appDir, { recursive: true, force: true, },);
        expect((caught as Error).message,).toBe(
          `the compositor ${compositor} does not exist; install it or set E2E_COMPOSITOR`,
        );
      },
    },),
    it({
      name: 'refuses a compositor older than the checks need, naming the upgrade and the override',
      fn: async ({ expect, }) => {
        const { compositor, message, } = await preconditionMessage({ versionOutput: 'monochromatic-nested-wayland-session 0.3.0', },);
        expect(message,).toContain(`the compositor ${compositor} is 0.3.0`,);
        expect(message,).toContain('mise upgrade cargo:monochromatic-nested-wayland-session',);
        expect(message,).toContain('E2E_COMPOSITOR',);
      },
    },),
    it({
      name: 'refuses a compositor whose version output cannot be read',
      fn: async ({ expect, }) => {
        const { compositor, message, } = await preconditionMessage({ versionOutput: 'no version here', },);
        expect(message,).toContain(`the compositor ${compositor} printed no version ("no version here")`,);
        expect(message,).toContain('E2E_COMPOSITOR',);
      },
    },),
    it({
      name: 'does not refuse a newer compositor for its version; the later checks may still stop the run',
      fn: async ({ expect, }) => {
        const newer = await preconditionMessage({ versionOutput: 'monochromatic-nested-wayland-session 9.9.9', },);
        expect(newer.message,).not.toContain('the compositor',);
        const prerelease = await preconditionMessage({ versionOutput: 'monochromatic-nested-wayland-session 0.5.0-dev', },);
        expect(prerelease.message,).not.toContain('the compositor',);
      },
    },),
  ],
},);
