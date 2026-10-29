import type { RunnerClosure, } from '@monochromatic-dev/translation-repair-insertion/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { FidelityReferenceSpec, } from '../fidelity-reference-model.ts';
import type {
  EditorDamageKind,
  EditorFidelityTrial,
} from '../editor-fidelity-trials.ts';
import type { AskEnd, } from './editor-fidelity-asks.ts';
import type { EditorCeiling, } from './editor-fidelity-ceiling.ts';
import { decisionRuleText, } from './editor-fidelity-decide.ts';
import type { EditorTrialReading, } from './editor-fidelity-trial.ts';
import type { ProbeRun, } from './probe-store.ts';

//region Editor fidelity plan
// What the editor fidelity probe writes down and says, built from values it already holds: the durable plan, one row
// for each card and trial, the tally of each card and the lines that report them. Kept apart from the driver so the
// driver stays a sequence of steps and each shape here has a case of its own.

/**
 One completed row: the reviewed twin's provenance beside how the card did on it.

 @example
 ```ts
 const row: EditorFidelityRow = { cardId, trialId: 'tabby-reference/insertion', referenceId, entryId, damageKind: 'insertion', changedChars: 12, damageDetail, reading, };
 ```
 */
export type EditorFidelityRow = {
  /**
   Card the row belongs to.
   */
  readonly cardId: RosterModelId;

  /**
   Trial the card was seated on.
   */
  readonly trialId: string;

  /**
   Reviewed reference the trial came from.
   */
  readonly referenceId: string;

  /**
   Corpus entry the reference came from.
   */
  readonly entryId: string;

  /**
   Defect the trial carries.
   */
  readonly damageKind: EditorDamageKind;

  /**
   Changed-content count the builder reviewed.
   */
  readonly changedChars: number;

  /**
   Safe structural description of the intentional delta.
   */
  readonly damageDetail: string;

  /**
   How the trial read.
   */
  readonly reading: EditorTrialReading;

  /**
   How each ask of the trial ended, in the order asked.
   */
  readonly stops: readonly AskEnd[];

  /**
   Which pass over the pool the row belongs to, from zero; absent for the first pass.
   */
  readonly pass?: number;
};

/**
 Joins a trial and a card to how the card did, without the trial's text.

 @param cardId - card seated

 @param trial - trial it was seated on

 @param reading - how the trial read

 @param stops - how each ask of the trial ended, in the order asked

 @returns Row as kept, which stays readable if a later row interrupts the run

 @example
 ```ts
 const row = editorFidelityRow({ cardId, trial, reading, stops: ['ok',], },);
 ```
 */
export function editorFidelityRow(
  {
    cardId,
    trial,
    reading,
    stops = [],
    pass,
  }: {
    readonly cardId: RosterModelId;
    readonly trial: EditorFidelityTrial;
    readonly reading: EditorTrialReading;
    readonly stops?: readonly AskEnd[];
    readonly pass?: number;
  },
): EditorFidelityRow {
  return {
    cardId,
    trialId: trial.trialId,
    referenceId: trial.spec
      .id,
    entryId: trial.spec
      .entryId,
    damageKind: trial.damageKind,
    changedChars: trial.changedChars,
    damageDetail: trial.damageDetail,
    reading,
    stops,
    // The member stays absent for the first pass, so a row of one pass is what it always was.
    ...((pass === undefined) ? {} : { pass, }),
  };
}

/**
 How a set of trials read, counted: the counts the card's tally keeps pooled and for each kind of defect.

 @example
 ```ts
 const counts: EditorKindTally = { trials: 4, exact: 1, equivalent: 0, defectOnly: 1, collateral: 1, missed: 1, refused: 0, unusable: 1, };
 ```
 */
export type EditorKindTally = {
  /**
   Trials counted.
   */
  readonly trials: number;

  /**
   Fixes that left the patched text equal to the reference.
   */
  readonly exact: number;

  /**
   Fixes that differ from the reference only in whitespace.
   */
  readonly equivalent: number;

  /**
   Replies that removed the defect from the region and changed something else too. They are not a clean fix and are
   not counted in the headline.
   */
  readonly defectOnly: number;

  /**
   Answered trials where something outside the defect changed, fixed or not.
   */
  readonly collateral: number;

  /**
   Answered trials where the defect was not fixed.
   */
  readonly missed: number;

  /**
   Trials where the card was heard and proposed no edit.
   */
  readonly refused: number;

  /**
   Trials with no usable reply.
   */
  readonly unusable: number;
};

/**
 How one card's trials read, counted, pooled and for each kind of defect.

 @example
 ```ts
 const tally: EditorCardTally = editorTallyOf({ cardId, rows, },);
 ```
 */
