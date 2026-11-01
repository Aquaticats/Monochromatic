/**
 Tests for the seam between the block pairing and the footnote relabel:
 definition blocks named as order-free, a crossing definition pair kept out
 of the slicing and read by label.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { parseDocument, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  crossingFinding,
  definitionIndexes,
  definitionLabelsOf,
  splitDefinitionPairs,
} from '../dist/final/node/index.mjs';

//region Fixtures

/**
 Original: the kitten next door is the first note, the old cat who took her in the second.
 */
const SOURCE = parseDocument({
  text: '## 生平\n\n团团[^2]收留了她，豆豆[^1]陪伴她。\n\n[^1]: 隔壁的小猫，总爱跟着她。\n\n[^2]: 收留她的老猫，像妈妈一样。\n',
},);

/**
 Archive: renumbered by first appearance, so its definitions cross the
 original's when paired by content.
 */
const TARGET = parseDocument({
  text: '## Life\n\nTuantuan[^1] took her in.\n\nDoudou[^2] kept her company.\n\n'
    + '[^1]: The old cat who took her in, like a mother.\n\n[^2]: The kitten next door, always following her.\n',
},);

//endregion Fixtures

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: definitionIndexes.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'names the footnote definition blocks by their chunk-local index and nothing else',
          fn: async () => {
            expect([ ...definitionIndexes({ nodes: SOURCE.nodes, },), ],).toStrictEqual([
              2,
              3,
            ],);
            expect([ ...definitionIndexes({ nodes: TARGET.nodes, },), ],).toStrictEqual([
              3,
              4,
            ],);
          },
        },),
      ],
    },),

    describe({
      name: definitionLabelsOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'reads the label a definition opens with and nothing off a body block',
          fn: async () => {
            expect(SOURCE.nodes
              .map(function toLabels(node,): readonly string[] {
                return definitionLabelsOf({ node, },);
              },),).toStrictEqual([
                [],
                [],
                [ '1', ],
                [ '2', ],
              ],);
          },
        },),
      ],
    },),

    describe({
      name: splitDefinitionPairs.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'keeps crossing definition pairs out of the slicing and reads them by label for the relabel',
          fn: async () => {
            /**
             The pairing six of eight voices gave on one entry's third launch: body
             paired in order, definitions paired by content, crossing.
             */
            const split = splitDefinitionPairs({
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
                  source: 1,
                  target: 2,
                },
                {
                  source: 2,
                  target: 4,
                },
                {
                  source: 3,
                  target: 3,
                },
              ],
              sourceNodes: SOURCE.nodes,
              targetNodes: TARGET.nodes,
            },);
            expect(split.crossing,).toBe(true,);
            expect(split.forSlicing,).toStrictEqual([
              {
                source: 0,
                target: 0,
              },
              {
                source: 1,
                target: 1,
              },
              {
                source: 1,
                target: 2,
              },
            ],);
            expect(split.definitionPairs,).toStrictEqual([
              {
                sourceLabel: '1',
                targetLabel: '2',
              },
              {
                sourceLabel: '2',
                targetLabel: '1',
              },
            ],);
          },
        },),

        it({
          name: 'hands every pair to the slicing, definitions included, where the definitions pair in order',
          fn: async () => {
            /**
             The same documents after the relabel and the reorder: definitions
             pair in order.
             */
            const split = splitDefinitionPairs({
              pairs: [
                {
                  source: 1,
                  target: 1,
                },
                {
                  source: 2,
                  target: 3,
                },
                {
                  source: 3,
                  target: 4,
                },
              ],
              sourceNodes: SOURCE.nodes,
              targetNodes: TARGET.nodes,
            },);
            expect(split.crossing,).toBe(false,);
            expect(split.forSlicing
              .length,).toBe(3,);
            expect(split.definitionPairs
              .length,).toBe(2,);
            expect(crossingFinding({ pairIndex: 4, },),).toContain('section 4',);
          },
        },),

        it({
          name: 'reads two definition pairs sharing an original as in order whichever comes first, '
            + 'since pairs tied on the original are ordered by the archive before any step back is sought',
          fn: async () => {
            /**
             One original note the archive split across both its definitions,
             given with the later archive block first.
             */
            const pairs = [
              {
                source: 2,
                target: 4,
              },
              {
                source: 2,
                target: 3,
              },
            ];
            const split = splitDefinitionPairs({
              pairs,
              sourceNodes: SOURCE.nodes,
              targetNodes: TARGET.nodes,
            },);
            expect(split.crossing,).toBe(false,);
            expect(split.forSlicing,).toStrictEqual(pairs,);
            expect(split.definitionPairs,).toStrictEqual([
              {
                sourceLabel: '1',
                targetLabel: '2',
              },
              {
                sourceLabel: '1',
                targetLabel: '1',
              },
            ],);
          },
        },),
      ],
    },),
  ],
},);
