import {
  applyConsistencyEdits,
  type ConsistencyInvariantError,
} from '@monochromatic-dev/translation-repair-consistency/ts';
import { withoutUnprovenGains, } from './pass-consistency-apply-gains.ts';
import { readmittedSlicesOf, } from './pass-consistency-apply-readmit.ts';
import {
  type FloorsAsked,
  floorsQuestionOf,
  type SettledSlice,
  type SlicePlan,
  sliceIndexOf,
} from './pass-consistency-apply-settled.ts';
import {
  workedPageOf,
  type WorkedPage,
} from './pass-consistency-apply-worked.ts';
import type { ConsistencyFloorVerdict, } from './pass-consistency-floors.ts';
import type { ConsistencySliceFacts, } from './pass-consistency-slices.ts';
import type { SettledGroup, } from './pass-consistency-types.ts';

//region Pass consistency apply plan
// WHAT EACH SLICE'S REPLACEMENTS COME TO BEFORE ANY IS RECORDED, decided over the whole page.
// A replacement is left out when it changes the words another group located and that group does
// not replace (`pass-consistency-apply-located.ts`), and a code splice is left out when the
// apostrophe it carries in is not proven on the page (`pass-consistency-apply-gains.ts`). The
// second is a question about EVERY SLICE AT ONCE, since a quotation mark in one slice, or one
// that a replacement of another slice carries in, can pair with the mark, so the plan of every
// slice is made before the first slice is applied. Nothing is recorded here: the apply stage
// records the outcomes and the findings slice by slice, in page order.
//
// THE FLOORS are the checks a slice with its replacements made must pass
// (`pass-consistency-floors.ts`): they judge the slice whole, and a slice they refuse keeps the
// wording it has now. A PLAN IS SETTLED when the floors have said what they say of each slice
// with its replacements made, the check of the apostrophes has been run on the page the passing
// slices make, and no step here would change either (`pass-consistency-apply-settled.ts`).
//
// THE CHECK OF THE APOSTROPHES AND THE FLOORS DECIDE TOGETHER (`settledSlicesOf`). The check
// reads the page as the replacements would make it, and the floors then refuse a slice whole,
// which keeps its wording and so puts back the words its replacements took out, a closing mark
// among them. The floors run on the plans, the slices they refuse are planned again as holding
// none of their replacements, and that repeats until no floor refuses another slice. A slice
// refused once stays refused, so the loop ends after at most one round more than the page has
// slices; the plan a refused slice is recorded with is the one it had when the floors refused it.
//
// THE CHECK LEAVES OUT EVERY UNPROVEN SPLICE AT ONCE, which can leave out one that would have
// stood, so the settled plan is read once more (`pass-consistency-apply-readmit.ts`): each
// splice left out is tried again on the final page, one at a time in page order, and put back
// when its mark is proven there, the marks of the splices that stand still are, and the floors
// pass its slice. What each plan reads of the groups (`pass-consistency-apply-worked.ts`) is
// read once for the pass, not once for each round.

/**
 Plans the replacements of every slice of a page already read, that carries English and has replacements.

 @param page - the slices that carry English with their replacements, and the text the published page carries between them

 @param refused - the indexes of the slices the floors refused, which keep their wording, so the page the apostrophes are
 checked on holds none of their replacements and they make none; none by default

 @returns The plan of each slice that has replacements, in page order

 @throws {@link ConsistencyInvariantError} when the text between the slices does
 not count one more than the slices

 @example
 ```ts
 const plans = plansOnPage({ page: workedPageOf({ settled, slices, },), refused: new Set(), },);
 ```
 */
