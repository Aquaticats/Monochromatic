/**
 Cross-process lock around the shadow log's read, deduplicate, and append section.

 Hook invocations overlap once a host runs them asynchronously,
 so two same-event records can scan the log before either appends.
 The lock serializes that section across processes;
 when it cannot be taken in time the section runs unlocked,
 preferring a rare duplicate observation over a dropped one.

 @module
 */

import { wait, } from '@monochromatic-dev/async-time-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  mkdir,
  rmdir,
  stat,
} from 'node:fs/promises';

/**
 Module logger for lock takes, releases, and the unlocked fallback.
 */
const loggerForLock = tagged({ tag: 'cost-dismissal-log-lock', },);

/**
 Suffix appended to the log path to name its lock directory.
 */
const LOCK_PATH_SUFFIX = '.lock' as const;

/**
 Age past which a lock directory counts as abandoned by a killed process,
 so the next section may reclaim it.
 */
const LOCK_STALE_MS = 5_000;

/**
 Longest total wait for the lock before the section runs unlocked.

 Twice {@link LOCK_STALE_MS} so a waiting run outlives staleness and reclaims
 an abandoned lock instead of degrading.
 */
const LOCK_RETRY_BUDGET_MS = 10_000;

/**
 Pause between lock take attempts.
 */
const LOCK_RETRY_INTERVAL_MS = 25;

/**
 Timing bounds of a lock take.
 */
type ShadowLogLockBounds = {
  /**
   Age past which an existing lock directory counts as abandoned.
   */
  readonly staleMs: number;

  /**
   Longest total wait before the section runs unlocked.
   */
  readonly retryBudgetMs: number;

  /**
   Pause between take attempts.
   */
  readonly retryIntervalMs: number;
};

/**
 Production lock bounds: staleness {@link LOCK_STALE_MS},
 budget {@link LOCK_RETRY_BUDGET_MS},
 interval {@link LOCK_RETRY_INTERVAL_MS}.
 */
const PRODUCTION_LOCK_BOUNDS: ShadowLogLockBounds = {
  staleMs: LOCK_STALE_MS,
  retryBudgetMs: LOCK_RETRY_BUDGET_MS,
  retryIntervalMs: LOCK_RETRY_INTERVAL_MS,
};

/**
 Handle for one held lock,
 whose disposal releases the lock directory it names.
 */
