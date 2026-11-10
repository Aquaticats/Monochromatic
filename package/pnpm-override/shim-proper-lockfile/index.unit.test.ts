import { existsSync, rmdirSync, rmSync, writeFileSync, } from 'node:fs';
import { mkdtemp, rm, } from 'node:fs/promises';
import { basename, dirname, join, resolve, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import lockfile from './index.cjs';

//region Fixtures

/** Retry budget for the async cases, small enough that an exhausted budget still reports quickly. */
const ASYNC_RETRIES = 2;
/** First backoff wait for the async cases, in milliseconds. */
const ASYNC_MIN_TIMEOUT_MS = 20;
/** How long a holder keeps the lock before the waiting `lock` call is allowed to succeed, in milliseconds. */
const HOLDER_RELEASE_DELAY_MS = 5;
/** Budget each async case gives `lock` before it gives up, in milliseconds. */
const ASYNC_CASE_TIMEOUT_MS = 5_000;
/** Name of the entry placed inside a lock directory to make its removal fail. */
const BLOCKING_ENTRY_NAME = 'holder-note';
/** The one removal failure the shim must tolerate, reported by `rmdirSync` when the directory is gone. */
const TOLERATED_REMOVAL_CODE = 'ENOENT';
/** The removal failure a lock directory with an entry left inside it reports. */
const BLOCKED_REMOVAL_CODE = 'ENOTEMPTY';
/** Lock conflict code the upstream contract defines. */
const HELD_CODE = 'ELOCKED';

/**
 Scratch directory holding one lockable target, removed with whatever lock state the case left behind once the
 declaring scope ends.
 */
type ScratchTarget = {
  readonly target: string;
  readonly lockDirectory: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Directory the shim locks, recomputed from the naming convention this package's README documents instead of read back
 from the implementation, so a case can observe the directory the caller is promised.
 */
function lockDirFor(target: string,): string {
  const absolute = resolve(target,);
  return resolve(dirname(absolute,), `.${basename(absolute,)}.lock`,);
}

/**
 Creates a scratch directory with one lockable target file.

 @returns the fixture; declare it with `await using` so the scratch directory is removed at the end of the scope
 */
async function makeTarget(): Promise<ScratchTarget> {
  const root = await mkdtemp(join(scratchRoot(), 'shim-proper-lockfile-',),);
  const target = join(root, 'auth.json',);
  writeFileSync(target, '{}',);
  return {
    target,
    lockDirectory: lockDirFor(target,),
    async [Symbol.asyncDispose]() {
      await rm(root, { recursive: true, force: true, },);
    },
  };
}

/**
 Calls `call` and hands back what it threw, so each case asserts on the error itself instead of on a wrapper.
 */
function caughtFrom(call: () => void,): unknown {
  try {
    call();
  }
  catch (error) {
    return error;
  }
  return undefined;
}

/**
 Awaits `call` and hands back what it rejected with, so each async case asserts on the error itself.
 */
async function rejectionFrom(call: () => Promise<unknown>,): Promise<unknown> {
  try {
    await call();
  }
  catch (error) {
    return error;
  }
  return undefined;
}

/**
 Places an entry inside the lock directory, so `rmdirSync` reports `ENOTEMPTY` the way an unremovable lock would.

 @param lockDirectory - Lock directory a case holds
 */
function blockRemoval(lockDirectory: string,): void {
  writeFileSync(join(lockDirectory, BLOCKING_ENTRY_NAME,), 'still here',);
}

//endregion Fixtures

await describe({
  name: '',
  children: [
    //region Export shape

    describe({
      name: 'export shape',
      children: [
        it({
          name: 'is callable and carries lock and lockSync members, matching the upstream function-form export',
          fn: async () => {
            expect(typeof lockfile,).toBe('function',);
            expect(typeof lockfile.lock,).toBe('function',);
            expect(typeof lockfile.lockSync,).toBe('function',);
          },
        },),
      ],
    },),
    //endregion Export shape

    //region lockSync

    describe({
      name: 'lockSync',
      children: [
        it({
          name: 'creates the .<basename>.lock directory and its release removes it',
          fn: async () => {
            await using fixture = await makeTarget();

            const release = lockfile.lockSync(fixture.target,);
            expect(existsSync(fixture.lockDirectory,),).toBe(true,);

            release();

            expect(existsSync(fixture.lockDirectory,),).toBe(false,);
          },
        },),
        it({
          name: 'throws ELOCKED naming the lock directory when another holder owns the lock',
          fn: async () => {
            await using fixture = await makeTarget();

            const held = lockfile.lockSync(fixture.target,);
            const thrown = caughtFrom(() => {
              lockfile.lockSync(fixture.target,);
            },);
            held();

            expect(thrown,).toHaveProperty('code', HELD_CODE,);
            expect((thrown as Error).message,).toContain(fixture.lockDirectory,);
          },
        },),
      ],
    },),
    //endregion lockSync

    //region release after a removal that fails for a reason other than the lock being gone

    describe({
      name: 'release after a removal that fails for a reason other than the lock being gone',
      children: [
        it({
          name: 'rethrows the filesystem error instead of reporting the lock released',
          fn: async () => {
            await using fixture = await makeTarget();

            const release = lockfile.lockSync(fixture.target,);
            blockRemoval(fixture.lockDirectory,);

            const thrown = caughtFrom(release,);

            expect(thrown,).toHaveProperty('code', BLOCKED_REMOVAL_CODE,);
            expect(existsSync(fixture.lockDirectory,),).toBe(true,);
          },
        },),
        it({
          name: 'retries the removal on the next release call once the obstacle is gone',
          fn: async () => {
            await using fixture = await makeTarget();

            const release = lockfile.lockSync(fixture.target,);
            blockRemoval(fixture.lockDirectory,);
            expect(caughtFrom(release,),).toBeDefined();

            rmSync(join(fixture.lockDirectory, BLOCKING_ENTRY_NAME,),);
            release();

            expect(existsSync(fixture.lockDirectory,),).toBe(false,);
          },
        },),
      ],
    },),
    //endregion release after a removal that fails for a reason other than the lock being gone

    //region release when the lock directory is already gone

    describe({
      name: 'release when the lock directory is already gone',
      children: [
        it({
          name: `tolerates the ${TOLERATED_REMOVAL_CODE} disappearance and stays quiet on every later release call`,
          fn: async () => {
            await using fixture = await makeTarget();

            const release = lockfile.lockSync(fixture.target,);
            rmdirSync(fixture.lockDirectory,);

            expect(caughtFrom(release,),).toBeUndefined();
            expect(caughtFrom(release,),).toBeUndefined();
          },
        },),
      ],
    },),
    //endregion release when the lock directory is already gone

    //region async lock

    describe({
      name: 'lock',
      children: [
        it({
          name: 'retries until the holder releases, then removes the lock directory on await release()',
          fn: async () => {
            await using fixture = await makeTarget();

            const held = lockfile.lockSync(fixture.target,);
            setTimeout(held, HOLDER_RELEASE_DELAY_MS,);

            const release = await lockfile.lock(fixture.target, {
              retries: { retries: ASYNC_RETRIES, minTimeout: ASYNC_MIN_TIMEOUT_MS, },
            },);
            await release();

            expect(existsSync(fixture.lockDirectory,),).toBe(false,);
          },
          timeout: ASYNC_CASE_TIMEOUT_MS,
        },),
        it({
          name: 'throws ELOCKED once the retry budget is exhausted',
          fn: async () => {
            await using fixture = await makeTarget();

            const held = lockfile.lockSync(fixture.target,);
            const thrown = await rejectionFrom(async () => {
              await lockfile.lock(fixture.target, {
                retries: { retries: 0, minTimeout: ASYNC_MIN_TIMEOUT_MS, },
              },);
            },);
            held();

            expect(thrown,).toHaveProperty('code', HELD_CODE,);
          },
          timeout: ASYNC_CASE_TIMEOUT_MS,
        },),
      ],
    },),
    //endregion async lock
  ],
},);
