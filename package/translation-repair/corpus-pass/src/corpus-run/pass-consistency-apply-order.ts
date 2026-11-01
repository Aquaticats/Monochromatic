import {
  candidatesOf,
  keyOf,
  type ReadmitLedger,
  replacedAt,
} from './pass-consistency-apply-ledger.ts';
import { screenSide, } from './pass-consistency-apply-screen.ts';
import type { FloorsAsked, } from './pass-consistency-apply-settled.ts';
import {
  floorsWithSplice,
  trialOf,
} from './pass-consistency-apply-trial.ts';
import type { ConsistencyFloorVerdict, } from './pass-consistency-floors.ts';

//region Pass consistency apply order
// THE SPLICES LEFT OUT ARE PUT BACK IN THE ORDER OF THE PAGE (`pass-consistency-apply-readmit.ts`), one at a time. A pass takes
// the screens first (`pass-consistency-apply-screen.ts`) and then a trial of each splice the screens spared
// (`pass-consistency-apply-trial.ts`); a splice put back changes what the next one is read against, so passes are repeated
// until one puts none back. The set this reaches depends on the order, which is why the readmission also looks for a larger set
// where few enough splices are left out (`pass-consistency-apply-largest.ts`).

/**
 One pass over the splices left out, in the order of the page: the screens, then a trial of each splice the screens spared.

 @param ledger - what the step knows of the page, replaced as a splice is put back

 @param asked - what the floors said of each set of replacements, mutated by this function

 @param floorsOn - runs the floors on a slice with its replacements made

 @returns How many splices the pass put back

 @example
 ```ts
 const putBack = passOf({ ledger, asked, floorsOn, },);
 ```
 */
function passOf(
  {
    ledger,
    asked,
    floorsOn,
  }: {
    readonly ledger: ReadmitLedger;
    readonly asked: Map<string, ConsistencyFloorVerdict>;
    readonly floorsOn: (arguments_: FloorsAsked,) => ConsistencyFloorVerdict;
  },
): number {
  screenSide({
    ledger,
    side: 'end',
  },);
  screenSide({
    ledger,
    side: 'start',
  },);
  /**
   The ledgers of the step, which the pass adds to.
   */
  const {
    blocked,
    reasonLog,
    verdictLog,
  } = ledger;
  /**
   How many splices this pass has put back.
   */
  const putBack = { count: 0, };
  for (const candidate of candidatesOf({ ledger, },)) {
    /**
     The splice and the place of its slice.
     */
    const {
      position,
      edit,
    } = candidate;
    /**
     The splice's key.
     */
    const key = keyOf({
      position,
      id: edit.id,
    },);
    if (blocked.has(key,))
      continue;
    /**
     What putting the splice back comes to on the page as it stands now.
     */
    const trial = trialOf({
      ledger,
      position,
      edit,
      asked,
      floorsOn,
    },);
    if (trial.kind === 'out') {
      reasonLog.push({
        key,
        reason: trial.reason,
      },);
      continue;
    }
    /**
     The splices of the slices still left out, as the page stands now.
     */
    const { dropped, } = ledger;
    /**
     The splices of this slice still left out.
     */
    const stillLeft = (dropped[position] ?? []).filter(function isLeft({ id, },): boolean {
      return id !== edit.id;
    },);
    ledger.made = replacedAt({
      list: ledger.made,
      position,
      entry: trial.made,
    },);
    ledger.toMake = replacedAt({
      list: ledger.toMake,
      position,
      entry: trial.edits,
    },);
    ledger.dropped = replacedAt({
      list: dropped,
      position,
      entry: stillLeft,
    },);
    // The reason the splice was left out for is no longer read: it is out of the splices still left out.
    verdictLog.push({
      position,
      verdict: trial.verdict,
    },);
    putBack.count += 1;
  }
  return putBack.count;
}

/**
 Gives the floors' refusal as the reason of each splice the screen blocked, where the floors refuse its slice with it.

 A trial states the refusal of the floors first (`pass-consistency-apply-trial.ts`), but the screen blocks a splice without a
 trial, with the reason that a mark of the page could pair with its apostrophe. That reason holds on the final page, and so
 does the floors' refusal, which holds whatever the other splices do and so comes first: the reason of a blocked splice
 whose slice the floors refuse with it is the refusal.

 @param ledger - what the step knows of the page after its last pass; the reasons it holds are added to

 @param asked - what the floors said of each set of replacements, mutated by this function

 @param floorsOn - runs the floors on a slice with its replacements made

 @example
 ```ts
 refusalsOfBlocked({ ledger, asked, floorsOn, },);
 ```
 */
function refusalsOfBlocked(
  {
    ledger,
    asked,
    floorsOn,
  }: {
    readonly ledger: ReadmitLedger;
    readonly asked: Map<string, ConsistencyFloorVerdict>;
    readonly floorsOn: (arguments_: FloorsAsked,) => ConsistencyFloorVerdict;
  },
): void {
  /**
   The splices still left out, the slices the floors refused, and the splices the screen blocked.
   */
  const {
    dropped,
    refused,
    blocked,
    reasonLog,
  } = ledger;
  dropped.forEach(function refusalsInSlice(
    edits,
    position,
  ): void {
    if (refused.has(position,))
      return;
    for (const edit of edits) {
      /**
       The splice's key.
       */
      const key = keyOf({
        position,
        id: edit.id,
      },);
      if (!blocked.has(key,))
        continue;
      /**
       What the floors say of the slice with the splice made, none where it overlaps one that stands.
       */
      const [verdict,] = floorsWithSplice({
        ledger,
        position,
        edit,
        asked,
        floorsOn,
      },);
      if (verdict?.kind === 'refused') {
        reasonLog.push({
          key,
          reason: {
            kind: 'floors',
            findings: verdict.findings,
          },
        },);
      }
    }
  },);
}

/**
 Puts back, one at a time in the order of the page, the splices left out that the page then carries, until a pass puts back
 none, and states the reason of the splices a trial blocked and the floors refuse.

 @param ledger - what the step knows of the page, replaced as splices are put back

 @param asked - what the floors said of each set of replacements, mutated by this function

 @param floorsOn - runs the floors on a slice with its replacements made

 @returns How many passes ran: every one but the last put back at least one splice, so at most one more than the splices left out

 @example
 ```ts
 const passes = putBackInPageOrder({ ledger, asked, floorsOn, },);
 ```
 */
export function putBackInPageOrder(
  {
    ledger,
    asked,
    floorsOn,
  }: {
    readonly ledger: ReadmitLedger;
    readonly asked: Map<string, ConsistencyFloorVerdict>;
    readonly floorsOn: (arguments_: FloorsAsked,) => ConsistencyFloorVerdict;
  },
): number {
  /**
   How many splices the latest pass put back, and how many passes ran; the step ends with a pass that puts back none.
   */
  const progress = {
    putBack: 1,
    passes: 0,
  };
  while (progress.putBack > 0) {
    progress.putBack = passOf({
      ledger,
      asked,
      floorsOn,
    },);
    progress.passes += 1;
  }
  refusalsOfBlocked({
    ledger,
    asked,
    floorsOn,
  },);
  return progress.passes;
}

//endregion Pass consistency apply order
