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
import { WritingBenchUnreachableError, } from '@monochromatic-dev/translation-repair-run/ts';
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
 Whether a failure is a writing bench error that carries NeuralWatt's return, which the pass may park for.

 @param error - what the entry raised

 @returns Whether the error names a return

 @example
 ```ts
 if (carriesNeuralWattReturn({ error, },))
   endRun?.({ error, },);
 ```
 */
function carriesNeuralWattReturn({ error, }: { readonly error: unknown; },): boolean {
  return (error instanceof WritingBenchUnreachableError) && (error.neuralwattReturnsInMs !== undefined);
}

/**
 The gate's answer when the pass is parked for the failure: the return the park waits for.
 */
export type ParkedAnswer = {
  /**
   The pass is parked for this failure.
   */
  readonly parked: true;

  /**
   Milliseconds between the read that named the return and the return.
   */
  readonly returnsInMs: number;

  /**
   Time of day the return falls at, with its zone, as the failure that parked the pass carried it.
   */
  readonly returnsAt: string;
};

/**
 The gate's answer when the pass is not parked for the failure.
 */
export type NotParkedAnswer = {
  /**
   The pass is not parked for this failure.
   */
  readonly parked: false;
};

/**
 What the gate answers when asked whether the pass is parked for a failure.
 */
export type ParkAnswer = ParkedAnswer | NotParkedAnswer;

/**
 A return time that does not end in ` UTC`, though the only writer of the time gives it with that zone.

 @example
 ```ts
 throw new ReturnTimeNotUtcError({ text: '19:15:08 CEST', },);
 ```
 */
export class ReturnTimeNotUtcError extends Error {
  /**
   Builds the diagnostic naming the text the failure carried.

   @param text - return time as the failure carried it

   @example
   ```ts
   new ReturnTimeNotUtcError({ text: '19:15:08 CEST', },);
   ```
   */
  public constructor({ text, }: { readonly text: string; },) {
    super(
      `the return time ${JSON.stringify(text,)} of a NeuralWatt failure does not end in " UTC", the zone its only `
        + 'writer (`returnTimeOf` in the provider package) gives; the TALLY line states the time as an ISO 8601 UTC '
        + 'time of day, so either that writer changed its zone or another writer was added, and the line then needs '
        + 'a conversion for that zone',
    );
    this.name = 'ReturnTimeNotUtcError';
  }
}

/**
 The time of day of a return as one token, so a reader splitting the line on spaces keeps the zone with it.

 @param text - time of day with its zone, as the failure carries it

 @returns ISO 8601 time of day with `Z` for UTC

 @throws {@link ReturnTimeNotUtcError} when the text does not end in ` UTC`, since its only writer gives that zone

 @example
 ```ts
 const token = oneTokenTimeOf({ text: '19:15:08 UTC', },);
 ```
 */
function oneTokenTimeOf({ text, }: { readonly text: string; },): string {
  if (!text.endsWith(' UTC',))
    throw new ReturnTimeNotUtcError({ text, },);
  return `${text.slice(
    0,
    -' UTC'.length,
  )}Z`;
}

/**
 The fields stating the NeuralWatt return a park waits for.

 @param answer - gate's answer for the entry's failure, which says the pass is parked for it

 @returns Leading-space fields: the distance and the time of day

 @throws {@link ReturnTimeNotUtcError} when the park's time of day is not in UTC

 @example
 ```ts
 const fields = returnFieldsOf({ answer: { parked: true, returnsInMs: 301_000, returnsAt: '19:15:08 UTC', }, },);
 ```
 */
function returnFieldsOf({ answer, }: { readonly answer: ParkedAnswer; },): string {
  return ` returnsInMs=${String(answer.returnsInMs,)} returnsAt=${oneTokenTimeOf({ text: answer.returnsAt, },)}`;
}

