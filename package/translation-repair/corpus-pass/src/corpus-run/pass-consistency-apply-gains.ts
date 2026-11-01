import {
  applyConsistencyEdits,
  bringsAtEdge,
  type ConsistencyEdit,
  NO_PLACE,
  type PageLayout,
  pageOfSlices,
  type PageSlice,
  positionOnPage,
  provenApostrophesInEnglish,
} from '@monochromatic-dev/translation-repair-consistency/ts';

//region Pass consistency apply gains
// THE GAIN CASE IS DECIDED ON THE PAGE THE REPLACEMENTS MAKE. A code splice may carry a
// proven apostrophe to an edge where the receiving words held none (`the parents’` over
// `the family`), and the planner (`consistency-splice.ts`) cannot say whether the mark
// is still an apostrophe there: that depends on every single quotation character of the
// receiving page, in any slice of it, and on the other replacements of the pass, since a
// mark carried in by one replacement can be the partner of a mark carried in by another
// (two gains can each be proven alone and neither together, in one paragraph or in two
// slices). So this step looks at the English of the WHOLE PAGE with every replacement of
// the pass made in every slice (`consistency-page-english.ts`), asks which of the
// apostrophes the replacements carried in are proven there
// (`consistency-proven-apostrophes.ts`), leaves out each replacement whose carried
// apostrophe is not, and looks again at the page without them, until no replacement is
// left out. Leaving one out takes its carried mark off the page and can let a replacement
// it overlapped be made, which the next round checks in its turn. A replacement left out
// leaves its place with its words, and every round leaves out at least one replacement or
// ends the choice, so it ends. Only a code splice is checked: the block a writer rewrote
// carries no apostrophe the pass chose, and its words are judged by the floors.
//
// A ROUND LEAVES OUT EVERY UNPROVEN REPLACEMENT AT ONCE, so it can leave out more than it must:
// two gains that pair are each unproven while the other stands, and either of them would have
// stood alone, yet both are left out. The same holds for a gain left out only because of a gain that a floor then
// refused. This step does not undo that; `readmittedSlicesOf`
// (`pass-consistency-apply-readmit.ts`) puts back, one at a time in page order, each
// replacement left out that the final page would carry.
//
// THE PAGE IS THE PUBLISHED PAGE. The optional text between the slices (`between`) is the text
// the archive carries around them, a sealed span among it, so a quotation character there is a
// partner like any other; without it the slices stand a blank line apart.
//
// A SLICE THE FLOORS REFUSE IS NOT THIS FILE'S TO SEE, and the caller names it by handing the
// slice with no replacements: the floors act on whole slices and a slice they refuse keeps its
// wording, so the page the check reads must hold none of its replacements, and the words they
// took out stand again. `settledSlicesOf` (`pass-consistency-apply-plan.ts`) runs the floors
// and this check in one loop until neither changes anything.

/**
 One slice of the page as the check reads it.
 */
export type GainSlice = {
  /**
   Index the artifact names the slice by.
   */
  readonly sliceIndex: number;

  /**
   The English of the slice before any replacement.
   */
  readonly english: string;

  /**
   The replacements addressed to the slice, in priority order, whose offsets address its English.
   */
  readonly edits: readonly ConsistencyEdit[];

  /**
   The names of the replacements that are code splices, the only ones whose carried apostrophe is checked.
   */
  readonly splices: ReadonlySet<string>;
};

/**
 What the check decided for one slice.
 */
export type GainVerdict = {
  /**
   Index the artifact names the slice by.
   */
  readonly sliceIndex: number;

  /**
   The replacements to make, still in priority order.
   */
  readonly kept: readonly ConsistencyEdit[];

  /**
   The replacements left out, in the order they were left out.
   */
  readonly dropped: readonly ConsistencyEdit[];
};

/**
 The sides of a replacement's words at which it brings an edge apostrophe in.
 */
export type GainedSides = readonly ('start' | 'end')[];

/**
 A slice with its replacements made.
 */
