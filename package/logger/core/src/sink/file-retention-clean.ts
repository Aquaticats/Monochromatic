/**
 Candidate collection, measurement, selection, and deletion for retention sweeps. @module
 */
import {
  readdir,
  rm,
  stat,
  unlink,
} from 'node:fs/promises';
import type { Stats, } from 'node:fs';
import { join, } from 'node:path';
import type { reportLoggerInternalError, } from '../error-format.ts';
import {
  loggerOwnedDay,
  NOT_LOGGER_OWNED,
} from './file-retention-name.ts';
import type { LogRetentionPolicy, } from './file-retention-policy.ts';

//region Missing paths are races, not faults
/**
 Sentinel for paths a concurrent sweep or cleanup already removed.
 */
export const PATH_ABSENT: unique symbol = Symbol('path already removed by a concurrent sweep or cleanup');

/**
 Files modified within this window are never deleted,
 because a running process keeps appending to them.
 */
const FRESH_FILE_GRACE_MS = 3_600_000;

/**
 Missing-path races against concurrent sweeps or cleanups
 are expected signals rather than faults.

 @param error - caught value from a filesystem call

 @returns Whether the fault only says the path is already gone

 @example
 ```ts
 if (isMissingPathFault(error)) return PATH_ABSENT;
 ```
 */
function isMissingPathFault(error: unknown,): boolean {
  return ((typeof error) === 'object')
    && (error !== null)
    && ('code' in error)
    && ((error as { readonly code: unknown, }).code === 'ENOENT');
}

/**
 Stat a path that may already be gone.

 @param path - target this sweep races other deletions for

 @returns File info, or {@link PATH_ABSENT} when a concurrent sweep won the race

 @throws rethrows unexpected faults for the caller's report path

 @example
 ```ts
 const info = await statIfPresent('/logs/2026-10-22/file.log.jsonl');
 ```
 */
export async function statIfPresent(path: string,): Promise<Stats | typeof PATH_ABSENT> {
  try {
    return await stat(path,);
  }
  catch (error: unknown) {
    if (isMissingPathFault(error,))
      return PATH_ABSENT;
    throw error;
  }
}
//endregion Missing paths are races, not faults

//region Candidates, measured and trimmed oldest first
/**
 One logger-owned file inside a destination directory.
 */
export type SweepEntry = {
  readonly path: string;
  readonly name: string;
  /**
   UTC day key of the name, the age key.
   */
  readonly day: string;
};

/**
 One measured file kept by the budget pass.
 */
export type MeasuredEntry = SweepEntry & {
  readonly size: number;
  readonly modified: number;
};

/**
 Collect logger-owned files inside the given day directories.

 @param directory - destination directory holding the day directories

 @param days - kept day directory entries from the root listing

 @returns Candidates with their day directory's age key

 @example
 ```ts
 const candidates = await collectDayFiles({ directory: '/logs', days: dayDirectories });
 ```
 */
export async function collectDayFiles({
  directory,
  days,
}: {
  readonly directory: string;
  readonly days: readonly SweepEntry[];
},): Promise<SweepEntry[]> {
  /**
   Names inside each kept day directory, read concurrently.
   */
  const dayNames = await Promise.all(days.map(function readDayNames(day: SweepEntry,) {
    return readdir(join(
      directory,
      day.name,
    ),);
  },),);
  /**
   Candidates collected from the kept day directories.
   */
  const collected: SweepEntry[] = [];
  for (const [index, day,] of days.entries()) {
    /**
     Names inside one day directory; a concurrently removed directory reads as empty.
     */
    const names = dayNames[index] ?? [];
    for (const name of names) {
      if (loggerOwnedDay(name,) === NOT_LOGGER_OWNED)
        continue;
      collected.push({
        path: join(
          directory,
          day.name,
          name,
        ),
        name,
        /**
         The day directory owns its contents wholesale,
         so its age key governs every file inside.
         */
        day: day.day,
      },);
    }
  }
  return collected;
}

/**
 Measure kept candidates concurrently, skipping files already gone.

 @param candidates - survivors of the age pass

 @param reportError - logger fault reporter injected for deterministic tests

 @returns Measurements for candidates still present

 @example
 ```ts
 const measured = await measureCandidates({ candidates, reportError });
 ```
 */
