import type {
  ConsistencyEdit,
  PageLayout,
} from '@monochromatic-dev/translation-repair-consistency/ts';
import {
  gainedBy,
  type GainedSides,
  type GainSlice,
  madeIn,
  type MadeSlice,
} from './pass-consistency-apply-gains.ts';
import {
  type GainReason,
  type SettledSlice,
  type SlicePlan,
  sliceIndexOf,
} from './pass-consistency-apply-settled.ts';
import type {
  Worked,
  WorkedPage,
} from './pass-consistency-apply-worked.ts';
import type { ConsistencyFloorVerdict, } from './pass-consistency-floors.ts';

//region Pass consistency apply ledger
// WHAT THE STEP THAT PUTS SPLICES BACK KNOWS OF THE PAGE WHILE IT WORKS (`pass-consistency-apply-readmit.ts`):
// for each slice that carries English, by its place in the page, the replacements that stand made, the
// splices left out, what the floors said and the reason each splice is left out for. It is replaced a
// slice at a time as a splice is put back, so a trial reads the page as it stands and never a page
// built from the round's first reading. IT IS CALLED THE LEDGER (`ReadmitLedger`): it holds the step's
// accounts of the page, the replacements made, the splices left out and the reasons, and is not the
// `standing` of `consistency-carry-standing.ts`, which says where a span stands among the blocks.

/**
 What the floors said of a slice's set of replacements.
 */
export type VerdictEntry = {
  /**
   The slice's place in the page.
   */
  readonly position: number;

  /**
   What the floors said.
   */
  readonly verdict: ConsistencyFloorVerdict;
};

/**
 Why a splice is left out.
 */
export type ReasonEntry = {
  /**
   The splice's key (`keyOf`).
   */
  readonly key: string;

  /**
   The reason.
   */
  readonly reason: GainReason;
};

/**
 What the step knows of the page while it works.
 */
export type ReadmitLedger = {
  /**
   The slices that carry English, in page order.
   */
  readonly worked: readonly Worked[];

  /**
   The plan of each slice that has replacements, by the slice's place in the page.
   */
  readonly plans: ReadonlyMap<number, SlicePlan>;

  /**
   The places of the slices the floors refused.
   */
  readonly refused: ReadonlySet<number>;

  /**
   The sides each code splice gains an edge apostrophe on, for each slice.
   */
  readonly gained: readonly ReadonlyMap<string, GainedSides>[];

  /**
   The text the published page carries around the slices: one piece more than there are slices, a blank line between each
   two where the facts carry no page.
   */
  readonly between: readonly string[];

  /**
   What the published page is assembled from, when the facts carry it: the page a mark is proved on is then assembled by the
   code that publishes it.
   */
  readonly layout?: PageLayout;

  /**
   Each slice with the replacements that stand made, replaced as a splice is put back.
   */
  made: readonly MadeSlice[];

  /**
   The replacements each slice makes, in priority order, replaced as a splice is put back.
   */
  toMake: readonly (readonly ConsistencyEdit[])[];

  /**
   The code splices each slice leaves out, in priority order, replaced as a splice is put back.
   */
  dropped: readonly (readonly ConsistencyEdit[])[];

  /**
   What the floors said of a slice's set of replacements, in the order they said it; the latest entry of a slice is the one
   that stands, and a slice never put through them has none. Appended to only by the step.
   */
  readonly verdictLog: VerdictEntry[];

  /**
   Why a splice is left out, in the order the step found it; the latest entry of a splice is the one that stands. Appended
   to only by the step.
   */
  readonly reasonLog: ReasonEntry[];

  /**
   The splices no trial can put back, by the slice's place and the splice's name. Mutated only by the step.
   */
  readonly blocked: Set<string>;
};

/**
 What the proof of an apostrophe reads the page by: the text around the slices, the layout of the published page when the
 facts carry it, and the index each slice is named by there.

 @param ledger - what the step knows of the page

 @returns The three, ready to hand to `unprovenOnPage`

 @example
 ```ts
 const unproven = unprovenOnPage({ made, gained, ...pageFrameOf({ ledger, },), },);
 ```
 */
export function pageFrameOf(
  { ledger, }: { readonly ledger: ReadmitLedger; },
): {
  readonly between: readonly string[];
  readonly layout?: PageLayout;
  readonly sliceIndexes: readonly number[];
} {
  /**
   The slices of the page and the text around them.
   */
  const {
    worked,
    between,
    layout,
  } = ledger;
  return {
    between,
    ...(layout === undefined) ? {} : { layout, },
    sliceIndexes: worked.map(function indexOfSlice({ facts, },): number {
      return facts.sliceIndex;
    },),
  };
}

