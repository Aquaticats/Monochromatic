import type {
  Root,
  RootContent,
} from 'mdast';

import type { DeepReadonlyData, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import type { BlockShape, } from './translate-skeleton.ts';

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

//region Explicit source line structure
// A single-block poem can miss the blank-block verse heuristic. Its explicit
// breaks still belong to the source when no archive rendering exists to choose.

/**
 Read-only syntax node borrowed from the shared document parser.
 */
type ReadonlyNode = DeepReadonlyData<RootContent>;

/**
 Counts explicit rendered breaks within each top-level block.
 Markdown hard breaks and intrinsic lowercase br elements are equivalent;
 soft newlines, code, custom components and paragraph boundaries are not.

 @param root - already parsed skeleton tree, avoiding another grammar pass

 @returns Break counts aligned with skeleton block order

 @example
 ```ts
 const explicitBreaks = explicitBreakCounts({ root, });
 ```
 */
export function explicitBreakCounts({ root, }: { readonly root: DeepReadonlyData<Root>; },): readonly number[] {
  return root.children
    .map(function countBlock(block,): number {
    /**
     Owned work list grows as descendant nodes are visited.
     */
      const pending: ReadonlyNode[] = [block,];
    /**
     Explicit breaks in this block, not in unrelated neighboring blocks.
     */
      let count = 0;
      for (const node of pending) {
        if (
          (node.type === 'break')
            || (((node.type === 'mdxJsxTextElement') || (node.type === 'mdxJsxFlowElement')) && (node.name === 'br'))
        )
          count += 1;
        if ('children' in node)
          pending.push(...node.children,);
      }
      return count;
    },);
}

/**
 Refuses an explicit-break shortfall where the original alone sets the floor.
 Nonempty archive text stays outside this check even if its parser found no
 blocks. Expansion remains legal; unrelated blocks cannot compensate.

 @param pageText - canonical incumbent text, empty where no archive span exists

 @param source - original break counts, one per top-level block

 @param candidate - corresponding candidate counts

 @returns Model-facing findings for missing rendered breaks

 @example
 ```ts
 sourceOnlyBreakFindings({ pageText: '', source: [2], candidate: [0] });
 ```
 */
export function sourceOnlyBreakFindings(
  {
    pageText,
    source,
    candidate,
  }: {
    readonly pageText: string;
    readonly source: readonly number[];
    readonly candidate: readonly number[];
  },
): readonly string[] {
  if (pageText !== '')
    return [];
  return source.flatMap(function compareBlock(
    owed,
    index,
  ): readonly string[] {
    /**
     Missing blocks carry no breaks; the block floor diagnoses their shape.
     */
    const carried = candidate[index] ?? 0;
    if (carried >= owed)
      return [];
    return [
      `ORIGINAL block ${String(index + 1,)} carries ${String(owed,)} explicit line break(s), `
        + `but your corresponding block carries ${String(carried,)}. No archive rendering exists here. `
        + 'Keep at least the original count with Markdown hard breaks or intrinsic <br/> elements; '
        + 'soft newlines render as spaces and do not preserve these lines. Keep your chosen wording.',
    ];
  },);
}

//endregion Explicit source line structure

//region Explicit breaks under a substitute page block
// CLASS FORTY-TWO (Mio25 slice 16, 2026-09-17). `sourceOnlyBreakFindings`
// keys on the whole slice's page text, and a page whose farewell paragraph is
// paired with a verse blockquote HAS text, so the verse's five authored breaks
// were never owed: the translate lane's rendering carried the quote with soft
// breaks only, the rule passed it, and the poem renders as one run-on
// paragraph. Mio23's rendering had kept them by the writer's choice, not the
// rule's. Counted per block kind rather than per index because the candidate
// carries the page's block beside the original's, so indexes do not align.

/**
 Refuses an explicit-break shortfall in a block kind the page never rendered,
 where the original alone sets that kind's floor.

 @param pageBlocks - shapes of the page as it stands, at least one

 @param sourceBlocks - original's shapes, aligned with `sourceBreaks`

 @param sourceBreaks - original's break counts, one per top-level block

 @param candidateBlocks - candidate's shapes, aligned with `candidateBreaks`

 @param candidateBreaks - candidate's break counts, one per top-level block

 @returns Model-facing findings, one per kind short of its floor

 @throws Error when the original's or the candidate's shapes and break counts
 differ in length, which cannot occur, since `readSliceSkeleton` reads both
 off one parse's top-level blocks

 @example
 ```ts
 substituteBreakFindings({ pageBlocks: [{ kind: 'paragraph', detail: '', },], sourceBlocks: [{ kind: 'blockquote', detail: '', },], sourceBreaks: [2,], candidateBlocks: [{ kind: 'blockquote', detail: '', },], candidateBreaks: [0,], },);
 ```
 */
export function substituteBreakFindings(
  {
    pageBlocks,
    sourceBlocks,
    sourceBreaks,
    candidateBlocks,
    candidateBreaks,
  }: {
    readonly pageBlocks: readonly BlockShape[];
    readonly sourceBlocks: readonly BlockShape[];
    readonly sourceBreaks: readonly number[];
    readonly candidateBlocks: readonly BlockShape[];
    readonly candidateBreaks: readonly number[];
  },
): readonly string[] {
  // EACH SIDE IS TWO PARALLEL LISTS, one entry per top-level block, and every
  // read in this function takes a block's breaks at that block's own index.
  // Lists of different lengths were not read off one parse, so an index no
  // longer names a block's own breaks: the sums would count breaks under the
  // wrong kind or read past the end. Each side is refused before any read.
  if (sourceBreaks.length !== sourceBlocks.length)
    throw new Error('unreachable: the block shapes and break counts of the original differ in length, where one parse yields both',);

  if (candidateBreaks.length !== candidateBlocks.length)
    throw new Error('unreachable: the block shapes and break counts of the candidate differ in length, where one parse yields both',);

  /**
   Kinds the page rendered, whose breaks the page floor governs.
   */
  const pageKinds = new Set(pageBlocks.map(function toKind(block,): string {
    return block.kind;
  },),);
  /**
   Kinds the page never rendered whose original blocks carry breaks, each
   once, in document order.
   */
  const owedKinds = [
    ...new Set(sourceBlocks
      .filter(function owesBreaks(
        block,
        index,
      ): boolean {
        return (!pageKinds.has(block.kind,)) && (nonNullishOrThrow(sourceBreaks[index],) > 0);
      },)
      .map(function toKind(block,): string {
        return block.kind;
      },),),
  ];
  return owedKinds.flatMap(function compareKind(kind,): readonly string[] {
    /**
     Breaks the original's blocks of this kind carry together.
     */
    const owed = sourceBlocks.reduce(
      function sumOwed(
        sum,
        block,
        index,
      ): number {
        return (block.kind === kind) ? sum + nonNullishOrThrow(sourceBreaks[index],) : sum;
      },
      0,
    );
    /**
     Breaks the candidate's blocks of this kind carry together.
     */
    const carried = candidateBlocks.reduce(
      function sumCarried(
        sum,
        block,
        index,
      ): number {
        return (block.kind === kind) ? sum + nonNullishOrThrow(candidateBreaks[index],) : sum;
      },
      0,
    );
    if (carried >= owed)
      return [];
    return [
      `ORIGINAL ${kind} block(s) carry ${String(owed,)} explicit line break(s), `
        + `but your ${kind} block(s) carry ${String(carried,)}. The PAGE AS IT STANDS has no ${kind} `
        + 'block here, so the original alone sets that floor. Keep at least the original count with '
        + 'Markdown hard breaks or intrinsic <br/> elements; soft newlines render as spaces and do not '
        + 'preserve these lines. Keep your chosen wording.',
    ];
  },);
}

//endregion Explicit breaks under a substitute page block
