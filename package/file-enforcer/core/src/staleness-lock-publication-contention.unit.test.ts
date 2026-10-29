import { mkdir, } from 'node:fs/promises';
import { join, } from 'node:path';
import { setTimeout as wait, } from 'node:timers/promises';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  acquireManifestLock,
  lockOwnerState,
  publishLockOwnerPublication,
  removeLockOwnerPublication,
  stageLockOwnerPublication,
} from '../dist/final/node/index.mjs';
import {
  setupStalenessLockFixture,
  teardownStalenessLockFixture,
} from './staleness-lock-regression-fixture.ts';

//region Contender observation helpers

/**
 Delay long enough for competing acquisition to observe staged lock.
 */
const CONTENDER_OBSERVATION_DELAY_MS = 50;

/**
 Result from competing acquisition or observation delay.
 */
type ContenderObservation =
  | Readonly<{
    readonly kind: 'acquired';
    readonly lock: AsyncDisposable;
  }>
    | Readonly<{
      readonly kind: 'pending';
    }>;

/**
 Acquires manifest lock and identifies successful acquisition.
 
 @param manifestPath - Manifest path whose lock already has staged owner.
 
 @returns Acquired lock result.
 
 @example
 ```ts
 const result = await acquireContender('/tmp/manifest.json');
 ```
 */
async function acquireContender(manifestPath: string,): Promise<ContenderObservation> {
  return {
    kind: 'acquired',
    lock: await acquireManifestLock(manifestPath,),
  };
}

/**
 Reports that contender remained pending through observation delay.
 
 @returns Pending observation result.
 
 @example
 ```ts
 const result = await observeContenderPending();
 ```
 */
async function observeContenderPending(): Promise<ContenderObservation> {
  await wait(CONTENDER_OBSERVATION_DELAY_MS,);
  return { kind: 'pending', };
}

//endregion Contender observation helpers

await describe({
  name: 'file-enforcer lock publication contention',
  concurrency: 1,
  children: [
    it({
      name: 'keeps competing acquisition pending until staged owner is published and released',
      timeout: 10_000,
      fn: async function keepsContenderPendingDuringOwnerStaging(): Promise<void> {
        const tempDir = await setupStalenessLockFixture();
        await using _cleanup = {
          [Symbol.asyncDispose](): Promise<void> {
            return teardownStalenessLockFixture(tempDir,);
          },
        };
        const manifestPath = join(
          tempDir,
          'contended-manifest.json',
        );
        const lockPath = `${manifestPath}.lock`;
        await mkdir(lockPath,);
        /**
         Complete live-owner metadata held at private publication stage.
         */
        const ownerText = `${JSON.stringify(
          {
            pid: process.pid,
            createdAt: new Date().toISOString(),
          },
          null,
          2,
        )}\n`;
        await stageLockOwnerPublication({
          lockPath,
          ownerText,
        },);

        /**
         Real acquisition started while owner remains privately staged.
         */
        const contenderPromise = acquireContender(manifestPath,);
        /**
         First result distinguishes premature acquisition from waiting.
         */
        const firstObservation = await Promise.race([
          contenderPromise,
          observeContenderPending(),
        ],);
        if (firstObservation.kind === 'acquired') {
          await firstObservation.lock[Symbol.asyncDispose]();
          throw new Error('Contender acquired manifest lock before staged owner publication',);
        }
        expect(await lockOwnerState(lockPath,),).toBe('absent',);

        await publishLockOwnerPublication(lockPath,);
        expect(await lockOwnerState(lockPath,),).toBe('live',);
        await removeLockOwnerPublication(lockPath,);

        /**
         Contender acquisition completed only after predecessor release.
         */
        const acquiredContender = await contenderPromise;
        if (acquiredContender.kind !== 'acquired')
          throw new Error('Contender did not acquire released manifest lock',);
        await using _contenderLock = acquiredContender.lock;
        expect(await lockOwnerState(lockPath,),).toBe('live',);
      },
    },),
  ],
},);
