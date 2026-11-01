import type { AlignmentStep, } from '@monochromatic-dev/translation-repair-document-structure/ts';

//region Unpartnered gap steps
// A GAP THE ROSTER LEFT UNPLACED ON BOTH SIDES IS ONE RENDERING. Between two
// paired steps a roster may leave originals and translation blocks alike
// unpartnered: it paired the blocks around a passage and nothing inside it,
// which happens when the translator merged and split the passage so that no
// block answers to one block. Read as bare steps, the originals become
// insertions (a coverage round then calls them carried or absent) and the
// translation blocks, undeclined because the pairing placed no original, ride
// into the next paired slice's span. On mikaela15 (2026-09-24) that shape
// shipped the HRT passage twice: the class one hundred eleven fold rendered
// the carried originals through their carrier and slice 13 kept the archive's
// rendering it had been handed (class one hundred twelve); on mikaela12 and 13
// the same shape lost the passage instead.
//
// THE FIRST PAIR OPENS THE RENDERING AND THE REST RIDE ALONG, which is the
// vocabulary a roster already uses for a merge: `paired` once, then
// `continuesPairing` on both sides. The grouper keeps continuations in one
// run, the anchor reader gives them no insertion, and the decline reader never
// declines them, so the gap reaches the lanes as one slice whose source is
// every unplaced original and whose span is every unplaced block.
//
// INTERIOR GAPS ONLY. A leading gap has no pairing before it and a trailing
// gap none after it; the tail bound (owner decision 2026-09-19) and the
// leading-skip rule already read those, and this pass leaves them as they
// are. A gap with blocks on one side alone is not a rendering and stays bare.

/**
 A maximal run of bare steps between two steps that belong to a rendering.
 */
type GapSpan = {
  /**
   Index of the first bare step.
   */
  readonly start: number;

  /**
   Index past the last bare step.
   */
  readonly end: number;
};

/**
 Whether a step belongs to a rendering: paired, or continuing one.

 @param step - alignment step

 @returns True for a paired step or a continuation

 @example
 ```ts
 const anchored = inRendering({ kind: 'paired', sourceIndex: 0, targetIndex: 0, },);
 ```
 */
function inRendering(step: AlignmentStep,): boolean {
  return (step.kind === 'paired') || (step.continuesPairing === true);
}

/**
 Interior gaps of bare steps, each bounded by rendering steps on both sides.

 @param steps - alignment steps in document order

 @returns Gap spans in document order

 @example
 ```ts
 const gaps = interiorGaps({ steps, },);
 ```
 */
function interiorGaps(
  { steps, }: { readonly steps: readonly AlignmentStep[]; },
): readonly GapSpan[] {
  /**
   Indices of steps that belong to a rendering.
   */
  const anchored = steps.flatMap(function anchoredAt(
    step,
    index,
  ): readonly number[] {
    return inRendering(step,) ? [index,] : [];
  },);
  return anchored.flatMap(function gapAfter(
    at,
    position,
  ): readonly GapSpan[] {
    /**
     The next rendering step, absent at the last one.
     */
    const next = anchored[position + 1];
    /**
     Index right after this rendering step.
     */
    const following = at + 1;
    if ((next === undefined) || (next === following))
      return [];
    return [{
      start: at + 1,
      end: next,
    },];
  },);
}

/**
 Reads one gap as a merge where it holds bare steps on both sides, else as it
 is.

 @param bare - the gap's steps

 @returns Steps the gap contributes

 @example
 ```ts
 const merged = mergeGap({ bare, },);
 ```
 */
function mergeGap(
  { bare, }: { readonly bare: readonly AlignmentStep[]; },
): readonly AlignmentStep[] {
  /**
   Original indices the gap holds, in document order.
   */
  const sources = bare.flatMap(function sourceOf(step,): readonly number[] {
    return (step.kind === 'source-only') ? [step.sourceIndex,] : [];
  },);
  /**
   Translation indices the gap holds, in document order.
   */
  const targets = bare.flatMap(function targetOf(step,): readonly number[] {
    return (step.kind === 'target-only') ? [step.targetIndex,] : [];
  },);
  /**
   The opening original, absent when the gap has none.
   */
  const [firstSource,] = sources;
  /**
   The opening translation block, absent when the gap has none.
   */
  const [firstTarget,] = targets;
  if ((firstSource === undefined) || (firstTarget === undefined))
    return bare;
  return [
    {
      kind: 'paired',
      sourceIndex: firstSource,
      targetIndex: firstTarget,
    },
    ...sources
      .slice(1,)
      .map(function continueSource(sourceIndex,): AlignmentStep {
        return {
          kind: 'source-only',
          sourceIndex,
          continuesPairing: true,
        };
      },),
    ...targets
      .slice(1,)
      .map(function continueTarget(targetIndex,): AlignmentStep {
        return {
          kind: 'target-only',
          targetIndex,
          continuesPairing: true,
        };
      },),
  ];
}

/**
 Pairs every interior gap that the roster left unplaced on both sides as one
 merge; every other step stays as it is.

 @param steps - alignment steps in document order

 @returns Steps with such gaps merged

 @example
 ```ts
 const merged = pairUnpartneredGaps({ steps, },);
 ```
 */
export function pairUnpartneredGaps(
  { steps, }: { readonly steps: readonly AlignmentStep[]; },
): readonly AlignmentStep[] {
  /**
   Interior gaps, in document order.
   */
  const gaps = interiorGaps({ steps, },);
  if (gaps.length === 0)
    return steps;
  // ONE PASS THAT APPENDS, since copying the steps rebuilt so far at every gap
  // costs the gap count times the step count (ledger B70).
  /**
   Steps rebuilt gap by gap: the steps before each gap, then the gap as read.
   */
  const rebuilt: AlignmentStep[] = [];
  /**
   Index of the first step not yet copied.
   */
  const cursor = { at: 0, };
  for (const gap of gaps) {
    rebuilt.push(
      ...steps.slice(
        cursor.at,
        gap.start,
      ),
      ...mergeGap({
        bare: steps.slice(
          gap.start,
          gap.end,
        ),
      },),
    );
    cursor.at = gap.end;
  }
  rebuilt.push(...steps.slice(cursor.at,),);
  return rebuilt;
}

//endregion Unpartnered gap steps