/**
 Prints the TALLY line for an entry that raised out of its pipeline, and
 returns the scheduler's disposition for it.

 THE STATUSES THIS LINE PRINTS, one per class of failure (a settled entry prints `SETTLED` and a declined one
 `DECLINED`, from their own lines):
 `ERROR` for a failure a whole-entry retry may resume, `INCOMPLETE` for stage-local work or a run-ending refusal that
 no retry would change, and `PARKED` for an entry the pass put back at the front of its queue to wait for NeuralWatt's
 energy to return, which earns no verdict and is attempted again after the wait. A `PARKED` line states the distance
 in `returnsInMs=` and the time of day in `returnsAt=` (ISO 8601, `Z` for UTC, one token) before the reason, so the
 cap on the reason's text never cuts them; both are the park's own, as the gate answers them, and never the entry's
 failure's, which may name no return or another distance; `error=` is the failure the entry ended with.

 @param entryId - entry that failed

 @param error - what it raised

 @param durationMs - time before it failed, on `monotonicMs`

 @param aborted - whether the entry's signal had aborted when it failed: the hard-ceiling abort fired, or the dry
 router stopped the run (the TALLY line prints it as `aborted=`)

 @param endRun - told of the failure when it ends the run or carries a NeuralWatt return the pass may park for, before
 the entry is filed, so the pass stops starting entries (or parks) however the failure reached the entry: a guarded call
 carries it by the shared abort, the seat reading and a layer that re-raises it reach here without one

 @param parked - asked after `endRun` has been told, naming this entry's failure: answers the return the pass is
 parked for when it is parked for that failure, so the entry goes back to the front of its queue and the line prints
 `PARKED` with that return and not the status the failure would have had; answers not parked for a failure that is not
 the park's own

 @returns Scheduling disposition, never a settlement

 @throws {@link ReturnTimeNotUtcError} when the entry is parked for a return whose time is not in UTC, before any
 TALLY line is printed

 @example
 ```ts
 return tallyCaughtEntry({ entryId: entry.id, error, durationMs, aborted, },);
 ```
 */
export function tallyCaughtEntry(
  {
    entryId,
    error,
    durationMs,
    aborted,
    endRun,
    parked,
  }: {
    readonly entryId: string;
    readonly error: unknown;
    readonly durationMs: number;
    readonly aborted: boolean;
    readonly endRun?: (args: { readonly error: unknown; },) => void;
    readonly parked?: (args: { readonly error: unknown; },) => ParkAnswer;
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
  // A bench error that carries NeuralWatt's return is handed over as well, for the pass to park on; the pass leaves
  // it alone when it cannot wait for the return, and the entry is filed as any bench error is.
  if (endsTheRun({ error, },) || carriesNeuralWattReturn({ error, },))
    endRun?.({ error, },);
  // THE FINDINGS REACH THE LOG, one line each, since the tally line carries the
  // reason alone and the sixth yuki418330012 pass of 2026-09-09 stopped twice
  // as carried-evidence-lost with nothing in the log saying which region.
  if (error instanceof TranslationRepairInterruptedError) {
    for (const finding of error.findings)
      console.log(`INTERRUPTED ${entryId}: ${finding}`,);
  }
  /**
   The gate's answer for this failure: the return the pass is parked for where it is parked for this failure and will
   attempt the entry again, and not parked otherwise or where the run gave no gate.
   */
  const answer: ParkAnswer = parked?.({ error, },) ?? { parked: false, };
  /**
   Status the line prints: the entry's own, or `PARKED`.
   */
  const status = answer.parked ? 'PARKED' : classified.status;
  /**
   The return a parked entry waits for, stated before the reason so the cap on the reason never cuts it.
   */
  const returnFields = answer.parked ? returnFieldsOf({ answer, },) : '';
  console.log(
    `TALLY ${entryId} status=${status}${returnFields} ms=${String(durationMs,)} aborted=${String(aborted,)} `
      + `error=${message}`,
  );
  return classified.outcome;
}

//endregion Entry failure scheduling
