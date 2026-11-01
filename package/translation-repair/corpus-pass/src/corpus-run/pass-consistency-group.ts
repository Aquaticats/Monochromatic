import type { RepeatGroup, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import type { ArtifactConsistencyRepeat, } from '@monochromatic-dev/translation-repair-artifact/ts';
import type { ConsistencySliceFacts, } from './pass-consistency-slices.ts';
import {
  ConsistencyInvariantError,
  type ConsistencyOccurrenceView,
} from '@monochromatic-dev/translation-repair-consistency/ts';

//region Pass consistency group
// WHAT THE PASS MAKES OF ONE REPEAT GROUP BEFORE ANY JUDGE IS ASKED: the
// places a slice holds whole, tied to the facts of their slices; the sheet's
// view of each; and whether the bench needs to be asked at all.

/**
 One place a slice holds whole, with its slice.

 @example
 ```ts
 const place: ConsistencyPlace = { startOffset: 0, endOffset: 12, line: 1, slice, };
 ```
 */
export type ConsistencyPlace = {
  /**
   Absolute offset in the original where the place starts.
   */
  readonly startOffset: number;

  /**
   Absolute exclusive offset where it ends.
   */
  readonly endOffset: number;

  /**
   One-based line it starts on.
   */
  readonly line: number;

  /**
   For a frame: absolute offset where the slot starts.
   */
  readonly slotStartOffset?: number;

  /**
   For a frame: absolute exclusive offset where the slot ends.
   */
  readonly slotEndOffset?: number;

  /**
   The slice holding the place.
   */
  readonly slice: ConsistencySliceFacts;
};

/**
 Whether the bench is asked about a group, or why it is not.

 @example
 ```ts
 const disposition: GroupDisposition = { kind: 'ask', };
 ```
 */
export type GroupDisposition =
  | {
    /**
     The places may render the wording differently, so the bench is asked.
     */
    readonly kind: 'ask';
  }
    | {
    /**
     The bench is not asked.
     */
      readonly kind: 'skip';

    /**
     Why: fewer than two places a slice holds whole, fewer than two with any
     English, or every place already reading alike.
     */
      readonly reason: 'too-few-placed' | 'no-english' | 'consistent';
    };

/**
 Ties each place of a group to the facts of its slice.

 @param group - repeat group of the original

 @param slices - what the pass knows of each slice

 @returns The places a slice holds whole, in page order

 @throws {@link Error} when a place names a slice the pass knows nothing of,
 since the groups are found over the same slices the facts are gathered from

 @example
 ```ts
 const places = placesOf({ group, slices, },);
 ```
 */
export function placesOf(
  {
    group,
    slices,
  }: {
    readonly group: RepeatGroup;
    readonly slices: readonly ConsistencySliceFacts[];
  },
): readonly ConsistencyPlace[] {
  return group.occurrences
    .map(function placeOf(occurrence,): ConsistencyPlace {
      /**
       The slice holding the place.
       */
      const slice = slices.find(function holdsIt(candidate,): boolean {
        return candidate.sourceIndex === occurrence.sliceIndex;
      },);
      if (slice === undefined) {
        throw new ConsistencyInvariantError({ invariant:
          `a repeat group names source slice ${String(occurrence.sliceIndex,)}, `
            + 'and the groups are found over the same slices the facts are gathered from', },);
      }
      return {
        startOffset: occurrence.startOffset,
        endOffset: occurrence.endOffset,
        line: occurrence.line,
        ...(('slotStartOffset' in occurrence)
          ? {
            slotStartOffset: occurrence.slotStartOffset,
            slotEndOffset: occurrence.slotEndOffset,
          }
          : {}),
        slice,
      };
    },);
}

/**
 The repeated wording or frame of a group, as the sheet and the artifact say it.

 @param group - repeat group of the original

 @param text - whole original

 @returns The frame of a refrain, or the run of a maximal repeat as the original writes it

 @example
 ```ts
 const repeat = repeatOf({ group, text, },);
 ```
 */
export function repeatOf(
  {
    group,
    text,
  }: {
    readonly group: RepeatGroup;
    readonly text: string;
  },
): ArtifactConsistencyRepeat {
  if (group.kind === 'refrain') {
    return {
      kind: 'refrain',
      prefix: group.prefix,
      suffix: group.suffix,
    };
  }
  /**
   Where the run first stands, placed or not.
   */
  const first = [
    ...group.occurrences,
    ...group.unplaced,
  ]
    .toSorted(function byStart(
      a,
      b,
    ): number {
      return a.startOffset - b.startOffset;
    },)
    .at(0,);
  if (first === undefined) {
    throw new ConsistencyInvariantError({ invariant:
      'a repeat group lists no place, and the detector reports a repeat only where it stands in two', },);
  }
  return {
    kind: 'maximal',
    wording: text.slice(
      first.startOffset,
      first.endOffset,
    ),
  };
}

/**
 Decides whether the bench is asked about a group.

 A GROUP READS ALIKE ONLY WHERE NO SLICE HOLDS TWO PLACES AND EVERY PLACE'S
 SLICE CARRIES ONE ENGLISH: the places in one slice share an English that may
 render the wording twice, and the pass cannot see inside an English without a
 judge.

 @param places - the places a slice holds whole

 @returns That the bench is asked, or why not

 @example
 ```ts
 const disposition = dispositionOf({ places, },);
 ```
 */
export function dispositionOf(
  { places, }: { readonly places: readonly ConsistencyPlace[]; },
): GroupDisposition {
  if (places.length < 2) {
    return {
      kind: 'skip',
      reason: 'too-few-placed',
    };
  }
  /**
   English of each place's slice, for the places that carry any.
   */
  const english = places.flatMap(function englishOf(place,): readonly string[] {
    return (place.slice
      .current
      .kind
      === 'present') ? [place.slice
        .current
        .text,] : [];
  },);
  if (english.length < 2) {
    return {
      kind: 'skip',
      reason: 'no-english',
    };
  }
  /**
   Whether one slice holds two places.
   */
  const sharesSlice = new Set(places.map(function indexOf(place,): number {
    return place.slice
      .sliceIndex;
  },),).size < places.length;
  if ((!sharesSlice) && (english.length === places.length)
    && english.every(function same(text,): boolean {
      return text === english[0];
    },)) {
    return {
      kind: 'skip',
      reason: 'consistent',
    };
  }
  return { kind: 'ask', };
}

/**
 The block of the original holding a place, with the place's offsets inside it.

 @param place - the place

 @param text - whole original

 @returns The block, or the whole slice's original where no block holds the
 place, and where the place starts and ends inside it

 @throws {@link Error} when the offsets do not read the same words in the
 block as in the whole original

 @example
 ```ts
 const { paragraph, markStart, } = paragraphOf({ place, text, },);
 ```
 */
function paragraphOf(
  {
    place,
    text,
  }: {
    readonly place: ConsistencyPlace;
    readonly text: string;
  },
): {
  readonly paragraph: string;
  readonly markStart: number;
  readonly markEnd: number;
} {
  /**
   Block of the slice holding the whole place, if one does.
   */
  const node = place.slice
    .nodes
    .find(function holdsIt(candidate,): boolean {
      return (candidate.startOffset <= place.startOffset) && (place.endOffset <= candidate.endOffset);
    },);
  /**
   The text shown and where it starts in the whole original.
   */
  const shown = (node === undefined)
    ? {
      paragraph: place.slice
        .sourceText,
      origin: place.slice
        .sourceStart,
    }
    : {
      paragraph: node.text,
      origin: node.startOffset,
    };
  /**
   The place's words in the whole original.
   */
  const words = text.slice(
    place.startOffset,
    place.endOffset,
  );
  /**
   Where the place stands in the text shown.
   */
  const markStart = place.startOffset - shown.origin;
  /**
   Where the place ends in the text shown.
   */
  const markEnd = place.endOffset - shown.origin;
  if (shown.paragraph
    .slice(
    markStart,
    markEnd,
    )
    !== words) {
    throw new ConsistencyInvariantError({ invariant:
      `the offsets of a place point at other words in the text shown than in the whole original (slice ${
        String(place.slice
          .sliceIndex,)
      }), and both are cut from the same offsets`, },);
  }
  return {
    paragraph: shown.paragraph,
    markStart,
    markEnd,
  };
}

/**
 What the sheet shows of one place.

 @param place - the place

 @param text - whole original

 @returns The place's paragraph with its marked wording, its slot for a frame,
 and the English of its slice

 @throws {@link Error} when the place's offsets do not read the same words in
 its block as in the whole original

 @example
 ```ts
 const view = occurrenceViewOf({ place, text, },);
 ```
 */
export function occurrenceViewOf(
  {
    place,
    text,
  }: {
    readonly place: ConsistencyPlace;
    readonly text: string;
  },
): ConsistencyOccurrenceView {
  return {
    sliceIndex: place.slice
      .sliceIndex,
    ...paragraphOf({
      place,
      text,
    },),
    ...(((place.slotStartOffset === undefined) || (place.slotEndOffset === undefined))
      ? {}
      : {
        slotSource: text.slice(
          place.slotStartOffset,
          place.slotEndOffset,
        ),
      }),
    // THE SLICE'S WHOLE ORIGINAL goes with the place, so the sheets cut the notes
    // between translators from the slice's English and keep the comments this
    // original holds (`consistency-shown-notes.ts`).
    sliceSource: place.slice
      .sourceText,
    english: place.slice
      .current,
  };
}

//endregion Pass consistency group
