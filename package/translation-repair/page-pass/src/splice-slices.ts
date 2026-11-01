import {
  type DocumentChunk,
  isInsertionChunk,
  type ReadonlyEntry,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  rendersAsNothing,
  wordForCount,
} from '@monochromatic-dev/translation-repair-typography/ts';
import {
  assertPlacementLayout,
  composedInsertion,
  documentLineEnding,
} from '@monochromatic-dev/translation-repair-corpus-source/ts';
import {
  assertSliceIndexing,
  type ChunkPair,
} from '@monochromatic-dev/translation-repair-slice-reading/ts';
import {
  contentWrite,
  type PlacedSlice,
  placementsOf,
  type SpliceEdit,
  type Write,
} from './splice-slice-places.ts';

//region Slice splicing
// Rebuilding a whole translation from per-slice results.
//
// Extracted because the driver assembles TWICE: once to get `T1`, whose
// definitions the naturalness gate needs in order to resolve references, and
// once more to get the text that ships. Two copies of an offset-descending
// splice is exactly the kind of duplication that drifts.
//
// LANE-NEUTRAL. It consumes replacements rather than repair outcomes, because
// the translate lane produces text for slices no repair outcome describes: a
// slice whose translation is absent has nothing repaired and everything to
// write. Reading `changed` off an outcome also asked the wrong question, since
// what assembly needs to know is whether there is text to write here, not
// whether some stage considers its work a change.

/**
 Raised when replacements cannot be written into the slices they name.

 @example
 ```ts
 throw new SliceSpliceError({ message: 'two replacements name one slice', },);
 ```
 */
export class SliceSpliceError extends Error {
  /**
   Builds refusal carrying what could not hold.

   @param message - which replacement cannot land, and what makes it impossible

   @example
   ```ts
   throw new SliceSpliceError({ message: 'two replacements name one slice', },);
   ```
   */
  public constructor({ message, }: { readonly message: string; },) {
    super(message,);
    this.name = 'SliceSpliceError';
  }
}

/**
 Text to write over one slice's span.

 Carries no `changed` flag: presence in the list IS the instruction to apply
 it, so a lane decides what changed and assembly decides where it goes.

 @example
 ```ts
 const replacement: SliceReplacement = { sliceIndex: 4, replacementText: 'The cat naps.', };
 ```
 */
export type SliceReplacement = {
  /**
   Global slice index, as `prepareDocumentPair` stamped it.
   */
  readonly sliceIndex: number;

  /**
   Text that replaces that slice's target span.

   May be empty, which deletes the span. May be written into a zero-length
   span, which inserts at that offset: that is how a slice with no existing
   translation receives one.
   */
  readonly replacementText: string;
};

/**
 One replacement joined to the span it names.

 @example
 ```ts
 const placed: PlacedReplacement = { replacement, span: slice.target, };
 ```
 */
type PlacedReplacement = {
  /**
   Text to write, as its lane emitted it.
   */
  readonly replacement: SliceReplacement;

  /**
   Target span it goes into, resolved from the slice list.

   NO SOURCE TEXT BESIDE IT. The original decides whether blank text may be
   written into a place that has nothing yet, and that question is answered
   where the pair is still in hand; carrying the answer's input forward would
   be a field nothing reads.
   */
  readonly span: DocumentChunk;
};

/**
 Plans every write, in the order they can be made without moving each other.

 DESCENDING, so writing one never shifts the offsets of those still pending.
 At one boundary the later slice is written first, which leaves the earlier
 one ahead of it: document order, and true only because an index IS a
 position, which {@link spliceSlices} asserts before this runs.

 @param placed - replacements joined to the spans they name

 @returns Edits in application order

 @throws {@link Error} when a boundary group holds nothing, which grouping
 cannot produce

 @example
 ```ts
 const edits = plannedEdits({ placed, },);
 ```
 */
