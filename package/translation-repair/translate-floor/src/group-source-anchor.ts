import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import type {
  AlignmentStep,
  DocumentNode,
} from '@monochromatic-dev/translation-repair-document-structure/ts';

//region Source anchor offsets
// Where a block the translation never rendered would be written, read off the
// monotone walk.
//
// WHY THE WALK CAN ANSWER THIS AND THE HEADING ALIGNER COULD NOT. The heading
// aligner reports one decision per source row and then every unclaimed target
// row, so it carries no cursor and its skip position had to be recovered from
// the table (`align-headings-optimal.ts`). This walk is monotone by
// construction: its steps ARE the cursor, in document order, so the place a
// source block is skipped at is simply where it sits in the sequence.
//
// A MERGE IS NOT AN OMISSION, and the walk already separates them. Two originals
// rendered as one translation block arrive as a `paired` step followed by a
// `source-only` step carrying `continuesPairing`, and that second original IS
// placed, against a block the first already claimed. Only a `source-only` step
// WITHOUT that marker is an original nothing rendered. This is the same
// predicate `declined-target-runs.ts` uses to decide whether a pairing left an
// original unplaced, kept identical on purpose: two readings of "unplaced" that
// drifted apart would disagree about the same document.

/**
 Stands for "no offset here", which is not an offset any document has: every
 real one is zero or greater.
 */
const NO_OFFSET = -1;

/**
 Offsets a walk step covers on the translation side.
 */
type RenderedSpan = {
  /**
   Where the block begins, or {@link NO_OFFSET} when the step consumes none.
   */
  readonly start: number;

  /**
   Where it ends, under the same convention.
   */
  readonly end: number;
};

/**
 Reads the block a walk step names on one side.

 LOUD RATHER THAN ABSENT. A step that named a block its side lacks used to
 read as a step carrying no block there: the grouper filtered it out, the
 anchors read it as a step that consumes no translation, and a walk pairing
 one original with a translation block past the last grouped to no runs at
 all, the whole section gone and nothing said. `walkIntoRuns` reads every
 step's blocks through here before anything else reads the walk, and
 `renderedSpan` reads through here too, so that state has one refusal and
 one wording.

 @param nodes - one side's blocks, which the walk indexes

 @param index - block the step names on that side

 @param side - which side the blocks are, for the refusal

 @param at - step's position in the walk, for the refusal

 @returns Block the step names

 @throws Error when the step names no block of that side, which no walk
 built in this package does: `alignBlocks` walks these very blocks, and
 `blockPairingToSteps` refuses a pairing that names a block its side lacks

 @example
 ```ts
 const node = blockAtStep({ nodes: targetNodes, index: step.targetIndex, side: 'translation', at: 3, },);
 ```
 */
export function blockAtStep(
  {
    nodes,
    index,
    side,
    at,
  }: {
    readonly nodes: readonly DocumentNode[];
    readonly index: number;
    readonly side: 'original' | 'translation';
    readonly at: number;
  },
): DocumentNode {
  /**
   Block at that index, absent only for a walk that names what its side lacks.
   */
  const node = nodes[index];
  if (node === undefined)
    throw new Error(
      `unreachable: walk step ${String(at,)} names ${side} block ${String(index,)}, and there are `
        + `${String(nodes.length,)}, though alignBlocks walks these very blocks and blockPairingToSteps `
        + 'refuses a pairing that names a block its side lacks',
    );
  return node;
}

/**
 Reads the span of the translation block a step consumes.

 RETURNS SENTINELS RATHER THAN AN ABSENT NODE, so callers compare numbers
 instead of narrowing, and a step naming only an original reads the same way
 as a position past the end of the walk.

 @param step - one walk step

 @param targetNodes - translation blocks the steps index

 @param at - step's position in the walk, named when the step is refused

 @returns Start and end offsets, both {@link NO_OFFSET} when the step consumes
 no translation block

 @throws Error when the step names a translation block the side lacks, in
 `blockAtStep`'s words

 @example
 ```ts
 const span = renderedSpan({ step, targetNodes, at: 0, },);
 ```
 */
function renderedSpan(
  {
    step,
    targetNodes,
    at,
  }: {
    readonly step: AlignmentStep;
    readonly targetNodes: readonly DocumentNode[];
    readonly at: number;
  },
): RenderedSpan {
  if (step.kind === 'source-only')
    return {
      start: NO_OFFSET,
      end: NO_OFFSET,
    };

  /**
   Block this step consumes.
   */
  const node = blockAtStep({
    nodes: targetNodes,
    index: step.targetIndex,
    side: 'translation',
    at,
  },);
  return {
    start: node.startOffset,
    end: node.endOffset,
  };
}

/**
 Reports whether a step names an original block nothing rendered.

 @param step - one walk step

 @returns Whether it leaves that original unplaced

 @example
 ```ts
 const unplaced = leavesOriginalUnplaced(step,);
 ```
 */