export type MadeSlice = {
  /**
   The English with the replacements that were made.
   */
  readonly text: string;

  /**
   The replacements that were made, in priority order.
   */
  readonly edits: readonly ConsistencyEdit[];
};

/**
 The state of the loop that settles the choice.
 */
type Settling = {
  /**
   The verdicts so far, one for each slice, replaced by each round.
   */
  verdicts: readonly GainVerdict[];

  /**
   Whether a round left nothing out.
   */
  settled: boolean;
};

/**
 Whether an edit brings a single mark in at an edge where the words it replaces
 held no edge apostrophe, whatever stands inside the mark (`bringsAtEdge`,
 `consistency-splice-readings.ts`): the page, not the words, says how it reads, so the mark is proven there or the
 edit is left out.

 @param english - the English before any edit

 @param edit - the replacement

 @returns The sides at which the replacement's words hold a single mark and the words they replace hold no edge apostrophe

 @example
 ```ts
 const sides = gainedSidesOf({ english: 'the family', edit: { id: 'a', start: 0, end: 10, text: 'the parents’', }, },); // ['end']
 ```
 */
function gainedSidesOf(
  {
    english,
    edit,
  }: {
    readonly english: string;
    readonly edit: ConsistencyEdit;
  },
): GainedSides {
  /**
   The words the replacement takes out.
   */
  const old = english.slice(
    edit.start,
    edit.end,
  );
  /**
   Both sides of the words, in the order they stand.
   */
  const sides = [
    'start',
    'end',
  ] as const;
  return sides.filter(function isGained(side,): boolean {
    return bringsAtEdge({
      text: edit.text,
      core: old,
      side,
    },);
  },);
}

/**
 Where a made replacement's words stand in the English the replacements made.

 @param edits - the replacements that were made

 @param edit - the replacement asked about

 @returns The offset of its first character in the edited English

 @example
 ```ts
 const at = editedStartOf({ edits, edit, },);
 ```
 */
export function editedStartOf(
  {
    edits,
    edit,
  }: {
    readonly edits: readonly ConsistencyEdit[];
    readonly edit: ConsistencyEdit;
  },
): number {
  return edits
    .filter(function standsBefore(other,): boolean {
      return other.start < edit.start;
    },)
    .reduce(
    function shifted(
      shift,
      other,
    ): number {
        /**
         How many characters longer the replacement's words are than the words they replace.
         */
      const { length, } = other.text;
      return shift + (length - (other.end - other.start));
    },
    edit.start,
    );
}

/**
 The sides each code splice of a slice gains an edge apostrophe on, by the splice's name.

 @param slice - the slice and its replacements

 @returns The sides of every splice, none for a replacement that is no splice

 @example
 ```ts
 const gained = gainedBy({ slice, },);
 ```
 */
export function gainedBy({ slice, }: { readonly slice: GainSlice; },): ReadonlyMap<string, GainedSides> {
  /**
   The sides by name, mutated only inside this function.
   */
  const sides = new Map<string, GainedSides>();
  /**
   The names of the replacements that are code splices.
   */
  const { splices, } = slice;
  for (const edit of slice.edits) {
    if (splices.has(edit.id,)) {
      sides.set(
        edit.id,
        gainedSidesOf({
          english: slice.english,
          edit,
        },),
      );
    }
  }
  return sides;
}

/**
 A slice with the replacements that still stand made.

 @param slice - the slice and its replacements

 @param remaining - the replacements still to be made

 @returns The English with them made and the replacements that were made

 @example
 ```ts
 const { text, edits, } = madeIn({ slice, remaining: slice.edits, },);
 ```
 */
export function madeIn(
  {
    slice,
    remaining,
  }: {
    readonly slice: GainSlice;
    readonly remaining: readonly ConsistencyEdit[];
  },
): MadeSlice {
  /**
   The English with them made, and the names of those that were.
   */
  const result = applyConsistencyEdits({
    english: slice.english,
    edits: remaining,
  },);
  return {
    text: result.text,
    edits: remaining.filter(function wasMade({ id, },): boolean {
      return result.applied
        .includes(id,);
    },),
  };
}

