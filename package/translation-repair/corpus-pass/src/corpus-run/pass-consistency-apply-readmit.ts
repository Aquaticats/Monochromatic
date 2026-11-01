import {
  applyConsistencyEdits,
  type ConsistencyInvariantError,
} from '@monochromatic-dev/translation-repair-consistency/ts';
import {
  candidatesOf,
  clonedLedgerOf,
  keyOf,
  NO_POSITION,
  positionOfSlice,
  type ReadmitLedger,
  readmitLedgerOf,
} from './pass-consistency-apply-ledger.ts';
import {
  EXACT_SEARCH_LIMIT,
  largestValidSetOf,
  NO_LARGER_SET,
  putBackSet,
} from './pass-consistency-apply-largest.ts';
import { putBackInPageOrder, } from './pass-consistency-apply-order.ts';
import { screenSide, } from './pass-consistency-apply-screen.ts';
import {
  type FloorsAsked,
  type GainReason,
  type SettledSlice,
  sliceIndexOf,
} from './pass-consistency-apply-settled.ts';
import type { WorkedPage, } from './pass-consistency-apply-worked.ts';
import type { ConsistencyFloorVerdict, } from './pass-consistency-floors.ts';

//region Pass consistency apply readmit
// THE GAINS THE CHECK LEFT OUT TOGETHER ARE PUT BACK, THE LARGEST SET THE PAGE CAN HOLD. The check of the apostrophes
// (`pass-consistency-apply-gains.ts`) leaves out every splice whose carried apostrophe is unproven on the
// page as it stood, all at once. That can leave out more than it must: two gains that pair are each
// unproven while the other stands, and either would have stood alone; a gain left out for a mark that a
// refused slice carried in could stand once that slice is out. So after the plan and the floors agree
// (`settledSlicesOf` in `pass-consistency-apply-plan.ts`), this step puts splices back in two ways.
//
// IN THE ORDER OF THE PAGE, one at a time. A TRIAL is the question asked of one splice: with it made beside the splices that
// stand, do its words overlap none of theirs, do the floors pass its slice, is the apostrophe it carries in proven on the
// page as it then ships, and is every apostrophe a splice that stands carries in still proven
// (`pass-consistency-apply-trial.ts`). A SCREEN is a cheaper question that needs no trial: one reading of the page with every
// splice left out whose mark the proof reads as the apostrophe of its edge on its own made (a letter or a digit inside the
// mark, a character that parts a mark from a word beside it) tells which of them no page could carry, because a mark of the
// page could pair with the apostrophe it carries in, and those splices are blocked without a trial
// (`pass-consistency-apply-screen.ts`). A splice whose mark reads free there (a symbol, a pictograph, a letter of another
// script or another single mark stands beside it) is not made in that reading, since a free mark is a possible partner of
// every other mark, and it goes to a trial. A splice put back
// changes what the next one is read against, so a pass over the splices left out, screens first and then a trial of each
// one the screens spared, is repeated until a pass puts none back.
//
// IT ENDS because the splices put back only ever grow and are bounded by the splices left out: a pass that
// puts back none is the last, and every other pass puts back at least one, so there are at most as many
// passes as there are splices left out, plus one (`putBackInPageOrder` in `pass-consistency-apply-order.ts` returns the count, and a test builds a page
// that needs every one of them). The reasons the findings give are those of the last
// pass, so each is true of the final page: its words overlap a replacement that is made, the floors refuse the
// slice with it (stated first because it holds whatever the other splices do, also for a splice the screen
// blocked), the splice's apostrophe is unproven there (a quotation mark of the page could pair with it, or what stands beside
// it leaves it free), the splice would unprove an apostrophe that stands (only when its span touches the span of that
// splice and changes the character beside the mark), or the floors refused the slice and it keeps its wording. The reason is what the
// record of the place states when the place ends as it was (`keptBecause`).
//
// AMONG THE SETS, where at most `EXACT_SEARCH_LIMIT` splices are left out in the slices the floors did not refuse, the largest
// set that the page can hold is looked for by looking at every set (`pass-consistency-apply-largest.ts`), because the
// order of the page can make a smaller one: a leading gain admitted first pairs with the trailing gains after it
// and blocks them all. Only sets larger than the order of the page's are looked at, so a tie goes to the order of the
// page's set, and among larger sets of one size the set chosen is a maximum set picked deterministically (the first the
// search finds in its fixed order of looking), not one promised to hold the earliest splices of the page, since the check of
// the apostrophes lets the later of two overlapping gains stand first. Above the limit the order of the page is the answer, since the sets grow with two to the power of the splices left out.
//
// A SLICE THE FLOORS REFUSED KEEPS ITS WORDING WHOLE, so none of its splices is tried: the owner's ruling
// is that a refused slice keeps the wording it has.
//
// THE FLOORS ARE ASKED ABOUT A SET OF REPLACEMENTS ONCE (`floorsQuestionOf`), the memory shared with the
// settling that came before, and they are not monotone: a splice the floors refused with one set of
// replacements may pass with another, which is why a pass is repeated.

