import {
  NaturalnessCompletenessError,
  NaturalnessRepairInterruptedError,
} from '@monochromatic-dev/translation-repair-adjudication/ts';
import {
  ArchiveOriginalCompletenessError,
  DroppedDestinationError,
  FrontMatterCompletenessError,
  PublishedPageDisagreesError,
  UnansweredContestSliceError,
  UnparseablePageError,
} from '@monochromatic-dev/translation-repair-page-check/ts';
import {
  CollapsedHeadingError,
  ContributorCompletenessError,
  SliceSpliceError,
} from '@monochromatic-dev/translation-repair-page-pass/ts';
import {
  endsTheRun,
  PromptPayloadStoreError,
} from '@monochromatic-dev/translation-repair-stage/ts';
import {
  lifecycleStamp,
  wallClockNow,
} from '@monochromatic-dev/translation-repair-markup/ts';
import { TranslationRepairInterruptedError, } from '@monochromatic-dev/translation-repair-translate-stage/ts';
import { VisualEvidenceInterruptedError, } from './visual-evidence-completeness.ts';
import type { EntryOutcome, } from './pass-entry-contract.ts';
import { tallyErrorText, } from './tally-error-text.ts';

//region Entry failure scheduling

/**
 Tally status and scheduler outcome for one caught entry error.

 @example
 ```ts
 const classified = entryErrorOutcome({ error, });
 ```
 */
export type EntryErrorOutcome = {
  /**
   Operational tally state.
   */
  readonly status: 'ERROR' | 'INCOMPLETE';

  /**
   Whether whole-entry scheduler may resume cached progress.
   */
  readonly outcome: Exclude<EntryOutcome, { readonly kind: 'settled'; }>;
};

/**
 Keeps stage-local quality work and completeness invariants out of whole-entry retry.

 @param error - caught entry failure

 @returns Tally status and scheduler disposition

 @example
 ```ts
 const classified = entryErrorOutcome({ error: new Error('transport'), });
 ```
 */
export function entryErrorOutcome(
  { error, }: { readonly error: unknown; },
): EntryErrorOutcome {
  /**
   Whether error names stage-local incomplete or invariant work.
   */
  const stopped = (error instanceof ArchiveOriginalCompletenessError)
    || (error instanceof ContributorCompletenessError)
    || (error instanceof DroppedDestinationError)
    // The owner's rule: every provider dry ends the run. A whole-entry retry meets the same dry router.
    || endsTheRun({ error, },)
    || (error instanceof FrontMatterCompletenessError)
    || (error instanceof NaturalnessRepairInterruptedError)
    || (error instanceof NaturalnessCompletenessError)
    || (error instanceof PromptPayloadStoreError)
    || (error instanceof TranslationRepairInterruptedError)
    || (error instanceof VisualEvidenceInterruptedError)
    // Ledger A7: deterministic refusals of the assembled page or the artifact,
    // which a retry resuming the same cached slices reproduces exactly.
    || (error instanceof CollapsedHeadingError)
    || (error instanceof UnparseablePageError)
    || (error instanceof PublishedPageDisagreesError)
    || (error instanceof UnansweredContestSliceError)
    || (error instanceof SliceSpliceError);
  return stopped
    ? {
      status: 'INCOMPLETE',
      outcome: { kind: 'stopped', },
    }
    : {
      status: 'ERROR',
      outcome: { kind: 'resumable-failure', },
    };
}

/**
 Prints the TALLY line for an entry that raised out of its pipeline, and
 returns the scheduler's disposition for it.

 THE STATUSES THIS LINE PRINTS, one per class of failure (a settled entry prints `SETTLED` and a declined one
 `DECLINED`, from their own lines):
 `ERROR` for a failure a whole-entry retry may resume, and `INCOMPLETE` for stage-local work or a run-ending refusal that
 no retry would change. Nothing parks the pass for NeuralWatt's day to start again (the owner's answer of 2026-10-29,
 "Never wait, relaunch"), so no entry is put back to wait and no line says `PARKED`.

 EVERY LINE CARRIES THE CLOCK TIME IT WAS PRINTED AT AND THE ATTEMPT IT BELONGS TO, as `at=` and `attempt=` straight
 after the status (after the entry on an `INTERRUPTED` line), before the capped reason. The hard
 ceiling ends an attempt and the next attempt begins under a fresh one, so a line that named neither could not be placed
 in a time window or tied to the attempt it closed (2026-10-22, `XingZ60`; ledger B960).

 @param entryId - entry that failed

 @param error - what it raised

 @param durationMs - time before it failed, on `monotonicMs`

 @param aborted - whether the entry's signal had aborted when it failed: the hard-ceiling abort fired, or the dry
 router stopped the run (the TALLY line prints it as `aborted=`)

 @param endRun - told of the failure when it ends the run, before the entry is filed, so the pass stops starting entries
 however the failure reached the entry: a guarded call carries it by the shared abort, the seat reading and a layer
 that re-raises it reach here without one

 @param attempt - number of the entry's attempt that failed, as the pass's attempts file holds it

 @param wallClock - clock the lines' time is read on; the wall clock unless a test fixes it

 @returns Scheduling disposition, never a settlement

 @throws `LifecycleAttemptError` when the attempt is not a whole number from 1

 @example
 ```ts
 return tallyCaughtEntry({ entryId: entry.id, error, durationMs, aborted, attempt, },);
 ```
 */
export function tallyCaughtEntry(
  {
    entryId,
    error,
    durationMs,
    aborted,
    endRun,
    attempt,
    wallClock = wallClockNow,
  }: {
    readonly entryId: string;
    readonly error: unknown;
    readonly durationMs: number;
    readonly aborted: boolean;
    readonly attempt: number;
    readonly wallClock?: () => Date;
    readonly endRun?: (args: { readonly error: unknown; },) => void;
  },
): EntryErrorOutcome['outcome'] {
  /**
   Failure text for the TALLY line, named or quoted per its class and capped.
   */
  const message = tallyErrorText({ error, },);
  /**
   Tally and retry classification for caught state.
   */
  const classified = entryErrorOutcome({ error, },);
  if (endsTheRun({ error, },))
    endRun?.({ error, },);
  // THE FINDINGS REACH THE LOG, one line each, since the tally line carries the
  // reason alone and the sixth yuki418330012 pass of 2026-09-09 stopped twice
  // as carried-evidence-lost with nothing in the log saying which region.
  if (error instanceof TranslationRepairInterruptedError) {
    for (const finding of error.findings)
      console.log(
        `INTERRUPTED ${entryId} ${
          lifecycleStamp({
            attempt,
            wallClock,
          },)
        }: ${finding}`,
      );
  }
  console.log(
    `TALLY ${entryId} status=${classified.status} ${
      lifecycleStamp({
        attempt,
        wallClock,
      },)
    } ms=${String(durationMs,)} aborted=${String(aborted,)} `
      + `error=${message}`,
  );
  return classified.outcome;
}

//endregion Entry failure scheduling
