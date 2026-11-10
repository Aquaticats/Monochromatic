import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { wait, } from '@monochromatic-dev/async-time-core/ts';
import {
  mkdir,
  mkdtempDisposable,
  rmdir,
  stat,
  utimes,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  LOCK_STALE_MS,
  withShadowLogLock,
  type ShadowLogLockBounds,
} from './log-lock.ts';

/**
 Scratch directory for this suite's lock fixtures, removed when this module's
 scope unwinds.
 */
await using scratchDirectory = await mkdtempDisposable(join(scratchRoot(), 'log-lock-',),);

/** Fixture root this suite shares; `scratchDirectory` owns its removal. */
const SCRATCH = scratchDirectory.path;

/**
 Concurrency a serializing lock permits at one time.
 */
const SERIAL_CONCURRENCY = 1;

/**
 Staleness bound for fixtures that must never look abandoned.
 */
const HELD_STALE_MS = 60_000;

/**
 Retry budget short enough to expire inside a test.
 */
const SHORT_BUDGET_MS = 40;

/**
 Retry pause short enough to keep a test fast.
 */
const SHORT_INTERVAL_MS = 5;

/**
 Retry budget long enough for a stale reclaim to land.
 */
const RECLAIM_BUDGET_MS = 5_000;

/**
 Age written on an abandoned lock fixture, past the staleness bound.
 */
const ABANDONED_AGE_MS = LOCK_STALE_MS * 2;

/**
 Probe a path that must be gone,
 returning the lookup error that proves its absence.

 @param path - Path that must no longer resolve

 @returns the lookup error

 @example
 ```ts
 await probeMissing('/state/shadow.jsonl.lock');
 ```
 */
async function probeMissing(path: string,): Promise<Error> {
  try {
    await stat(path,);
  } catch (error) {
    if (Error.isError(error,)) return error;
    throw error;
  }
  throw new Error(`path still resolves: ${path}`,);
}

await describe({
  name: '',
  children: [
    describe({
      name: withShadowLogLock.name,
      children: [
        //region Contended sections

        it({
          name: 'serializes overlapping sections across concurrent callers',
          fn: async () => {
            /**
             Log path whose lock both callers take.
             */
            const logPath = join(
              SCRATCH,
              'serialize.jsonl',
            );
            /**
             Sections inside the critical section at one time.
             */
            let inside = 0;
            /**
             Peak observed concurrency, which the lock holds at the serial bound.
             */
            let peak = 0;
            await Promise.all([
              withShadowLogLock({
                logPath,
                work: async function firstSection() {
                  inside += 1;
                  peak = Math.max(
                    peak,
                    inside,
                  );
                  await wait(20,);
                  inside -= 1;
                  return 'first';
                },
              },),
              withShadowLogLock({
                logPath,
                work: async function secondSection() {
                  inside += 1;
                  peak = Math.max(
                    peak,
                    inside,
                  );
                  await wait(20,);
                  inside -= 1;
                  return 'second';
                },
              },),
            ]);
            expect(peak,).toBe(SERIAL_CONCURRENCY,);
          },
        },),

        it({
          name: 'returns the section result when uncontended',
          fn: async () => {
            const result = await withShadowLogLock({
              logPath: join(
                SCRATCH,
                'uncontended.jsonl',
              ),
              work: async function alone() {
                return 'ran';
              },
            },);
            expect(result,).toBe('ran',);
          },
        },),

        //endregion Contended sections

        //region Lock lifecycle

        it({
          name: 'releases the lock once the section finishes',
          fn: async () => {
            /**
             Log path whose lock directory must not outlive the section.
             */
            const logPath = join(
              SCRATCH,
              'released.jsonl',
            );
            await withShadowLogLock({
              logPath,
              work: async function finishing() {
                return true;
              },
            },);
            /**
             Lookup error proving the lock directory is gone.
             */
            const missingError = await probeMissing(`${logPath}.lock`,);
            expect(missingError.message,)
              .toContain('ENOENT',);
          },
        },),

        it({
          name: 'releases the lock when the section throws',
          fn: async () => {
            /**
             Log path whose lock directory must not outlive a failing section.
             */
            const logPath = join(
              SCRATCH,
              'throws.jsonl',
            );
            let caught: unknown;
            try {
              await withShadowLogLock({
                logPath,
                work: async function failing() {
                  throw new Error('section failed',);
                },
              },);
            } catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(Error,);
            expect((caught as Error).message,)
              .toContain('section failed',);
            /**
             Lookup error proving the lock directory is gone.
             */
            const missingError = await probeMissing(`${logPath}.lock`,);
            expect(missingError.message,)
              .toContain('ENOENT',);
          },
        },),

        //endregion Lock lifecycle

        //region Degraded takes

        it({
          name: 'runs the section unlocked once the retry budget expires',
          fn: async () => {
            /**
             Log path whose lock a foreign holder keeps.
             */
            const logPath = join(
              SCRATCH,
              'budget.jsonl',
            );
            /**
             Lock directory held by the fixture, not by a section.
             */
            const lockPath = `${logPath}.lock`;
            await mkdir(lockPath,);
            /**
             Bounds that expire before staleness,
             so the held lock cannot be reclaimed.
             */
            const bounds: ShadowLogLockBounds = {
              staleMs: HELD_STALE_MS,
              retryBudgetMs: SHORT_BUDGET_MS,
              retryIntervalMs: SHORT_INTERVAL_MS,
            };
            const result = await withShadowLogLock({
              logPath,
              bounds,
              work: async function degradedSection() {
                return 'ran-unlocked';
              },
            },);
            expect(result,).toBe('ran-unlocked',);
            /**
             Foreign lock directory metadata,
             which the fallback must leave untouched.
             */
            const keptLockStat = await stat(lockPath,);
            expect(keptLockStat.isDirectory(),).toBe(true,);
            await rmdir(lockPath,);
          },
        },),

        it({
          name: 'reclaims a lock abandoned past the staleness bound',
          fn: async () => {
            /**
             Log path whose lock fixture is backdated like a killed process left it.
             */
            const logPath = join(
              SCRATCH,
              'stale.jsonl',
            );
            /**
             Lock directory abandoned by the fixture.
             */
            const lockPath = `${logPath}.lock`;
            await mkdir(lockPath,);
            /**
             Abandoned take time, older than the staleness bound under test.
             */
            const abandonedAt = new Date(Date.now() - ABANDONED_AGE_MS,);
            await utimes(
              lockPath,
              abandonedAt,
              abandonedAt,
            );
            const result = await withShadowLogLock({
              logPath,
              bounds: {
                staleMs: LOCK_STALE_MS,
                retryBudgetMs: RECLAIM_BUDGET_MS,
                retryIntervalMs: SHORT_INTERVAL_MS,
              },
              work: async function afterReclaim() {
                return 'ran-after-reclaim';
              },
            },);
            expect(result,).toBe('ran-after-reclaim',);
            /**
             Lookup error proving the reclaimed lock directory is gone again.
             */
            const missingError = await probeMissing(lockPath,);
            expect(missingError.message,)
              .toContain('ENOENT',);
          },
        },),

        //endregion Degraded takes
      ],
    },),
  ],
},);