export type EditorCardTally = EditorKindTally & {
  /**
   Card counted.
   */
  readonly cardId: RosterModelId;

  /**
   The headline: trials whose defect was fixed cleanly, exact plus equivalent. Defect-only replies are not in it.
   */
  readonly fixed: number;

  /**
   The same counts for the trials of each kind of defect.
   */
  readonly byKind: Readonly<Record<EditorDamageKind, EditorKindTally>>;
};

/**
 How many of the rows had one fix kind, among the rows of one card.

 @param rows - rows counted

 @param fixed - fix kind counted

 @returns Rows answered with that fix kind

 @example
 ```ts
 const exact = countFixed({ rows, fixed: 'exact', },);
 ```
 */
function countFixed(
  {
    rows,
    fixed,
  }: {
    readonly rows: readonly EditorFidelityRow[];
    readonly fixed: 'exact' | 'equivalent' | 'defect-only' | 'none';
  },
): number {
  return rows.filter(function has(row,): boolean {
    return (row.reading
      .kind
      === 'answered') && (row.reading
        .score
        .fixed
        === fixed);
  },)
    .length;
}

/**
 Counts each way a set of trials read.

 @param rows - rows counted, all of one card

 @returns Counts; zero trials for no rows

 @example
 ```ts
 const counts = kindTallyOf({ rows, },);
 ```
 */
function kindTallyOf({ rows, }: { readonly rows: readonly EditorFidelityRow[]; },): EditorKindTally {
  return {
    trials: rows.length,
    exact: countFixed({
      rows,
      fixed: 'exact',
    },),
    equivalent: countFixed({
      rows,
      fixed: 'equivalent',
    },),
    defectOnly: countFixed({
      rows,
      fixed: 'defect-only',
    },),
    collateral: rows.filter(function touched(row,): boolean {
      return (row.reading
        .kind
        === 'answered')
        && row.reading
          .score
          .collateral;
    },)
      .length,
    missed: countFixed({
      rows,
      fixed: 'none',
    },),
    refused: rows.filter(function isRefused(row,): boolean {
      return row.reading
        .kind
        === 'refused';
    },)
      .length,
    unusable: rows.filter(function isUnusable(row,): boolean {
      return row.reading
        .kind
        === 'unusable';
    },)
      .length,
  };
}

/**
 Counts each way one card's trials read.

 @param cardId - card counted

 @param rows - every row of the run, of every card

 @returns Tally; zero trials for a card with no rows

 @example
 ```ts
 const tally = editorTallyOf({ cardId, rows, },);
 ```
 */
export function editorTallyOf(
  {
    cardId,
    rows,
  }: {
    readonly cardId: RosterModelId;
    readonly rows: readonly EditorFidelityRow[];
  },
): EditorCardTally {
  /**
   The card's own rows.
   */
  const own = rows.filter(function isOwn(row,): boolean {
    return row.cardId === cardId;
  },);
  /**
   The card's counts over every kind of defect.
   */
  const pooled = kindTallyOf({ rows: own, },);
  return {
    cardId,
    ...pooled,
    fixed: pooled.exact + pooled.equivalent,
    byKind: {
      insertion: kindTallyOf({
        rows: own.filter(function isInsertion(row,): boolean {
          return row.damageKind === 'insertion';
        },),
      },),
      alteration: kindTallyOf({
        rows: own.filter(function isAlteration(row,): boolean {
          return row.damageKind === 'alteration';
        },),
      },),
    },
  };
}

/**
 Says one set of counts, the clean fix first and the defect-only count outside it.

 @param counts - counts said

 @returns Counts as the lines write them

 @example
 ```ts
 const said = countsSaid({ counts, },);
 ```
 */
function countsSaid({ counts, }: { readonly counts: EditorKindTally; },): string {
  return `clean fix ${String(counts.exact + counts.equivalent,)} of ${String(counts.trials,)} (exact ${
    String(counts.exact,)
  }, equivalent ${String(counts.equivalent,)}), defect only ${String(counts.defectOnly,)}, collateral ${
    String(counts.collateral,)
  }, missed ${String(counts.missed,)}, refused ${String(counts.refused,)}, unusable ${String(counts.unusable,)}`;
}

/**
 The line that reports a card's tally, pooled over every kind of defect.

 @param tally - card's tally

 @returns Line, without a newline

 @example
 ```ts
 log.info(editorTallyLine({ tally, },),);
 ```
 */
export function editorTallyLine({ tally, }: { readonly tally: EditorCardTally; },): string {
  return `EDITOR ${tally.cardId}: ${countsSaid({ counts: tally, },)}`;
}

