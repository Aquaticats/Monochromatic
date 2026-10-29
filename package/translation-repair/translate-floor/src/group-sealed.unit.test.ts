/**
 Tests for grouping with sealed blocks: the blocks the archive's note seals
 reach no run, take their paired originals with them, and stand as
 boundaries for the originals either side of them.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { parseDocument, } from '@monochromatic-dev/translation-repair-shared-contract';
import type { DocumentNode, } from '@monochromatic-dev/translation-repair-document-structure';
import { blockPairingToSteps, } from '@monochromatic-dev/translation-repair-stage';
import {
  type AlignedRun,
  groupNodesSealed,
} from '../dist/final/node/index.mjs';

//region Fixtures

/**
 Budget wide enough that nothing closes on size.
 */
const WIDE_BUDGET = 100_000;

/**
 Four originals: an intro, a letter, a closing line and a footnote definition
 the archive never rendered.
 */
const SOURCE_TEXT = '她在门口放了一封信。\n\n> 罐头在第二个柜子里\n\n是时候去睡午觉了。\n\n[^1]: 即 Cat Nap Planner\n';

/**
 Three renderings: the intro, the letter and the closing line.
 */
const TARGET_TEXT = 'She put a letter by the door.\n\n> The cans are in the second cupboard,\n\nTime for an afternoon nap.\n';

/**
 Blocks of a text.

 @param text - document

 @returns Its top-level blocks

 @example
 ```ts
 const nodes = blocksOf({ text: SOURCE_TEXT, },);
 ```
 */
function blocksOf({ text, }: { readonly text: string; },): readonly DocumentNode[] {
  return parseDocument({ text, },).nodes;
}

/**
 Node at a position, or a thrown absence.

 @param nodes - blocks

 @param at - position

 @returns Block

 @example
 ```ts
 const letter = nodeAt({ nodes: targetNodes, at: 1, },);
 ```
 */
function nodeAt(
  {
    nodes,
    at,
  }: {
    readonly nodes: readonly DocumentNode[];
    readonly at: number;
  },
): DocumentNode {
  /**
   The block.
   */
  const node = nodes[at];
  if (node === undefined)
    throw new Error(`no block at ${String(at,)}`,);
  return node;
}

/**
 Ids a list of runs carries on the translation side, in order.

 @param runs - settled runs

 @returns Target ids

 @example
 ```ts
 const ids = targetIdsOf({ runs, },);
 ```
 */
function targetIdsOf({ runs, }: { readonly runs: readonly AlignedRun[]; },): readonly string[] {
  return runs.flatMap(function toIds(run,): readonly string[] {
    return (run.kind === 'insertion')
      ? []
      : run.targetRun
        .map(function toId(node,): string {
          return node.id;
        },);
  },);
}

/**
 Ids a list of runs carries on the original side, in order.

 @param runs - settled runs

 @returns Source ids

 @example
 ```ts
 const ids = sourceIdsOf({ runs, },);
 ```
 */
function sourceIdsOf({ runs, }: { readonly runs: readonly AlignedRun[]; },): readonly string[] {
  return runs.flatMap(function toIds(run,): readonly string[] {
    return run.sourceRun
      .map(function toId(node,): string {
        return node.id;
      },);
  },);
}

//endregion Fixtures

