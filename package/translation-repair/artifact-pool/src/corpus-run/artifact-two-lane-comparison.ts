import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import type {
  ArtifactComparisonRow,
  ArtifactDecisionComparison,
  ArtifactDeliveryRow,
  ArtifactLaneRelation,
  ArtifactSliceOutcome,
} from './artifact-two-lane-vocabulary.ts';

import { ArtifactComparisonError, } from './artifact-two-lane-comparison-fault.ts';
import { comparisonRowDifferences, } from './artifact-two-lane-row-equality.ts';

//region Artifact version 2 comparison
// How version 2 decides what a comparison row SAYS, frozen the way its
// vocabulary is.
//
// The vocabulary froze the words and this freezes the rules, which are two
// different things and only one of them was done. `buildSettledTwoLaneArtifact`
// derived its persisted comparison by calling the LIVE comparator and
// projecting the answer, so a later change to how a verdict is decided would
// have silently reinterpreted every artifact already on disk: nothing in the
// file records which rules produced it, and a reader recomputing with the same
// changed comparator would agree with itself while both disagreed with what the
// artifact meant when it was written.
//
// So the rules live here, under a version 2 name, over version 2 ROWS. Changing
// them means editing a file whose name says which generation it defines, which
// is as loud as the union snapshot next door. The live comparator keeps its own
// job: it checks two ledgers against each other and refuses pairs that cannot
// be compared at all, which is a runtime invariant rather than a persisted
// meaning.
//
// DELIBERATELY DUPLICATED, therefore, and the duplication is checked rather
// than trusted: `buildSettledTwoLaneArtifact` runs both and refuses a disagreement,
// so the day the live rules move, a corpus pass stops rather than quietly
// writing artifacts that mean something new under an unchanged version number.

/**
 Names how one slice's two carried wordings relate, as version 2 decides it.

 @param repairText - wording the repair document carries

 @param translateText - wording the translate document carries

 @param incumbentKind - whether the archive holds any wording here, which is
 what separates its wording standing from a passage still missing

 @param incumbentText - archive wording both fall back to

 @returns Verdict for this slice

 @example
 ```ts
 const verdict = judgeTwoLaneSlice({ repairText, translateText, incumbentKind, incumbentText, },);
 ```
 */
function judgeTwoLaneSlice(
  {
    repairText,
    translateText,
    incumbentKind,
    incumbentText,
  }: {
    readonly repairText: string;
    readonly translateText: string;
    readonly incumbentKind: 'present' | 'absent';
    readonly incumbentText: string;
  },
): ArtifactLaneRelation {
  /**
   Whether the repair document moved off the archive wording.
   */
  const repairMoved = repairText !== incumbentText;

  /**
   Whether the translate document did.
   */
  const translateMoved = translateText !== incumbentText;
  if (repairMoved && translateMoved)
    return (repairText === translateText) ? 'both-agree' : 'both-differ';
  if (repairMoved)
    return 'repair-only';
  if (translateMoved)
    return 'translate-only';

  // NEITHER DOCUMENT MOVED, which means the archive's wording stands only where
  // the archive HAS wording. At an anchor it means the passage is still
  // missing, and reporting that as the archive standing tells a reader a
  // translation is being kept where none has ever existed.
  return (incumbentKind === 'absent') ? 'gap-remains' : 'archive-stands';
}

/**
 Compares what the two lanes DECIDED, as version 2 records it.

 @param repair - what the repair lane did about this slice

 @param translate - what the translate lane did

 @returns Whether the two decisions were comparable, and how they came out

 @example
 ```ts
 const decisions = compareTwoLaneDecisions({ repair, translate, },);
 ```
 */
function compareTwoLaneDecisions(
  {
    repair,
    translate,
  }: {
    readonly repair: ArtifactSliceOutcome;
    readonly translate: ArtifactSliceOutcome;
  },
): ArtifactDecisionComparison {
  /**
   Lanes with no wording to compare, collected rather than returned at the
   first one found: a slice neither lane decided is a fact about both.
   */
  const undecidedLanes = [
    ...(repair.kind === 'decided') ? [] : ['repair',] as const,
    ...(translate.kind === 'decided') ? [] : ['translate',] as const,
  ];
  if ((repair.kind !== 'decided') || (translate.kind !== 'decided')) {
    return {
      kind: 'not-comparable',
      undecidedLanes,
    };
  }
  return {
    kind: 'comparable',
    verdict: (repair.acceptedText === translate.acceptedText) ? 'same' : 'different',
  };
}

/**
 Derives the whole comparison from two version 2 delivery ledgers.

 OVER PROJECTED ROWS rather than live records, so what it reads is exactly
 what the artifact carries: a reader holding only the file can run this and
 get the same answer the writer did, which is what makes the persisted copy
 checkable rather than merely present.

 @param repair - repair lane's rows, in document order

 @param translate - translate lane's rows, in the same order

 @returns One row per slice, in the order the repair ledger reports them

 @throws {@link ArtifactComparisonError} when the two ledgers differ in
 length or disagree at any position about which slice it is, which original
 it carries, or what the archive holds there

 @example
 ```ts
 const rows = compareLanes({ repair: lanes.repair.delivery, translate: lanes.translate.delivery, },);
 ```
 */