type ShadowLogLockHandle = {
  /**
   Lock directory this handle releases.
   */
  readonly lockPath: string;

  /**
   Releases the lock directory when the `await using` scope exits.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Reports whether the lock directory was left behind by a killed process.

 @param lockPath - Lock directory to inspect

 @param staleMs - Age past which the lock counts as abandoned

 @returns whether the lock is older than the staleness bound

 @example
 ```ts
 await isLockStale({ lockPath: '/state/shadow.jsonl.lock', staleMs: 5_000, });
 ```
 */
async function isLockStale(
  {
    lockPath,
    staleMs,
  }: {
    readonly lockPath: string;
    readonly staleMs: number;
  },
): Promise<boolean> {
  try {
    /**
     Lock directory metadata, whose mtime is the take time.
     */
    const lockStat = await stat(lockPath,);
    /**
     Age of the lock take, in milliseconds.
     */
    const lockAgeMs = Date.now() - lockStat.mtimeMs;
    return lockAgeMs > staleMs;
  } catch (error) {
    loggerForLock.debug(`lock vanished while inspected: ${String(error,)}`,);
    return false;
  }
}

/**
 Removes the lock directory,
 tolerating a lock a peer already reclaimed and released.

 @param lockPath - Lock directory to remove

 @example
 ```ts
 await releaseLock({ lockPath: '/state/shadow.jsonl.lock', });
 ```
 */
async function releaseLock(
  {
    lockPath,
  }: {
    readonly lockPath: string;
  },
): Promise<void> {
  try {
    await rmdir(lockPath,);
  } catch (error) {
    loggerForLock.warn(`lock directory did not release cleanly: ${String(error,)}`,);
  }
}

/**
 Builds the disposable handle for a taken lock.

 @param lockPath - Lock directory the handle releases

 @returns handle whose disposal removes the lock directory

 @example
 ```ts
 await using lock = createLockHandle({ lockPath: '/state/shadow.jsonl.lock', });
 ```
 */
function createLockHandle(
  {
    lockPath,
  }: {
    readonly lockPath: string;
  },
): ShadowLogLockHandle {
  return {
    lockPath,
    async [Symbol.asyncDispose]() {
      await releaseLock({ lockPath, },);
    },
  };
}

/**
 Runs one take attempt: create the lock directory,
 or reclaim and report a retry when a stale lock blocks it.

 @param lockPath - Lock directory to take

 @param staleMs - Age past which a blocking lock counts as abandoned

 @returns whether this attempt now holds the lock

 @example
 ```ts
 await tryTakeLock({ lockPath: '/state/shadow.jsonl.lock', staleMs: 5_000, });
 ```
 */
async function tryTakeLock(
  {
    lockPath,
    staleMs,
  }: {
    readonly lockPath: string;
    readonly staleMs: number;
  },
): Promise<boolean> {
  try {
    await mkdir(lockPath,);
    return true;
  } catch (error) {
    loggerForLock.debug(`lock take failed: ${String(error,)}`,);
    if (await isLockStale({
      lockPath,
      staleMs,
    },)) {
      loggerForLock.debug('reclaiming an abandoned log lock',);
      await releaseLock({ lockPath, },);
    }
    return false;
  }
}

/**
 Reports whether a take sequence's retry budget still allows another attempt.

 @param startedAt - When the first take started

 @param budgetMs - Longest total wait before the section runs unlocked

 @returns whether another attempt fits inside the budget

 @example
 ```ts
 takeBudgetRemains({ startedAt: Date.now(), budgetMs: 10_000, });
 ```
 */
function takeBudgetRemains(
  {
    startedAt,
    budgetMs,
  }: {
    readonly startedAt: number;
    readonly budgetMs: number;
  },
): boolean {
  /**
   Age of the take sequence, in milliseconds.
   */
  const elapsedMs = Date.now() - startedAt;
  return elapsedMs < budgetMs;
}

/**
 Runs the critical section with a taken lock held,
 releasing the lock when the section settles.

 @param lockPath - Lock directory the section holds

 @param work - Critical section to run

 @returns whatever the critical section returns

 @example
 ```ts
 await runSectionUnderLock({ lockPath: '/state/shadow.jsonl.lock', work: append, });
 ```
 */
async function runSectionUnderLock<TWorkResult,>(
  {
    lockPath,
    work,
  }: {
    readonly lockPath: string;
    readonly work: () => Promise<TWorkResult>;
  },
): Promise<TWorkResult> {
  /**
   Lock handle disposed at scope exit, releasing the lock directory.
   */
  await using lock = createLockHandle({ lockPath, },);
  loggerForLock.debug(`shadow log section running under the lock ${lock.lockPath}`,);
  return await work();
}

/**
 Runs one shadow log critical section under the shadow log's cross-process lock.

 The lock is a directory beside the log, taken with an atomic create.
 Takes wait out a live holder and reclaim a lock abandoned past its staleness
 bound; when the retry budget expires the section still runs, unlocked,
 and a warning marks the degraded pass,
 so an observation is never dropped over lock contention.

 @param logPath - Shadow log whose section to serialize

 @param work - Critical section to serialize

 @param bounds - Timing bounds, for tests and disposable runs

 @returns whatever the critical section returns

 @example
 ```ts
 await withShadowLogLock({ logPath: '/state/shadow.jsonl', work: append, });
 ```
 */
async function withShadowLogLock<TWorkResult,>(
  {
    logPath,
    work,
    bounds,
  }: {
    readonly logPath: string;
    readonly work: () => Promise<TWorkResult>;
    readonly bounds?: ShadowLogLockBounds;
  },
): Promise<TWorkResult> {
  /**
   Lock directory identity, derived from the log it protects.
   */
  const lockPath = `${logPath}${LOCK_PATH_SUFFIX}`;
  /**
   Timing bounds in force: production defaults unless a caller overrides.
   */
  const activeBounds = bounds ?? PRODUCTION_LOCK_BOUNDS;
  /**
   When the first take started, for the retry budget check.
   */
  const startedAt = Date.now();
  while (takeBudgetRemains({
    startedAt,
    budgetMs: activeBounds.retryBudgetMs,
  },)) {
    /* oxlint-disable eslint/no-await-in-loop -- take attempt N+1 must wait for attempt N's stale check and pause. */
    /**
     Whether this attempt now holds the lock.
     */
    const taken = await tryTakeLock({
      lockPath,
      staleMs: activeBounds.staleMs,
    },);
    /* oxlint-enable eslint/no-await-in-loop */
    if (taken) return runSectionUnderLock({
      lockPath,
      work,
    },);
    // oxlint-disable-next-line eslint/no-await-in-loop -- retry attempt N+1 must wait for attempt N's backoff pause.
    await wait(activeBounds.retryIntervalMs,);
  }
  loggerForLock.warn(
    `log lock still unavailable after ${String(activeBounds.retryBudgetMs,)}ms; running the section unlocked`,
  );
  loggerForLock.debug('shadow log section running unlocked',);
  return await work();
}

export type { ShadowLogLockBounds, };

export {
  LOCK_RETRY_BUDGET_MS,
  LOCK_RETRY_INTERVAL_MS,
  LOCK_STALE_MS,
  withShadowLogLock,
};
