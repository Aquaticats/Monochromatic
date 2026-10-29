import { join, } from 'node:path';

import {
  lifecycleClockStamp,
  wallClockNow,
} from '@monochromatic-dev/translation-repair-markup/ts';
import { inEntryLogContext, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  type AttemptMap,
  attemptsOf,
  countAttempt,
  writeAttemptMap,
} from '@monochromatic-dev/translation-repair-glossary/ts';
import {
  type CorpusPair,
  countCachedSlices,
  type EntryOutcome,
  runAttemptQueue,
  stopBeforeNextEntry,
} from '@monochromatic-dev/translation-repair-slice-cache/ts';

import type { PassDryStop, } from './pass-dry-stop.ts';

//region Corpus pass queue
// Runs the pending entries one attempt at a time under the pass's budgets.

/**
 Prints the pass's stop line if the dry gate has tripped and the line has not been printed.

 THE LINE ENDS WITH THE CLOCK TIME AND NAMES NO ATTEMPT: it belongs to the pass, and the gate builds its text without a
 clock, so the time is read when the line is printed (ledger B960).

 @param dryStop - gate the pass's client trips when every provider reads dry

 @param wallClock - clock the time is read on

 @example
 ```ts
 announceDryStop({ dryStop, wallClock, },);
 ```
 */
function announceDryStop(
  {
    dryStop,
    wallClock,
  }: {
    readonly dryStop: PassDryStop;
    readonly wallClock: () => Date;
  },
): void {
  for (const note of dryStop.takeStopNotes())
    console.log(`${note} ${lifecycleClockStamp({ wallClock, },)}`,);
}

/**
 Runs every pending entry once, then re-runs the ones that earned it, counting
 and persisting each attempt before it starts and stopping at the soft budget
 or the spend ceiling.

 @param pending - entries in run order

 @param attempts - attempt counts this run keeps and persists

 @param attemptsPath - where the counts are persisted

 @param sliceCacheDir - root of the per-entry slice caches

 @param pipelineDigest - generation every log line of an attempt carries

 @param dryStop - gate the pass's client trips when every provider reads dry; once tripped no further entry starts,
 no REATTEMPT or STALLED line is printed for the entry that tripped it, and the reason is printed once, also when the
 queue ends with the entry that tripped it

 @param softBudgetMs - time after which no new entry starts

 @param spendCeilingUsd - USD the run may spend before it stops starting entries

 @param start - instant the processing loop began, on the clock `now` reads (ledger B78)

 @param now - clock the budget is read on, in milliseconds

 @param wallClock - clock every lifecycle line's time is read on, in the lines this queue prints (the stop lines too) and
 in those the settlement prints; the wall clock unless a test fixes it

 @param settle - settles one entry, given that entry and the number of the attempt about to run (what the attempts file
 holds once this queue has counted it); the pass's whole per-entry procedure

 @mutates attempts - counts one more attempt for an entry ahead of the attempt itself

 @example
 ```ts
 await runPassQueue({ pending, attempts, attemptsPath, sliceCacheDir, pipelineDigest, dryStop, softBudgetMs, spendCeilingUsd, start, now, settle, },);
 ```
 */
export async function runPassQueue(
  {
    pending,
    attempts,
    attemptsPath,
    sliceCacheDir,
    pipelineDigest,
    dryStop,
    softBudgetMs,
    spendCeilingUsd,
    start,
    now,
    wallClock = wallClockNow,
    settle,
  }: {
    readonly pending: readonly CorpusPair[];
    readonly attempts: AttemptMap;
    readonly attemptsPath: string;
    readonly sliceCacheDir: string;
    readonly pipelineDigest: string;
    readonly dryStop: PassDryStop;
    readonly softBudgetMs: number;
    readonly spendCeilingUsd: number;
    readonly start: number;
    readonly now: () => number;
    readonly wallClock?: () => Date;
    readonly settle: (
      input: {
        readonly entry: CorpusPair;
        readonly attempt: number;
        readonly wallClock: () => Date;
      },
    ) => Promise<EntryOutcome>;
  },
): Promise<void> {
  await runAttemptQueue({
    pending,

    cachedCountFor: function cachedCountFor({ entry, },): Promise<number> {
      return countCachedSlices({
        dir: join(
          sliceCacheDir,
          entry.id,
        ),
      },);
    },

    // THE NUMBER THE ATTEMPTS FILE HOLDS, not a count of the queue's own: the lines this queue prints for an attempt
    // and the lines its settlement prints must name the same number, and the file is what outlives the process.
    attemptOf: function attemptOf({ entry, },): number {
      return attemptsOf({
        attempts,
        id: entry.id,
      },);
    },

    wallClock,

    stopBeforeNext: function stopBeforeNext(): boolean {
      if (dryStop.tripped()) {
        announceDryStop({
          dryStop,
          wallClock,
        },);
        return true;
      }
      return stopBeforeNextEntry({
        elapsedMs: now() - start,
        softBudgetMs,
        ceilingUsd: spendCeilingUsd,
        wallClock,
      },);
    },

    runHasStopped: function runHasStopped(): boolean {
      return dryStop.tripped();
    },

    attempt: async function attempt({ entry, },): Promise<EntryOutcome> {
      countAttempt({
        attempts,
        id: entry.id,
        attemptsPath,
      },);
      // Persisted before the attempt so a crash still records that it happened.
      await writeAttemptMap({
        attemptsPath,
        attempts,
      },);

      /**
       Number of the attempt about to run, as the attempts file now holds it.
       */
      const attemptNumber = attemptsOf({
        attempts,
        id: entry.id,
      },);

      // Every line and ledger record written for this entry names it and the attempt (ledger A11, B960).
      return await inEntryLogContext({
        entry: entry.id,
        generation: pipelineDigest,
        attempt: attemptNumber,
        run: async function settleInContext(): Promise<EntryOutcome> {
          return await settle({
            entry,
            attempt: attemptNumber,
            wallClock,
          },);
        },
      },);
    },
  },);

  // THE STOP IS ANNOUNCED WHEN THE QUEUE ENDS TOO, because `stopBeforeNext` is asked only before an attempt: the entry
  // that met the dry router may have been the last one, and a queue that empties never asks again. The note is handed
  // out once, so an announcement already made is not repeated.
  announceDryStop({
    dryStop,
    wallClock,
  },);
}

//endregion Corpus pass queue