function plannedEdits(
  { placed, }: { readonly placed: readonly PlacedReplacement[]; },
): readonly SpliceEdit[] {
  /**
   Writes over existing text, one per replacement.
   */
  const overText = placed
    .filter(function coversText(entry,): boolean {
      return !isInsertionChunk(entry.span,);
    },)
    .map(function toContentEdit(entry,): SpliceEdit {
      return {
        kind: 'content',
        startOffset: entry.span
          .startOffset,
        endOffset: entry.span
          .endOffset,
        orderIndex: entry.replacement
          .sliceIndex,
        text: entry.replacement
          .replacementText,
      };
    },);

  /**
   Writes into a place, gathered by the boundary they share.
   */
  const byBoundary = Map.groupBy(
    placed.filter(function namesAPlace(entry,): boolean {
      return isInsertionChunk(entry.span,);
    },),
    function toBoundary(entry,): number {
      return entry.span
        .startOffset;
    },
  );

  /**
   One edit per boundary, carrying its fragments in document order.
   */
  const intoPlaces = [...byBoundary,].map(function toInsertionEdit(entry: ReadonlyEntry<number, readonly PlacedReplacement[]>,): SpliceEdit {
    /**
     Boundary and the replacements anchored there.
     */
    const [offset, group,] = entry;

    /**
     Those replacements in document order, which is slice order.
     */
    const inOrder = group.toSorted(function byIndex(
      left,
      right,
    ): number {
      /**
       Slice the left replacement names.
       */
      const leftIndex = left.replacement
        .sliceIndex;

      /**
       Slice the right one names.
       */
      const rightIndex = right.replacement
        .sliceIndex;
      return leftIndex - rightIndex;
    },);

    /**
     Earliest of them, which orders the whole group.
     */
    const [first,] = inOrder;
    if (first === undefined)
      throw new Error('unreachable: grouping produced a boundary with no replacement',);
    return {
      kind: 'insertion',
      startOffset: offset,
      endOffset: offset,
      orderIndex: first.replacement
        .sliceIndex,
      fragments: inOrder.map(function toText(anchored,): string {
        return anchored.replacement
          .replacementText;
      },),
      sliceIndexes: inOrder.map(function toIndex(anchored,): number {
        return anchored.replacement
          .sliceIndex;
      },),
    };
  },);
  return [
    ...overText,
    ...intoPlaces,
  ].toSorted(function byOffsetDescending(
    left,
    right,
  ): number {
    /**
     Offset gap, which decides every pair of distinct boundaries.
     */
    const byOffset = right.startOffset - left.startOffset;
    if (byOffset !== 0)
      return byOffset;
    return right.orderIndex - left.orderIndex;
  },);
}

/**
 Rebuilds the translation with every replacement written in.

 Edits apply in DESCENDING document order, so writing one never shifts the
 offsets of those still pending. An edit is not a replacement: every anchor
 sharing one boundary becomes a SINGLE edit whose fragments are joined in
 slice order, because the separators between them are decided once for the
 whole group rather than guessed at by each write in turn.

 @param targetText - translation the slices were cut from

 @param slices - slice pairs in document order

 @param replacements - text to write, in any order

 @returns Translation with every replacement applied

 @throws {@link Error} when a replacement names a slice that does not exist,
 when two name the same slice, or when two slices carry one index: each means
 the caller and the slicing disagree, and each silently drops text

 @example
 ```ts
 const assembled = spliceSlices({ targetText, slices, replacements, },);
 ```
 */
export function spliceSlices(
  {
    targetText,
    slices,
    replacements,
  }: {
    readonly targetText: string;
    readonly slices: readonly ChunkPair[];
    readonly replacements: readonly SliceReplacement[];
  },
): string {
  return placeSlices({
    targetText,
    slices,
    replacements,
  },)
    .text;
}

/**
 Rebuilds the translation with every replacement written in and says where each replacement's words landed.

 THE ONE PLACE THE PAGE IS ASSEMBLED: {@link spliceSlices} is this function's text, so a reader of the page that wants to
 know where a slice's English stands in it asks here and not a reconstruction.

 @param targetText - translation the slices were cut from

 @param slices - slice pairs in document order

 @param replacements - text to write, in any order

 @returns The translation, and for each replacement the range its words occupy and where its first character stands; an
 anchored fragment that shows a reader nothing writes nothing and has no entry

 @throws {@link Error} when a replacement names a slice that does not exist,
 when two name the same slice, or when two slices carry one index: each means
 the caller and the slicing disagree, and each silently drops text

 @example
 ```ts
 const { text, placed, } = placeSlices({ targetText, slices, replacements, },);
 ```
 */