await describe({
  name: groupNodesSealed.name,
  children: [
    it({
      name: 'drops a sealed block with the original paired to it, keeps the runs either side apart, and '
        + 'writes an original behind the seal at the seal end rather than before it',
      fn: async () => {
        const sourceNodes = blocksOf({ text: SOURCE_TEXT, },);
        const targetNodes = blocksOf({ text: TARGET_TEXT, },);
        /**
         The letter and the closing line, sealed.
         */
        const letter = nodeAt({
          nodes: targetNodes,
          at: 1,
        },);
        const closing = nodeAt({
          nodes: targetNodes,
          at: 2,
        },);
        const {
          runs,
          sealedSourceIds,
        } = groupNodesSealed({
          sourceNodes,
          targetNodes,
          sourceBudget: WIDE_BUDGET,
          targetBudget: WIDE_BUDGET,
          steps: blockPairingToSteps({
            pairs: [
              {
                source: 0,
                target: 0,
              },
              {
                source: 1,
                target: 1,
              },
              {
                source: 2,
                target: 2,
              },
            ],
            sourceCount: sourceNodes.length,
            targetCount: targetNodes.length,
          },),
          sealed: new Set([
            letter.id,
            closing.id,
          ],),
        },);
        // THE INTRO IS THE ONE PAIRED RUN; the footnote definition is an
        // insertion at the END of the sealed closing line, not after the intro.
        expect(targetIdsOf({ runs, },),).toStrictEqual([ nodeAt({
          nodes: targetNodes,
          at: 0,
        },).id, ],);
        expect(sourceIdsOf({ runs, },),).toStrictEqual([
          nodeAt({
            nodes: sourceNodes,
            at: 0,
          },).id,
          nodeAt({
            nodes: sourceNodes,
            at: 3,
          },).id,
        ],);
        /**
         The insertion run.
         */
        const insertion = runs.find(function isInsertion(run,): boolean {
          return run.kind === 'insertion';
        },);
        if ((insertion === undefined) || (insertion.kind !== 'insertion'))
          throw new Error('expected an insertion run for the footnote definition',);
        expect(insertion.targetOffset,).toBe(closing.endOffset,);
        expect([ ...sealedSourceIds, ],).toStrictEqual([
          nodeAt({
            nodes: sourceNodes,
            at: 1,
          },).id,
          nodeAt({
            nodes: sourceNodes,
            at: 2,
          },).id,
        ],);
      },
    },),

    it({
      name: 'seals one block in the middle and keeps the blocks either side in separate runs',
      fn: async () => {
        const sourceNodes = blocksOf({ text: SOURCE_TEXT, },);
        const targetNodes = blocksOf({ text: TARGET_TEXT, },);
        const letter = nodeAt({
          nodes: targetNodes,
          at: 1,
        },);
        const { runs, } = groupNodesSealed({
          sourceNodes,
          targetNodes,
          sourceBudget: WIDE_BUDGET,
          targetBudget: WIDE_BUDGET,
          steps: blockPairingToSteps({
            pairs: [
              {
                source: 0,
                target: 0,
              },
              {
                source: 1,
                target: 1,
              },
              {
                source: 2,
                target: 2,
              },
            ],
            sourceCount: sourceNodes.length,
            targetCount: targetNodes.length,
          },),
          sealed: new Set([ letter.id, ],),
        },);
        /**
         Paired runs, which must not have merged across the seal.
         */
        const paired = runs.filter(function isPaired(run,): boolean {
          return run.kind === 'paired';
        },);
        expect(paired.length,).toBe(2,);
        expect(targetIdsOf({ runs, },),).toStrictEqual([
          nodeAt({
            nodes: targetNodes,
            at: 0,
          },).id,
          nodeAt({
            nodes: targetNodes,
            at: 2,
          },).id,
        ],);
      },
    },),

    it({
      name: 'seals under the scorer walk too, when no roster pairing was supplied',
      fn: async () => {
        const sourceNodes = blocksOf({ text: SOURCE_TEXT, },);
        const targetNodes = blocksOf({ text: TARGET_TEXT, },);
        const letter = nodeAt({
          nodes: targetNodes,
          at: 1,
        },);
        const { runs, } = groupNodesSealed({
          sourceNodes,
          targetNodes,
          sourceBudget: WIDE_BUDGET,
          targetBudget: WIDE_BUDGET,
          sealed: new Set([ letter.id, ],),
        },);
        /**
         Every translation id the runs carry.
         */
        const ids = targetIdsOf({ runs, },);
        expect(ids,).not
          .toContain(letter.id,);
        expect(ids,).toContain(nodeAt({
          nodes: targetNodes,
          at: 0,
        },).id,);
        expect(ids,).toContain(nodeAt({
          nodes: targetNodes,
          at: 2,
        },).id,);
      },
    },),

    it({
      name: 'WRITES AN ORIGINAL LEFT BEHIND THE LAST SEAL AT THE SEAL\'S END under the scorer walk, where no pairing '
        + 'anchors it and nothing after the seal can take it (ledger T8)',
      fn: async () => {
        const sourceNodes = blocksOf({ text: SOURCE_TEXT, },);
        const targetNodes = blocksOf({ text: TARGET_TEXT, },);
        /**
         The closing line, sealed, which leaves the footnote definition behind
         the seal with no run after it.
         */
        const closing = nodeAt({
          nodes: targetNodes,
          at: 2,
        },);
        const { runs, } = groupNodesSealed({
          sourceNodes,
          targetNodes,
          sourceBudget: WIDE_BUDGET,
          targetBudget: WIDE_BUDGET,
          sealed: new Set([ closing.id, ],),
        },);
        expect(runs.at(-1,),).toStrictEqual({
          kind: 'insertion',
          sourceRun: [ nodeAt({
            nodes: sourceNodes,
            at: 3,
          },), ],
          targetOffset: closing.endOffset,
        },);
      },
    },),

    it({
      name: 'carries a continuation\'s original with the seal that covers its rendering, so the sealed '
        + 'block and the originals it renders leave together rather than the continuation becoming a '
        + 'sealed run of its own',
      fn: async () => {
        /**
         Originals whose renderings the one sealed block covers.
         */
        const sourceNodes = blocksOf({ text: '猫猫在窗台上睡觉。\n\n猫猫追蝴蝶。\n', },);

        /**
         The single rendering, sealed.
         */
        const targetNodes = blocksOf({ text: 'The cat sleeps on the windowsill, and chases butterflies.\n', },);
        const {
          runs,
          sealedSourceIds,
        } = groupNodesSealed({
          sourceNodes,
          targetNodes,
          sourceBudget: WIDE_BUDGET,
          targetBudget: WIDE_BUDGET,
          steps: [
            {
              kind: 'paired',
              sourceIndex: 0,
              targetIndex: 0,
            },
            {
              kind: 'source-only',
              sourceIndex: 1,
              continuesPairing: true,
            },
          ],
          sealed: new Set([ nodeAt({
            nodes: targetNodes,
            at: 0,
          },).id, ],),
        },);
        expect([ ...sealedSourceIds, ],).toStrictEqual([
          nodeAt({
            nodes: sourceNodes,
            at: 0,
          },).id,
          nodeAt({
            nodes: sourceNodes,
            at: 1,
          },).id,
        ],);
        expect(runs,).toStrictEqual([],);
      },
    },),
  ],
},);