export function compareLanes(
  {
    repair,
    translate,
  }: {
    readonly repair: readonly ArtifactDeliveryRow[];
    readonly translate: readonly ArtifactDeliveryRow[];
  },
): readonly ArtifactComparisonRow[] {
  if (repair.length !== translate.length) {
    throw new ArtifactComparisonError({
      fault: {
        kind: 'ledger-lengths',
        repairRows: repair.length,
        translateRows: translate.length,
      },
    },);
  }
  return repair.map(function toRow(
    mine,
    position,
  ): ArtifactComparisonRow {
    /**
     Row the other ledger holds at this POSITION, which is where a ledger
     built over the same preparation holds the same slice; the two hold the
     same number of rows, so there is one.
     */
    const theirs = nonNullishOrThrow(translate[position],);
    if (theirs.sliceIndex !== mine.sliceIndex) {
      throw new ArtifactComparisonError({
        fault: {
          kind: 'slice-positions',
          position,
          repairSliceIndex: mine.sliceIndex,
          translateSliceIndex: theirs.sliceIndex,
        },
      },);
    }
    if (theirs.sourceText !== mine.sourceText) {
      throw new ArtifactComparisonError({
        fault: {
          kind: 'different-originals',
          sliceIndex: mine.sliceIndex,
        },
      },);
    }
    if (theirs.incumbentText !== mine.incumbentText) {
      throw new ArtifactComparisonError({
        fault: {
          kind: 'different-archive-wordings',
          sliceIndex: mine.sliceIndex,
        },
      },);
    }

    // TEXT IS NOT ENOUGH. A blank content slice and a place the archive never
    // translated both carry the empty string, so equal text leaves exactly the
    // pair this comparison must not confuse still equal.
    if (theirs.incumbentKind !== mine.incumbentKind) {
      throw new ArtifactComparisonError({
        fault: {
          kind: 'different-archive-kinds',
          sliceIndex: mine.sliceIndex,
          repairKind: mine.incumbentKind,
          translateKind: theirs.incumbentKind,
        },
      },);
    }
    return {
      sliceIndex: mine.sliceIndex,
      incumbentKind: mine.incumbentKind,
      incumbentText: mine.incumbentText,
      repairText: mine.shippedText,
      translateText: theirs.shippedText,
      laneRelation: judgeTwoLaneSlice({
        repairText: mine.shippedText,
        translateText: theirs.shippedText,
        incumbentKind: mine.incumbentKind,
        incumbentText: mine.incumbentText,
      },),
      repairOutcome: mine.outcome,
      translateOutcome: theirs.outcome,
      decisionComparison: compareTwoLaneDecisions({
        repair: mine.outcome,
        translate: theirs.outcome,
      },),
      repairDelivery: mine.delivery,
      translateDelivery: theirs.delivery,
    };
  },);
}

/**
 Refuses a run whose frozen and live comparisons disagree.

 The one place the duplication is worth its cost. Version 2's rules are frozen
 here and the pipeline's live rules keep evolving; while they agree, either
 derivation answers for the other, and the moment they stop, an artifact
 written under the live rules would mean something the version number does not
 say. A stopped pass is the cheap outcome: whoever changed the rules decides
 whether version 2 changed with them, which is a version 3, or whether the
 change was a defect.

 COMPARED FIELD BY FIELD through {@link comparisonRowDifferences}, so key order
 does not decide it: two rows carrying the same values in a different order
 are the same row, and the reader comparing a recorded row read off disk
 against a derived one needs exactly that reading.

 @param frozen - rows version 2's own rules produced

 @param live - rows the pipeline's comparator produced, projected into version
 2's vocabulary

 @throws {@link ArtifactComparisonError} at the first row they disagree on

 @example
 ```ts
 assertDerivationsAgree({ frozen, live, },);
 ```
 */
export function assertDerivationsAgree(
  {
    frozen,
    live,
  }: {
    readonly frozen: readonly ArtifactComparisonRow[];
    readonly live: readonly ArtifactComparisonRow[];
  },
): void {
  if (frozen.length !== live.length) {
    throw new ArtifactComparisonError({
      fault: {
        kind: 'derivation-lengths',
        frozenRows: frozen.length,
        liveRows: live.length,
      },
    },);
  }
  for (const [
    position,
    row,
  ] of frozen.entries()) {
    /**
     What the live comparator said about the same position; the two hold the
     same number of rows, so there is one.
     */
    const theirs = nonNullishOrThrow(live[position],);

    /**
     Fields the two derivations disagree on, named and never quoted: the rows
     carry slice text, and this message reaches the pass's stdout.
     */
    const differing = comparisonRowDifferences({
      left: row,
      right: theirs,
    },);
    if (differing.length > 0) {
      throw new ArtifactComparisonError({
        fault: {
          kind: 'derivations-differ',
          sliceIndex: row.sliceIndex,
          fields: differing,
        },
      },);
    }
  }
}

//endregion Artifact version 2 comparison