export function placeSlices(
  {
    targetText,
    slices,
    replacements,
  }: {
    readonly targetText: string;
    readonly slices: readonly ChunkPair[];
    readonly replacements: readonly SliceReplacement[];
  },
): {
  readonly text: string;
  readonly placed: readonly PlacedSlice[];
} {
  /**
   Target span per slice index, so a replacement is resolved by the index it
   names rather than by its position in the slice list.
   */
  const spans = new Map(slices.map(function toSpan(slice,) {
    return [
      slice.target
        .sliceIndex,
      slice,
    ] as const;
  },),);
  if (spans.size !== slices.length) {
    throw new SliceSpliceError({
      message: 'two slices carry one index: a map keyed by index keeps only the last of '
        + 'them, so one slice becomes unreachable while its replacement lands '
        + 'on the other',
    },);
  }

  // AFTER identity, BEFORE any offset is read. Assembly is where a malformed
  // span does its damage, because slicing a string clamps: a span running
  // past the end, or backwards, produces plausible text rather than a
  // failure. Identity comes first because a list that names one slice twice
  // is a caller disagreeing with the slicer, which explains every offset
  // complaint that would follow it.
  // POSITIONAL INDICES, which this function's sort depends on and it
  // cannot otherwise assume. Two anchors at one boundary are written in
  // descending index order so they land in ascending order, which is document
  // order only while an index IS a position. A caller handing in slices whose
  // indices are unique but shuffled gets a document with its insertions
  // reversed and no complaint from anything else here.
  assertSliceIndexing({ slices, },);
  assertPlacementLayout({
    slices,
    targetText,
  },);

  /**
   Replacements paired with the span each names, refusing anything that
   cannot be placed.

   Resolved BEFORE sorting. Sorting first would have to invent an offset for
   an unresolvable index, and every fabricated offset orders the rest wrongly
   while looking like an ordinary sort.
   */
  const placed = replacements.map(function toPlaced(replacement,): PlacedReplacement {
    /**
     Span this replacement names.
     */
    const slice = spans.get(replacement.sliceIndex,);
    if (slice === undefined) {
      throw new SliceSpliceError({
        message: `no slice ${String(replacement.sliceIndex,)} to write into: `
          + `the document was sliced into ${String(slices.length,)} ${
          wordForCount({
            count: slices.length,
            one: 'slice',
            many: 'slices',
          },)
        }`,
      },);
    }

    /**
     Where it goes.
     */
    const span = slice.target;

    /**
     Whether this is a place rather than existing wording.
     */
    const missingTranslation = isInsertionChunk(span,);

    /**
     Original this slice renders.
     */
    const sourceText = slice.source
      .text;
    /**
     Whether the text offered for this slice shows a reader nothing, invisible
     characters `trim()` keeps among it (ledger B40).
     */
    const writesNothing = rendersAsNothing({ text: replacement.replacementText, },);

    /**
     Whether the original says something.
     */
    const sourceSaysSomething = !rendersAsNothing({ text: sourceText, },);
    if (missingTranslation
      && writesNothing
      && sourceSaysSomething) {
      throw new SliceSpliceError({
        message: `slice ${String(replacement.sliceIndex,)} has no translation and writes none: an anchor is `
          + 'where a rendering belongs, so blank text there leaves the passage missing while the run '
          + 'reports it delivered',
      },);
    }
    return {
      replacement,
      span,
    };
  },);
  if (new Set(placed.map(function toIndex(entry,): number {
    return entry.replacement
      .sliceIndex;
  },),).size !== placed.length) {
    throw new SliceSpliceError({
      message: 'two replacements name one slice: whichever applied second would '
        + 'overwrite the other, and the winner would depend on sort order',
    },);
  }

  /**
   Every edit this call makes, with the anchors sharing one boundary gathered
   into a single one.

   GATHERED RATHER THAN SEQUENCED, because the separators between them are
   decided once for the whole group: written one at a time, each would have to
   guess what the others had already put there.
   */
  const edits = plannedEdits({ placed, },);

  /**
   Line ending this document separates its blocks with.
   */
  const eol = documentLineEnding({ targetText, },);

  /**
   What each write left behind, in the order the writes were made, mutated only inside this function.
   */
  const writes: Write[] = [];
  /**
   The translation once every write is made.
   */
  const text = edits.reduce(
    function spliceOne(
      current: string,
      edit,
    ): string {
      /**
       Everything before this edit, which no earlier write has touched: edits
       run backwards through the document, so the prefix is still the archive.
       */
      const head = current.slice(
        0,
        edit.startOffset,
      );

      /**
       Everything after it, as it will stand: later offsets were written
       first, so this is what the edit's text will actually meet.
       */
      const tail = current.slice(edit.endOffset,);

      /**
       The text the span held, whose line-ending edges a content span keeps.
       */
      const replaced = current.slice(
        edit.startOffset,
        edit.endOffset,
      );

      /**
       Text going in. A content span carries its lane's text with the span's
       own line-ending edges (class seventy-five: a lane's trailing newline
       written verbatim ahead of the archive's separator doubled the blank
       line). An anchor has no span to sit between, so assembly composes its
       separators.
       */
      const written = (edit.kind === 'insertion')
        ? composedInsertion({
          fragments: edit.fragments,
          before: head,
          after: tail,
          eol,
        },)
        : contentWrite({
          replaced,
          text: edit.text,
        },);
      writes.push({
        edit,
        written,
        replacedLength: edit.endOffset - edit.startOffset,
      },);
      return head
        + written.text
        + tail;
    },
    targetText,
  );
  return {
    text,
    placed: placementsOf({ writes, },),
  };
}

//endregion Slice splicing
