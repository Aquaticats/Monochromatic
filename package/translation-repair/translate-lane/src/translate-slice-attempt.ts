import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import type { ArchiveDispute, } from '@monochromatic-dev/translation-repair-archive/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import { isInsertionChunk, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { exchangeFailureLogText, } from '@monochromatic-dev/translation-repair-provider-stream/ts';
import type { PreparedDocumentPair, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import {
  failureWritten,
  type StageSettings,
} from '@monochromatic-dev/translation-repair-stage/ts';
import {
  TranslateAbsenceError,
  type TranslateAbsenceReason,
} from '@monochromatic-dev/translation-repair-translate-floor/ts';
import type {
  TranslateModels,
  TranslateSliceRecord,
} from './translate-document-contract.ts';
import { settleTranslateSlice, } from './translate-slice.ts';

//region Translate slice attempt
// One slice's round, with settled and unfilled shapes named.
//
// The stage repairs an absent passage at fixed depth two; when both rounds
// leave it unwritten the slice settles UNFILLED here, one slice rather than
// the entry, and publication's no-page terminal accounts for it
// (doc/planning/translation-repair-no-loop-design.md).
//
// SPLIT FROM THE DRIVER so the union has a home and the driver keeps its line
// budget for the loop it exists to run. What lives here is only the shape of
// one attempt; every decision about the document stays with the driver.
//
// THE ABORT COMES FIRST, before the refusal. Under an abort every exchange is
// torn down and arrives as silence, so a slice with no incumbent reports
// exactly what a genuinely fruitless round reports. Blaming the models for a
// spent deadline would make an entry look unfillable when nothing was asked.

/**
 What one slice's round produced.

 @example
 ```ts
 const attempt: SliceAttempt = { kind: 'settled', record, };
 ```
 */
export type SliceAttempt = {
  /**
   Round produced a record, whatever it decided.
   */
  readonly kind: 'settled';

  /**
   That record.
   */
  readonly record: TranslateSliceRecord;
} | {
  /**
   Slice has no translation in the archive and this round produced none, so
   there is nothing to record and nothing to write.
   */
  readonly kind: 'unfilled';

  /**
   Why nothing could be written.
   */
  readonly reason: TranslateAbsenceReason;

  /**
   What the stage had gathered before it gave up.
   */
  readonly findings: readonly string[];
};

/**
 Runs one slice and reports which of the two endings it reached.

 @param client - injected model client

 @param slice - prepared slice pair

 @param prepared - document the slice came from

 @param models - translator and judge rosters

 @param archiveDispute - dispute over this slice's archive rendering, passed on

 @param signal - entry abort honored by every exchange

 @param perCallTimeoutMs - deadline per exchange

 @param l - driver logger

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns Settled record, or the fact that this passage stays missing

 @throws Whatever the slice throws that is not an absence refusal over an
 insertion slice (a refusal over a slice the archive translates among them),
 and the caller's abort reason by identity when the signal fired

 @example
 ```ts
 const attempt = await attemptTranslateSlice({ client, slice, prepared, models, signal, ... },);
 ```
 */
export async function attemptTranslateSlice(
  {
    client,
    slice,
    prepared,
    models,
    neighbouringIncumbentText,
    neighbouringSourceText,
    pictureContext,
    pictureFindings,
    archiveDispute,
    signal,
    perCallTimeoutMs,
    l,
    stageSettings,
  }: ForeignBorrowed<{
    readonly client: ModelClient;
    readonly slice: ChunkPair;
    readonly prepared: PreparedDocumentPair;
    readonly models: TranslateModels;
    readonly archiveDispute?: ArchiveDispute;

    /**
     Archive English either side of this slice, computed once by the driver so
     the cache key and the call provably name the same window.
     */
    readonly neighbouringIncumbentText: string;

    /**
     Original either side of this slice, computed the same way.
     */
    readonly neighbouringSourceText: string;

    /**
     What the pictures this slice and its neighbours show were read as,
     gathered once per document so every slice showing one picture is shown
     the same words about it.
     */
    readonly pictureContext: string;

    /**
     One line per picture no reading is available for.
     */
    readonly pictureFindings: readonly string[];
    readonly signal: AbortSignal;
    readonly perCallTimeoutMs: number;
    readonly l: Logger;
    readonly stageSettings: StageSettings;
  }>,
): Promise<SliceAttempt> {
  try {
    return {
      kind: 'settled',
      record: await settleTranslateSlice({
        client,
        slice,
        prepared,
        models,
        neighbouringIncumbentText,
        neighbouringSourceText,
        pictureContext,
        pictureFindings,
        ...((archiveDispute === undefined) ? {} : { archiveDispute, }),
        signal,
        perCallTimeoutMs,
        l,
        stageSettings,
      },),
    };
  }
  catch (error) {
    // An aborted run fails BECAUSE it was aborted; whichever torn-down exchange
    // happened to surface is a symptom. The caller has to tell a spent deadline
    // apart from a provider fault by identity alone, and only one of those is
    // worth retrying the entry over.
    if (signal.aborted) {
      // A FAILURE A LOWER LAYER ALREADY WROTE keeps its words on that line
      // alone, such as a repair re-ask `repairOneCandidate` wrote with its
      // seat (ledger B403 left this open as a reading, B412 ran it); this
      // line then says only that the slice was abandoned.
      /**
       What surfaced, in parentheses, where no lower layer wrote it.
       */
      const surfaced = failureWritten({ error, },) ? '' : ` (${exchangeFailureLogText({ error, },)})`;
      l.warn(
        `slice ${String(slice.target
          .sliceIndex,)}: abandoned by the caller's abort${surfaced}`,
      );
      throw signal.reason;
    }
    // SECOND BACKSTOP, and deliberately not trusting the error alone. Only a
    // slice with nothing in the archive can be unfilled; a content slice
    // reported that way would record a passage the archive DOES translate as
    // one it never did, and every count of missing passages would inherit it.
    // So an absence on a content slice is rethrown with every other failure.
    // Since ledger B39 none arrives: wording the archive has and the floor
    // refuses stands on the stage's follow-up round rather than raising. A
    // slate nobody was heard on never arrives as an absence either: the stage
    // turns it into a provider interruption (`translate-stage-repair.ts`).
    if ((error instanceof TranslateAbsenceError) && isInsertionChunk(slice.target,)) {
      return {
        kind: 'unfilled',
        reason: error.reason,
        findings: error.findings,
      };
    }
    throw error;
  }
}

//endregion Translate slice attempt
