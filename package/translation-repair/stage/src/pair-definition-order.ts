import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import {
  type DocumentNode,
  scanGfmReferenceLiterals,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import type { BlockPair, } from './pair-blocks-wire.ts';

//region Definition pairs and their order
// FOOTNOTE DEFINITIONS ARE ORDER-FREE BLOCKS. A page renders its notes by the
// order they are referenced, not by where their definitions sit, so two
// documents may carry the same notes in a different definition order and both
// be right. The block pairing's rule that a correspondence never steps
// backwards is a rule about prose, and on the third `yuki418330012` launch of
// 2026-09-08 it refused six of eight voices for pairing the two definitions by
// content, since the archive had renumbered them (the nineteenth class). The
// pairing starved to two voices and three of eight blocks.
//
// THIS MODULE IS THE SEAM. The wire reader exempts the definition blocks it is
// told about from the order rule; this module names those blocks, keeps a
// crossing definition pair out of the slicer (whose runs must be in document
// order on both sides), and hands the pair's labels to the relabel, which
// makes the archive's labels and definition order the original's, so the next
// preparation pairs them in order like any other block.

/**
 Zone a parsed footnote definition block carries.
 */
const DEFINITION_ZONE = 'footnote-definition';

/**
 One definition the roster paired with one definition, by label.

 @example
 ```ts
 const pair: DefinitionLabelPair = { sourceLabel: '2', targetLabel: '1', };
 ```
 */
export type DefinitionLabelPair = {
  /**
   Label of the original's definition.
   */
  readonly sourceLabel: string;

  /**
   Label of the archive's definition paired with it.
   */
  readonly targetLabel: string;
};

/**
 What one chunk's pairing splits into.

 @example
 ```ts
 const split: SplitDefinitionPairs = splitDefinitionPairs({ pairs, sourceNodes, targetNodes, },);
 ```
 */
export type SplitDefinitionPairs = {
  /**
   Pairs the slicer may walk: every body pair, and the definition pairs too
   unless they cross.
   */
  readonly forSlicing: readonly BlockPair[];

  /**
   Every definition pair, by label, for the relabel.
   */
  readonly definitionPairs: readonly DefinitionLabelPair[];

  /**
   Whether the definition pairs cross, and so were kept out of the slicing.
   */
  readonly crossing: boolean;
};

/**
 Indices, among one chunk's nodes, of the footnote definition blocks.

 @param nodes - one side of a chunk

 @returns Chunk-local indices of the definitions

 @example
 ```ts
 const free = { source: definitionIndexes({ nodes: sourceNodes, },), target: definitionIndexes({ nodes: targetNodes, },), };
 ```
 */
export function definitionIndexes(
  { nodes, }: { readonly nodes: readonly DocumentNode[]; },
): ReadonlySet<number> {
  return new Set(
    nodes
      .map(function toIndex(
        node,
        index,
      ): number {
        return (node.zone === DEFINITION_ZONE) ? index : (-1);
      },)
      .filter(function isDefinition(index,): boolean {
        return index >= 0;
      },),
  );
}

/**
 Label a definition block opens with, as a list of one, empty for any other
 block.

 @param node - block to read

 @returns One opening label or an empty collection; this reader never returns multiple labels

 @example
 ```ts
 definitionLabelsOf({ node, },);
 // => ['2']
 ```
 */
export function definitionLabelsOf(
  { node, }: { readonly node: DocumentNode; },
): readonly string[] {
  if (node.zone !== DEFINITION_ZONE)
    return [];
  /**
   The first marker literal in the block, the opener when it sits at offset
   zero.
   */
  const [first,] = scanGfmReferenceLiterals({ slice: node.text, },);
  if ((first === undefined) || (first.localOffset !== 0))
    return [];
  return [ first.identifier, ];
}

/**
 Whether pairs, sorted by original then translation, ever step backwards on
 the translation side.

 @param pairs - pairs to read

 @returns Whether any two cross

 @example
 ```ts
 crosses({ pairs: [ { source: 7, target: 12, }, { source: 8, target: 11, }, ], },);
 // => true
 ```
 */
function crosses({ pairs, }: { readonly pairs: readonly BlockPair[]; },): boolean {
  /**
   Pairs in document order on the original side.
   */
  const sorted = pairs.toSorted(function bySourceThenTarget(
    left,
    right,
  ): number {
    return (left.source - right.source) || (left.target - right.target);
  },);
  return sorted.some(function stepsBack(
    pair,
    at,
  ): boolean {
    /**
     The pair before this one in that order.
     */
    const previous = sorted[at - 1];
    return (previous !== undefined) && (pair.target < previous.target);
  },);
}

/**
 One pair whose blocks are both definitions, with those blocks.
 */
type DefinitionBlockPair = {
  /**
   The pair as the roster agreed it.
   */
  readonly pair: BlockPair;

  /**
   The original's definition block.
   */
  readonly sourceNode: DocumentNode;

  /**
   The archive's definition block.
   */
  readonly targetNode: DocumentNode;
};

/**
 The block a pair names on one side.

 @param nodes - one side of a chunk

 @param index - block a pair names

 @returns Block

 @throws Error when the index names no block, which would mean the pair was
 read against some other chunk

 @example
 ```ts
 const node = blockAt({ nodes: sourceNodes, index: 7, },);
 ```
 */
function blockAt(
  {
    nodes,
    index,
  }: {
    readonly nodes: readonly DocumentNode[];
    readonly index: number;
  },
): DocumentNode {
  // A PAIR NAMES A BLOCK OF THE CHUNK IT WAS ASKED ABOUT: the roster's reads are
  // range-checked against these sides by `readBlockPairing`, and
  // `prepareBlockPairing` reads a cached record against them through
  // `readBlockPairing` too, before it splits the record, missing on one that
  // does not fit.
  return nonNullishOrThrow(nodes[index],);
}

/**
 The label a definition block opens with.

 @param node - footnote definition block

 @returns Its label

 @throws Error when the block opens with no label the marker scanner reads,
 which would mean the scanner and the parser disagree about a label

 @example
 ```ts
 const label = openingLabel({ node: sourceNode, },);
 ```
 */
function openingLabel({ node, }: { readonly node: DocumentNode; },): string {
  // A FOOTNOTE DEFINITION OPENS WITH ITS LABEL: a block is zoned as one only
  // for the parser's `footnoteDefinition` node, whose text starts `[^label]:`,
  // and `gfmMarkerSpans` reads a label by micromark's own rule. A pair dropped
  // here instead would leave the relabel one definition short, unsaid.
  /**
   The label, alone in its list for a definition block.
   */
  const [label,] = definitionLabelsOf({ node, },);
  return nonNullishOrThrow(label,);
}

/**
 Splits one chunk's agreed pairing into what the slicer walks and what the
 relabel reads.

 @param pairs - pairs the roster agreed, chunk-local

 @param sourceNodes - original side of the chunk

 @param targetNodes - archive side of the chunk

 @returns Split

 @example
 ```ts
 const split = splitDefinitionPairs({ pairs, sourceNodes, targetNodes, },);
 blockPairings.set(pairIndex, split.forSlicing,);
 ```
 */
export function splitDefinitionPairs(
  {
    pairs,
    sourceNodes,
    targetNodes,
  }: {
    readonly pairs: readonly BlockPair[];
    readonly sourceNodes: readonly DocumentNode[];
    readonly targetNodes: readonly DocumentNode[];
  },
): SplitDefinitionPairs {
  /**
   Pairs whose both sides are definitions, with their blocks.
   */
  const definitionBlocks = pairs.flatMap(function asDefinitionPair(pair,): readonly DefinitionBlockPair[] {
    /**
     The original's block.
     */
    const sourceNode = blockAt({
      nodes: sourceNodes,
      index: pair.source,
    },);
    /**
     The archive's block.
     */
    const targetNode = blockAt({
      nodes: targetNodes,
      index: pair.target,
    },);
    if ((sourceNode.zone !== DEFINITION_ZONE) || (targetNode.zone !== DEFINITION_ZONE))
      return [];
    return [ {
      pair,
      sourceNode,
      targetNode,
    }, ];
  },);
  /**
   The definition pairs themselves.
   */
  const definitionBlockPairs = definitionBlocks.map(function toPair(definition,): BlockPair {
    return definition.pair;
  },);
  /**
   The same pairs by label.
   */
  const definitionPairs = definitionBlocks.map(function toLabels(definition,): DefinitionLabelPair {
    return {
      sourceLabel: openingLabel({ node: definition.sourceNode, },),
      targetLabel: openingLabel({ node: definition.targetNode, },),
    };
  },);
  /**
   Whether the definitions cross, in which case the slicer walks the body
   pairs alone.
   */
  const crossing = crosses({ pairs: definitionBlockPairs, },);
  return {
    forSlicing: crossing
      ? pairs.filter(function isBodyPair(pair,): boolean {
        return !definitionBlockPairs.includes(pair,);
      },)
      : pairs,
    definitionPairs,
    crossing,
  };
}

/**
 Finding and log line for a chunk whose definition pairs cross.

 @param pairIndex - aligned chunk

 @returns Line, in the block-pairing findings' wording

 @example
 ```ts
 findings.push(crossingFinding({ pairIndex, },),);
 ```
 */
export function crossingFinding({ pairIndex, }: { readonly pairIndex: number; },): string {
  return `block-pairing section ${String(pairIndex,)}: the footnote definitions cross, kept out of the slicing `
    + 'and read for the relabel';
}

//endregion Definition pairs and their order
