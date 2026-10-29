/**
 Tests for Pi's own discovery and loading of the built bash-poke package.

 @module
 */

import { mkdtemp, rm, } from 'node:fs/promises';
import { join, } from 'node:path';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import {
  createEventBus,
  discoverAndLoadExtensions,
} from '@earendil-works/pi-coding-agent';
import { wait, } from '@monochromatic-dev/async-time-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { constants, } from '../dist/final/node/index.mjs';

import { createFakeContext, } from './test-fixtures.ts';

//region Fixtures

/**
 Package directory whose built manifest Pi discovers.
 */
const packageDir = join(import.meta.dirname, '..', );

/**
 Milliseconds a started job is given to finish before the case ends.
 */
const SETTLE_MS = 500;

/**
 Loads the built package through Pi's own discovery path.
 
 @param agentDir - empty agent directory, so no unrelated global extension loads
 
 @returns Pi's discovery result for this package alone
 
 @example
 ```ts
 await loadThroughPi(agentDir);
 ```
 */
async function loadThroughPi(agentDir: string, ) {
  return await discoverAndLoadExtensions(
    [packageDir, ],
    packageDir,
    agentDir,
    createEventBus(),
  );
}

//endregion Fixtures

await describe({
  name: 'Pi discovery',
  concurrency: 1,
  children: [
    it({
      name: 'loads the built package with no discovery errors',
      fn: async () => {
        /**
         Empty agent directory isolating discovery from global extensions.
         */
        const agentDir = await mkdtemp(join(scratchRoot(), 'bash-poke-agent-', ), );
        await using cleanup = {
          async [Symbol.asyncDispose](): Promise<void> {
            await rm(agentDir, { recursive: true, force: true, }, );
          },
        };

        /**
         Discovery result for this package.
         */
        const result = await loadThroughPi(agentDir, );
        expect(result.errors, ).toEqual([]);
        expect(result.extensions, ).toHaveLength(1);
      },
    }, ),
    it({
      name: 'registers the poke renderer and both events, and no commands or shortcuts',
      fn: async () => {
        /**
         Empty agent directory isolating discovery from global extensions.
         */
        const agentDir = await mkdtemp(join(scratchRoot(), 'bash-poke-agent-', ), );
        await using cleanup = {
          async [Symbol.asyncDispose](): Promise<void> {
            await rm(agentDir, { recursive: true, force: true, }, );
          },
        };

        /**
         Sole extension Pi loaded from this package.
         */
        const result = await loadThroughPi(agentDir, );
        const [extension] = result.extensions;
        if (extension === undefined)
          throw new Error('expected one discovered extension');
        expect(extension.handlers.has('user_bash', ), ).toBe(true);
        expect(extension.handlers.has('session_start', ), ).toBe(true);
        expect(extension.messageRenderers.has(constants.POKE_CUSTOM_TYPE, ), ).toBe(true);
        // The design cuts every slash command and every registered shortcut, so
        // a surprise registration here means the surface grew unnoticed.
        expect(extension.commands.size, ).toBe(0);
        expect(extension.shortcuts.size, ).toBe(0);
      },
    }, ),
    it({
      name: 'leaves a hidden command to Pi through the loaded handler',
      fn: async () => {
        /**
         Empty agent directory isolating discovery from global extensions.
         */
        const agentDir = await mkdtemp(join(scratchRoot(), 'bash-poke-agent-', ), );
        await using cleanup = {
          async [Symbol.asyncDispose](): Promise<void> {
            await rm(agentDir, { recursive: true, force: true, }, );
          },
        };

        /**
         Loaded extension whose handler Pi would call for a bash command.
         */
        const result = await loadThroughPi(agentDir, );
        const [extension] = result.extensions;
        if (extension === undefined)
          throw new Error('expected one discovered extension');

        /**
         Handler Pi registered for user bash commands.
         */
        const [handler] = extension.handlers.get('user_bash', ) ?? [];
        if (handler === undefined)
          throw new Error('expected a user_bash handler');

        /**
         Fake interactive context the handler binds against.
         */
        const context = createFakeContext({ mode: 'tui', hasUI: true, }, );

        /**
         Result of a hidden command, which must fall through to Pi.
         */
        const hidden = await handler({
          type: 'user_bash',
          command: 'printf hidden',
          excludeFromContext: true,
          cwd: agentDir,
        }, context.ctx, );
        expect(hidden, ).toBeUndefined();
      },
    }, ),
    it({
      name: 'returns the placeholder for a visible command through the loaded handler',
      fn: async () => {
        /**
         Empty agent directory isolating discovery from global extensions.
         */
        const agentDir = await mkdtemp(join(scratchRoot(), 'bash-poke-agent-', ), );
        await using cleanup = {
          async [Symbol.asyncDispose](): Promise<void> {
            await rm(agentDir, { recursive: true, force: true, }, );
          },
        };

        /**
         Loaded extension whose handler Pi would call for a bash command.
         */
        const result = await loadThroughPi(agentDir, );
        const [extension] = result.extensions;
        if (extension === undefined)
          throw new Error('expected one discovered extension');

        /**
         Handler Pi registered for user bash commands.
         */
        const [handler] = extension.handlers.get('user_bash', ) ?? [];
        if (handler === undefined)
          throw new Error('expected a user_bash handler');

        /**
         Fake interactive context the handler binds against.
         */
        const context = createFakeContext({ mode: 'tui', hasUI: true, }, );

        /**
         Result Pi records while the job runs in the background.
         */
        const taken = await handler({
          type: 'user_bash',
          command: 'printf discovery',
          excludeFromContext: false,
          cwd: agentDir,
        }, context.ctx, ) as { result?: { output?: string; }; };
        expect(taken.result?.output, ).toBe(constants.PENDING_NOTE);

        // The job keeps running after the handler returns, and its poke cannot
        // be delivered because discovery never bound a session runtime. Waiting
        // here proves that undeliverable poke is contained instead of escaping
        // as an unhandled rejection.
        await wait(SETTLE_MS, );
      },
    }, ),
  ],
}, );
