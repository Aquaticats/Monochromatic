import {
  type ConsistencyEdit,
  ConsistencyInvariantError,
} from '@monochromatic-dev/translation-repair-consistency/ts';
import {
  type MadeSlice,
  unprovenOnPage,
} from './pass-consistency-apply-gains.ts';
import {
  type Candidate,
  pageFrameOf,
  type ReadmitLedger,
  replacedAt,
} from './pass-consistency-apply-ledger.ts';
import type { FloorsAsked, } from './pass-consistency-apply-settled.ts';
import {
  floorsWithSet,
  madeWithSet,
} from './pass-consistency-apply-trial.ts';
import type { ConsistencyFloorVerdict, } from './pass-consistency-floors.ts';

//region Pass consistency apply largest
// THE LARGEST SET OF SPLICES THE PAGE CAN HOLD, FOUND BY LOOKING AT EVERY SET (`pass-consistency-apply-readmit.ts`). Putting the
// splices back one at a time in the order of the page leaves out as many as it must not (`pass-consistency-apply-trial.ts`
// admits a leading gain that pairs with the trailing marks after it, and blocks them all), because whether a splice can stand
// depends on the others that do, and an apostrophe carried in can prove or unprove another. A SET IS VALID when, with exactly its
// splices made besides the ones that stand, the words of no two overlap, the floors pass every slice it touches, and the
// apostrophe every gain of the page carries in is proven on the page that ships. Where few enough splices are left out, every
// set of them is looked at, the largest sizes first, and the first valid set found, in the order of the page, is the largest.
// A set too large to look at is left to the order of the page, whose cost grows with the splices and not with their sets.
//
// THE COST IS THE SETS: up to two to the power of the splices left out (`EXACT_SEARCH_LIMIT`), each needing the floors for the
// slices it touches, asked once for each set of replacements a slice can make (`floorsQuestionOf`), and one reading of the whole
// page. The overlap and the floors are asked before the page is read, so a set that cannot be made never costs a page.

/**
 How many splices a page can leave out for the sets of them to be looked at: two to the power of this many sets at most.
 */
export const EXACT_SEARCH_LIMIT = 12;

/**
 What the page and the floors say of the splices of one slice made together, asked once: either the slice cannot be made with
 them (their words overlap a replacement that stands or one another, or the floors refuse the slice with them), or what it
 comes to with them made.
 */
type SliceSet =
  | {
    /**
     The slice cannot be made with them.
     */
    readonly holds: false;
  }
    | {
    /**
     The splices overlap no replacement that stands or one another and the floors pass the slice with them made.
     */
      readonly holds: true;

    /**
     The replacements the slice makes with them, in priority order.
     */
      readonly edits: readonly ConsistencyEdit[];

    /**
     The slice with them made.
     */
      readonly made: MadeSlice;

    /**
     What the floors said of the slice with them made.
     */
      readonly verdict: ConsistencyFloorVerdict;
    };

/**
 What {@link largestValidSetOf} answers when no set larger than the size it was given is valid.
 */
export const NO_LARGER_SET: unique symbol = Symbol('every group of splices bigger than the one-at-a-time result fails validation',);

/**
 Every set of a size over a number of items, as the places of the items, in the order a list of places sorts: the set that holds
 the earliest items first.

 @param count - how many items there are

 @param size - how many each set holds

 @returns Each set's places, ascending, the sets in that order

 @example
 ```ts
 const sets = [...combinationsOf({ count: 3, size: 2, },),]; // [[0, 1], [0, 2], [1, 2]]
 ```
 */
function* combinationsOf(
  {
    count,
    size,
  }: {
    readonly count: number;
    readonly size: number;
  },
): Generator<readonly number[]> {
  if (size === 0) {
    yield [];
    return;
  }
  for (let first = 0; first <= (count - size); first += 1) {
    /**
     How many items come after this one, and so how many the rest of the set is chosen among.
     */
    const after = (count - first) - 1;
    /**
     How far a place among the items after this one is from its place among all of them.
     */
    const shift = first + 1;
    for (const rest of combinationsOf({
      count: after,
      size: size - 1,
    },)) {
      yield [
        first,
        ...rest.map(function afterFirst(place,): number {
          return place + shift;
        },),
      ];
    }
  }
}