export function leavesOriginalUnplaced(step: AlignmentStep,): boolean {
  return (step.kind === 'source-only') && (step.continuesPairing !== true);
}

/**
 Reads where each unplaced original block's translation would be written.

 ANCHORED BEFORE THE NEXT RENDERED BLOCK, or after the last one when nothing
 follows. Anchoring after the PREVIOUS block instead would be the same place
 in a document with no gap between blocks and a different one wherever the
 translation carries anything between them, so the two are not
 interchangeable and the forward reading is the one that keeps the insertion
 outside a rendering rather than inside it.

 A WALK THAT CONSUMES NO TRANSLATION BLOCK AT ALL yields no anchors. That is a
 section whose translation is empty, which needs a body-insertion boundary
 rather than a block one, and inventing offset zero for it would write into
 whatever the section actually begins with.

 @param walk - monotone steps in document order

 @param targetNodes - translation blocks the steps index

 @returns Walk position of each unplaced original, mapped to the offset its
 rendering would be written at

 @throws Error when a step names a translation block the side lacks, in
 `blockAtStep`'s words, or when an unplaced original has no rendered block
 before or after it in a walk that pairs a step; no walk built in this
 package does either

 @example
 ```ts
 const anchors = anchorOffsets({ walk, targetNodes, },);
 ```
 */
export function anchorOffsets(
  {
    walk,
    targetNodes,
  }: {
    readonly walk: readonly AlignmentStep[];
    readonly targetNodes: readonly DocumentNode[];
  },
): ReadonlyMap<number, number> {
  // A PAIRING THAT PLACED NOTHING DOES NOT MAKE EVERY ORIGINAL ABSENT. With no
  // paired step anywhere, every original reads as unplaced and every
  // translation block as unclaimed, which is a pairing that failed rather than a
  // page missing its whole source. Reading it as absence would propose writing
  // the entire original into a page that already carries a translation of it,
  // and would leave the unclaimed translation blocks with no run to belong to.
  //
  // `declined-target-runs.ts` refuses the mirror of this for the same reason:
  // "Nor does a pairing that placed nothing at all decline everything."
  if (!walk.some(function placesSomething(step,): boolean {
    return step.kind === 'paired';
  },))
    return new Map<number, number>();

  /**
   Offset the next rendered block begins at, per walk position, using
   {@link NO_OFFSET} where no rendered block follows.

   Filled by scanning BACKWARDS so each position reads the answer the position
   after it already computed, which makes the whole pass linear.
   */
  const nextOffsets: number[] = Array.from(
    { length: walk.length, },
    function unknown(): number {
      return NO_OFFSET;
    },
  );

  /**
   Offset carried backwards through the scan.
   */
  const scan = { next: NO_OFFSET, };
  for (let at = walk.length - 1; at >= 0; at -= 1) {
    /**
     Step at this position, always present since the loop counts down from
     the walk's own length.
     */
    const step = nonNullishOrThrow(walk.at(at,),);

    /**
     Span the step consumes on the translation side.
     */
    const span = renderedSpan({
      step,
      targetNodes,
      at,
    },);
    if (span.start !== NO_OFFSET)
      scan.next = span.start;

    nextOffsets[at] = scan.next;
  }

  /**
   End of the last translation block the walk consumes, where a trailing
   unplaced original belongs, or {@link NO_OFFSET} when the walk consumes none.
   */
  const tail = walk.reduce(
    function lastRendered(
      standing: number,
      step,
      at,
    ): number {
      /**
       Span this step consumes on the translation side.
       */
      const span = renderedSpan({
        step,
        targetNodes,
        at,
      },);

      return (span.end === NO_OFFSET) ? standing : span.end;
    },
    NO_OFFSET,
  );

  return new Map(walk.flatMap(function toAnchor(
    step,
    at,
  ): readonly (readonly [
    number,
    number,
  ])[] {
    if (!leavesOriginalUnplaced(step,))
      return [];

    /**
     Where this original's rendering would go: before the next rendered block
     when one follows, after the last one otherwise.
     */
    const forward = nonNullishOrThrow(nextOffsets.at(at,),);

    /**
     That, falling back to the tail for an original past every rendering.
     */
    const offset = (forward === NO_OFFSET) ? tail : forward;

    // LOUD RATHER THAN ANCHORED AT `NO_OFFSET`. A paired step is what let
    // the walk past the opening check, a paired step names a translation
    // block (`blockAtStep` refuses one that names a block the side lacks),
    // and that block gives `tail` a real end; so no walk leaves both
    // readings empty, and one that did is broken rather than a page with
    // nowhere to write (ledger M113).
    if (offset === NO_OFFSET)
      throw new Error(
        'unreachable: an unplaced original with no rendered block before or after it, in a walk that '
          + 'pairs a step, though a paired step names a translation block and that block ends at an offset',
      );

    return [[
      at,
      offset,
    ],];
  },),);
}

//endregion Source anchor offsets
