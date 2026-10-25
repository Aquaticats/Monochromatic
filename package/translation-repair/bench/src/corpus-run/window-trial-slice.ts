import type { Logger, } from '@monochromatic-dev/module-logger/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import type { PreparedSheetFacts, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import {
  isInsertionChunk,
  type RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { neighbouringSource, } from '@monochromatic-dev/translation-repair-repair/ts';
import {
  judgeTranslateSlate,
  produceTranslateSlate,
} from '@monochromatic-dev/translation-repair-translate-stage/ts';
import {
  appendTrialRow,
  trialKey,
  type WindowTrialRow,
} from './window-trial-ledger.ts';
import {
  armOrderFor,
  TRIAL_ARM_SET,
} from './window-trial-order.ts';
import { TrialSliceRefusalError, } from './window-trial-refusal.ts';
import { TRIAL_ARMS, } from './window-trial-report.ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import type { StageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Window trial slice
// One slice's three arms, over ONE slate.
//
// PRODUCING ONCE IS THE WHOLE POINT. `runTranslateStage` writes candidates and
// judges them in one call, so asking the same slice twice used to resample the
// candidates: the two answers then differed in the slate as well as in the
// evidence, and no reading could say which moved the verdict. The split
// stage lets this buy the slate once and put it to the judges three times.
//
// THE THIRD ARM IS NOT A LUXURY. Judges are stochastic, so a narrow-to-wide
// difference means nothing until it beats the difference between two narrow runs
// of the same slate. That second narrow arm is the only thing that supplies it.

/**
 Runs the arms one slice still owes, appending each as it completes.

 SKIPS THE WHOLE SLICE when the ledger already holds all three arms, without
 producing a slate. Producing is the expensive half, and a resumed run that
 bought a slate only to throw it away would pay most of the cost of the work it
 is skipping.

 @param client - injected model client

 @param slices - every prepared slice of this entry, for the window

 @param sliceIndex - position of the slice under trial

 @param sliceClass - class the screen flagged, or the control label

 @param entryId - entry the slice belongs to

 @param protocol - digest this run buys under

 @param ledgerPath - where completed arms are appended

 @param done - arms already bought, as keys

 @param models - translator and judge rosters

 @param signal - caller abort honored by every exchange

 @param perCallTimeoutMs - deadline per exchange

 @param l - run logger

 @param sheetFacts - what the entry's preparation tells every judging sheet of
 the page, so the sheets carry the tense block and the identity block the pass's
 do; the translators are told the identity block alone, as in the pass

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns Rows this call appended, empty when the slice was already complete

 @throws {@link TrialSliceRefusalError} when the slice index is not one the prepared slices carry or its
 neighbouring original holds no text

 @throws Whatever the stage throws; a slice that cannot be judged is a defect
 rather than a datum, and recording it as a keep would report a failed arm as
 the judges preserving the archive

 @example
 ```ts
 const rows = await runSliceArms({ client, slices, sliceIndex, ... },);
 ```
 */
