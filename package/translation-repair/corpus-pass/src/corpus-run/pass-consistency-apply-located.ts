import {
  consistencyLabel,
  type PageEnglish,
  standsOf,
} from '@monochromatic-dev/translation-repair-consistency/ts';
import { pageDefinitionsOf, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  replacedKeyOf,
  type SettledIndex,
} from './pass-consistency-apply-index.ts';
import type { PendingEdit, } from './pass-consistency-types.ts';

//region Pass consistency apply located
// A REPLACEMENT MUST NOT CHANGE THE WORDS ANOTHER GROUP LOCATED, UNLESS THAT
// GROUP IS REPLACING THEM ITSELF. A writer's block is written for one group,
// with the words of the other groups listed as words to keep, and the candidate
// is withheld when it loses them. The groups settle at the same time, though,
// and a writer is only asked to keep, so the replacement is checked again where
// all the groups' replacements meet: a block replacement that covers a located
// place of another group and changes its words is left out. The test reads the
// English as it stood when the pass began and the located words of every group,
// so it does not depend on the order the groups come in.
//
// A CODE SPLICE IS HELD TO THE PLACES ANOTHER GROUP DOES NOT REPLACE. It
// replaces the words of its own receiving places, so it can change another
// group's words only where they overlap. Where the other group has a pending
// replacement of its own over that place in the slice, the two replacements
// meet and the earlier group wins (`pass-consistency-apply.ts`). Where it has
// none (the place is its chosen place, a place the majority keeps for its
// context, a place that already reads as the chosen one, a place whose carry
// ended with its own English standing, or any other place no replacement of
// the group carries), no group's replacement is meant to change the place, so
// the splice that does is left out. What a place's group replaces is read from
// the group's pending replacements, never from the name of the place's
// provisional outcome.
//
// A SPLICE THAT IS MADE NEVER LEAVES A PLACE IT CHANGED UNRECORDED. The place a
// splice is spared is carried by a replacement of its group that holds the
// place's words, so the two replacements overlap and the later of them is left
// out (`pass-consistency-apply.ts`), recording the place `overlapped`.

/**
 A located place of one group, with where its words start.

 @example
 ```ts
 const place: ChangedPlace = { group: 1, position: 2, label: 'OCCURRENCE 3', start: 26, };
 ```
 */
type ChangedPlace = {
  /**
   Position of the place's group on the page, from zero.
   */
  readonly group: number;

  /**
   Position of the place among its group's places, from zero.
   */
  readonly position: number;

  /**
   Label of the place.
   */
  readonly label: string;

  /**
   Offset in the slice's English where the place's located words start.
   */
  readonly start: number;
};

/**
 A replacement left out for changing words another group located, with the
 place whose words it changes.

 @example
 ```ts
 const left: LeftOutEdit = { group: 0, edit, other: { group: 1, label: 'OCCURRENCE 3', start: 26, }, };
 ```
 */
export type LeftOutEdit = {
  /**
   Position of the replacement's group on the page, from zero.
   */
  readonly group: number;

  /**
   The replacement.
   */
  readonly edit: PendingEdit;

  /**
   The place of another group whose words it changes.
   */
  readonly other: {
    /**
     Position of that group on the page, from zero.
     */
    readonly group: number;

    /**
     Label of the place.
     */
    readonly label: string;

    /**
     Offset in the slice's English where the place's located words start.
     */
    readonly start: number;
  };
};

/**
 Whether a replacement changes located words: it covers part of them, or
 covers all of them and holds them fewer times than the text it replaces.

 @param edit - the replacement

 @param english - the slice's English as it stood when the pass began

 @param words - the located words, with where they start

 @param definitions - the page's link and footnote definitions (`pageDefinitionsOf`), which a reference in the replacement or in the
 English resolves against whichever slice holds the definition; none where the slice carries no page (n173)

 @returns Whether the replacement changes them

 @example
 ```ts
 const changes = changesWords({ edit, english, words: { start: 4, core: 'The cat naps', }, definitions: '', },);
 ```
 */
