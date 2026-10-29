import {
  mkdtemp,
  rm,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';
import { setTimeout as wait, } from 'node:timers/promises';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  l,
  startWatching,
} from '../dist/final/node/index.mjs';

/**
 Maximum time allowed for bounded startWatching restart failures to surface.
 */
const WATCH_RESTART_FAILURE_WAIT_MS = 1_000;

/**
 Sentinel returned when startWatching does not fail within the regression window.
 */
const START_WATCHING_STILL_PENDING = Symbol('startWatching still pending');

//region Fixture helpers

/**
 Creates isolated temp directory for startWatching regression tests.
 
 @returns Temp directory path.
 
 @example
 ```ts
 const tempDir = await setup();
 ```
 */
async function setup(): Promise<string> {
  return await mkdtemp(join(
    scratchRoot(),
    'file-enforcer-watch-start-regression-',
  ),);
}

/**
 Removes isolated temp directory.
 
 @param tempDir - Directory returned by {@link setup}.
 
 @example
 ```ts
 await teardown(tempDir);
 ```
 */
async function teardown(tempDir: string,): Promise<void> {
  await rm(
    tempDir,
    {
      recursive: true,
      force: true,
    },
  );
}

//endregion Fixture helpers

//region Error capture helpers

/**
 Captures startWatching rejection for assertions.
 
 @param configPath - Config path passed to startWatching.
 
 @returns Caught error, or undefined if startWatching unexpectedly resolves.
 
 @example
 ```ts
 const caught = await captureStartWatchingFailure('/tmp/missing/config.ts');
 ```
 */
async function captureStartWatchingFailure(configPath: string,): Promise<unknown> {
  try {
    await startWatching(configPath,);
  }
  catch (error: unknown) {
    return error;
  }

  return undefined;
}

/**
 Returns a sentinel after the bounded watch restart failure window.
 
 @returns Pending sentinel.
 
 @example
 ```ts
 const result = await pendingStartWatchingSentinel();
 ```
 */
async function pendingStartWatchingSentinel(): Promise<typeof START_WATCHING_STILL_PENDING> {
  await wait(WATCH_RESTART_FAILURE_WAIT_MS,);
  return START_WATCHING_STILL_PENDING;
}

//endregion Error capture helpers

await describe({
  name: startWatching.name,
  concurrency: 1,
  children: [
    it({
      name: 'restarts failed watchers only up to a bounded limit',
      fn: async function startWatchingFailsAfterRestartLimit({ sinon, },): Promise<void> {
        const errorStub = sinon.stub(l, 'error',);
        const infoStub = sinon.stub(l, 'info',);
        const tempDir = await setup();
        await using _cleanup = {
          [Symbol.asyncDispose](): Promise<void> {
            return teardown(tempDir,);
          },
        };
        const missingConfigPath = join(
          tempDir,
          'missing-config-root',
          'file-enforcer.config.ts',
        );

        const result = await Promise.race([
          captureStartWatchingFailure(missingConfigPath,),
          pendingStartWatchingSentinel(),
        ],);

        expect(result,).not.toBe(START_WATCHING_STILL_PENDING,);
        expect(result,).toBeInstanceOf(Error,);
        expect((result as Error).message,)
          .toContain('watcher restart limit exceeded',);
        expect(errorStub,).toHaveBeenCalledWith(
          expect.stringContaining('watcher failed in',),
        );
        expect(errorStub,).toHaveBeenCalledWith(
          expect.stringContaining('watcher restart limit exceeded',),
        );
        expect(infoStub,).toHaveBeenCalledWith(
          expect.stringContaining('restarting watcher in',),
        );
      },
    },),
  ],
},);