export async function runSliceArms(
  {
    client,
    slices,
    sliceIndex,
    sliceClass,
    entryId,
    protocol,
    ledgerPath,
    done,
    models,
    signal,
    perCallTimeoutMs,
    l,
    sheetFacts,
    stageSettings,
  }: ForeignBorrowed<{
    readonly client: ModelClient;
    readonly slices: readonly ChunkPair[];
    readonly sliceIndex: number;
    readonly sliceClass: string;
    readonly entryId: string;
    readonly protocol: string;
    readonly ledgerPath: string;
    readonly done: ReadonlySet<string>;
    readonly models: {
      readonly translatorModelIds: readonly RosterModelId[];
      readonly judgeModelIds: readonly RosterModelId[];
    };
    readonly signal: AbortSignal;
    readonly perCallTimeoutMs: number;
    readonly l: Logger;
    readonly sheetFacts: PreparedSheetFacts;
    readonly stageSettings: StageSettings;
  }>,
): Promise<readonly WindowTrialRow[]> {
  /**
   Order this slice buys its arms in, assigned from its own identity so the
   wide arm is not always the last call.
   */
  const order = armOrderFor({
    protocol,
    entryId,
    sliceIndex,
  },);

  /**
   Arms this slice still owes, in buying order.
   */
  const owed = order.filter(function notBought(arm,): boolean {
    // THROUGH `trialKey`, never hand-joined. The two builders disagreed once,
    // on a NUL separator against a space, which no reader shows and which made
    // every resumed run re-buy arms the ledger already held.
    return !done.has(trialKey({ row: {
      protocol,
      entryId,
      sliceIndex,
      arm,
    }, },),);
  },);
  if (owed.length === 0)
    return [];

  // A PARTLY BOUGHT SLICE IS SPOILED, NOT RESUMABLE. The slate cannot be
  // reproduced, so finishing the remaining arms here would judge different
  // candidates from the arms already on disk, and the ledger would then hold a
  // triple that looks complete while its arms disagree about the slate as well
  // as the evidence. That is precisely the confound the stage was split to remove,
  // arriving through resumption instead of through the stage. Left as it is,
  // the slice stays incomplete, and the report already excludes it and says so.
  if (owed.length !== TRIAL_ARM_SET.length) {
    l.warn(
      `${entryId}/${String(sliceIndex,)}: ${
        String(TRIAL_ARM_SET.length - owed.length,)
      } of ${String(TRIAL_ARM_SET.length,)} arms ${
        wordForCount({
          count: TRIAL_ARM_SET.length - owed.length,
          one: 'survives',
          many: 'survive',
        },)
      } from an interrupted run; `
        + `skipping rather than finishing them over a slate the earlier arms `
        + `never saw`,
    );
    return [];
  }

  /**
   Slice under trial.
   */
  const slice = slices[sliceIndex];
  if (slice === undefined)
    throw new TrialSliceRefusalError({
      entryId,
      sliceIndex,
      kind: 'slice-missing',
    },);

  /**
   Neighbouring original, which only the wide arm is shown.

   Computed BEFORE any call, so a slice whose window turns out empty is
   refused here rather than after two thirds of its quota is spent.
   */
  const neighbouringSourceText = neighbouringSource({
    slices,
    slicePosition: sliceIndex,
  },);
  // TRIMMED, not compared against the empty string. A window of blank lines is a
  // window in name only: the wide sheet would differ from the narrow one by
  // whitespace, and the pair would report a false null under a label saying the
  // judges had been shown the neighbouring original.
  if (neighbouringSourceText.trim() === '')
    throw new TrialSliceRefusalError({
      entryId,
      sliceIndex,
      kind: 'window-empty',
    },);

  /**
   Whether this bed treats its slices as governed by the verse rule.

   IT DOES NOT, and the value is named rather than written twice so the
   producer and the judges provably agree. This trial varies ONE thing, the
   width of the window, and every other input has to be identical across the
   arms and between the halves. A slate built under the verse rule and judged
   without it would vary two things and the difference would be credited to
   the window.

   NOT DERIVED FROM THE SLICE, deliberately. Production decides this per
   enclosing CHUNK, from a set this bed does not carry, so deriving it here
   from the slice text alone would answer a different question than
   production asks and the bed would stop measuring production.
   */
  const TRIAL_TREATS_ITS_SLICES_AS_PROSE = false;

  /**
   Slate every arm judges, bought once.
   */
  const produced = await produceTranslateSlate({
    incumbentKind: isInsertionChunk(slice.target,) ? 'absent' : 'present',
    client,
    translatorModelIds: models.translatorModelIds,
    sourceText: slice.source
      .text,
    incumbentText: slice.target
      .text,
    ...((sheetFacts.identityContext === undefined) ? {} : { identityContext: sheetFacts.identityContext, }),
    lineStructured: TRIAL_TREATS_ITS_SLICES_AS_PROSE,
    signal,
    perCallTimeoutMs,
    l,
    stageSettings,
  },);

  /**
   Rows this call appended.
   */
  const appended: WindowTrialRow[] = [];
  for (const [position, arm,] of order.entries()) {
    /* oxlint-disable no-await-in-loop -- arms are bought one at a time on purpose, so a kill loses one arm rather than three */
    /**
     What the judges made of the same slate under this arm's evidence.
     */
    const decided = await judgeTranslateSlate({
      client,
      produced,
      judgeModelIds: models.judgeModelIds,
      sourceText: slice.source
        .text,
      incumbentText: slice.target
        .text,
      incumbentKind: isInsertionChunk(slice.target,) ? 'absent' : 'present',
      // THE PAGE FACTS EVERY ARM SHARES, so the sheets differ in the window alone
      // and carry what the pass's do: the declared names, the death with the
      // tense candidates of the ballot texts, and the subject's names.
      ...sheetFacts,
      ...((arm === TRIAL_ARMS.wide) ? { neighbouringSourceText, } : {}),
      lineStructured: TRIAL_TREATS_ITS_SLICES_AS_PROSE,
      // EVERY SEAT, because the report drops a triple whose arm heard fewer judges than it seated and the window
      // check counts every judge of the first wide arm. The production default asks a window of quorum plus one on
      // a bench of four since the weight minimum that sized it was removed (2026-10-25); the trial compares arms
      // on whole panels, as it did when that window fell short and the whole bench was asked.
      fanOut: 'whole-bench',
      signal,
      perCallTimeoutMs,
      l,
      stageSettings,
    },);
    /* oxlint-enable no-await-in-loop */

    /**
     Row recording it.
     */
    const row: WindowTrialRow = {
      protocol,
      entryId,
      sliceIndex,
      arm,
      sliceClass,
      shipped: decided.origin !== 'incumbent',
      decision: decided.decision,
      winnerText: decided.text,
      judgesHeard: decided.tally
        .ballots,
      judgesSeated: decided.tally
        .judgesAvailable,
      position,
    };

    // APPENDED BEFORE THE NEXT ARM STARTS, which is what makes a kill cost one
    // arm rather than the slice.
    /* oxlint-disable-next-line no-await-in-loop -- durability is the reason this is sequential */
    await appendTrialRow({
      path: ledgerPath,
      row,
    },);
    appended.push(row,);
  }
  return appended;
}

//endregion Window trial slice
