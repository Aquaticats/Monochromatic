import {
  type ConsistencyEdit,
  ConsistencyInvariantError,
  type PageLayout,
  pageOfSlices,
  pageReadingOf,
  type PageSlice,
} from '@monochromatic-dev/translation-repair-consistency/ts';
import {
  editsChangingLocatedWords,
  type LeftOutEdit,
} from './pass-consistency-apply-located.ts';
import {
  type SettledIndex,
  settledIndexOf,
} from './pass-consistency-apply-index.ts';
import {
  editId,
  pendingNamed,
  type SliceEdits,
  sliceEditsOf,
} from './pass-consistency-apply-record.ts';
import type { ConsistencySliceFacts, } from './pass-consistency-slices.ts';
import type { SettledGroup, } from './pass-consistency-types.ts';

//region Pass consistency apply worked
// THE PAGE AS THE PLAN READS IT, read once. Which replacements a slice has, which of them change the
// words another group located, and which are code splices depend on the groups and on the English the
// pass began with, and on nothing the floors decide later, so the plan reads them once for the pass
// (`pass-consistency-apply-plan.ts`) and not once for each round of the floors. The text the published
// page carries between the slices is read off the page the facts carry (`pass-consistency-slices.ts`).

/**
 The replacements that change the words another group located and that group does not replace, which are not made.
 */
export type LeftOut = readonly {
  /**
   Name of the replacement.
   */
  readonly id: string;

  /**
   The place of the other group whose located words it changes.
   */
  readonly other: LeftOutEdit['other'];
}[];

/**
 One slice that carries English, with its replacements before the check of the apostrophes they carry in.
 */
export type Worked = {
  /**
   What the pass knows of the slice.
   */
  readonly facts: ConsistencySliceFacts;

  /**
   The wording the slice carries now.
   */
  readonly currentText: string;

  /**
   The slice's replacements, the places each carries and the group each belongs to.
   */
  readonly named: SliceEdits;

  /**
   The replacements that change the words another group located and that group does not replace.
   */
  readonly leftOut: LeftOut;

  /**
   The replacements that are not left out for changing another group's located words.
   */
  readonly candidates: readonly ConsistencyEdit[];

  /**
   The names of the candidates that are code splices.
   */
  readonly splices: ReadonlySet<string>;
};

/**
 The slices of a page that carry English, with the text the published page carries around them.
 */
export type WorkedPage = {
  /**
   The slices that carry English, in page order.
   */
  readonly slices: readonly Worked[];

  /**
   The text the published page carries before the first of them, between each two and after the last, one piece more than
   there are slices; absent when the facts carry no page, whose slices then stand a blank line apart.
   */
  readonly between?: readonly string[];

  /**
   What the published page is assembled from, when the facts of every slice carry the same: the proof of an apostrophe writes
   the wording of the slices into it with the code that publishes the page, so `between` is read off the page as the slices
   stand now and serves only the screen's reading of what stands beside a mark. Absent when the facts carry none.
   */
  readonly layout?: PageLayout;
};

/**
 The names of the replacements that are code splices.

 @param named - a slice's replacements with the pending replacement under each name

 @param candidates - the replacements to look at

 @returns The names of those that are code splices

 @example
 ```ts
 const splices = splicesAmong({ named, candidates, },);
 ```
 */
function splicesAmong(
  {
    named,
    candidates,
  }: {
    readonly named: SliceEdits;
    readonly candidates: readonly ConsistencyEdit[];
  },
): ReadonlySet<string> {
  return new Set(candidates
    .filter(function isSplice({ id, },): boolean {
      /**
       The pending replacement under this name.
       */
      const { edit, } = pendingNamed({
        pendings: named.pendings,
        id,
      },);
      return edit.kind === 'splice';
    },)
    .map(function idOf({ id, },): string {
      return id;
    },),);
}

/**
 The slice that carries English, with its replacements and the ones another group's located words leave out.

 @param index - the page's groups read by slice

 @param facts - what the pass knows of the slice

 @returns The slice as a list of one, none when the slice carries no English

 @throws {@link ConsistencyInvariantError} when two replacements of one group overlap in the slice

 @example
 ```ts
 const [worked,] = workedOn({ index, facts, },);
 ```
 */