/**
 A settled slice as the step leaves it: with the reason each splice it leaves out is left out for, and with the splices put
 back made.

 @param ledger - what the step knows of the page after its last pass

 @param slice - the settled slice as it came

 @param refused - the indexes of the slices the floors refused

 @returns The settled slice

 @example
 ```ts
 const settled = settledSliceOf({ ledger, slice, refused, },);
 ```
 */
function settledSliceOf(
  {
    ledger,
    slice,
    refused,
  }: {
    readonly ledger: ReadmitLedger;
    readonly slice: SettledSlice;
    readonly refused: ReadonlySet<number>;
  },
): SettledSlice {
  /**
   What the step knows of the page after its last pass.
   */
  const {
    worked,
    dropped,
    toMake,
    verdictLog,
    reasonLog,
  } = ledger;
  /**
   The index of the slice.
   */
  const sliceIndex = sliceIndexOf({ slice, },);
  /**
   Its place in the page.
   */
  const position = positionOfSlice({
    worked,
    sliceIndex,
  },);
  if (position === NO_POSITION)
    return slice;
  /**
   The plan the slice came with.
   */
  const { plan, } = slice;
  /**
   The splices it left out then.
   */
  const { unprovenGains, } = plan;
  if (refused.has(sliceIndex,)) {
    return {
      ...slice,
      gainReasons: new Map(unprovenGains.map(function refusedReason({ id, },): readonly [
        string,
        GainReason,
      ] {
        return [
          id,
          { kind: 'refused', },
        ];
      },),),
    };
  }
  /**
   The splices it leaves out in the end.
   */
  const left = dropped[position] ?? [];
  /**
   The reason each splice was found out for, the latest finding of a splice being the one that stands.
   */
  const found = new Map(reasonLog.map(function foundOf({
    key,
    reason,
  },): readonly [
    string,
    GainReason,
  ] {
    return [
      key,
      reason,
    ];
  },),);
  /**
   The reason each is left out for.
   */
  const gainReasons = new Map(left.map(function reasonOf({ id, },): readonly [
    string,
    GainReason,
  ] {
    return [
      id,
      found.get(keyOf({
        position,
        id,
      },),) ?? { kind: 'pairs', },
    ];
  },),);
  if (left.length === unprovenGains.length) {
    return {
      ...slice,
      gainReasons,
    };
  }
  /**
   What the slice makes now.
   */
  const kept = toMake[position] ?? [];
  /**
   What the floors said of it.
   */
  const said = verdictLog.findLast(function isSlice(entry,): boolean {
    return entry.position === position;
  },);
  /**
   Its verdict, absent when the floors never said anything of it.
   */
  const verdict = said?.verdict;
  return {
    plan: {
      ...plan,
      toMake: kept,
      unprovenGains: left,
    },
    made: applyConsistencyEdits({
      english: plan.currentText,
      edits: kept,
    },),
    ...(verdict === undefined) ? {} : { verdict, },
    gainReasons,
  };
}

/**
 How many splices a ledger leaves out in the slices the floors did not refuse.

 @param ledger - what the step knows of the page

 @returns The count

 @example
 ```ts
 const count = leftOutCountOf({ ledger, },);
 ```
 */
function leftOutCountOf({ ledger, }: { readonly ledger: ReadmitLedger; },): number {
  return candidatesOf({ ledger, },)
    .length;
}

/**
 The ledger of the splices put back: the largest set the page can hold where few enough are left out, the set the order of the
 page makes otherwise, and the order of the page's set where it is as large (`pass-consistency-apply-largest.ts`).

 @param ledger - what the step knows of the page as the check left it, which is not changed

 @param asked - what the floors said of each set of replacements, mutated by this function

 @param floorsOn - runs the floors on a slice with its replacements made

 @returns The ledger after the splices are put back, with the reason each splice still left out is left out for

 @example
 ```ts
 const readmitted = readmittedLedgerOf({ ledger, asked, floorsOn, },);
 ```
 */
