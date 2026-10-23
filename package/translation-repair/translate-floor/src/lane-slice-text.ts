import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import { isInsertionChunk, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { LaneSliceCoverageError, } from './lane-slice-coverage-error.ts';
import { validateNamedSets, } from './lane-slice-sets.ts';

//region Lane slice text
// What one lane DECIDED for each slice, beside what the archive already said.
//
// Both lanes already report which slices the returned document carries a change
// for. That names the slices and says nothing about the wording, so the question
// the two-lane design exists to answer, whether repair and translate produce the
// SAME English where they both touch a slice, cannot be asked of either result.
//
// Deliberately ACCEPTED-SIDE ONLY. Whether a slice shipped is a fact about one
// document, decided by an assembly guard that reads the whole of it, and the
// same slice can ship in one run and be withdrawn in the next when a
// neighbouring replacement changes. Membership in a result's shipped and
// withdrawn index sets is that fact; repeating it here would put a per-run
// verdict on a per-slice record, which is the defect class this file exists
// downstream of.

/**
 What became of one slice inside a lane.

 A DISCRIMINATED UNION rather than an optional wording, since 2026-08-16. The
 optional field had to mean everything that was not a decision, and four
 different things are not a decision. Two of them, a slice nobody reached and
 a slice everybody failed at, were read as the same fact by every consumer,
 and the second was reported to graders as the archive's wording standing.

 Named for the OUTCOME rather than for reach on purpose: three of the four
 mean the lane reached the slice, so a field called `reach` invites the next
 reader to write `reach === 'decided'` and rebuild the defect this replaced.

 @example
 ```ts
 const outcome: LaneSliceOutcome = { kind: 'decided', acceptedText: 'The cat is napping.', };
 ```
 */
export type LaneSliceOutcome = {
  /**
   Lane produced a wording for this slice.

   Its text equals the incumbent when the lane examined the slice and chose to
   leave it alone, which is a decision rather than an absence.
   */
  readonly kind: 'decided';

  /**
   Wording the lane decided on, whether or not the document carries it.
   */
  readonly acceptedText: string;
} | {
  /**
   Lane never reached this slice.

   The repair lane's whole-document block produces exactly this: it stops at
   the earliest crossing, so every later slice went unexamined.
   */
  readonly kind: 'not-evaluated';
} | {
  /**
   Lane reached the slice, produced nothing, and had nothing to fall back on
   because the archive holds no wording here either.

   The passage is MISSING, which is a different fact from every other member:
   `decided` would claim a wording, `not-evaluated` would claim nobody looked,
   and `incumbent-fallback` would claim something stands here.
   */
  readonly kind: 'unfilled';

  /**
   Present when the lane never bought a candidate for the passage because the bench that decides it read below its
   floor, absent when a panel was asked and declined; the artifact records it from generation twenty-one.
   */
  readonly benchBelowFloor?: true;
} | {
  /**
   Lane reached the slice, produced nothing, and the archive's own wording
   therefore stands BY DEFAULT rather than by anyone's choice.

   The translate lane produces this whenever no translator was heard. It was
   recorded as a decision equal to the incumbent until 2026-08-16, so a stage
   that heard nobody read exactly like a panel that examined the slice and
   kept the archive, which is the same defect the window trial had to fix for
   judges.
   */
  readonly kind: 'incumbent-fallback';
} | {
  /**
   Lane reached the slice and the work it does has no input here at all.

   The repair lane at a passage the archive never translated is this and
   nothing else: it mends existing English, and there is none, so it never had
   an opinion to record. It reported `decided` with the empty string until
   2026-08-16, which made a lane comparison state that the two lanes chose
   DIFFERENT wordings wherever the translate lane filled the passage.

   SEPARATE FROM `unfilled` on purpose, though both leave the passage missing.
   `unfilled` is a lane that tried and produced nothing, which is a rate worth
   measuring; folding this into it would make the repair lane's decline rate
   equal the count of gaps in the archive, a constant of the document that
   measures nothing about the lane.
   */
  readonly kind: 'not-applicable';
};

/**
 One slice's wording as a lane left it.

 @example
 ```ts
 const wording: LaneSliceText = {
   sliceIndex: 3,
   incumbentKind: 'present',
   incumbentText: 'The cat naps.',
   outcome: { kind: 'decided', acceptedText: 'The cat is napping.', },
 };
 ```
 */
export type LaneSliceText = {
  /**
   Global slice index, which is what a comparison joins two lanes on.
   */
  readonly sliceIndex: number;

  /**
   Whether the archive holds any wording at this slice at all.

   A SEPARATE AXIS from the outcome, and not inferable from the text: a
   content slice may legitimately be blank, so an empty
   {@link LaneSliceText.incumbentText} cannot answer this. Without it a reader
   meeting an unchanged slice cannot tell the archive's wording standing from
   a gap left where the archive never had any.
   */
  readonly incumbentKind: 'present' | 'absent';

  /**
   Archive's own English for this slice, before either lane touched it.
   */
  readonly incumbentText: string;

  /**
   What this lane did about the slice.
   */
  readonly outcome: LaneSliceOutcome;
};

/**
 What a builder does about a prepared slice the lane never decided.

 @example
 ```ts
 const undecided: UndecidedSlicePolicy = 'refuse';
 ```
 */
export type UndecidedSlicePolicy =
  /**
   Treat it as a defect, which is right wherever the lane is meant to visit
   every slice: a short list is otherwise read by every later count as a
   smaller document.
   */
  | 'refuse'
  /**
   Record it as unexamined, which is right only where the lane stopped early
   BY DESIGN and says so in its status.
   */
    | 'not-evaluated';

/**
 Names one slice's outcome from what the lane reported about it.

 ORDERED DELIBERATELY: a decision wins over both named sets, since a lane
 naming a slice both decided and unreachable is a contradiction the caller
 already refuses, and reading the sets first here would hide it rather than
 let that check speak.

 @param sliceIndex - slice being named, for the failure message

 @param byIndex - wordings the lane reported, by slice index; every decision
 carries its wording as a string, the empty string included, so a slice the
 map holds is a decided slice

 @param unfilledHere - whether the lane named this slice as reached and
 unfillable

 @param belowFloorHere - whether it was left unfilled for want of a bench, which marks the outcome only where the
 slice is unfilled

 @param unheardHere - whether the lane named it as reached with no voice heard

 @param undecided - what an unnamed gap means for this lane

 @returns Outcome for this slice

 @throws {@link LaneSliceCoverageError} when a gap is left under `refuse`

 @example
 ```ts
 const outcome = outcomeOf({ sliceIndex, byIndex, unfilledHere, belowFloorHere, unheardHere, notApplicableHere, undecided, },);
 ```
 */
function outcomeOf(
  {
    sliceIndex,
    byIndex,
    unfilledHere,
    belowFloorHere,
    unheardHere,
    notApplicableHere,
    undecided,
  }: {
    readonly sliceIndex: number;
    readonly byIndex: ReadonlyMap<number, string>;
    readonly unfilledHere: boolean;
    readonly belowFloorHere: boolean;
    readonly unheardHere: boolean;
    readonly notApplicableHere: boolean;
    readonly undecided: UndecidedSlicePolicy;
  },
): LaneSliceOutcome {
  /**
   Wording the lane decided for this slice, absent where it decided none.
   */
  const acceptedText = byIndex.get(sliceIndex,);
  if (acceptedText !== undefined)
    return {
      kind: 'decided',
      acceptedText,
    };

  // NAMED RATHER THAN INFERRED, and checked before the policy, because a slice
  // the lane reached and could not fill is neither of the two shapes the policy
  // describes: refusing it would fail a run that did nothing wrong, and treating
  // it as an early stop would let every later slice pass unexamined.
  if (unfilledHere) {
    return belowFloorHere
      ? {
        kind: 'unfilled',
        benchBelowFloor: true,
      }
      : { kind: 'unfilled', };
  }
  if (unheardHere)
    return { kind: 'incumbent-fallback', };
  if (notApplicableHere)
    return { kind: 'not-applicable', };
  if (undecided === 'refuse')
    throw new LaneSliceCoverageError({
      fault: {
        kind: 'left-undecided',
        sliceIndex,
      },
    },);
  return { kind: 'not-evaluated', };
}

// Declared in a file of its own since the per-list checks moved to
// `lane-slice-sets.ts`, and re-exported here because every caller of this
// builder catches it by this name.
export { LaneSliceCoverageError, } from './lane-slice-coverage-error.ts';

/**
 Pairs each prepared slice with the wording a lane decided for it.

 Built at the DOCUMENT level rather than stored per slice, so neither lane's
 cache schema has to carry it and a resumed slice cannot serve a stale
 incumbent from a preparation that has since changed.

 @param slices - prepared slice pairs, which supply both the denominator and
 every incumbent

 @param decided - what the lane accepted, keyed by the same global index

 @param undecided - what to do about a prepared slice with no decision;
 `refuse` wherever the lane visits every slice, `not-evaluated` only where it
 stops early by design

 @param unfilledChunkIndices - slices the lane REACHED and could not decide a
 wording for, which is neither an early stop nor a dropped slice; named one by
 one rather than by policy, so every other gap is still refused

 @param belowFloorChunkIndices - slices among them the lane never bought a candidate for because the bench that
 decides them read below its floor; a name here marks the outcome only where the slice is also named unfilled

 @returns One entry per prepared slice, in document order

 @throws LaneSliceCoverageError when a decision names a slice preparation
 never produced, a prepared slice has no decision under `refuse` without being
 named unfilled, or a slice is named unfilled and decided at once

 @example
 ```ts
 const wordings = buildLaneSliceTexts({ slices, decided, undecided: 'refuse', },);
 ```
 */
export function buildLaneSliceTexts(
  {
    slices,
    decided,
    undecided,
    unfilledChunkIndices = [],
    belowFloorChunkIndices = [],
    unheardChunkIndices = [],
    notApplicableChunkIndices = [],
  }: {
    readonly slices: readonly ChunkPair[];
    readonly decided: readonly {
      readonly sliceIndex: number;
      readonly text: string;
    }[];
    readonly undecided: UndecidedSlicePolicy;
    readonly unfilledChunkIndices?: readonly number[];
    readonly belowFloorChunkIndices?: readonly number[];
    readonly unheardChunkIndices?: readonly number[];
    readonly notApplicableChunkIndices?: readonly number[];
  },
): readonly LaneSliceText[] {
  /**
   Wording decided for each slice index.
   */
  const byIndex = new Map(decided.map(function toEntry(one,): [
    number,
    string,
  ] {
    return [
      one.sliceIndex,
      one.text,
    ];
  },),);

  /**
   Indices preparation actually produced, so a decision naming any other one
   is caught rather than dropped by a lookup that finds nothing.
   */
  const prepared = new Set(slices.map(function toIndex(slice,): number {
    return slice.target
      .sliceIndex;
  },),);

  // `byIndex` and `prepared` would both swallow a repeat: the last entry would win and the
  // list would still be the right length, so a decision would be silently
  // reused for one slice and lost for another.
  if (prepared.size !== slices.length)
    throw new LaneSliceCoverageError({
      fault: {
        kind: 'preparation-repeats',
        slices: slices.length,
        distinct: prepared.size,
      },
    },);
  if (byIndex.size !== decided.length)
    throw new LaneSliceCoverageError({
      fault: {
        kind: 'decisions-repeat',
        decisions: decided.length,
        distinct: byIndex.size,
      },
    },);
  for (const one of decided) {
    if (!prepared.has(one.sliceIndex,))
      throw new LaneSliceCoverageError({
        fault: {
          kind: 'decided-unproduced',
          sliceIndex: one.sliceIndex,
        },
      },);
  }

  /**
   Every list the lane named, validated once against the preparation and
   against each other, then read back in the order the sets were given.
   */
  const [
    unfilled = new Set<number>(),
    unheard = new Set<number>(),
    notApplicable = new Set<number>(),
  ] = validateNamedSets({
    slices,
    decidedIndices: new Set(byIndex.keys(),),
    sets: [
      {
        label: 'unfilled',
        indices: unfilledChunkIndices,
        incumbent: 'absent',
      },
      {
        label: 'unheard',
        indices: unheardChunkIndices,
        incumbent: 'present',
      },
      {
        label: 'not-applicable',
        indices: notApplicableChunkIndices,
        incumbent: 'absent',
      },
    ],
  },);

  /**
   Slices left unfilled for want of a bench, to look up by index.
   */
  const belowFloor = new Set(belowFloorChunkIndices,);

  /**
   Whether some earlier slice in document order went undecided.

   `not-evaluated` describes ONE shape and no other: a lane that stopped, so
   an evaluated prefix followed by an unevaluated suffix. Decisions for slices
   0 and 2 with 1 unexamined is not that shape, and accepting it would let a
   dropped slice pass as an early stop.
   */
  const stopped = { already: false, };

  return slices.map(function toWording(slice,): LaneSliceText {
    /**
     This slice's global index.
     */
    const { sliceIndex, } = slice.target;

    /**
     What this lane did about the slice, before the stopped-prefix rule sees
     it.
     */
    const outcome = outcomeOf({
      sliceIndex,
      byIndex,
      unfilledHere: unfilled.has(sliceIndex,),
      belowFloorHere: belowFloor.has(sliceIndex,),
      unheardHere: unheard.has(sliceIndex,),
      notApplicableHere: notApplicable.has(sliceIndex,),
      undecided,
    },);

    // EVERY REACHED OUTCOME, not decisions alone. A lane that stopped cannot
    // report reaching a later slice by any route, and checking only decisions
    // let an unfilled slice sit after an unexamined one, which asserts the lane
    // resumed after stopping.
    if (stopped.already && (outcome.kind !== 'not-evaluated')) {
      throw new LaneSliceCoverageError({
        fault: {
          kind: 'reached-after-stop',
          sliceIndex,
        },
      },);
    }
    if (outcome.kind === 'not-evaluated')
      stopped.already = true;

    return {
      sliceIndex,
      // READ OFF THE PREPARED CHUNK, which is the only thing that knows. An
      // anchor names a place the archive never translated; a content slice that
      // happens to be blank is wording the archive does hold.
      incumbentKind: isInsertionChunk(slice.target,) ? 'absent' : 'present',
      incumbentText: slice.target
        .text,
      outcome,
    };
  },);
}

//endregion Lane slice text
