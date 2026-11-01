import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import type { GroupedRun, } from './group-aligned.ts';

//region Insertion anchors read off the settled runs
// Where a block the translation never rendered gets written, decided from the
// runs that ship rather than from the walk they were built out of.
//
// WHY THE WALK IS NO LONGER THE AUTHORITY. `anchorOffsets` reads the walk, and
// the walk is a faithful description of the layout right up until
// `mergeOneSidedRuns` folds an unclaimed translation block into a neighbour. A
// run's span is cut from its first node to its last, so absorbing a block
// STRETCHES the span over it, and an anchor that named that block's start is
// then pointing into the middle of a passage rather than at a boundary
// between two.
//
// `assertPlacementLayout` catches it and refuses the document: 431 of 910
// randomised reader-legal pairings over the pinned corpus, and 689 of 4000
// over synthetic ones, all of them with a message about one slice starting
// before the one before it ends.
//
// THE SMALLEST CASE IS TWO PARAGRAPHS AGAINST TWO. Pair only the first, and
// the second original becomes an insertion anchored at the second
// translation's start, while that same translation folds into the first
// run. The anchor and the span then describe the same bytes.
//
// FORWARD FIRST, which keeps the meaning `anchorOffsets` gave it: the missing
// rendering belongs immediately BEFORE the passage that follows, not after the
// one behind. Only when nothing follows does the boundary become the end of
// what came before, which is where a page whose closing passages were never
// rendered needs them written.

/**
 Stands for "no boundary on this side", which no offset can be.
 */
const NO_BOUNDARY = -1;

/**
 Reads, for each run, where the nearest translation blocks AFTER it start.

 @param runs - settled runs in document order

 @returns Offset per run, {@link NO_BOUNDARY} where nothing follows

 @example
 ```ts
 const starts = nextTargetStarts({ runs, },);
 ```
 */
function nextTargetStarts(
  { runs, }: { readonly runs: readonly GroupedRun[]; },
): readonly number[] {
  /**
   Nearest start seen while walking backwards, one named record rather than a
   loose counter.
   */
  const scan = { next: NO_BOUNDARY, };
  return runs
    .toReversed()
    .map(function toNextStart(run,): number {
      /**
       Answer for this run, read before its own blocks can overwrite it.
       */
      const answer = scan.next;

      /**
       First translation block this run carries, absent for an insertion; a
       sealed run's blocks are boundaries like any other's.
       */
      const first = (run.kind === 'insertion')
        ? undefined
        : run.targetRun[0];
      if (first !== undefined)
        scan.next = first.startOffset;
      return answer;
    },)
    .toReversed();
}

/**
 Reads, for each run, where the nearest translation blocks BEFORE it end.

 @param runs - settled runs in document order

 @returns Offset per run, {@link NO_BOUNDARY} where nothing precedes

 @example
 ```ts
 const ends = previousTargetEnds({ runs, },);
 ```
 */
function previousTargetEnds(
  { runs, }: { readonly runs: readonly GroupedRun[]; },
): readonly number[] {
  /**
   Nearest end seen while walking forwards.
   */
  const scan = { previous: NO_BOUNDARY, };
  return runs.map(function toPreviousEnd(run,): number {
    /**
     Answer for this run, read before its own blocks can overwrite it.
     */
    const answer = scan.previous;

    /**
     Last translation block this run carries, absent for an insertion; a
     sealed run's blocks are boundaries like any other's.
     */
    const last = (run.kind === 'insertion')
      ? undefined
      : run.targetRun
        .at(-1,);
    if (last !== undefined)
      scan.previous = last.endOffset;
    return answer;
  },);
}

/**
 Rewrites every insertion's anchor as a boundary between the runs beside it.

 @param runs - settled runs in document order

 @returns Same runs, each insertion anchored where it can actually be written

 @example
 ```ts
 const placeable = reanchorInsertions({ runs, },);
 ```
 */
export function reanchorInsertions(
  { runs, }: { readonly runs: readonly GroupedRun[]; },
): readonly GroupedRun[] {
  /**
   Where the next translation blocks start, per run.
   */
  const nextStart = nextTargetStarts({ runs, },);

  /**
   Where the previous translation blocks end, per run.
   */
  const previousEnd = previousTargetEnds({ runs, },);
  return runs.map(function toAnchored(
    run,
    at,
  ): GroupedRun {
    if (run.kind !== 'insertion')
      return run;

    /**
     Where the nearest translation blocks after this insertion start, which
     these runs always carry: `anchorOffsets` names an anchor only where a
     step paired, and a paired step leaves its blocks in a run.
     */
    const next = nonNullishOrThrow(nextStart.at(at,),);

    /**
     Where the nearest translation blocks before this insertion end.
     */
    const previous = nonNullishOrThrow(previousEnd.at(at,),);

    /**
     Boundary this insertion writes at, preferring the passage that follows.
     Nothing either side carries a translation block only when no walk builds
     one: `anchorOffsets` names an anchor only where a step paired, and a
     paired step always leaves a translation block in the runs for these
     scans to find (ledger T8).
     */
    const settled = (next === NO_BOUNDARY) ? previous : next;

    // LOUD RATHER THAN WRITTEN. Without this check the insertion would carry
    // `NO_BOUNDARY` as its offset, a position no page has, and the splice
    // would be left to make what it could of it (ledger M113).
    if (settled === NO_BOUNDARY)
      throw new Error(
        'unreachable: an insertion with no translation block on either side, though an anchor is named only '
          + 'where a step paired and a paired step leaves its translation block in a run',
      );
    return {
      kind: 'insertion',
      sourceRun: run.sourceRun,
      targetOffset: settled,
    };
  },);
}

//endregion Insertion anchors read off the settled runs
