import { compareCodePoints, } from '@monochromatic-dev/translation-repair-markup/ts';
import {
  type CarryBlock,
  ConsistencyInvariantError,
  type SpliceOnlyKind,
} from '@monochromatic-dev/translation-repair-consistency/ts';
import { crossingOfUnit, } from './pass-consistency-carry-crossing.ts';
import type { ConsistencyPlace, } from './pass-consistency-group.ts';
import type { ConsistencySliceFacts, } from './pass-consistency-slices.ts';
import type {
  CarryReceiver,
  CarryUnit,
  DecidedGroup,
  UnwritableReason,
} from './pass-consistency-types.ts';

//region Pass consistency carry units
// WHICH RECEIVING PLACES ARE CARRIED TO TOGETHER. A rewrite of a block carries
// every receiving place the block holds, and the splice of the chosen words
// moves every one of them, so receiving places are grouped by the stretch of
// English that holds them: the innermost block, or, for words that lie in no
// single block, the whole English of the slice, where only the code splice and
// the current English stand for the place. A place of the group that is not a
// receiving place (the chosen place, a place the majority keeps for its context,
// a place that already reads as the chosen place does) keeps its words inside a
// block that is rewritten: they are listed with the unit, and a rewrite that
// changes them is withheld. So do the located words of every place of every
// OTHER group of the page that stand in the block: another group's carry may
// replace them, and that is settled where the edits are applied, but this
// group's rewrite of a block never takes them away unasked.
//
// A TABLE, A CODE BLOCK, A SETEXT HEADING OR A PREFORMATTED BLOCK HOLDS NO
// PARAGRAPH A WRITER CAN REWRITE. The unit of such a place is the whole English,
// as for words that lie in no block, and it names the kind, so the finding says
// what the place stood in.
//
// A BLOCK THAT OVERLAPS THE WORDS OF A RECEIVING PLACE OUTSIDE IT THAT HAS A
// SPLICE IS NOT WRITABLE. A place whose words lie in no single block (they cross a
// paragraph boundary) is carried by its splice, which may run into the next
// paragraph; a rewrite of that paragraph for another place of the group would then
// address the same words as the splice, and two replacements of one group may not
// (`pass-consistency-apply.ts`). The unit keeps its real block and is marked not
// writable, so its places are carried by their splices or keep their English, and
// the finding says the paragraph overlaps another receiving place. A receiving
// place whose plan is `unavailable` has no splice and so no edit that could meet a
// rewrite: it is left out of this test, because the plan is known before any writer
// is asked and a place no edit addresses cannot make the block unwritable. A place
// whose words cross a block boundary has that plan whenever the chosen words hold a
// different number of block breaks than its own words do (`consistency-block-breaks.ts`),
// since no page with such a splice passes the structure floor, so only a place whose
// chosen words cross as many boundaries as its own can make its neighbour unwritable.
// What keeps that place's record true is the unit's `crossing` list: the stretch of its words that lies inside the
// block, with the edge of the block it stands at, which a writer's paragraph must hold there
// (`pass-consistency-carry-crossing.ts`).
//
// THE WORDS OF A PLACE THAT MUST STAY AND RUN OVER AN EDGE OF THE BLOCK. A place of the
// group that is not a receiving place (the chosen place, a place the majority keeps for
// its context, a place that already reads as the chosen place does) is listed in
// `keeping` only when its words lie wholly inside the block. When they run over an edge
// of the block, the stretch inside the block is listed in `crossing` instead, with the
// edge it stands at, exactly as for a receiving place whose plan is `unavailable`. A
// receiving place whose plan is `replace` is in neither list, since its words are
// replaced, and the located words of a place of another group are in neither list
// either, since the apply stage holds them (`pass-consistency-apply-located.ts`).

/**
 A receiving place with the block of its English that holds its words.

 @example
 ```ts
 const held: HeldReceiver = { receiver, block: [], };
 ```
 */
export type HeldReceiver = {
  /**
   The receiving place.
   */
  readonly receiver: CarryReceiver;

  /**
   The innermost block of its English that holds its words, as a list of zero
   or one: none where the words lie in no single block.
   */
  readonly block: readonly CarryBlock[];

  /**
   The kind of block (a table, a code block, a setext heading, a preformatted
   block) that holds its words where no paragraph rewrite can carry words to
   them, as a list of zero or one.
   */
  readonly spliceOnlyIn: readonly SpliceOnlyKind[];
};

/**
 Words of another place of the group, with where they stand.

 @example
 ```ts
 const kept: KeptWords = { sliceIndex: 0, start: 4, core: 'The cat naps', };
 ```
 */