/**
 The key a splice is kept under: its slice's place in the page and its name.

 @param position - the slice's place in the page

 @param id - name of the splice

 @returns The key

 @example
 ```ts
 const key = keyOf({ position: 2, id: 'group 1 OCCURRENCE 2', },); // '2 group 1 OCCURRENCE 2'
 ```
 */
export function keyOf(
  {
    position,
    id,
  }: {
    readonly position: number;
    readonly id: string;
  },
): string {
  return `${String(position,)} ${id}`;
}

/**
 The names of some replacements.

 @param edits - the replacements

 @returns Their names

 @example
 ```ts
 const ids = idsOf({ edits, },);
 ```
 */
export function idsOf({ edits, }: { readonly edits: readonly ConsistencyEdit[]; },): ReadonlySet<string> {
  return new Set(edits.map(function idOf({ id, },): string {
    return id;
  },),);
}

/**
 A list with one entry replaced.

 @param list - the list

 @param position - the place of the entry

 @param entry - what goes there

 @returns A copy of the list with the entry replaced

 @example
 ```ts
 const next = replacedAt({ list: [1, 2,], position: 0, entry: 3, },); // [3, 2]
 ```
 */
export function replacedAt<Entry,>(
  {
    list,
    position,
    entry,
  }: {
    readonly list: readonly Entry[];
    readonly position: number;
    readonly entry: Entry;
  },
): readonly Entry[] {
  return list.with(
    position,
    entry,
  );
}

/**
 A slice as the check of the apostrophes reads it, with all the replacements that were not left out for another group's
 words.

 @param worked - the slice

 @returns The slice and its replacements

 @example
 ```ts
 const slice = gainSliceOf({ worked, },);
 ```
 */
export function gainSliceOf({ worked, }: { readonly worked: Worked; },): GainSlice {
  /**
   Everything the check of the apostrophes reads of the slice.
   */
  const {
    facts,
    currentText,
    candidates,
    splices,
  } = worked;
  return {
    sliceIndex: facts.sliceIndex,
    english: currentText,
    edits: candidates,
    splices,
  };
}

/**
 The text a page of slices a blank line apart carries around them.

 @param count - how many slices the page has

 @returns Nothing before the first and after the last, and a blank line between each two

 @example
 ```ts
 const between = blankBetween({ count: 2, },); // ['', '\n\n', '']
 ```
 */
function blankBetween({ count, }: { readonly count: number; },): readonly string[] {
  return Array.from(
    { length: count + 1, },
    function gapAt(
      _slice,
      at,
    ): string {
      return ((at === 0) || (at === count)) ? '' : '\n\n';
    },
  );
}

/**
 The place `positionOfSlice` gives a slice that carries no English.
 */
export const NO_POSITION = -1;

/**
 The place of a slice in the page.

 @param worked - the slices that carry English, in page order

 @param sliceIndex - the index the artifact names the slice by

 @returns The place, or {@link NO_POSITION} when the slice carries no English

 @example
 ```ts
 const position = positionOfSlice({ worked, sliceIndex: 4, },);
 ```
 */
export function positionOfSlice(
  {
    worked,
    sliceIndex,
  }: {
    readonly worked: readonly Worked[];
    readonly sliceIndex: number;
  },
): number {
  return worked.findIndex(function isSlice({ facts, },): boolean {
    return facts.sliceIndex === sliceIndex;
  },);
}

/**
 The page and the plans as the step starts from them.

 @param page - the slices that carry English and the text between them

 @param round - the settled slices of the page, each with the plan the floors agreed to

 @param refused - the indexes of the slices the floors refused

 @returns What the step knows of the page

 @example
 ```ts
 const ledger = readmitLedgerOf({ page, round, refused, },);
 ```
 */
