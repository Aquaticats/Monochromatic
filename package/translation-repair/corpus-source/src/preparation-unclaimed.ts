import {
  chunkByHeadings,
  type SectionAlignment,
} from '@monochromatic-dev/translation-repair-slice-reading/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import type { DocumentNode, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import type { RepairDocument, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { UnclaimedTargetBlock, } from './prepared-document-pair.ts';

//region Target blocks outside alignment
// The archive blocks no aligned section owns, which no slice can review and
// which the block correction round is handed instead, split out of
// `document-preparation.ts` at its line budget.

/**
 Target blocks outside every aligned section, less the ones a seal covers.

 A SEALED BLOCK IS NOT UNCLAIMED: it is out of review by rule, and the block
 correction round must not be handed it either.

 @param alignment - aligned section pairs over both documents

 @param targetDocument - parsed archive

 @param sealedTargetIds - ids of every translation block a seal covers

 @returns Unclaimed blocks in document order, located by their section

 @example
 ```ts
 const unclaimed = unclaimedOutsideAlignment({ alignment, targetDocument, sealedTargetIds, },);
 ```
 */
export function unclaimedOutsideAlignment(
  {
    alignment,
    targetDocument,
    sealedTargetIds,
  }: {
    readonly alignment: SectionAlignment;
    readonly targetDocument: RepairDocument;
    readonly sealedTargetIds: ReadonlySet<string>;
  },
): readonly UnclaimedTargetBlock[] {
  /**
   Target node ids belonging to some aligned section.
   */
  const alignedTargetIds = new Set(
    alignment.pairs
      .flatMap(function toTargetIds(pair,): readonly string[] {
        return pair.target
          .nodes
          .map(function toId(node,): string {
            return node.id;
          },);
      },),
  );
  return chunkByHeadings({ document: targetDocument, },)
    .flatMap(function outsideAlignment(
      chunk,
      sectionIndex,
    ): readonly UnclaimedTargetBlock[] {
      return chunk.nodes
        .filter(function isOutside(node,): boolean {
          return (!alignedTargetIds.has(node.id,)) && (!sealedTargetIds.has(node.id,));
        },)
        .map(function toUnclaimed(node,): UnclaimedTargetBlock {
          return {
            location: {
              kind: 'target-section',
              sectionIndex,
            },
            blockId: node.id,
            startOffset: node.startOffset,
            endOffset: node.endOffset,
          };
        },);
    },);
}

/**
 Records of translation blocks inside one aligned chunk that reach no slice.

 @param pairIndex - aligned chunk the blocks sit in

 @param blocks - the blocks, in document order

 @returns One record per block, located by the chunk

 @example
 ```ts
 const unclaimed = unclaimedInAlignedPair({ pairIndex: 0, blocks: pair.target.nodes, },);
 ```
 */
export function unclaimedInAlignedPair(
  {
    pairIndex,
    blocks,
  }: {
    readonly pairIndex: number;
    readonly blocks: readonly DocumentNode[];
  },
): readonly UnclaimedTargetBlock[] {
  return blocks.map(function toUnclaimed(node,): UnclaimedTargetBlock {
    return {
      location: {
        kind: 'aligned-pair',
        pairIndex,
      },
      blockId: node.id,
      startOffset: node.startOffset,
      endOffset: node.endOffset,
    };
  },);
}

/**
 Alignment finding naming translation blocks of one chunk that reach no slice,
 by their positional ids, so the decision is legible from the artifact alone.

 @param label - what kind of finding this is, the word after `alignment`

 @param pairIndex - aligned chunk the blocks sit in

 @param blocks - the blocks, in document order

 @param reason - why no slice holds them, finishing the count sentence

 @returns Finding in the alignment channel's wording

 @example
 ```ts
 const finding = unclaimedFinding({
   label: 'target-unplaced',
   pairIndex: 0,
   blocks,
   reason: 'no run could take',
 },);
 ```
 */
export function unclaimedFinding(
  {
    label,
    pairIndex,
    blocks,
    reason,
  }: {
    readonly label: string;
    readonly pairIndex: number;
    readonly blocks: readonly DocumentNode[];
    readonly reason: string;
  },
): string {
  /**
   Characters the blocks hold, as offset spans.
   */
  const characters = blocks.reduce(
    function addChars(
      sum,
      node,
    ): number {
      return sum + (node.endOffset - node.startOffset);
    },
    0,
  );
  return `alignment ${label} (pair ${String(pairIndex,)}: ${String(blocks.length,)} translation ${
    wordForCount({
      count: blocks.length,
      one: 'block',
      many: 'blocks',
    },)
  } ${reason}, ${String(characters,)} ${
    wordForCount({
      count: characters,
      one: 'character',
      many: 'characters',
    },)
  }: ${blocks.map(function toId(node,): string {
    return node.id;
  },)
    .join(', ',)})`;
}

//endregion Target blocks outside alignment