function workedOn(
  {
    index,
    facts,
  }: {
    readonly index: SettledIndex;
    readonly facts: ConsistencySliceFacts;
  },
): readonly Worked[] {
  if (facts.current
    .kind
    !== 'present')
    return [];
  /**
   The wording the slice carries now.
   */
  const currentText = facts.current
    .text;
  /**
   This slice's replacements, the places each carries and the group each belongs to.
   */
  const named = sliceEditsOf({
    index,
    sliceIndex: facts.sliceIndex,
  },);
  /**
   The replacements that change the words another group located and that group does not replace.
   */
  const leftOut = (named.edits
    .length
    === 0)
    ? []
    : editsChangingLocatedWords({
      index,
      sliceIndex: facts.sliceIndex,
      english: currentText,
      ...((facts.page === undefined) ? {} : { page: facts.page, }),
    },)
      .map(function namedLeftOut({
        group,
        edit,
        other,
      },): LeftOut[number] {
        return {
          id: editId({
            group,
            places: edit.placePositions,
          },),
          other,
        };
      },);
  /**
   The replacements that are not left out for changing another group's located words.
   */
  const candidates = named.edits
    .filter(function isCandidate({ id, },): boolean {
      return !leftOut.some(function isLeftOut(left,): boolean {
        return left.id === id;
      },);
    },);
  return [
    {
      facts,
      currentText,
      named,
      leftOut,
      candidates,
      splices: splicesAmong({
        named,
        candidates,
      },),
    },
  ];
}

/**
 The text the published page carries around the slices that carry English, read off the page the facts carry.

 @param worked - the slices that carry English, in page order

 @returns The text before the first slice, between each two and after the last, none when a slice carries no page or the
 slices carry different pages, which is how facts built without a page look

 @throws {@link ConsistencyInvariantError} when a slice does not lie inside its page, or starts before the slice before it ends

 @example
 ```ts
 const between = textAroundWorked({ worked, },);
 ```
 */
function textAroundWorked(
  { worked, }: { readonly worked: readonly Worked[]; },
): Pick<WorkedPage, 'between' | 'layout'> {
  /**
   The page of the first slice.
   */
  const [first,] = worked;
  if (first === undefined)
    return {};
  /**
   The facts of the first slice, which name the page.
   */
  const { facts: firstFacts, } = first;
  /**
   What the published page is assembled from, when the facts carry it.
   */
  const { layout, } = firstFacts;
  if (layout !== undefined) {
    if (worked.some(function holdsOtherLayout({ facts, },): boolean {
      return facts.layout !== layout;
    },))
      return {};
    /**
     The text around the slices on the page assembled with the wording they carry now.
     */
    const { around, } = pageOfSlices({
      slices: worked.map(function sliceOf({
        facts,
        currentText,
      },): PageSlice {
        return {
          sliceIndex: facts.sliceIndex,
          english: currentText,
        };
      },),
      layout,
    },);
    return {
      layout,
      between: around,
    };
  }
  /**
   The page the first slice belongs to.
   */
  const { page: firstPage, } = firstFacts;
  if (firstPage === undefined)
    return {};
  /**
   The text of that page.
   */
  const { text: pageText, } = firstPage;
  /**
   The pieces cut so far, mutated only inside this function.
   */
  const pieces: string[] = [];
  /**
   Where the slice before ended in the page, mutated only inside this function.
   */
  const reached = { end: 0, };
  for (const slice of worked) {
    /**
     What the pass knows of the slice and the wording it carries now.
     */
    const {
      facts,
      currentText,
    } = slice;
    /**
     The page this slice belongs to.
     */
    const { page, } = facts;
    if (page?.text !== pageText)
      return {};
    /**
     The slice's English and where it starts in the page, which the page must hold.
     */
    const {
      offset,
      shipped,
    } = pageReadingOf({
      english: currentText,
      page,
    },);
    if (offset < reached.end) {
      throw new ConsistencyInvariantError({ invariant:
        `slice ${String(facts.sliceIndex,)} starts at ${String(offset,)} in the page, before ${
          String(reached.end,)
        }, where the slice before it ends, so the slices do not follow each other in the page`, },);
    }
    pieces.push(pageText.slice(
      reached.end,
      offset,
    ),);
    reached.end = offset + (shipped ?? currentText).length;
  }
  pieces.push(pageText.slice(reached.end,),);
  return { between: pieces, };
}

/**
 Reads the slices of a page that carry English, with their replacements and the text the published page carries around
 them.

 @param settled - the groups in page order, with their pending replacements

 @param slices - what the pass knows of each slice, in page order

 @returns The slices that carry English and the text around them

 @throws {@link ConsistencyInvariantError} when two replacements of one group overlap in a slice, or the facts' page does
 not hold the slices in order

 @example
 ```ts
 const page = workedPageOf({ settled, slices, },);
 ```
 */
export function workedPageOf(
  {
    settled,
    slices,
  }: {
    readonly settled: readonly SettledGroup[];
    readonly slices: readonly ConsistencySliceFacts[];
  },
): WorkedPage {
  /**
   The groups read by slice, once for the whole page.
   */
  const index = settledIndexOf({ settled, },);
  /**
   Each slice that carries English with its replacements.
   */
  const worked = slices.flatMap(function workedIn(facts,): readonly Worked[] {
    return workedOn({
      index,
      facts,
    },);
  },);
  /**
   The text around them.
   */
  const around = textAroundWorked({ worked, },);
  return {
    slices: worked,
    ...around,
  };
}

//endregion Pass consistency apply worked
