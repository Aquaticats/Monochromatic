import type { FragmentPlace, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import {
  leadingLineEndings,
  matchSpanEdges,
} from '@monochromatic-dev/translation-repair-slice-reading/ts';

//region Splice slice places
// WHERE EACH REPLACEMENT'S WORDS LANDED once assembly (`splice-slices.ts`) has written every edit. Edits run from the end of
// the document to its start, so the text before an edit is untouched when it is written; the final place of an edit's words
// is its start offset moved by the net growth of every edit written after it, which stands before it in the finished text.

/**
 One write assembly performs, which is not one replacement.

 Several anchors can share a boundary, and their separators are decided once
 for the whole group rather than by each in turn, so the group is ONE edit.

 @example
 ```ts
 const edit: SpliceEdit = { kind: 'insertion', startOffset: 12, endOffset: 12, orderIndex: 3, fragments, sliceIndexes: [3,], };
 ```
 */
export type SpliceEdit = {
  /**
   Whether this writes over existing text or into a place where none is.
   */
  readonly kind: 'content';

  /**
   Where the span being written over starts.
   */
  readonly startOffset: number;

  /**
   Where it ends, exclusive.
   */
  readonly endOffset: number;

  /**
   Slice this edit is ordered by, which for a group is its earliest.
   */
  readonly orderIndex: number;

  /**
   Text to write, exactly as its lane produced it.
   */
  readonly text: string;
} | {
  /**
   Whether this writes over existing text or into a place where none is.
   */
  readonly kind: 'insertion';

  /**
   Boundary the anchors share.
   */
  readonly startOffset: number;

  /**
   Same boundary, since an anchor covers nothing.
   */
  readonly endOffset: number;

  /**
   Earliest slice anchored here, which orders this group against the rest.
   */
  readonly orderIndex: number;

  /**
   What the lanes produced for those slices, in document order.
   */
  readonly fragments: readonly string[];

  /**
   The slice each fragment was produced for, in the order of the fragments.
   */
  readonly sliceIndexes: readonly number[];
};

/**
 The text one write puts in, and where each replacement's words stand in it.

 @example
 ```ts
 const written: Written = { text: 'The cat naps.', placed: [{ fragment: 0, englishAt: 0, from: 0, to: 13, },], };
 ```
 */
export type Written = {
  /**
   The text written.
   */
  readonly text: string;

  /**
   Where the words of each replacement the write carries stand in the text; a replacement that writes nothing has no entry.
   */
  readonly placed: readonly FragmentPlace[];
};

/**
 One write made, with what it replaced.

 @example
 ```ts
 const write: Write = { edit, written, replacedLength: 15, };
 ```
 */
export type Write = {
  /**
   The edit.
   */
  readonly edit: SpliceEdit;

  /**
   What it put in.
   */
  readonly written: Written;

  /**
   How long the text it replaced was.
   */
  readonly replacedLength: number;
};

/**
 Where one replacement's words landed in the translation assembly built.

 @example
 ```ts
 const placed: PlacedSlice = { sliceIndex: 4, start: 120, end: 150, englishAt: 120, };
 ```
 */
export type PlacedSlice = {
  /**
   Global slice index of the replacement.
   */
  readonly sliceIndex: number;

  /**
   Where the range the replacement's words occupy starts in the translation: for a content span the whole text written over
   it, edges included; for an anchored fragment its body, without the blank lines assembly put around it.
   */
  readonly start: number;

  /**
   Where that range ends, exclusive.
   */
  readonly end: number;

  /**
   Where the replacement's first character would stand in the translation, so that a character of it past the line endings
   and blank lines assembly drops stands at this offset plus its own; the line endings the replacement carries at its edges
   are dropped (a content span) and so are the blank lines before an anchored fragment, which can move this before `start`.
   */
  readonly englishAt: number;
};

/**
 The text a content span is written over with, and where its words stand in it.

 @param replaced - text the span held, whose line-ending edges the write keeps

 @param text - the lane's text, whose own line-ending edges the write drops

 @returns The text written, and the one place of its words: the offset where the lane's first character would stand, which is
 before the written text when the lane carried more line endings at its start than the span did

 @example
 ```ts
 const { text, placed, } = contentWrite({ replaced: 'The cat sleeps.', text: '\nThe cat naps.\n', },);
 ```
 */
export function contentWrite(
  {
    replaced,
    text,
  }: {
    readonly replaced: string;
    readonly text: string;
  },
): Written {
  /**
   The text written over the span.
   */
  const matched = matchSpanEdges({
    replaced,
    text,
  },);
  /**
   The line endings the span keeps at its start.
   */
  const kept = leadingLineEndings({ text: replaced, },);
  /**
   The line endings the lane carried at its start, which the write drops.
   */
  const dropped = leadingLineEndings({ text, },);
  return {
    text: matched,
    placed: [
      {
        fragment: 0,
        englishAt: kept.length - dropped.length,
        from: 0,
        to: matched.length,
      },
    ],
  };
}

/**
 Where each replacement's words stand in the translation once every write is made.

 Writes run from the end of the document to its start, so the text before a write is untouched when it is made and each
 write is moved by the net growth of every write made after it, which stands before it.

 @param writes - the writes in the order they were made

 @returns One entry for each replacement that wrote something, in the order of the translation

 @example
 ```ts
 const placed = placementsOf({ writes, },);
 ```
 */
export function placementsOf({ writes, }: { readonly writes: readonly Write[]; },): readonly PlacedSlice[] {
  /**
   How much every write made after the one at each position grew the text, mutated only inside this function.
   */
  const after = { growth: 0, };
  /**
   The places found so far, mutated only inside this function.
   */
  const found: PlacedSlice[] = [];
  for (const {
    edit,
    written,
    replacedLength,
  } of writes.toReversed()) {
    /**
     Where this write's text starts in the finished translation.
     */
    const base = edit.startOffset + after.growth;
    /**
     The text this write put in.
     */
    const { text, } = written;
    after.growth += text.length - replacedLength;
    /**
     The slice each fragment of the write was produced for: the one of a content span, or those of the anchors in a group.
     */
    const sliceIndexes = (edit.kind === 'content') ? [edit.orderIndex,] : edit.sliceIndexes;
    for (const place of written.placed) {
      /**
       The slice these words were written for.
       */
      const sliceIndex = sliceIndexes.at(place.fragment,);
      if (sliceIndex === undefined)
        continue;
      found.push({
        sliceIndex,
        start: base + place.from,
        end: base + place.to,
        englishAt: base + place.englishAt,
      },);
    }
  }
  return found.toSorted(function byStart(
    left,
    right,
  ): number {
    return left.start - right.start;
  },);
}

//endregion Splice slice places
