import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import type {
  CarriedInsertion,
  FoldedInsertion,
  InsertionAdmission,
  PreparedDocumentPair,
} from '@monochromatic-dev/translation-repair-corpus-source/ts';
import {
  type AnchorTarget,
  parseDocument,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { decideFold, } from './insertion-carried-decide.ts';
import {
  shiftedFinding,
  shiftSlices,
  widenSource,
} from './insertion-carried-shift.ts';

//region Carried insertion fold
// THE PASSAGE BELONGS WITH ITS RENDERING. A carried insertion is a source-only
// slice whose English the roster found already on the page; where that
// English sits inside the archive span the pairing gave the NEIGHBOURING
// slice, the archive merged two originals into one rendering and the pairing
// left one of them unpaired. The lanes then write the neighbour from its own
// source alone: on mikaela12 (2026-09-24) the translate lane's judges rightly
// preferred the candidate without "the surrounding passage", the contest took
// it, and the guard at publish stopped the entry with the carried sentence
// gone (class one hundred ten). Folding the carried source into the carrier
// makes the pair honest, so both lanes render the passage as part of the
// slice that carries it; the folded slice stays an insertion the lanes skip,
// and leaves the guard, whose evidence the lanes may now reword.
//
// THE EVIDENCE MAY RUN INTO BOTH NEIGHBOURS. On mikaela13 (2026-09-24) the
// archive rendered the carried passage across the two paired slices around
// it (the HRT sentences inside the earlier span, "originally just a typical
// thing" as the later span's own line), the one ballot quoted both, and a
// fold that wanted one holder stood aside while the class one hundred seven
// stand-ins on both neighbours dropped the passage (class one hundred
// eleven). So the fold places the evidence block by block: every block must
// sit in a paired slice next to the carried one, and the carrier is the
// neighbour holding the larger share of the quoted text (the earlier on a
// tie), whose source must abut the carried source across blank space alone.
// The other neighbour keeps its own source; the words of the passage it
// rendered are the carrier's to write now.
//
// THE CARRIER'S SPAN MAY RENDER THE PASSAGE ALONE. On TianqiChen66611
// (2026-09-26) the pairing left one off and the plain fold made the carrier
// render its own source twice over; insertion-carried-shift.ts moves the
// carrier's own source to its far neighbour instead (class one hundred
// seventy-nine).
//
// EVERY STAND-ASIDE SAYS WHY, so a log reader can tell a passage the archive
// really scattered from one the fold could not place.

/**
 Finding prefix for a passage folded into its carrier.
 */
export const CARRIED_FOLDED_FINDING = 'insertion-carried-folded';

/**
 Slices and records as the folds so far left them.
 */
type FoldState = {
  /**
   Slices with every carrier folded so far widened.
   */
  readonly slices: readonly ChunkPair[];

  /**
   Passages still carried, in admission order.
   */
  readonly kept: readonly CarriedInsertion[];

  /**
   Passages folded into a carrier.
   */
  readonly folded: readonly FoldedInsertion[];

  /**
   One finding per fold.
   */
  readonly findings: readonly string[];

  /**
   One line per passage that stayed carried, naming why.
   */
  readonly asides: readonly string[];
};

/**
 One pass over the passages still carried: each folded in turn over the
 slices the earlier folds left, so two passages folding into one carrier both
 widen it. Stand-asides of earlier passes are dropped; the pass rewrites them.

 @param start - slices and records as the passes so far left them

 @param pending - passages still carried, in admission order

 @param sourceText - whole original

 @param target - archive parsed for anchoring

 @returns State after the pass

 @example
 ```ts
 const after = foldPass({ start: state, pending: state.kept, sourceText, target, },);
 ```
 */
function foldPass(
  {
    start,
    pending,
    sourceText,
    target,
  }: {
    readonly start: FoldState;
    readonly pending: readonly CarriedInsertion[];
    readonly sourceText: string;
    readonly target: AnchorTarget;
  },
): FoldState {
  // ONE PASS THAT APPENDS: each passage folds over the slices the passages
  // before it left, and copying the four records at every passage bought
  // nothing (ledger B70).
  /**
   Slices with every carrier folded so far widened.
   */
  const now = { slices: start.slices, };
  /**
   Passages still carried, in admission order.
   */
  const kept: CarriedInsertion[] = [];
  /**
   Passages folded into a carrier, the earlier passes' first.
   */
  const folded = [...start.folded,];
  /**
   One finding per fold, the earlier passes' first.
   */
  const findings = [...start.findings,];
  /**
   One line per passage that stayed carried in this pass, naming why.
   */
  const asides: string[] = [];
  for (const candidate of pending) {
    /**
     Whether and where this passage folds.
     */
    const decision = decideFold({
      slices: now.slices,
      sourceText,
      target,
      candidate,
    },);
    if (decision.kind === 'aside') {
      kept.push(candidate,);
      asides.push(`slice ${String(candidate.sliceIndex,)} stays carried: ${decision.reason}`,);
      continue;
    }
    /**
     The carrier as the earlier folds left it, and the carried slice.
     */
    const {
      carrier,
      carried,
    } = decision;
    /**
     Stable index the lanes report the carrier under.
     */
    const carrierSliceIndex = carrier.slice
      .target
      .sliceIndex;
    // ON A SHIFT the carrier holds the carried source and its own joins the
    // receiver (class one hundred seventy-nine); on a plain fold the carrier
    // widens over both.
    now.slices = (decision.kind === 'shift')
      ? shiftSlices({
        sourceText,
        slices: now.slices,
        carrier,
        receiver: decision.receiver,
        carried,
      },)
      : now.slices
        .map(function replaceCarrier(
          slice,
          position,
        ): ChunkPair {
          return (position === carrier.position)
            ? widenSource({
              sourceText,
              widened: carrier.slice,
              absorbed: carried,
            },)
            : slice;
        },);
    folded.push({
      position: candidate.position,
      sliceIndex: candidate.sliceIndex,
      carrierSliceIndex,
    },);
    findings.push(`${CARRIED_FOLDED_FINDING} (slice ${String(candidate.sliceIndex,)} into slice ${
      String(carrierSliceIndex,)
    })`,);
    // The shift's own finding follows the fold's; a plain fold has none.
    if (decision.kind === 'shift') {
      findings.push(shiftedFinding({
        carrierSliceIndex,
        receiverSliceIndex: decision.receiver
          .slice
          .target
          .sliceIndex,
        carriedSliceIndex: candidate.sliceIndex,
      },),);
    }
  }
  return {
    slices: now.slices,
    kept,
    folded,
    findings,
    asides,
  };
}

/**
 Folds every unambiguously placed carried passage into its carrier.

 @param prepared - preparation both lanes run over

 @param admission - insertion admission as read

 @returns Preparation with carriers widened, the admission with folded
 passages moved out of `carried`, one finding per fold and one line per
 passage that stayed carried, naming why

 @example
 ```ts
 const folded = foldCarriedInsertions({ prepared, admission, },);
 ```
 */
export function foldCarriedInsertions(
  {
    prepared,
    admission,
  }: {
    readonly prepared: PreparedDocumentPair;
    readonly admission: InsertionAdmission;
  },
): {
  readonly prepared: PreparedDocumentPair;
  readonly admission: InsertionAdmission;
  readonly findings: readonly string[];
  readonly asides: readonly string[];
} {
  /**
   Passages the roster found carried.
   */
  const carried = admission.carried ?? [];
  if (carried.length === 0) {
    return {
      prepared,
      admission,
      findings: [],
      asides: [],
    };
  }
  /**
   Archive parsed once for every region's anchoring.
   */
  const target = parseDocument({ text: prepared.targetText, },);
  /**
   Passes run until nothing more folds, at most one per carried passage: a
   passage refused for a carried neighbour in the way folds once that
   neighbour has folded (mikaela14). A pass that folds nothing repeats the
   same result, so the bound is the passages themselves.
   */
  const outcome = carried.reduce(
    function passOnce(progress: FoldState,): FoldState {
      /**
       Passages still carried before this pass.
       */
      const pendingCount = progress.kept
        .length;
      if (pendingCount === 0)
        return progress;
      return foldPass({
        start: progress,
        pending: progress.kept,
        sourceText: prepared.sourceText,
        target,
      },);
    },
    {
      slices: prepared.slices,
      kept: carried,
      folded: [],
      findings: [],
      asides: [],
    },
  );
  return {
    prepared: {
      ...prepared,
      slices: outcome.slices,
    },
    admission: {
      ...admission,
      carried: outcome.kept,
      folded: outcome.folded,
      findings: [
        ...admission.findings,
        ...outcome.findings,
      ],
    },
    findings: outcome.findings,
    asides: outcome.asides,
  };
}

//endregion Carried insertion fold