export function plansOnPage(
  {
    page,
    refused,
  }: {
    readonly page: WorkedPage;
    readonly refused: ReadonlySet<number>;
  },
): readonly SlicePlan[] {
  /**
   The slices that carry English and the text the published page carries between them.
   */
  const {
    slices: workedSlices,
    between,
    layout,
  } = page;
  /**
   What the apostrophes the splices carry in come to on the whole page, in the order of the slices.
   */
  const verdicts = withoutUnprovenGains({
    slices: workedSlices.map(function gainSliceOf({
      facts,
      currentText,
      candidates,
      splices,
    },) {
      return {
        sliceIndex: facts.sliceIndex,
        english: currentText,
        // A slice the floors refused keeps its wording, so the page holds none of its replacements.
        edits: refused.has(facts.sliceIndex,) ? [] : candidates,
        splices,
      };
    },),
    ...(between === undefined) ? {} : { between, },
    ...(layout === undefined) ? {} : { layout, },
  },);
  return workedSlices.flatMap(function planOf(
    slice,
    position,
  ): readonly SlicePlan[] {
    /**
     The verdict of the check for this slice.
     */
    const verdict = verdicts[position];
    /**
     The slice's replacements before any is left out.
     */
    const { edits, } = slice.named;
    if ((edits.length === 0) || (verdict === undefined))
      return [];
    return [
      {
        ...slice.named,
        facts: slice.facts,
        currentText: slice.currentText,
        leftOut: slice.leftOut,
        toMake: verdict.kept,
        unprovenGains: verdict.dropped,
      },
    ];
  },);
}

/**
 Whether the floors refused the slice with its plan made.

 @param slice - the settled slice

 @returns True when the floors refused it

 @example
 ```ts
 const refusedNow = isRefusedByFloors({ slice, },);
 ```
 */
function isRefusedByFloors({ slice, }: { readonly slice: SettledSlice; },): boolean {
  /**
   What the floors said, when they were asked.
   */
  const { verdict, } = slice;
  return verdict?.kind === 'refused';
}

/**
 The plans of one round, each put through the floors, a slice the floors refused in an earlier round standing as it did.

 @param page - the slices that carry English with their replacements, and the text between them

 @param refused - the slices the floors refused in earlier rounds, by the index of the slice

 @param asked - what the floors said of a slice with a set of replacements made, by the slice and the replacements, so that
 a slice whose replacements did not change is not put through the floors again; mutated by this function

 @param floorsOn - runs the floors on a slice with its plan made

 @returns The settled slices of the round, in page order

 @example
 ```ts
 const round = roundOf({ page, refused: new Map(), asked: new Map(), floorsOn, },);
 ```
 */
function roundOf(
  {
    page,
    refused,
    asked,
    floorsOn,
  }: {
    readonly page: WorkedPage;
    readonly refused: ReadonlyMap<number, SettledSlice>;
    readonly asked: Map<string, ConsistencyFloorVerdict>;
    readonly floorsOn: (arguments_: FloorsAsked,) => ConsistencyFloorVerdict;
  },
): readonly SettledSlice[] {
  return plansOnPage({
    page,
    refused: new Set(refused.keys(),),
  },)
    .map(function settleOne(plan,): SettledSlice {
      /**
       The index of the slice in page order.
       */
      const { sliceIndex, } = plan.facts;
      /**
       The replacements the plan makes.
       */
      const { toMake, } = plan;
      /**
       What the floors said of the slice in an earlier round, when they refused it.
       */
      const before = refused.get(sliceIndex,);
      if (before !== undefined)
        return before;
      if (toMake.length === 0)
        return {
          plan,
          gainReasons: new Map(),
        };
      /**
       The slice with the plan's replacements made.
       */
      const made = applyConsistencyEdits({
        english: plan.currentText,
        edits: toMake,
      },);
      /**
       The slice and the replacements the floors are asked about, which is what makes the same question the same.
       */
      const question = floorsQuestionOf({
        sliceIndex,
        edits: toMake,
      },);
      /**
       What the floors said when they were asked this question before.
       */
      const known = asked.get(question,);
      /**
       What the floors say of the slice with the plan made.
       */
      const verdict = known ?? floorsOn({
        plan,
        candidateText: made.text,
      },);
      asked.set(
        question,
        verdict,
      );
      return {
        plan,
        made,
        verdict,
        gainReasons: new Map(),
      };
    },);
}