export type KeptWords = {
  /**
   Slice whose English holds the words.
   */
  readonly sliceIndex: number;

  /**
   Offset in that English where the words start.
   */
  readonly start: number;

  /**
   The words.
   */
  readonly core: string;

  /**
   Present when the words are the located words of a place of another group of the page, and absent when they are
   those of a place of this group. A stretch of this group's words that runs over an edge of a block is held at the
   edge by the block's unit; one of another group's is held where the groups' replacements meet.
   */
  readonly ofOtherGroup?: true;
};

/**
 Provisional outcomes of a place whose words a carry must leave as they are.
 */
const KEPT_OUTCOMES: ReadonlySet<string> = new Set([
  'chosen',
  'kept-context',
  'consistent',
],);

/**
 Whether a place with this provisional outcome keeps its words: the chosen
 place, a place the majority keeps for its context, or a place that already
 reads as the chosen place does.

 @param outcome - the place's provisional outcome

 @returns Whether a carry must leave the place's words as they are

 @example
 ```ts
 const keeps = isKeptOutcome({ outcome: 'chosen', },); // true
 ```
 */
export function isKeptOutcome({ outcome, }: { readonly outcome: string; },): boolean {
  return KEPT_OUTCOMES.has(outcome,);
}

/**
 The words of the group's places that a carry must leave as they are: the
 chosen place, the places the majority keeps for their context and the places
 that already read as the chosen place does, each as its located rendering.

 The words of every place of every other group of the page that has a located
 rendering are listed with them, whatever became of the place: that group's
 carry may replace them, and this group's rewrite of a block must not. Those
 words are marked as another group's.

 @param decided - the group with its renderings and provisional outcomes

 @param others - the page's other groups, in any order; the result does not depend on it

 @returns The words with their slice and offset, in page order and without
 repeats; where this group and another locate the very same words, the entry is this group's

 @example
 ```ts
 const kept = keptWordsOf({ decided, others: [], },);
 ```
 */
export function keptWordsOf(
  {
    decided,
    others = [],
  }: {
    readonly decided: DecidedGroup;
    readonly others?: readonly DecidedGroup[];
  },
): readonly KeptWords[] {
  /**
   The words of this group that stay, then the located words of every other group.
   */
  const listed: readonly KeptWords[] = [
    ...decided.places
      .flatMap(function keptOf(
        place,
        position,
      ): readonly KeptWords[] {
        /**
         This place's rendering.
         */
        const rendering = decided.renderings[position];
        /**
         This place's provisional outcome.
         */
        const outcome = decided.outcomes[position];
        if (rendering === undefined)
          return [];
        if (rendering.kind !== 'located')
          return [];
        if (outcome === undefined)
          return [];
        if (!isKeptOutcome({ outcome, },))
          return [];
        return [
          {
            sliceIndex: place.slice
              .sliceIndex,
            start: rendering.start,
            core: rendering.core,
          },
        ];
      },),
    ...others.flatMap(function locatedOf(other,): readonly KeptWords[] {
      return other.places
        .flatMap(function locatedPlaceOf(
          place,
          position,
        ): readonly KeptWords[] {
          /**
           This place's rendering.
           */
          const rendering = other.renderings[position];
          return (rendering?.kind === 'located')
            ? [
              {
                sliceIndex: place.slice
                  .sliceIndex,
                start: rendering.start,
                core: rendering.core,
                ofOtherGroup: true,
              },
            ]
            : [];
        },);
    },),
  ];
  return listed
    .toSorted(function inPageOrder(
      left,
      right,
    ): number {
      return (left.sliceIndex - right.sliceIndex) || (left.start - right.start)
        || compareCodePoints({
          left: left.core,
          right: right.core,
        },);
    },)
    .filter(function isFirstOfItsKind(
      words,
      at,
      sorted: readonly KeptWords[],
    ): boolean {
      /**
       The words just before these in page order.
       */
      const before = sorted[at - 1];
      if (before === undefined)
        return true;
      return (before.sliceIndex !== words.sliceIndex)
        || (before.start !== words.start)
        || (before.core !== words.core);
    },);
}

/**
 The English of a place's slice, which a located place has.

 @param place - the place

 @returns The slice's English

 @throws {@link ConsistencyInvariantError} when the slice carries none, since a rendering is located only in English

 @example
 ```ts
 const english = englishOf({ place, },);
 ```
 */
export function englishOf({ place, }: { readonly place: ConsistencyPlace; },): string {
  if (place.slice
    .current
    .kind
    === 'absent') {
    throw new ConsistencyInvariantError({ invariant:
      `slice ${String(place.slice
        .sliceIndex,)} carries no English, and a rendering is located only in English`, },);
  }
  return place.slice
    .current
    .text;
}

/**
 The slice that holds the receiving places of a unit, which they share.

 @param unit - the unit

 @returns The slice's facts

 @throws {@link ConsistencyInvariantError} when the unit has no receiving place

 @example
 ```ts
 const facts = sliceOfUnit({ unit, },);
 ```
 */
