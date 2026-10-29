/**
 Age-window sweeps over one log destination directory. @module
 */
import {
  readdir,
  utimes,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { reportLoggerInternalError, } from '../error-format.ts';
import {
  collectDayFiles,
  deleteSelected,
  measureCandidates,
  PATH_ABSENT,
  selectDeletions,
  statIfPresent,
  type MeasuredEntry,
  type SweepEntry,
} from './file-retention-clean.ts';
import {
  DAY_KEY_LENGTH,
  loggerOwnedDay,
  NOT_LOGGER_OWNED,
} from './file-retention-name.ts';
import type { LogRetentionPolicy, } from './file-retention-policy.ts';

//region Sweep constants
/**
 Milliseconds in one UTC day, the age-key granularity.
 */
const MS_PER_DAY = 86_400_000;

/**
 Milliseconds between sweeps of one destination directory,
 throttling housekeeping off the logging path.
 */
export const SWEEP_INTERVAL_MS = 300_000;

/**
 Stamp file name a sweep claims before walking its destination directory.
 */
const SWEEP_STAMP_NAME = '.logger-retention-sweep';
//endregion Sweep constants

//region Age windows key every owned name by its UTC day prefix
/**
 Oldest day key still inside the age window,
 derived from the name prefix rather than file timestamps,
 so one linear pass ages a whole directory.

 @param now - sweep instant

 @param maxAgeDays - configured age bound of the policy

 @returns Day key of the oldest still-kept day

 @example
 ```ts
 const cutoff = cutoffDay({ now: Date.now(), maxAgeDays: 14 });
 ```
 */
export function cutoffDay({
  now,
  maxAgeDays,
}: {
  readonly now: number;
  readonly maxAgeDays: number;
},): string {
  /**
   Start of the oldest still-kept day in milliseconds.
   */
  const cutoffMs = now - (maxAgeDays * MS_PER_DAY);
  return new Date(cutoffMs,)
    .toISOString()
    .slice(
    0,
    DAY_KEY_LENGTH,
    );
}
//endregion Age windows key every owned name by its UTC day prefix

//region Sweeps expire by age first, then trim the oldest files to the budget
/**
 Bring one destination directory back inside its retention window.

 Sibling loggers and deliberate neighbors are never touched:
 only `YYYY-MM-DD` directories,
 timestamp `*.log.jsonl` files,
 and timestamp `*.probe.tmp` residue qualify for deletion.
 A stamp file throttles the walk,
 and every fault reports through the logger without breaking logging.

 @param directory - destination directory owned by one logger family

 @param policy - retention bounds to restore

 @param now - sweep instant

 @param stampIntervalMs - minimum milliseconds between sweeps of one directory

 @param reportError - logger fault reporter injected for deterministic tests

 @example
 ```ts
 await sweepLogDirectory({ directory: '/repo/node_modules/.monochromatic', policy });
 ```
 */
export async function sweepLogDirectory({
  directory,
  policy,
  now = Date.now(),
  stampIntervalMs = SWEEP_INTERVAL_MS,
  reportError = reportLoggerInternalError,
}: {
  readonly directory: string;
  readonly policy: LogRetentionPolicy;
  readonly now?: number;
  readonly stampIntervalMs?: number;
  readonly reportError?: typeof reportLoggerInternalError;
},): Promise<void> {
  /**
   Stamp path this sweep claims before walking the directory.
   */
  const stampPath = join(
    directory,
    SWEEP_STAMP_NAME,
  );
  try {
    /**
     A fresh stamp means another process swept this directory recently.
     */
    const stamp = await statIfPresent(stampPath,);
    if ((stamp !== PATH_ABSENT) && ((now - stamp.mtimeMs) < stampIntervalMs))
      return;
    /**
     Claiming the stamp before the walk makes concurrent sweeps skip it;
     duplicate sweeps stay harmless because deletions are idempotent.
     */
    await writeFile(
      stampPath,
      `${new Date(now,)
        .toISOString()}\n`,
      { mode: 0o600, },
    );
    await utimes(
      stampPath,
      new Date(now,),
      new Date(now,),
    );
  }
  catch (error: unknown) {
    reportError({
      context: `log retention could not stamp ${JSON.stringify(directory,)}`,
      error,
    },);
    return;
  }
  try {
    /**
     Root listing, partitioned into logger-owned directories and files.
     */
    const entries = await readdir(
      directory,
      { withFileTypes: true, },
    );
    /**
     Logger-owned day directories of the destination directory.
     */
    const dayDirectories: SweepEntry[] = [];
    /**
     Logger-owned stray files at the destination directory root.
     */
    const rootFiles: SweepEntry[] = [];
    for (const entry of entries) {
      /**
       Day key of the listing entry; foreign neighbors stay out of both lists.
       */
      const day = loggerOwnedDay(entry.name,);
      if (day === NOT_LOGGER_OWNED)
        continue;
      /**
       Absolute path of the listing entry under the destination directory.
       */
      const path = join(
        directory,
        entry.name,
      );
      if (entry.isDirectory())
        dayDirectories.push({
          path,
          name: entry.name,
          day,
        },);
      else if (entry.isFile())
        rootFiles.push({
          path,
          name: entry.name,
          day,
        },);
    }
    /**
     Cutoff day key of the configured age window.
     */
    const cutoff = cutoffDay({
      now,
      maxAgeDays: policy.maxAgeDays,
    },);
    /**
     Expired day directories, removed wholesale along with their contents.
     */
    const expiredDayPaths = dayDirectories
      .filter(function olderThanCutoff(entry: SweepEntry,) {
        return entry.day < cutoff;
      },)
      .map(function toPath(entry: SweepEntry,) {
        return entry.path;
      },);
    await deleteSelected({
      paths: expiredDayPaths,
      recursive: true,
      context: 'log retention could not remove expired day directory',
      reportError,
    },);
    /**
     Expired stray files, the legacy flat layout aging out by name prefix.
     */
    const expiredFilePaths = rootFiles
      .filter(function olderThanCutoff(entry: SweepEntry,) {
        return entry.day < cutoff;
      },)
      .map(function toPath(entry: SweepEntry,) {
        return entry.path;
      },);
    await deleteSelected({
      paths: expiredFilePaths,
      recursive: false,
      context: 'log retention could not delete expired file',
      reportError,
    },);
    /**
     Kept day directories within the age window.
     */
    const keptDays = dayDirectories.filter(function withinWindow(entry: SweepEntry,) {
      return entry.day >= cutoff;
    },);
    /**
     Kept stray files within the age window.
     */
    const keptRootFiles = rootFiles.filter(function withinWindow(entry: SweepEntry,) {
      return entry.day >= cutoff;
    },);
    /**
     Kept files inside kept day directories, carrying their directory's age key.
     */
    const dayFiles = await collectDayFiles({
      directory,
      days: keptDays,
    },);
    /**
     All kept files, sorted oldest first for the budget pass.
     */
    const candidates = [
      ...dayFiles,
      ...keptRootFiles,
    ]
      .toSorted(function oldestFirst(
        left: SweepEntry,
        right: SweepEntry,
      ) {
        return (left.day === right.day)
          ? left.name
            .localeCompare(right.name,)
          : left.day
            .localeCompare(right.day,);
      },);
    /**
     Measurements of the kept candidates, present files only.
     */
    const measured = await measureCandidates({
      candidates,
      reportError,
    },);
    /**
     Deletions bringing both budget bounds back within limits.
     */
    const deletions = selectDeletions({
      measured,
      policy,
      now,
    },);
    /**
     Paths of the selected deletions.
     */
    const trimPaths = deletions.map(function toPath(entry: MeasuredEntry,) {
      return entry.path;
    },);
    await deleteSelected({
      paths: trimPaths,
      recursive: false,
      context: 'log retention could not trim file',
      reportError,
    },);
  }
  catch (error: unknown) {
    reportError({
      context: `log retention sweep of ${JSON.stringify(directory,)} failed`,
      error,
    },);
  }
}
//endregion Sweeps expire by age first, then trim the oldest files to the budget