export function readmitLedgerOf(
  {
    page,
    round,
    refused,
  }: {
    readonly page: WorkedPage;
    readonly round: readonly SettledSlice[];
    readonly refused: ReadonlySet<number>;
  },
): ReadmitLedger {
  /**
   The slices that carry English and the text the published page carries between them.
   */
  const {
    slices: worked,
    between,
    layout,
  } = page;
  /**
   The plan of each settled slice by its place, mutated only inside this function.
   */
  const plans = new Map<number, SlicePlan>();
  /**
   The places of the slices the floors refused, mutated only inside this function.
   */
  const refusedAt = new Set<number>();
  /**
   What the floors said of each slice, in page order, mutated only inside this function.
   */
  const verdictLog: VerdictEntry[] = [];
  for (const slice of round) {
    /**
     The index of the slice.
     */
    const sliceIndex = sliceIndexOf({ slice, },);
    /**
     Where it stands in the page.
     */
    const position = positionOfSlice({
      worked,
      sliceIndex,
    },);
    if (position === NO_POSITION)
      continue;
    /**
     The plan and the verdict of the floors the slice came to.
     */
    const {
      plan,
      verdict,
    } = slice;
    plans.set(
      position,
      plan,
    );
    if (refused.has(sliceIndex,))
      refusedAt.add(position,);
    if (verdict !== undefined) {
      verdictLog.push({
        position,
        verdict,
      },);
    }
  }
  return {
    worked,
    plans,
    refused: refusedAt,
    gained: worked.map(function gainsOf(slice,): ReadonlyMap<string, GainedSides> {
      return gainedBy({ slice: gainSliceOf({ worked: slice, },), },);
    },),
    between: between ?? blankBetween({ count: worked.length, },),
    ...(layout === undefined) ? {} : { layout, },
    made: worked.map(function madeOf(
      slice,
      position,
    ): MadeSlice {
      /**
       The plan of the slice, absent when it has no replacements.
       */
      const planned = plans.get(position,);
      // A slice the floors refused keeps its wording whole, so the page holds none of its replacements.
      if ((planned === undefined) || refusedAt.has(position,)) {
        return {
          text: slice.currentText,
          edits: [],
        };
      }
      return madeIn({
        slice: gainSliceOf({ worked: slice, },),
        remaining: planned.toMake,
      },);
    },),
    toMake: worked.map(function toMakeOf(
      _slice,
      position,
    ): readonly ConsistencyEdit[] {
      /**
       The plan of the slice, absent when it has no replacements.
       */
      const planned = plans.get(position,);
      return (planned === undefined) ? [] : planned.toMake;
    },),
    dropped: worked.map(function droppedOf(
      _slice,
      position,
    ): readonly ConsistencyEdit[] {
      /**
       The plan of the slice, absent when it has no replacements.
       */
      const planned = plans.get(position,);
      return (planned === undefined) ? [] : planned.unprovenGains;
    },),
    verdictLog,
    reasonLog: [],
    blocked: new Set(),
  };
}

/**
 A splice left out, with the place of the slice it is addressed to.

 @example
 ```ts
 const candidate: Candidate = { position: 1, edit, };
 ```
 */
export type Candidate = {
  /**
   The slice's place in the page.
   */
  readonly position: number;

  /**
   The splice.
   */
  readonly edit: ConsistencyEdit;
};

/**
 The splices left out, in the order of the page, in the slices the floors did not refuse.

 @param ledger - what the step knows of the page

 @returns Each splice with the place of its slice

 @example
 ```ts
 const candidates = candidatesOf({ ledger, },);
 ```
 */
export function candidatesOf({ ledger, }: { readonly ledger: ReadmitLedger; },): readonly Candidate[] {
  /**
   The splices each slice leaves out, and the slices the floors refused.
   */
  const {
    dropped,
    refused,
  } = ledger;
  return dropped.flatMap(function inSlice(
    edits,
    position,
  ): readonly Candidate[] {
    if (refused.has(position,))
      return [];
    return edits
      .toSorted(function inPageOrder(
        left,
        right,
      ): number {
        return left.start - right.start;
      },)
      .map(function placed(edit,): Candidate {
        return {
          position,
          edit,
        };
      },);
  },);
}

/**
 A ledger that goes its own way from this one: the ledgers it keeps are copies, so what one puts back the other does not see.

 @param ledger - what the step knows of the page

 @returns The same page, plans and replacements, with ledgers of its own

 @example
 ```ts
 const copy = clonedLedgerOf({ ledger, },);
 ```
 */
export function clonedLedgerOf({ ledger, }: { readonly ledger: ReadmitLedger; },): ReadmitLedger {
  return {
    ...ledger,
    verdictLog: [...ledger.verdictLog,],
    reasonLog: [...ledger.reasonLog,],
    blocked: new Set(ledger.blocked,),
  };
}

//endregion Pass consistency apply ledger