export async function measureCandidates({
  candidates,
  reportError,
}: {
  readonly candidates: readonly SweepEntry[];
  readonly reportError: typeof reportLoggerInternalError;
},): Promise<MeasuredEntry[]> {
  /**
   Concurrent stats, one per candidate, so a large tree costs one wait.
   */
  const measurements = await Promise.all(candidates.map(async function measure(candidate: SweepEntry,) {
    try {
      /**
       File info of the candidate, or the absence sentinel for a lost race.
       */
      const info = await statIfPresent(candidate.path,);
      return info === PATH_ABSENT
        ? PATH_ABSENT
        : {
          path: candidate.path,
          name: candidate.name,
          day: candidate.day,
          size: info.size,
          modified: info.mtimeMs,
        };
    }
    catch (error: unknown) {
      reportError({
        context: `log retention could not measure ${JSON.stringify(candidate.path,)}`,
        error,
      },);
      return PATH_ABSENT;
    }
  },),);
  /**
   Measurements kept from the concurrent stat pass.
   */
  const measured: MeasuredEntry[] = [];
  for (const measurement of measurements) {
    if (measurement !== PATH_ABSENT)
      measured.push(measurement,);
  }
  return measured;
}

/**
 Select the oldest deletions that bring both budget bounds within limits,
 never selecting files a running process may still append to.

 @param measured - kept candidates with sizes and modification times

 @param policy - configured budget bounds

 @param now - sweep instant

 @returns Candidates to delete, oldest first, stopping before fresh files

 @example
 ```ts
 const deletions = selectDeletions({ measured, policy, now: Date.now() });
 ```
 */
export function selectDeletions({
  measured,
  policy,
  now,
}: {
  readonly measured: readonly MeasuredEntry[];
  readonly policy: LogRetentionPolicy;
  readonly now: number;
},): MeasuredEntry[] {
  /**
   Kept totals of the measured tree, counted before any deletion is selected.
   */
  const kept = {
    bytes: 0,
    files: 0,
  };
  for (const entry of measured) {
    kept.bytes += entry.size;
    kept.files += 1;
  }
  /**
   Deletion list built oldest first until both bounds hold.
   */
  const deletions: MeasuredEntry[] = [];
  for (const entry of measured) {
    if ((kept.bytes <= policy.maxBytes) && (kept.files <= policy.maxFiles))
      break;
    /**
     Freshly modified files keep their records intact even over budget,
     because the writing process still owns them,
     while aged entries behind them still fund the bounds.
     */
    if ((now - entry.modified) < FRESH_FILE_GRACE_MS)
      continue;
    deletions.push(entry,);
    kept.bytes -= entry.size;
    kept.files -= 1;
  }
  return deletions;
}

/**
 Delete the selected paths together,
 reporting unexpected failures without discarding the rest.

 @param paths - absolute paths this sweep owns and removes

 @param recursive - remove whole day directories instead of single files

 @param context - fault report context naming the deletion step

 @param reportError - logger fault reporter injected for deterministic tests

 @example
 ```ts
 await deleteSelected({ paths, recursive: false, context: 'log retention trim', reportError });
 ```
 */
export async function deleteSelected({
  paths,
  recursive,
  context,
  reportError,
}: {
  readonly paths: readonly string[];
  readonly recursive: boolean;
  readonly context: string;
  readonly reportError: typeof reportLoggerInternalError;
},): Promise<void> {
  /**
   Concurrent removals; one failure must not discard the others.
   */
  const outcomes = await Promise.allSettled(paths.map(function removePath(path: string,) {
    return recursive
      ? rm(
        path,
        {
          recursive: true,
          force: true,
        },
      )
      : unlink(path,);
  },),);
  for (const outcome of outcomes) {
    if (outcome.status !== 'rejected')
      continue;
    /**
     Reason of a failed removal; an expected race skips instead of reporting.
     */
    const reason: unknown = outcome.reason;
    if (isMissingPathFault(reason,))
      continue;
    reportError({
      context,
      error: reason,
    },);
  }
}
//endregion Candidates, measured and trimmed oldest first
