import { join, } from 'node:path';

import { inEntryLogContext, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  type AttemptMap,
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

 @param dryStop - gate the pass's client trips when every provider reads dry

 @example
 ```ts
 announceDryStop({ dryStop, },);
 ```
 */
function announceDryStop({ dryStop, }: { readonly dryStop: PassDryStop; },): void {
  for (const note of dryStop.takeStopNotes())
    console.log(note,);
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

 @param dryStop - gate the pass's client trips when every provider reads dry, or parks when the return of NeuralWatt's
 energy fits what the pass has left; once tripped no further entry starts,
 no REATTEMPT or STALLED line is printed for the entry that tripped it, and the reason is printed once, also when the
 queue ends with the entry that tripped it

 @param softBudgetMs - time after which no new entry starts

 @param spendCeilingUsd - USD the run may spend before it stops starting entries

 @param start - instant the processing loop began, on the clock `now` reads (ledger B78)

 @param now - clock the budget is read on, in milliseconds

 @param settle - settles one entry, given that entry; the pass's whole per-entry procedure

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
    readonly settle: (input: { readonly entry: CorpusPair; },) => Promise<EntryOutcome>;
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

    stopBeforeNext: function stopBeforeNext(): boolean {
      if (dryStop.tripped()) {
        announceDryStop({ dryStop, },);
        return true;
      }
      return stopBeforeNextEntry({
        elapsedMs: now() - start,
        softBudgetMs,
        ceilingUsd: spendCeilingUsd,
      },);
    },

    runHasStopped: function runHasStopped(): boolean {
      return dryStop.tripped();
    },

    // A PARKED PASS WAITS BETWEEN ENTRIES, before the budget is asked whether the pass goes on: the wait spends the
    // budget, and a pass whose budget ran out while it waited ends without starting the entry again.
    beforeNext: async function beforeNext(): Promise<void> {
      await dryStop.awaitPark();
    },

    interrupted: function interrupted(): boolean {
      return dryStop.parked();
    },

    attempt: async function attempt({ entry, },): Promise<EntryOutcome> {
      countAttempt({
        attempts,
        id: entry.id,
      },);
      // Persisted before the attempt so a crash still records that it happened.
      await writeAttemptMap({
        attemptsPath,
        attempts,
      },);

      // Every line and ledger record written for this entry names it (ledger A11).
      return await inEntryLogContext({
        entry: entry.id,
        generation: pipelineDigest,
        run: async function settleInContext(): Promise<EntryOutcome> {
          return await settle({ entry, },);
        },
      },);
    },
  },);

  // THE STOP IS ANNOUNCED WHEN THE QUEUE ENDS TOO, because `stopBeforeNext` is asked only before an attempt: the entry
  // that met the dry router may have been the last one, and a queue that empties never asks again. The note is handed
  // out once, so an announcement already made is not repeated.
  announceDryStop({ dryStop, },);
}

//endregion Corpus pass queue