/**
 The lines that report a card's tally for each kind of defect, insertion first.

 @param tally - card's tally

 @returns One line for each kind, without newlines

 @example
 ```ts
 for (const said of editorKindLines({ tally, },))
   log.info(said,);
 ```
 */
export function editorKindLines({ tally, }: { readonly tally: EditorCardTally; },): readonly string[] {
  /**
   The counts of each kind.
   */
  const {
    insertion,
    alteration,
  } = tally.byKind;
  return [
    `EDITOR ${tally.cardId} insertion: ${countsSaid({ counts: insertion, },)}`,
    `EDITOR ${tally.cardId} alteration: ${countsSaid({ counts: alteration, },)}`,
  ];
}

/**
 The line that reports the replies that removed the defect and changed something else, which the headline leaves out.

 @param tally - card's tally

 @returns Line, without a newline

 @example
 ```ts
 log.info(editorDefectOnlyLine({ tally, },),);
 ```
 */
export function editorDefectOnlyLine({ tally, }: { readonly tally: EditorCardTally; },): string {
  /**
   The counts of each kind.
   */
  const {
    insertion,
    alteration,
  } = tally.byKind;
  return `EDITOR ${tally.cardId} defect only, not in the clean fix count: ${String(tally.defectOnly,)} of ${
    String(tally.trials,)
  } (insertion ${String(insertion.defectOnly,)}, alteration ${String(alteration.defectOnly,)})`;
}

/**
 The rule the arms are read by, stated before any arm asks: which counts are compared, what difference counts, what a
 tie keeps and what the insertion trials measure. The text is `decisionRuleText`, the one the read step applies.

 @returns Line, without a newline

 @example
 ```ts
 log.info(editorDecisionRuleLine(),);
 ```
 */
export function editorDecisionRuleLine(): string {
  return `RULE ${decisionRuleText()}`;
}

/**
 One planned row as the durable plan lists it.

 @example
 ```ts
 const row: EditorPlannedRow = { position: 0, cardId, trialId: 'tabby-reference/insertion', damageKind: 'insertion', };
 ```
 */
export type EditorPlannedRow = {
  /**
   Where the row sits among the planned ones, from zero.
   */
  readonly position: number;

  /**
   Card the row seats.
   */
  readonly cardId: RosterModelId;

  /**
   Trial the card is seated on.
   */
  readonly trialId: string;

  /**
   Defect the trial carries.
   */
  readonly damageKind: EditorDamageKind;

  /**
   Which pass over the pool the row belongs to, from zero; absent for the first pass.
   */
  readonly pass?: number;
};

/**
 What a run is and what it is about to attempt, written before any model is asked so an interrupted run cannot be
 mistaken for a finished one.

 @example
 ```ts
 const plan: EditorFidelityPlan = editorFidelityPlan({ startedAt, pipelineDigest, runnerClosure, referenceManifestDigest, specs, cardIds, trials, planned, ceilings, },);
 ```
 */
export type EditorFidelityPlan = {
  /**
   Always `planned`: a completed run is a different file.
   */
  readonly status: 'planned';

  /**
   When this invocation began.
   */
  readonly startedAt: string;

  /**
   Digest of the built output that ran.
   */
  readonly pipelineDigest: string;

  /**
   Chunks the executing entry imports.
   */
  readonly runnerClosure: RunnerClosure;

  /**
   Digest of the reviewed manifest this run used.
   */
  readonly referenceManifestDigest: string;

  /**
   The reviewed specifications themselves.
   */
  readonly referenceManifest: readonly FidelityReferenceSpec[];

  /**
   Cards the request seated, each alone in its own gather.
   */
  readonly requestedCards: readonly RosterModelId[];

  /**
   Whether the plan holds every trial the request materialized.
   */
  readonly completeRequestedMatrix: boolean;

  /**
   Rows the whole request holds: every trial for every card.
   */
  readonly fullMatrixRows: number;

  /**
   Rows this run will attempt.
   */
  readonly plannedRows: number;

  /**
   The cost ceilings the run printed and was held to.
   */
  readonly ceilings: readonly EditorCeiling[];

  /**
   The attempted rows, in order.
   */
  readonly rows: readonly EditorPlannedRow[];
};

/**
 One card and trial to attempt.

 @example
 ```ts
 const pair: EditorPair = { cardId, trial, };
 ```
 */
export type EditorPair = {
  /**
   Card seated.
   */
  readonly cardId: RosterModelId;

  /**
   Trial it is seated on.
   */
  readonly trial: EditorFidelityTrial;

  /**
   Which pass over the pool the ask belongs to, from zero; absent for the first pass.
   */
  readonly pass?: number;
};

