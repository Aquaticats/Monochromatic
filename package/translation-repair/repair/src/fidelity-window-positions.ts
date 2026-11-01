import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';

/**
 Selects shared source/archive positions without treating a forward heading as its body.

 Ordinary immediate neighbors stay unchanged. A standalone following heading
 may bring exactly its next body slice into view. Another heading or metadata
 stops the extension. Backward headings never pull in an earlier section body.
 Media and unknown nonempty slices count as bodies rather than being skipped.

 @param slices - paired physical slices in document order

 @param slicePosition - current position already validated by the public window reader

 @returns Immediate positions plus at most one heading-associated forward body

 @example
 ```ts
 const positions = fidelityWindowPositions({ slices, slicePosition: 0 });
 ```
 */
export function fidelityWindowPositions(
  {
    slices,
    slicePosition,
  }: {
    readonly slices: readonly ChunkPair[];
    readonly slicePosition: number;
  },
): readonly number[] {
  /**
   Immediate successor whose role may need its following body.
   */
  const nextPosition = slicePosition + 1;
  /**
   Candidate body belongs after that heading, never before it.
   */
  const bodyPosition = nextPosition + 1;
  /**
   Structural evidence from source, shared by both language views.
   */
  const next = slices[nextPosition];
  /**
   One further slice is the entire extension budget.
   */
  const body = slices[bodyPosition];
  /**
   Existing physical window survives every uncertain or forbidden extension.
   */
  const immediate = [
    slicePosition - 1,
    nextPosition,
  ];
  if ((next === undefined)
    || (next.syntax === 'front-matter')
    || (body === undefined)
    || (body.syntax === 'front-matter')) {
    return immediate;
  }
  /**
   Source structure must positively establish a standalone heading.
   */
  const { nodes: headingNodes, } = next.source;
  if (headingNodes.length === 0)
    return immediate;
  if (!headingNodes.every(function heading(node,): boolean {
    return node.kind === 'heading';
  },)) {
    return immediate;
  }
  /**
   Nonempty unknown nodes remain content; mixed heading/body starts another boundary.
   */
  const {
    nodes: bodyNodes,
    text: bodyText,
  } = body.source;
  if (bodyText.trim() === '')
    return immediate;
  if (bodyNodes.some(function heading(node,): boolean {
    return node.kind === 'heading';
  },)) {
    return immediate;
  }
  return [
    ...immediate,
    bodyPosition,
  ];
}