/**
 The code splices whose carried apostrophe is no proven apostrophe on the published page the slices make, in one reading
 of that page.

 @param made - every slice of the page that carries English, in page order, with its replacements made

 @param gained - for each slice, in the order given, the sides each code splice brings an edge apostrophe in on, by name

 @param between - the text the published page carries before the first slice, between each two and after the last, one piece
 more than there are slices; absent where the slices stand a blank line apart; read only where no layout is given

 @param layout - what the published page is assembled from; the page is then assembled by the code that publishes it, with the
 slices' wording written in, and `between` is not read

 @param sliceIndexes - the index the artifact names each slice by, which the layout names its slices by; the slice's place in
 the list by default

 @returns For each slice, in the order given, the made replacements whose carried apostrophe is not proven on that page

 @throws {@link ConsistencyInvariantError} when the text between the slices does not hold one more piece than there are slices

 @example
 ```ts
 const [unproven,] = unprovenOnPage({ made: [{ text: 'Then the parents’ naps.', edits, },], gained, },);
 ```
 */
export function unprovenOnPage(
  {
    made,
    gained,
    between,
    layout,
    sliceIndexes,
  }: {
    readonly made: readonly MadeSlice[];
    readonly gained: readonly ReadonlyMap<string, GainedSides>[];
    readonly between?: readonly string[];
    readonly layout?: PageLayout;
    readonly sliceIndexes?: readonly number[];
  },
): readonly (readonly ConsistencyEdit[])[] {
  /**
   The index a slice of the page is named by: the artifact's, which the layout of the published page names its slices by,
   else the slice's place in the list.

   @param position - the slice's place in the list

   @returns The index
   */
  function indexOf(position: number,): number {
    return sliceIndexes?.[position] ?? position;
  }
  /**
   The English of the whole page with every replacement made, and where each slice starts in it.
   */
  const page = pageOfSlices({
    slices: made.map(function pageSliceOf(
      { text, },
      position,
    ): PageSlice {
      return {
        sliceIndex: indexOf(position,),
        english: text,
      };
    },),
    ...(between === undefined) ? {} : { between, },
    ...(layout === undefined) ? {} : { layout, },
  },);
  /**
   The proven apostrophes of that page.
   */
  const {
    leading,
    trailing,
  } = provenApostrophesInEnglish({ english: page.text, },);
  return made.map(function unprovenIn(
    {
      edits: madeEdits,
      text: written,
    },
    position,
  ): readonly ConsistencyEdit[] {
    /**
     Where this slice starts in the page, none where the page writes nothing for it.
     */
    const offset = page.offsets
      .get(indexOf(position,),);
    /**
     What the page holds for the slice, which is the replacements made unless typography or a page-assembly pass rewrote it.
     */
    const shipped = page.shipped
      .get(indexOf(position,),);
    return madeEdits.filter(function carriesUnprovenMark(edit,): boolean {
      /**
       The sides the replacement brings an apostrophe in on.
       */
      const sides = gained[position]
        ?.get(edit.id,)
        ?? [];
      if (offset === undefined)
        return sides.length > 0;
      /**
       Where the replacement's words start in the slice's English as made.
       */
      const startInSlice = editedStartOf({
        edits: madeEdits,
        edit,
      },);
      /**
       How long the replacement's words are.
       */
      const { length: textLength, } = edit.text;
      /**
       Where the replacement's words end in the slice's English as made, exclusive.
       */
      const endInSlice = startInSlice + textLength;
      /**
       Where the replacement's words stand on the page, {@link NO_PLACE} where a page-assembly pass changed the character there.
       */
      const start = positionOnPage({
        written,
        shipped: shipped ?? written,
        offset,
        at: startInSlice,
      },);
      /**
       Where its last character stands there.
       */
      const last = positionOnPage({
        written,
        shipped: shipped ?? written,
        offset,
        at: endInSlice - 1,
      },);
      return sides.some(function isUnproven(side,): boolean {
        return (side === 'start')
          ? ((start === NO_PLACE) || (!leading.has(start,)))
          : ((last === NO_PLACE) || (!trailing.has(last,)));
      },);
    },);
  },);
}