export function sliceOfUnit({ unit, }: { readonly unit: CarryUnit; },): ConsistencySliceFacts {
  /**
   The first receiving place, whose slice every other shares.
   */
  const [first,] = unit.receivers;
  if (first === undefined) {
    throw new ConsistencyInvariantError({ invariant:
      'a unit has at least one receiving place, and a unit is made from a group of receivers', },);
  }
  return first.place
    .slice;
}

/**
 The English of the slice that holds the receiving places of a unit.

 @param unit - the unit

 @returns The slice's English

 @throws {@link ConsistencyInvariantError} when the unit has no receiving place or its slice carries no English

 @example
 ```ts
 const english = englishOfUnit({ unit, },);
 ```
 */
export function englishOfUnit({ unit, }: { readonly unit: CarryUnit; },): string {
  /**
   The first receiving place, whose slice every other shares.
   */
  const [first,] = unit.receivers;
  if (first === undefined) {
    throw new ConsistencyInvariantError({ invariant:
      'a unit has at least one receiving place, and a unit is made from a group of receivers', },);
  }
  return englishOf({ place: first.place, },);
}

/**
 The labels of the receiving places of a unit, for the findings that name them.

 @param unit - the unit

 @returns The labels joined by `and`, in page order

 @example
 ```ts
 const named = unitLabelsOf({ unit, },); // 'OCCURRENCE 2 and OCCURRENCE 3'
 ```
 */
export function unitLabelsOf({ unit, }: { readonly unit: CarryUnit; },): string {
  return unit.receivers
    .map(function labelOf({ label, },): string {
      return label;
    },)
    .join(' and ',);
}

/**
 The name a stretch of English is grouped under: its slice and its offsets, or
 its slice alone for the whole English.

 @param held - a receiving place with its block

 @returns The grouping key

 @example
 ```ts
 const key = unitKeyOf({ held, },); // '1:0:34'
 ```
 */
function unitKeyOf({ held, }: { readonly held: HeldReceiver; },): string {
  /**
   The slice that holds the place.
   */
  const slice = String(held.receiver
    .place
    .slice
    .sliceIndex,);
  /**
   The block, none where the words lie in no single block.
   */
  const [block,] = held.block;
  if (block !== undefined)
    return `${slice}:${String(block.start,)}:${String(block.end,)}`;
  /**
   The kind of block that holds the words where no paragraph can, none for words in no block.
   */
  const [kind,] = held.spliceOnlyIn;
  return `${slice}:whole:${kind ?? ''}`;
}

/**
 The stretch of English a group of held receivers is carried to.

 @param members - the receivers of one stretch, in page order

 @returns The block of their English, whether a writer can be given it, the kind of block no paragraph rewrite can carry words into that holds the words, and, as a list of zero or one, why a unit in no such kind is not writable

 @throws {@link ConsistencyInvariantError} when the receivers' slice carries no English

 @example
 ```ts
 const { block, writable, } = stretchOf({ members, },);
 ```
 */
function stretchOf(
  { members, }: { readonly members: readonly HeldReceiver[]; },
): {
  readonly block: CarryBlock;
  readonly writable: boolean;
  readonly spliceOnlyIn: readonly SpliceOnlyKind[];
  readonly unwritable: readonly UnwritableReason[];
} {
  /**
   The first receiver, whose slice and block every member shares.
   */
  const [first,] = members;
  if (first === undefined) {
    throw new ConsistencyInvariantError({ invariant:
      'a stretch of English is carried to at least one receiving place, and a unit is made from a group of receivers', },);
  }
  /**
   The block, none where the words lie in no single block.
   */
  const [block,] = first.block;
  if (block !== undefined) {
    return {
      block,
      writable: true,
      spliceOnlyIn: [],
      unwritable: [],
    };
  }
  /**
   The kind of block that holds the words, none where no kind does.
   */
  const [kind,] = first.spliceOnlyIn;
  /**
   The whole English of the receivers' slice.
   */
  const english = englishOf({
    place: first.receiver
      .place,
  },);
  return {
    block: {
      start: 0,
      end: english.length,
      text: english,
    },
    writable: false,
    spliceOnlyIn: first.spliceOnlyIn,
    unwritable: (kind === undefined) ? ['no-single-block',] : [],
  };
}

/**
 Whether a block overlaps the located words of a receiving place that is carried
 apart from it by a splice. A block and the splice of such a place would address
 the same words, and two replacements of one group may not. A receiving place
 whose plan is not a replacement has no splice and is not counted.

 @param block - the block of a unit

 @param members - the unit's own receiving places

 @param held - every receiving place of the group with its block

 @returns Whether the words of a receiving place outside the unit that has a splice lie, wholly or in part, inside the block

 @example
 ```ts
 const overlapped = overlapsOutsideReceiver({ block, sliceIndex: 1, members, held, },);
 ```
 */