/**
 Builds the durable plan.

 @param startedAt - when this invocation began

 @param pipelineDigest - digest of the built output that ran

 @param runnerClosure - chunks the executing entry imports

 @param referenceManifestDigest - digest of the reviewed manifest used

 @param specs - reviewed specifications the request selected

 @param cardIds - cards the request seats

 @param trials - every trial the request materialized

 @param pairs - card and trial pairs this run attempts, in order

 @param passes - how many times the run is asked to walk the pool, one when it is not said

 @param ceilings - ceilings printed before the run

 @returns Plan, ready to serialize

 @example
 ```ts
 const plan = editorFidelityPlan({ startedAt, pipelineDigest, runnerClosure, referenceManifestDigest, specs, cardIds, trials, pairs, passes: 2, ceilings, },);
 ```
 */
export function editorFidelityPlan(
  {
    startedAt,
    pipelineDigest,
    runnerClosure,
    referenceManifestDigest,
    specs,
    cardIds,
    trials,
    pairs,
    passes = 1,
    ceilings,
  }: {
    readonly startedAt: string;
    readonly pipelineDigest: string;
    readonly runnerClosure: RunnerClosure;
    readonly referenceManifestDigest: string;
    readonly specs: readonly FidelityReferenceSpec[];
    readonly cardIds: readonly RosterModelId[];
    readonly trials: readonly EditorFidelityTrial[];
    readonly pairs: readonly EditorPair[];
    readonly passes?: number;
    readonly ceilings: readonly EditorCeiling[];
  },
): EditorFidelityPlan {
  return {
    status: 'planned',
    startedAt,
    pipelineDigest,
    runnerClosure,
    referenceManifestDigest,
    referenceManifest: specs,
    requestedCards: cardIds,
    completeRequestedMatrix: pairs.length === (trials.length * (passes * cardIds.length)),
    fullMatrixRows: trials.length * (passes * cardIds.length),
    plannedRows: pairs.length,
    ceilings,
    rows: pairs.map(function plannedRowOf(
      pair,
      position,
    ): EditorPlannedRow {
      return {
        position,
        cardId: pair.cardId,
        trialId: pair.trial
          .trialId,
        damageKind: pair.trial
          .damageKind,
        // The member stays absent for the first pass, so a plan of one pass reads as it always did.
        ...((pair.pass === undefined) ? {} : { pass: pair.pass, }),
      };
    },),
  };
}

/**
 Everything a finished run is kept as, apart from the clock and the build's identity, which the driver supplies.

 @param plan - durable plan written before the run

 @param rows - every completed row

 @param tallies - tally of each card

 @param finishedAt - when the run ended

 @param promptPayloadDir - where completed payloads were kept

 @param corpusPin - corpus commit the references were read at

 @param entriesRequested - entry filter as typed, empty for every entry

 @param trialCap - cap on attempted trials

 @param damageKinds - defect families requested

 @param maxUsd - amount the run was held to

 @returns Record `persistProbeRun` writes

 @example
 ```ts
 const run = editorKeptRun({ plan, rows, tallies, finishedAt, promptPayloadDir, corpusPin, entriesRequested, trialCap, damageKinds, maxUsd, },);
 ```
 */
export function editorKeptRun(
  {
    plan,
    rows,
    tallies,
    finishedAt,
    promptPayloadDir,
    corpusPin,
    entriesRequested,
    trialCap,
    damageKinds,
    maxUsd,
  }: {
    readonly plan: EditorFidelityPlan;
    readonly rows: readonly EditorFidelityRow[];
    readonly tallies: readonly EditorCardTally[];
    readonly finishedAt: string;
    readonly promptPayloadDir: string;
    readonly corpusPin: string;
    readonly entriesRequested: readonly string[];
    readonly trialCap: number;
    readonly damageKinds: readonly EditorDamageKind[];
    readonly maxUsd: number;
  },
): ProbeRun {
  return {
    startedAt: plan.startedAt,
    finishedAt,
    pipelineDigest: plan.pipelineDigest,
    runnerClosure: plan.runnerClosure,
    roster: plan.requestedCards,
    subject: {
      status: 'completed',
      completeRequestedMatrix: plan.completeRequestedMatrix,
      fullMatrixRows: plan.fullMatrixRows,
      plannedRows: plan.plannedRows,
      completedRows: rows.length,
      referenceManifestDigest: plan.referenceManifestDigest,
      promptPayloadDir,
      corpusPin,
      referenceManifest: plan.referenceManifest,
      entriesRequested,
      trialCap,
      damageKinds,
      maxUsd,
      ceilings: plan.ceilings,
      tallies,
    },
    rows,
  };
}

//endregion Editor fidelity plan