/**
 Plans the replacements of every slice, puts each changed slice through the floors, and plans again without the slices
 the floors refused, until no floor refuses another slice; then puts back the splices the final page would carry.

 A slice the floors refuse keeps its wording, so the page the pass ends with holds none of its replacements, and the
 apostrophes the other replacements carry in are checked on that page: a closing mark that a refused splice took out stands
 again and can pair with a mark carried in earlier (`’round` before a `parents’` that comes back), so such a splice is left
 out, and the slice it left changes, so the floors are asked again about what is left of it. EACH ROUND EITHER REFUSES A SLICE
 NOT REFUSED BEFORE OR ENDS THE LOOP, and a refused slice stays refused, so the loop ends after at most one round more than
 the page has slices. A round does not carry over what the check left out in the round before: it plans from the replacements
 of the slices not refused, so a splice the check had left out for a mark a refused slice carried in is made once that slice
 is out.

 The check leaves out every unproven splice at once, so the splices left out are then tried again, one at a time in page
 order, on the page the loop ended with (`readmittedSlicesOf`), which puts back each one the page would carry and gives the
 reason for each one that stays out.

 @param settled - the groups in page order, with their pending replacements

 @param slices - what the pass knows of each slice, in page order

 @param floorsOn - runs the floors on a slice with its plan made

 @returns The settled slices in page order: the plan of each slice that has replacements, the slice with the plan made and the
 floors' verdict, and for a refused slice the plan as it stood when the floors refused it; each with the reason every splice
 it leaves out is left out for

 @throws {@link ConsistencyInvariantError} when two replacements of one group overlap in a slice

 @example
 ```ts
 const result = settledSlicesOf({ settled, slices, floorsOn, },);
 ```
 */
export function settledSlicesOf(
  {
    settled,
    slices,
    floorsOn,
  }: {
    readonly settled: readonly SettledGroup[];
    readonly slices: readonly ConsistencySliceFacts[];
    readonly floorsOn: (arguments_: FloorsAsked,) => ConsistencyFloorVerdict;
  },
): readonly SettledSlice[] {
  /**
   The slices that carry English with their replacements, read once for every round.
   */
  const page = workedPageOf({
    settled,
    slices,
  },);
  /**
   The slices the floors refused, by index, and the plan each had when they refused it, mutated only inside this function.
   */
  const refused = new Map<number, SettledSlice>();
  /**
   What the floors said of each question they were asked, mutated only inside this function.
   */
  const asked = new Map<string, ConsistencyFloorVerdict>();
  /**
   The slices of the latest round, replaced by each round.
   */
  const state = {
    round: roundOf({
      page,
      refused,
      asked,
      floorsOn,
    },),
  };
  /**
   The slices of a round that the floors refused and that no earlier round had refused.

   @returns The slices of the latest round that are newly refused
   */
  function newlyRefused(): readonly SettledSlice[] {
    return state.round
      .filter(function isNewlyRefused(slice,): boolean {
        return isRefusedByFloors({ slice, },) && (!refused.has(sliceIndexOf({ slice, },),));
      },);
  }
  /**
   Whether the latest round holds a refusal that no earlier round held.

   @returns True when another round is needed
   */
  function hasNewRefusal(): boolean {
    /**
     The slices newly refused in the latest round.
     */
    const fresh = newlyRefused();
    return fresh.length > 0;
  }
  while (hasNewRefusal()) {
    for (const slice of newlyRefused()) {
      refused.set(
        sliceIndexOf({ slice, },),
        slice,
      );
    }
    state.round = roundOf({
      page,
      refused,
      asked,
      floorsOn,
    },);
  }
  return readmittedSlicesOf({
    page,
    round: state.round,
    refused: new Set(refused.keys(),),
    asked,
    floorsOn,
  },);
}

//endregion Pass consistency apply plan