function overlapsOutsideReceiver(
  {
    block,
    members,
    held,
  }: {
    readonly block: CarryBlock;
    readonly members: readonly HeldReceiver[];
    readonly held: readonly HeldReceiver[];
  },
): boolean {
  /**
   The first member of the unit, whose slice every member shares.
   */
  const [first,] = members;
  if (first === undefined)
    return false;
  /**
   The slice the unit stands in.
   */
  const { sliceIndex, } = first.receiver
    .place
    .slice;
  return held.some(function reachesBlock({ receiver, },): boolean {
    /**
     Offset where the place's located words start, and the words themselves.
     */
    const {
      start,
      core,
    } = receiver.rendering;
    /**
     Exclusive offset where they end.
     */
    const end = start + core.length;
    /**
     Whether the place is one of the unit's own receiving places.
     */
    const isMember = members.some(function sameReceiver(member,): boolean {
      return member.receiver === receiver;
    },);
    /**
     The slice the place stands in.
     */
    const { sliceIndex: placeSlice, } = receiver.place
      .slice;
    if (isMember)
      return false;
    if (placeSlice !== sliceIndex)
      return false;
    // A PLACE WITH NO SPLICE HAS NO EDIT, so nothing addresses its words and a rewrite of the block cannot meet one. The
    // plan was made before any writer was asked (`pass-consistency-decide.ts`) and is read here as it stands.
    /**
     How the chosen words are moved into the place.
     */
    const { plan, } = receiver;
    if (plan.kind !== 'replace')
      return false;
    return (start < block.end) && (block.start < end);
  },);
}

/**
 Groups the receiving places of a group by the stretch of English that holds
 them.

 @param held - every receiving place with its block, in page order

 @param kept - the words of the group's other places that must stay as they are

 @returns One unit for each stretch, in the page order of its first receiving place

 @throws {@link ConsistencyInvariantError} when a slice carries no English

 @example
 ```ts
 const units = unitsOf({ held, kept: [], },);
 ```
 */
export function unitsOf(
  {
    held,
    kept,
  }: {
    readonly held: readonly HeldReceiver[];
    readonly kept: readonly KeptWords[];
  },
): readonly CarryUnit[] {
  /**
   Receivers of each stretch, keyed by the stretch, in the order the stretches are first met.
   */
  const stretches = new Map<string, HeldReceiver[]>();
  for (const entry of held) {
    /**
     The stretch this receiver's words stand in.
     */
    const key = unitKeyOf({ held: entry, },);
    /**
     The receivers already met in this stretch.
     */
    const members = stretches.get(key,);
    if (members === undefined) {
      stretches.set(
        key,
        [entry,],
      );
    }
    else {
      members.push(entry,);
    }
  }
  return [...stretches.values(),]
    .map(function unitOf(members,): CarryUnit {
      /**
       The stretch the members are carried to.
       */
      const {
        block,
        writable,
        spliceOnlyIn,
        unwritable,
      } = stretchOf({ members, },);
      /**
       The slice the members stand in.
       */
      const sliceIndex = members[0]
        ?.receiver
        .place
        .slice
        .sliceIndex;
      /**
       Whether no receiving place outside the unit has its words in part inside the block; where one has, the block's
       rewrite and that place's splice would address the same words.
       */
      const overlapped = overlapsOutsideReceiver({
        block,
        members,
        held,
      },);
      /**
       Whether the block is one a writer could be given and the overlap is what takes it away.
       */
      const overlapTakesIt = writable && overlapped;
      /**
       Why the unit is not writable, as a list of zero or one: the overlap, or what the stretch says.
       */
      const [unwritableBecause,] = overlapTakesIt ? ['overlaps-outside-receiver',] as const : unwritable;
      /**
       Whether the unit stays writable.
       */
      const stays = !overlapTakesIt;
      return {
        block,
        writable: writable && stays,
        ...((unwritableBecause === undefined) ? {} : { unwritableBecause, }),
        spliceOnlyIn,
        receivers: members.map(function receiverOf({ receiver, },): CarryReceiver {
          return receiver;
        },),
        crossing: crossingOfUnit({
          block,
          members,
          held,
          kept,
        },),
        keeping: kept
          .filter(function standsInBlock(
            {
              sliceIndex: wordsSlice,
              start: wordsStart,
              core,
            },
          ): boolean {
            return (wordsSlice === sliceIndex) && (block.start <= wordsStart)
              && ((wordsStart + core.length) <= block.end);
          },)
          .map(function coreOf({ core, },): string {
            return core;
          },),
      };
    },);
}

//endregion Pass consistency carry units