/**
 The replacements of the slices of a page without the code splices whose carried apostrophe would
 not be one on the page the remaining replacements make.

 @param slices - every slice of the page that carries English, in page order, with the replacements addressed to it; a slice with none is read as it stands

 @param between - the text the published page carries before the first slice, between each two and after the last, one piece
 more than there are slices; absent where the slices stand a blank line apart; read only where no layout is given

 @param layout - what the published page is assembled from; the page is then assembled by the code that publishes it, with the
 slices' wording written in, and `between` is not read

 @returns For each slice, in the order given, the replacements to make, still in priority order, and those left out, in the order they were left out

 @throws {@link Error} when a replacement's span lies outside the English or is empty

 @throws {@link ConsistencyInvariantError} when the text between the slices does not hold one more piece than there are slices

 @example
 ```ts
 const [verdict,] = withoutUnprovenGains({ slices: [{ sliceIndex: 0, english, edits, splices: new Set(['a',],), },], },);
 ```
 */
export function withoutUnprovenGains(
  {
    slices,
    between,
    layout,
  }: {
    readonly slices: readonly GainSlice[];
    readonly between?: readonly string[];
    readonly layout?: PageLayout;
  },
): readonly GainVerdict[] {
  /**
   The index each slice is named by, which the layout of the published page names its slices by.
   */
  const sliceIndexes = slices.map(function indexOfSlice({ sliceIndex, },): number {
    return sliceIndex;
  },);
  /**
   The sides each code splice gains, so a page that no replacement gains an apostrophe on is never read.
   */
  const gained = slices.map(function gainsOf(slice,): ReadonlyMap<string, GainedSides> {
    return gainedBy({ slice, },);
  },);
  /**
   The verdicts so far, replaced by each round of the loop that settles the choice.
   */
  const state: Settling = {
    verdicts: slices.map(function undecided({
      sliceIndex,
      edits,
    },): GainVerdict {
      return {
        sliceIndex,
        kept: edits,
        dropped: [],
      };
    },),
    settled: false,
  };
  while (!state.settled) {
    /**
     Each slice with the replacements still to be made made.
     */
    const made = slices.map(function madeOf(
      slice,
      position,
    ): MadeSlice {
      return madeIn({
        slice,
        remaining: state.verdicts[position]
          ?.kept
          ?? [],
      },);
    },);
    /**
     Whether any replacement still to be made brings an apostrophe in.
     */
    const mayGain = made.some(function holdsGain(
      { edits, },
      position,
    ): boolean {
      return edits.some(function gains({ id, },): boolean {
        return (gained[position]
          ?.get(id,)
          ?.length
          ?? 0)
          > 0;
      },);
    },);
    if (!mayGain) {
      // A page that no remaining replacement gains an apostrophe on is never read.
      state.settled = true;
      continue;
    }
    /**
     In each slice, the splices whose carried apostrophe is no proven apostrophe on the page the replacements make.
     */
    const unproven = unprovenOnPage({
      made,
      gained,
      sliceIndexes,
      ...(between === undefined) ? {} : { between, },
      ...(layout === undefined) ? {} : { layout, },
    },);
    if (unproven.every(function none(edits,): boolean {
      return edits.length === 0;
    },)) {
      state.settled = true;
      continue;
    }
    state.verdicts = state.verdicts
      .map(function without(
        verdict,
        position,
      ): GainVerdict {
        /**
         The replacements of this slice left out in this round.
         */
        const left = unproven[position] ?? [];
        return {
          sliceIndex: verdict.sliceIndex,
          kept: verdict.kept
            .filter(function stays(edit,): boolean {
              return !left.includes(edit,);
            },),
          dropped: [
            ...verdict.dropped,
            ...left,
          ],
        };
      },);
  }
  return state.verdicts;
}

//endregion Pass consistency apply gains
