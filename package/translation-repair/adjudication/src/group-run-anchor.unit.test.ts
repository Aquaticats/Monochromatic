/**
 Tests for where an insertion is anchored once the runs have settled.

 THE ANCHOR AND THE SPAN USED TO DESCRIBE THE SAME BYTES. `anchorOffsets`
 reads the monotone walk, which describes the layout right up until
 `mergeOneSidedRuns` folds an unclaimed translation block into a neighbour. A
 run's span is cut from its first node to its last, so absorbing that block
 stretches the span over it, and an anchor naming its start then points into
 the middle of a passage. `assertPlacementLayout` refused the whole document:
 431 of 910 randomised reader-legal pairings over the pinned corpus.

 THE SMALLEST CASE IS TWO PARAGRAPHS AGAINST TWO, which is what these use.
 Pair only the first, and the second original becomes an insertion while the
 second translation folds into the first run.

 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { parseDocument, } from '@monochromatic-dev/translation-repair-shared-contract';
import { blockPairingToSteps, } from '@monochromatic-dev/translation-repair-stage';
import { prepareDocumentPair, } from '../dist/final/node/index.mjs';
import { groupWithNothingSealed, } from '@monochromatic-dev/translation-repair-translate-floor/ts/group-aligned.test-fixture.ts';

/**
 Two originals, the second of which no pairing will place.
 */
const SOURCE_TEXT = '猫猫在窗台上睡觉。\n\n猫猫追蝴蝶。\n';

/**
 Two translations, the second of which no original will claim.
 */
const TARGET_TEXT = 'The cat sleeps on the windowsill.\n\nThe cat chases butterflies.\n';

/**
 Pairing naming the first paragraph on each side and nothing else, which
 leaves an original unplaced AND a translation unclaimed. Leaving an original
 unplaced is what switches `declinedTargetIds` off, so the unclaimed
 translation is folded rather than declined.
 */
const FIRST_ONLY = [
  {
    source: 0,
    target: 0,
  },
];

/**
 Budget large enough that nothing splits.
 */
const WIDE_BUDGET = 100_000;

await describe({
  name: 'insertion anchors after merging',
  children: [
    it({
      name: 'ANCHORS outside every span, so the anchor names a boundary between passages rather '
        + 'than an offset inside one, which is the difference between writing a missing rendering '
        + 'and overwriting a rendering that is already there',
      fn: async () => {
        /**
         Translation blocks, for the offsets the runs are checked against.
         */
        const targetNodes = parseDocument({ text: TARGET_TEXT, },).nodes;

        /**
         Runs as they ship.
         */
        const runs = groupWithNothingSealed({
          sourceNodes: parseDocument({ text: SOURCE_TEXT, },).nodes,
          targetNodes,
          sourceBudget: WIDE_BUDGET,
          targetBudget: WIDE_BUDGET,
          steps: blockPairingToSteps({
            pairs: FIRST_ONLY,
            sourceCount: 2,
            targetCount: 2,
          },),
        },);

        for (const run of runs) {
          if (run.kind !== 'insertion')
            continue;
          for (const other of runs) {
            if (other.kind !== 'paired')
              continue;

            /**
             That run's span on the translation side.
             */
            const span = {
              start: other.targetRun[0]
                ?.startOffset,
              end: other.targetRun
                .at(-1,)
                ?.endOffset,
            };
            if ((span.start === undefined) || (span.end === undefined))
              continue;

            // AT A BOUNDARY OR OUTSIDE, never strictly within: an anchor equal
            // to a span's start means before it and one equal to its end means
            // after it, and both are placeable.
            expect((run.targetOffset <= span.start) || (run.targetOffset >= span.end),).toBe(
              true,
            );
          }
        }
      },
    },),

    it({
      name: 'PREPARES the document rather than refusing it, which is the boundary an operator sees: '
        + 'the anchor and the span disagreeing killed the entry outright',
      fn: async () => {
        /**
         Slices this pairing produces.
         */
        const prepared = prepareDocumentPair({
          sourceText: SOURCE_TEXT,
          targetText: TARGET_TEXT,
          blockPairings: new Map([
            [
              0,
              FIRST_ONLY,
            ],
          ],),
        },);
        expect(prepared.slices
          .length,).toBeGreaterThan(0,);
      },
    },),
  ],
},);