/**
 The splices of a set, by the place of their slice, in the order of the page.

 @param set - splices with the place of their slices, in the order of the page

 @returns Each place that holds splices of the set, with the splices

 @example
 ```ts
 const bySlice = bySliceOf({ set, },); // Map { 1 => [edit], 3 => [edit, edit] }
 ```
 */
function bySliceOf({ set, }: { readonly set: readonly Candidate[]; },): ReadonlyMap<number, readonly ConsistencyEdit[]> {
  /**
   The splices of each slice, mutated only inside this function.
   */
  const grouped = new Map<number, ConsistencyEdit[]>();
  for (const {
    position,
    edit,
  } of set) {
    /**
     The splices of this slice so far, none before its first.
     */
    const placed = grouped.get(position,);
    if (placed === undefined) {
      grouped.set(
        position,
        [edit,],
      );
    }
    else {
      placed.push(edit,);
    }
  }
  return grouped;
}

/**
 What the page and the floors say of the splices of one slice made together.

 @param ledger - what the step knows of the page

 @param position - the slice's place in the page

 @param set - the splices of that slice

 @param asked - what the floors said of each set of replacements, mutated by this function

 @param floorsOn - runs the floors on a slice with its replacements made

 @returns Whether the slice can be made with them, and what it comes to

 @example
 ```ts
 const { holds, } = sliceSetOf({ ledger, position: 1, set, asked, floorsOn, },);
 ```
 */
function sliceSetOf(
  {
    ledger,
    position,
    set,
    asked,
    floorsOn,
  }: {
    readonly ledger: ReadmitLedger;
    readonly position: number;
    readonly set: readonly ConsistencyEdit[];
    readonly asked: Map<string, ConsistencyFloorVerdict>;
    readonly floorsOn: (arguments_: FloorsAsked,) => ConsistencyFloorVerdict;
  },
): SliceSet {
  /**
   The floors' verdict on the slice with the splices made, none where their words overlap.
   */
  const [verdict,] = floorsWithSet({
    ledger,
    position,
    set,
    asked,
    floorsOn,
  },);
  if ((verdict === undefined) || (verdict.kind === 'refused'))
    return { holds: false, };
  /**
   The slice with the splices made.
   */
  const {
    edits,
    made,
  } = madeWithSet({
    ledger,
    position,
    set,
  },);
  return {
    holds: true,
    edits,
    made,
    verdict,
  };
}

/**
 Looks for the largest set of the splices left out that the page can hold, among the sets larger than a size.

 @param ledger - what the step knows of the page, which is not changed

 @param candidates - the splices left out that can still be put back, in the order of the page

 @param toBeat - the size the sets must beat: the sets of this size and fewer are not looked at

 @param asked - what the floors said of each set of replacements, mutated by this function

 @param floorsOn - runs the floors on a slice with its replacements made

 @returns The first valid set, in the order of the page, of the largest size that has one, {@link NO_LARGER_SET} when no set
 larger than `toBeat` is valid

 @example
 ```ts
 const set = largestValidSetOf({ ledger, candidates, toBeat: 1, asked, floorsOn, },);
 ```
 */