function changesWords(
  {
    edit,
    english,
    words,
    definitions,
  }: {
    readonly edit: PendingEdit;
    readonly english: string;
    readonly words: {
      readonly start: number;
      readonly core: string;
    };
    readonly definitions: string;
  },
): boolean {
  /**
   Where the located words start, and the words.
   */
  const {
    start,
    core,
  } = words;
  /**
   Exclusive offset where the located words end.
   */
  const end = start + core.length;
  if ((edit.end <= start) || (end <= edit.start))
    return false;
  if ((start < edit.start) || (edit.end < end))
    return true;
  return standsOf({
    haystack: edit.text,
    words: core,
    definitions,
  },) < standsOf({
    haystack: english.slice(
      edit.start,
      edit.end,
    ),
    words: core,
    definitions,
  },);
}

/**
 The replacements of a slice that change the located words of a place of another
 group: a block replacement against any located place, a code splice against
 every located place its group does not replace in the slice.

 @param index - the page's groups read by slice, which holds the slice's replacements and located places and no other
 slice's, so the cost of a slice does not grow with the number of slices

 @param sliceIndex - the slice

 @param english - the slice's English as it stood when the pass began

 @param page - the English of the whole page the slice belongs to, whose link and footnote definitions a reference in the English or
 in a replacement resolves against (n173); absent for a slice read alone

 @returns Each such replacement once, with the first place it changes in page order, in the order of the groups

 @example
 ```ts
 const leftOut = editsChangingLocatedWords({ index: settledIndexOf({ settled, },), sliceIndex: 2, english, },);
 ```
 */
export function editsChangingLocatedWords(
  {
    index,
    sliceIndex,
    english,
    page,
  }: {
    readonly index: SettledIndex;
    readonly sliceIndex: number;
    readonly english: string;
    readonly page?: PageEnglish;
  },
): readonly LeftOutEdit[] {
  /**
   The link and footnote definitions of the page, none for a slice read alone.
   */
  const definitions = (page === undefined) ? '' : pageDefinitionsOf({ pageText: page.text, },);
  /**
   What the page's groups hold by slice, and the places they replace in their slices.
   */
  const {
    places,
    edits: indexedEdits,
    replaced,
  } = index;
  /**
   The located places in the slice, in the order of the groups.
   */
  const placesHere = places.get(sliceIndex,) ?? [];
  /**
   The replacements addressed to the slice, in the order of the groups.
   */
  const editsHere = indexedEdits.get(sliceIndex,) ?? [];
  return editsHere.flatMap(function changedBy({
    group: groupPosition,
    edit,
  },): readonly LeftOutEdit[] {
    /**
     The places of the other groups, in this slice, whose located words the replacement changes.
     */
    const changed = placesHere
      .filter(function isChanged(place,): boolean {
        if (place.group === groupPosition)
          return false;
        // A place its group replaces is arbitrated by order against the other replacements, so a code splice is not held to it.
        if ((edit.kind === 'splice') && replaced.has(replacedKeyOf({
          group: place.group,
          position: place.position,
          sliceIndex,
        },),))
          return false;
        /**
         Where the located words of the place start and what they are.
         */
        const {
          start,
          core,
        } = place.rendering;
        return changesWords({
          edit,
          english,
          words: {
            start,
            core,
          },
          definitions,
        },);
      },)
      .map(function changedPlace({
        group,
        position,
        rendering,
      },): ChangedPlace {
        return {
          group,
          position,
          label: consistencyLabel({ position, },),
          start: rendering.start,
        };
      },);
    /**
     The first of them in page order.
     */
    const [first,] = changed.toSorted(function inPageOrder(
      left,
      right,
    ): number {
      return (left.start - right.start) || (left.group - right.group);
    },);
    return (first === undefined)
      ? []
      : [
        {
          group: groupPosition,
          edit,
          other: {
            group: first.group,
            label: first.label,
            start: first.start,
          },
        },
      ];
  },);
}

//endregion Pass consistency apply located
