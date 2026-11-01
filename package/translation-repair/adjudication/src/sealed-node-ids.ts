import type { ArchiveOriginalSpan, } from '@monochromatic-dev/translation-repair-corpus-source/ts';

//region Original-language interval geometry has no runtime note-classification dependency

/**
 What a seal needs of a block: its id and where it sits.
 */
type PlacedBlock = {
  /**
   Stable node id.
   */
  readonly id: string;

  /**
   First offset the block owns.
   */
  readonly startOffset: number;

  /**
   Exclusive end offset.
   */
  readonly endOffset: number;
};

/**
 Ids of the nodes a set of spans seals: every node lying wholly inside one.

 WHOLLY, because a block straddling a seal boundary belongs to neither side
 cleanly; the note sits between blocks in both pinned pages, and a block cut
 by a seal would be a page shape this rule has not met.

 @param nodes - document nodes, any subset

 @param spans - sealed spans in the same offsets

 @returns Ids of the sealed nodes

 @example
 ```ts
 const sealed = sealedNodeIds({ nodes: pair.target.nodes, spans, },);
 ```
 */
export function sealedNodeIds(
  {
    nodes,
    spans,
  }: {
    readonly nodes: readonly PlacedBlock[];
    readonly spans: readonly ArchiveOriginalSpan[];
  },
): ReadonlySet<string> {
  return new Set(
    nodes
      .filter(function isSealed(node,): boolean {
        return spans.some(function covers(span,): boolean {
          return (node.startOffset >= span.startOffset) && (node.endOffset <= span.endOffset);
        },);
      },)
      .map(function toId(node,): string {
        return node.id;
      },),
  );
}

//endregion Original-language interval geometry has no runtime note-classification dependency