export function largestValidSetOf(
  {
    ledger,
    candidates,
    toBeat,
    asked,
    floorsOn,
  }: {
    readonly ledger: ReadmitLedger;
    readonly candidates: readonly Candidate[];
    readonly toBeat: number;
    readonly asked: Map<string, ConsistencyFloorVerdict>;
    readonly floorsOn: (arguments_: FloorsAsked,) => ConsistencyFloorVerdict;
  },
): readonly Candidate[] | typeof NO_LARGER_SET {
  /**
   What each set of splices of one slice comes to, by the slice's place and the splices' names, mutated only inside this function.
   */
  const remembered = new Map<string, SliceSet>();
  /**
   Whether a set of splices is valid: every slice it touches can be made and the floors pass it, and every gain of the page is
   proven on the page that ships with the set made.

   @param set - the splices with the place of their slices, in the order of the page

   @returns Whether the page can hold them
   */
  function isValid(set: readonly Candidate[],): boolean {
    /**
     The page's slices with the ones the set touches made with it.
     */
    const made = { list: ledger.made, };
    for (const [position, splices,] of bySliceOf({ set, },)) {
      /**
       The names of these splices of this slice.
       */
      const names = splices.map(function nameOf({ id, },): string {
        return id;
      },);
      /**
       What names this slice with these splices.
       */
      const key =`${String(position,)} ${names.join('\u0000',)}`;
      /**
       What the slice comes to with them, asked once for these splices.
       */
      const sliceSet = remembered.get(key,) ?? sliceSetOf({
        ledger,
        position,
        set: splices,
        asked,
        floorsOn,
      },);
      remembered.set(
        key,
        sliceSet,
      );
      if (!sliceSet.holds)
        return false;
      made.list = replacedAt({
        list: made.list,
        position,
        entry: sliceSet.made,
      },);
    }
    /**
     The splices whose carried apostrophe is not proven on the page with the set made, in each slice.
     */
    const unproven = unprovenOnPage({
      made: made.list,
      gained: ledger.gained,
      ...pageFrameOf({ ledger, },),
    },);
    return unproven.every(function holdsNone(edits,): boolean {
      return edits.length === 0;
    },);
  }
  for (const size of Array.from(
    { length: Math.max(
      candidates.length - toBeat,
      0,
    ), },
    function sizeAt(
      _slot,
      at,
    ): number {
      return candidates.length - at;
    },
  )) {
    for (const places of combinationsOf({
      count: candidates.length,
      size,
    },)) {
      /**
       The splices of this set, in the order of the page.
       */
      const set = places.flatMap(function candidateAt(place,): readonly Candidate[] {
        /**
         The splice at this place.
         */
        const candidate = candidates[place];
        return (candidate === undefined) ? [] : [candidate,];
      },);
      if (isValid(set,))
        return set;
    }
  }
  return NO_LARGER_SET;
}

/**
 Puts a set of splices back: each slice it touches is made with them, and the ledger says so.

 @param ledger - what the step knows of the page, replaced slice by slice

 @param set - the splices with the place of their slices, in the order of the page

 @param asked - what the floors said of each set of replacements, mutated by this function

 @param floorsOn - runs the floors on a slice with its replacements made

 @throws {@link ConsistencyInvariantError} when a slice cannot be made with the splices of the set, which a valid set can

 @example
 ```ts
 putBackSet({ ledger, set, asked, floorsOn, },);
 ```
 */
export function putBackSet(
  {
    ledger,
    set,
    asked,
    floorsOn,
  }: {
    readonly ledger: ReadmitLedger;
    readonly set: readonly Candidate[];
    readonly asked: Map<string, ConsistencyFloorVerdict>;
    readonly floorsOn: (arguments_: FloorsAsked,) => ConsistencyFloorVerdict;
  },
): void {
  /**
   The record of what the floors said of each slice put back, which the ledger keeps.
   */
  const { verdictLog, } = ledger;
  for (const [position, splices,] of bySliceOf({ set, },)) {
    /**
     What the slice comes to with the splices.
     */
    const sliceSet = sliceSetOf({
      ledger,
      position,
      set: splices,
      asked,
      floorsOn,
    },);
    if (!sliceSet.holds) {
      throw new ConsistencyInvariantError({ invariant:
        `slice ${String(position,)} was put back with a set of splices it cannot be made with, and only a valid set is put back`, },);
    }
    /**
     The names of the splices put back in this slice.
     */
    const names = new Set(splices.map(function nameOf({ id, },): string {
      return id;
    },),);
    ledger.made = replacedAt({
      list: ledger.made,
      position,
      entry: sliceSet.made,
    },);
    ledger.toMake = replacedAt({
      list: ledger.toMake,
      position,
      entry: sliceSet.edits,
    },);
    /**
     The splices of this slice still left out.
     */
    const stillLeft = (ledger.dropped[position] ?? []).filter(function isLeft({ id, },): boolean {
      return !names.has(id,);
    },);
    ledger.dropped = replacedAt({
      list: ledger.dropped,
      position,
      entry: stillLeft,
    },);
    verdictLog.push({
      position,
      verdict: sliceSet.verdict,
    },);
  }
}

//endregion Pass consistency apply largest
