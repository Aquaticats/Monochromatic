import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  alignBlocks,
  estimateExpansion,
  FALLBACK_EXPANSION,
  scorePairing,
  tokenize,
} from '@monochromatic-dev/translation-repair-document-structure';
import { parseDocument, } from '../dist/final/node/index.mjs';

//region Block alignment tests
// The regression fixture reproduces the STRUCTURE read off the pinned corpus,
// with cat-themed invented text: both sides carry the same block count, yet the
// translation drops a short lead-in paragraph and folds it into the quotation
// that follows. Index pairing silently drifts from that point on; a correct
// alignment leaves the lead-in unpartnered and pairs the quotations.

/**
 Original side: a lead-in paragraph introduces the quotation.
 */
const SOURCE_PAGE = `## Memories

Mittens loved tuna and naps in equal measure.

Her sister said to Mittens:

> I will wait for you by the window.

In 2023 Mittens moved to Catbury.
`;

/**
 Translation side: same block count, but the lead-in is folded away and a
 closing paragraph appears instead.
 */
const TARGET_PAGE = `## Memories

Mittens loved tuna and naps in equal measure.

> I will wait for you by the window, her sister said.

In 2023 Mittens moved to Catbury.

She never did settle in.
`;

await describe({
  name: '',
  children: [
    describe({
      name: tokenize.name,
      children: [
        it({
          name: 'collects Latin word and digit runs, lowercased and deduplicated',
          fn: async () => {
            expect(
              [ ...tokenize({ text: 'Mittens played THE FINALS in 2023, mittens!', },), ],
            ).toEqual([
              'mittens',
              'played',
              'the',
              'finals',
              'in',
              '2023',
            ],);
          },
        },),

        it({
          name: 'ignores cjk, whose characters would match across unrelated blocks',
          fn: async () => {
            expect([ ...tokenize({ text: '她喜欢金枪鱼', },), ],).toHaveLength(0,);
          },
        },),

        it({
          name: 'drops single characters, which collide constantly',
          fn: async () => {
            expect([ ...tokenize({ text: 'a b cd', },), ],).toEqual([ 'cd', ],);
          },
        },),

        it({
          name: 'folds an accented word to one token, opening or carrying the accent, composed or combining, '
            + 'which ASCII runs cut at the accent (ledger B18)',
          fn: async () => {
            expect([ ...tokenize({ text: 'Émile napped in the Château, the cha\u{0302}teau and à Chateau', },), ],)
              .toEqual([
                'emile',
                'napped',
                'in',
                'the',
                'chateau',
                'and',
              ],);
          },
        },),
      ],
    },),

    describe({
      name: scorePairing.name,
      children: [
        it({
          name: 'does not let a one-token block outscore a richer genuine partner',
          fn: async () => {
            // The inverted-evidence bug: dividing by the smaller token set let
            // a block carrying a single shared token score a perfect match, so
            // MORE evidence scored worse. Both blocks of this case share exactly
            // one token, so the length fit must decide.
            /**
             Original blocks: a rich paragraph, then a one-token stub.
             */
            const { nodes: sourceNodes, } = parseDocument({
              text: `## H

Mittens adored tuna, and would sing for it every morning without fail.

Mittens.
`,
            },);

            /**
             Blocks of the page the candidates compete against.
             */
            const { nodes: partnerNodes, } = parseDocument({
              text: `## H

Mittens adored tuna, and would sing for it every single morning without ever failing.
`,
            },);

            /**
             The block both candidates compete to partner.
             */
            const partner = nonNullishOrThrow(partnerNodes.at(1,),);

            expect(
              scorePairing({
                source: nonNullishOrThrow(sourceNodes.at(1,),),
                target: partner,
              },),
            ).toBeGreaterThan(
              scorePairing({
                source: nonNullishOrThrow(sourceNodes.at(2,),),
                target: partner,
              },),
            );
          },
        },),

        it({
          name: 'scores a same-kind pairing above a different-kind one',
          fn: async () => {
            /**
             Same sentence as a paragraph and as a quotation, so kind is the
             only thing that differs.
             */
            const { nodes, } = parseDocument({
              text: `Mittens waited by the window.

> Mittens waited by the window.
`,
            },);

            /**
             Paragraph form.
             */
            const paragraph = nonNullishOrThrow(nodes.at(0,),);

            /**
             Quotation form.
             */
            const quote = nonNullishOrThrow(nodes.at(1,),);

            expect(
              scorePairing({
                source: paragraph,
                target: paragraph,
              },),
            ).toBeGreaterThan(
              scorePairing({
                source: paragraph,
                target: quote,
              },),
            );
          },
        },),

        it({
          name: 'READS NO LENGTH FIT FOR A BLOCK WITH NO CHARACTERS: two empty blocks score as an empty block does '
            + 'beside a worded one, a finite number rather than the 0/0 of their lengths',
          fn: async () => {
            /**
             A worded paragraph.
             */
            const paragraph = nonNullishOrThrow(parseDocument({ text: 'Mittens.\n', },).nodes
              .at(0,),);

            /**
             The same block with no characters, which a caller of the
             exported scorer may hand it though no parse writes one.
             */
            const empty = {
              ...paragraph,
              text: '',
            };

            /**
             Score of two empty blocks.
             */
            const bothEmpty = scorePairing({
              source: empty,
              target: empty,
            },);
            expect(Number.isFinite(bothEmpty,),).toBe(true,);
            expect(bothEmpty,).toBe(scorePairing({
              source: empty,
              target: paragraph,
            },),);
          },
        },),
      ],
    },),

    describe({
      name: alignBlocks.name,
      children: [
        it({
          name: 'leaves a dropped lead-in unpartnered instead of drifting, the graded failure',
          fn: async () => {
            /**
             Both sides carry the same count, which is exactly what made the
             old lockstep pairing believe correspondence was guaranteed.
             */
            const { nodes: sourceNodes, } = parseDocument({ text: SOURCE_PAGE, },);

            /**
             Translation blocks in document order.
             */
            const { nodes: targetNodes, } = parseDocument({ text: TARGET_PAGE, },);

            expect(sourceNodes.length,).toBe(targetNodes.length,);

            /**
             Monotone alignment over the two block lists.
             */
            const steps = alignBlocks({
              sourceNodes,
              targetNodes,
            },);

            /**
             Index of the source-side lead-in paragraph.
             */
            const leadInIndex = sourceNodes.findIndex(function isLeadIn(node,) {
              return node.text
                .includes('Her sister said',);
            },);

            expect(
              steps.some(function skipsLeadIn(step,) {
                return (step.kind === 'source-only')
                  && (step.sourceIndex === leadInIndex);
              },),
            ).toBe(true,);

            // The quotations are the pair index-based walking got wrong.
            /**
             Source-side quotation index.
             */
            const sourceQuote = sourceNodes.findIndex(function isQuote(node,) {
              return node.kind === 'blockquote';
            },);

            /**
             Translation-side quotation index.
             */
            const targetQuote = targetNodes.findIndex(function isQuote(node,) {
              return node.kind === 'blockquote';
            },);

            expect(
              steps.some(function pairsQuotes(step,) {
                return (step.kind === 'paired')
                  && (step.sourceIndex === sourceQuote)
                  && (step.targetIndex === targetQuote);
              },),
            ).toBe(true,);
          },
        },),

        it({
          name: 'pairs every block when both sides truly correspond',
          fn: async () => {
            const { nodes, } = parseDocument({ text: SOURCE_PAGE, },);

            /**
             Aligning a document against itself must pair everything.
             */
            const steps = alignBlocks({
              sourceNodes: nodes,
              targetNodes: nodes,
            },);

            expect(steps,).toHaveLength(nodes.length,);
            expect(
              steps.every(function isPaired(step,) {
                return step.kind === 'paired';
              },),
            ).toBe(true,);
          },
        },),

        it({
          name: 'covers every block on both sides exactly once',
          fn: async () => {
            const { nodes: sourceNodes, } = parseDocument({ text: SOURCE_PAGE, },);
            const { nodes: targetNodes, } = parseDocument({ text: TARGET_PAGE, },);

            /**
             Alignment whose coverage is under test.
             */
            const steps = alignBlocks({
              sourceNodes,
              targetNodes,
            },);

            /**
             Source indices the alignment touched.
             */
            const sourceSeen = steps.flatMap(function toSource(step,) {
              return step.kind === 'target-only'
                ? []
                : [ step.sourceIndex, ];
            },);

            /**
             Target indices the alignment touched.
             */
            const targetSeen = steps.flatMap(function toTarget(step,) {
              return step.kind === 'source-only'
                ? []
                : [ step.targetIndex, ];
            },);

            expect(sourceSeen.toSorted(function ascending(a, b,) {
              return a - b;
            },),).toEqual(sourceNodes.map(function toIndex(_node, index,) {
              return index;
            },),);
            expect(targetSeen.toSorted(function ascending(a, b,) {
              return a - b;
            },),).toEqual(targetNodes.map(function toIndex(_node, index,) {
              return index;
            },),);
          },
        },),

        it({
          name: 'keeps order monotone on both sides',
          fn: async () => {
            const { nodes: sourceNodes, } = parseDocument({ text: SOURCE_PAGE, },);
            const { nodes: targetNodes, } = parseDocument({ text: TARGET_PAGE, },);

            /**
             Alignment whose ordering is under test.
             */
            const steps = alignBlocks({
              sourceNodes,
              targetNodes,
            },);

            /**
             Highest source index emitted so far.
             */
            let lastSource = -1;

            /**
             Highest target index emitted so far.
             */
            let lastTarget = -1;
            for (const step of steps) {
              if (step.kind !== 'target-only') {
                expect(step.sourceIndex,).toBeGreaterThan(lastSource,);
                lastSource = step.sourceIndex;
              }
              if (step.kind !== 'source-only') {
                expect(step.targetIndex,).toBeGreaterThan(lastTarget,);
                lastTarget = step.targetIndex;
              }
            }
          },
        },),

        it({
          name: 'aligns an empty side as all skips rather than failing',
          fn: async () => {
            const { nodes, } = parseDocument({ text: SOURCE_PAGE, },);

            expect(
              alignBlocks({
                sourceNodes: nodes,
                targetNodes: [],
              },),
            ).toHaveLength(nodes.length,);
            expect(
              alignBlocks({
                sourceNodes: [],
                targetNodes: [],
              },),
            ).toHaveLength(0,);
          },
        },),
      ],
    },),

    describe({
      name: estimateExpansion.name,
      children: [
        it({
          name: 'reads the ratio off both whole block lists, since a single block is exactly the thing '
            + 'whose pairing is in question and cannot be used to judge itself',
          fn: async () => {
            expect(estimateExpansion({
              sourceNodes: parseDocument({ text: '猫猫睡觉。\n\n猫猫追蝴蝶。\n', },).nodes,
              targetNodes: parseDocument({
                text: 'The cat sleeps.\n\nThe cat chases butterflies.\n',
              },).nodes,
            },),).toBe(42 / 11,);
          },
        },),
        it({
          name: 'FALLS BACK where either side carries no characters, rather than dividing by nothing or '
            + 'reading the expansion as zero. A ratio of zero would score every pairing on length '
            + 'pointing the wrong way, which is the reading '
            + '`doc/audit/the-critics-are-shown-the-wrong-paragraph.md` records, and an empty side is '
            + 'exactly the page this lane exists to fill',
          fn: async () => {
            expect(estimateExpansion({
              sourceNodes: parseDocument({ text: '猫猫睡觉。\n', },).nodes,
              targetNodes: [],
            },),).toBe(FALLBACK_EXPANSION,);
            expect(estimateExpansion({
              sourceNodes: [],
              targetNodes: parseDocument({ text: 'The cat sleeps.\n', },).nodes,
            },),).toBe(FALLBACK_EXPANSION,);
          },
        },),
      ],
    },),
  ],
},);

//endregion Block alignment tests