function readmittedLedgerOf(
  {
    ledger,
    asked,
    floorsOn,
  }: {
    readonly ledger: ReadmitLedger;
    readonly asked: Map<string, ConsistencyFloorVerdict>;
    readonly floorsOn: (arguments_: FloorsAsked,) => ConsistencyFloorVerdict;
  },
): ReadmitLedger {
  /**
   The ledger after the order of the page has put its splices back.
   */
  const inPageOrder = clonedLedgerOf({ ledger, },);
  putBackInPageOrder({
    ledger: inPageOrder,
    asked,
    floorsOn,
  },);
  /**
   How many splices the page leaves out.
   */
  const leftOut = leftOutCountOf({ ledger, },);
  if (leftOut > EXACT_SEARCH_LIMIT)
    return inPageOrder;
  /**
   The ledger the sets are looked at on, with the splices the screens block left out of them.
   */
  const searched = clonedLedgerOf({ ledger, },);
  screenSide({
    ledger: searched,
    side: 'end',
  },);
  screenSide({
    ledger: searched,
    side: 'start',
  },);
  /**
   The largest valid set, none when the order of the page already puts back as many as any set could.
   */
  const largest = largestValidSetOf({
    ledger: searched,
    candidates: candidatesOf({ ledger: searched, },)
      .filter(function isOpen({
        position,
        edit,
      },): boolean {
        return !searched.blocked
          .has(keyOf({
            position,
            id: edit.id,
          },),);
      },),
    toBeat: leftOut - leftOutCountOf({ ledger: inPageOrder, },),
    asked,
    floorsOn,
  },);
  if (largest === NO_LARGER_SET)
    return inPageOrder;
  putBackSet({
    ledger: searched,
    set: largest,
    asked,
    floorsOn,
  },);
  putBackInPageOrder({
    ledger: searched,
    asked,
    floorsOn,
  },);
  return searched;
}

/**
 Puts back the code splices the check of the apostrophes left out that the final page would carry: the largest set the page can
 hold where few enough are left out, otherwise one at a time in the order of the page, until a pass puts back none.

 @param page - the slices that carry English and the text the published page carries between them

 @param round - the settled slices of the page: each plan as the floors and the check agreed to it

 @param refused - the indexes of the slices the floors refused, which keep their wording and put none of their splices back

 @param asked - what the floors said of each slice with a set of replacements made, shared with the settling that came
 before, mutated by this function

 @param floorsOn - runs the floors on a slice with its replacements made

 @returns The settled slices, each with the reason every splice it leaves out is left out for; a slice that puts a splice back
 has the plan, the made slice and the verdict of the floors with it, and the round itself where no slice left a splice out

 @throws {@link ConsistencyInvariantError} when the text between the slices does
 not count one more than the slices

 @example
 ```ts
 const settled = readmittedSlicesOf({ page, round, refused, asked, floorsOn, },);
 ```
 */
export function readmittedSlicesOf(
  {
    page,
    round,
    refused,
    asked,
    floorsOn,
  }: {
    readonly page: WorkedPage;
    readonly round: readonly SettledSlice[];
    readonly refused: ReadonlySet<number>;
    readonly asked: Map<string, ConsistencyFloorVerdict>;
    readonly floorsOn: (arguments_: FloorsAsked,) => ConsistencyFloorVerdict;
  },
): readonly SettledSlice[] {
  /**
   Whether no slice of the round left a splice out.
   */
  const leavesNone = round.every(function leavesNoSplice({ plan, },): boolean {
    /**
     The splices the slice left out.
     */
    const { unprovenGains, } = plan;
    return unprovenGains.length === 0;
  },);
  if (leavesNone)
    return round;
  /**
   What the step knows of the page after the splices are put back.
   */
  const ledger = readmittedLedgerOf({
    ledger: readmitLedgerOf({
      page,
      round,
      refused,
    },),
    asked,
    floorsOn,
  },);
  return round.map(function settledOf(slice,): SettledSlice {
    return settledSliceOf({
      ledger,
      slice,
      refused,
    },);
  },);
}

//endregion Pass consistency apply readmit
