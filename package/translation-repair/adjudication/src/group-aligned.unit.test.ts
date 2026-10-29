/**
 Tests for grouping an aligned block pair into budget-bounded slice runs.

 The grouping had no test. It is read here through `groupNodesSealed` with
 nothing sealed (`group-aligned.test-fixture.ts`, ledger B30), the function
 the slicing calls. Its contract is a coverage claim: the runs
 cover every block on both sides exactly once. That claim is what makes the
 rest of the pipeline safe, because a slice's text is cut from its first to
 its last offset, so a block left out of a run is NOT left out of the text the
 critics read. It is only left out of the record of what the slice was built
 from, which means a claim anchored to it has nowhere to land.

 So the coverage invariant gets asserted on every shape in this file rather than
 once, and the module's own stated exception, an entirely one-sided section,
 is asserted as the exception it is.

 A walk handed over whole is read off `groupNodesSealed` itself: its whole
 result where the walk is well formed, and its refusal where a step names a
 block its side lacks or no step names a block, each of which once grouped a
 section to nothing.

 Fixtures go through `parseDocument`, so the nodes carry the offsets and text
 the aligner really scores on rather than offsets I chose to make a case pass.
 Cat-themed invention throughout.

 @module
 */

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { parseDocument, } from '@monochromatic-dev/translation-repair-shared-contract';
import { blockPairingToSteps, } from '@monochromatic-dev/translation-repair-stage';
import { archiveOriginalReadingOf, } from '@monochromatic-dev/translation-repair-corpus-source';
import type {
  AlignmentStep,
  DocumentNode,
} from '@monochromatic-dev/translation-repair-document-structure';
import {
  type AlignedRun,
  groupNodesSealed,
} from '@monochromatic-dev/translation-repair-translate-floor';
import {
  prepareDocumentPair,
  sealedNodeIds,
} from '../dist/final/node/index.mjs';
import { groupWithNothingSealed, } from '@monochromatic-dev/translation-repair-translate-floor/ts/group-aligned.test-fixture.ts';

/**
 Budget large enough that nothing splits, for shape cases.
 */
const WIDE_BUDGET = 100_000;

/**
 Parses a document and hands back its blocks.

 @param text - markdown source

 @returns Blocks in document order

 @example
 ```ts
 const nodes = blocksOf({ text: 'The cat naps.\n', },);
 ```
 */
function blocksOf({ text, }: { readonly text: string; },): readonly DocumentNode[] {
  return parseDocument({ text, },).nodes;
}

/**
 Asserts the coverage contract: every block appears exactly once, in order.

 This is the invariant worth repeating on every shape. A dropped block still
 reaches the critics through the slice text, so its absence shows up only
 later, as a claim that cannot anchor.

 @param runs - grouped runs under test

 @param sourceNodes - original blocks handed to grouping

 @param targetNodes - translation blocks handed to grouping

 @example
 ```ts
 expectCoversEveryBlockOnce({ runs, sourceNodes, targetNodes, },);
 ```
 */
function expectCoversEveryBlockOnce(
  {
    runs,
    sourceNodes,
    targetNodes,
  }: {
    readonly runs: readonly AlignedRun[];
    readonly sourceNodes: readonly DocumentNode[];
    readonly targetNodes: readonly DocumentNode[];
  },
): void {
  expect(
    runs.flatMap(function toSourceIds(run,) {
      return run.sourceRun
        .map(function toId(node,) {
          return node.id;
        },);
    },),
  ).toStrictEqual(sourceNodes.map(function toId(node,) {
    return node.id;
  },),);

  // AN INSERTION RUN CARRIES NO TRANSLATION BLOCKS, by construction: it names
  // originals nothing rendered and the place their rendering belongs. The
  // coverage claim is unchanged by that, since every translation block still
  // appears exactly once across the runs that hold any.
  expect(
    runs.flatMap(function toTargetIds(run,) {
      return (run.kind === 'insertion')
        ? []
        : run.targetRun
          .map(function toId(node,) {
            return node.id;
          },);
    },),
  ).toStrictEqual(targetNodes.map(function toId(node,) {
    return node.id;
  },),);
}

/**
 Original with four paragraphs.
 */
const SOURCE_TEXT = '猫猫在窗台上睡觉。\n\n太阳移动时她会醒来。\n\n'
  + '她追蝴蝶，很喜欢它们。\n\n晚上她在门口等着。\n';

/**
 Translation with the same four paragraphs.
 */
const TARGET_TEXT = 'The cat sleeps on the windowsill.\n\n'
  + 'She wakes when the sun moves.\n\n'
  + 'She chases butterflies, and she loves them.\n\n'
  + 'In the evening she waits by the door.\n';

/**
 Stands for "no insertion run was produced", which no offset can be.
 */
const NO_RUN = -1;

//region Roster-pairing disposal
// What happens to blocks a supplied pairing left one-sided, which is the path
// the deterministic scorer never produces.

/**
 Six originals, which is the smallest count reaching every disposal site.
 */
const SIX_SOURCE_TEXT = '猫猫一号在窗台上睡觉。\n\n猫猫二号追蝴蝶。\n\n猫猫三号在门口等着。\n\n'
  + '猫猫四号喝牛奶。\n\n猫猫五号爬树。\n\n猫猫六号晒太阳。\n';

/**
 Five translations, so two originals have no counterpart.
 */
const FIVE_TARGET_TEXT = 'Cat one sleeps on the windowsill.\n\n'
  + 'Cat two chases butterflies.\n\n'
  + 'Cat three waits by the door.\n\n'
  + 'Cat four drinks milk.\n\n'
  + 'Cat five climbs the tree.\n';

/**
 Three translations, the shape that leaves an unclaimed one at the very end.
 */
const THREE_TARGET_TEXT = 'Cat one sleeps on the windowsill.\n\n'
  + 'Cat two chases butterflies.\n\n'
  + 'Cat three waits by the door.\n';

/**
 Groups a document pair under a roster pairing, at a budget nothing splits.

 @param sourceText - whole original

 @param targetText - whole translation

 @param pairs - correspondences a roster agreed on

 @returns Runs, beside the blocks they were built from

 @example
 ```ts
 const { runs, } = groupUnderPairing({ sourceText, targetText, pairs, },);
 ```
 */
function groupUnderPairing(
  {
    sourceText,
    targetText,
    pairs,
  }: {
    readonly sourceText: string;
    readonly targetText: string;
    readonly pairs: readonly { readonly source: number; readonly target: number; }[];
  },
) {
  /**
   Original blocks in document order.
   */
  const sourceNodes = blocksOf({ text: sourceText, },);

  /**
   Translation blocks in document order.
   */
  const targetNodes = blocksOf({ text: targetText, },);
  return {
    sourceNodes,
    targetNodes,
    runs: groupWithNothingSealed({
      sourceNodes,
      targetNodes,
      sourceBudget: WIDE_BUDGET,
      targetBudget: WIDE_BUDGET,
      steps: blockPairingToSteps({
        pairs,
        sourceCount: sourceNodes.length,
        targetCount: targetNodes.length,
      },),
    },),
  };
}

//endregion Roster-pairing disposal

//region Supplied walks
// What the grouper makes of a walk handed to it whole, which is how a roster's
// pairing reaches it once `blockPairingToSteps` has converted it.

/**
 Two originals, the smallest pair a walk can both pair and leave unplaced.
 */
const TWO_SOURCE_TEXT = '猫睡了。\n\n猫醒了。\n';

/**
 One translation, fourteen characters long.
 */
const ONE_TARGET_TEXT = 'The cat slept.\n';

/**
 Why the grouper holds a step that names a block its side lacks to be
 unreachable, as its refusal words it after the step, the block and the count.
 */
const WALK_INVARIANT = ', though alignBlocks walks these very blocks and blockPairingToSteps refuses a pairing '
  + 'that names a block its side lacks';

/**
 What the grouper throws for a walk over the two originals and the one
 translation, as text.

 @param steps - walk handed to the grouper

 @returns Refusal's class and whole message

 @example
 ```ts
 const refusal = walkRefusal({ steps: [{ kind: 'paired', sourceIndex: 0, targetIndex: 3, },], },);
 ```
 */
function walkRefusal({ steps, }: { readonly steps: readonly AlignmentStep[]; },): string {
  return String(caught(function groupsTheWalk(): unknown {
    return groupNodesSealed({
      sourceNodes: blocksOf({ text: TWO_SOURCE_TEXT, },),
      targetNodes: blocksOf({ text: ONE_TARGET_TEXT, },),
      sourceBudget: WIDE_BUDGET,
      targetBudget: WIDE_BUDGET,
      steps,
      sealed: new Set<string>(),
    },);
  },),);
}

//endregion Supplied walks

//region Sealed pages
// An archive page whose translators' note says everything below it is the
// English original, read the way the preparation reads it: the note's span
// seals every block after it up to the next heading.

/**
 The translators' note that seals what follows it.
 */
const SEALING_NOTE = '<!-- 这段话以下全部，原文都是英文，中文是反向翻译的 -->';

/**
 Letter the original carries, which the archive's sealed back-translation
 renders.
 */
const LETTER_SOURCE = 'Dear cat, come home.\n';

/**
 Archive with a shop greeting ahead of the sealing note and the letter after it.
 */
const GREETED_ARCHIVE = `A note from the shop.\n\n${SEALING_NOTE}\n\nDear cat, come home.\n`;

/**
 Original whose heading has no translation, followed by four paragraphs.
 */
const HEADED_SOURCE = '## Cats\n\nThe tabby naps.\n\nThe kitten purrs.\n\nThe cat stretches.\n\nThe whiskers twitch.\n';

/**
 Archive with a heading ahead of the sealing note and one sealed paragraph after it.
 */
const HEADED_ARCHIVE = `## Gatos\n\n${SEALING_NOTE}\n\nThe tabby naps in the sun.\n`;

/**
 Archive whose greeting has an original of its own beside it, ahead of the note.
 */
const PAIRED_GREETING_SOURCE = `A word from the shop.\n\n${LETTER_SOURCE}`;

/**
 Ids of the blocks a sealing note covers in an archive.

 @param target - parsed archive

 @returns Ids of its sealed blocks

 @example
 ```ts
 const sealed = sealedIdsOf({ target: parseDocument({ text: GREETED_ARCHIVE, },), },);
 ```
 */
function sealedIdsOf({ target, }: { readonly target: ReturnType<typeof parseDocument>; },): ReadonlySet<string> {
  /**
   What the archive's notes say.
   */
  const reading = archiveOriginalReadingOf({ document: target, },);
  return sealedNodeIds({
    nodes: target.nodes,
    spans: (reading.kind === 'spans') ? reading.spans : [],
  },);
}

/**
 Groups an original against a sealed archive.

 @param sourceText - whole original

 @param targetText - whole archive

 @returns Runs and ids of the originals the seal took with it

 @example
 ```ts
 const { runs, } = groupSealedPage({ sourceText: LETTER_SOURCE, targetText: GREETED_ARCHIVE, },);
 ```
 */
function groupSealedPage(
  {
    sourceText,
    targetText,
  }: {
    readonly sourceText: string;
    readonly targetText: string;
  },
): ReturnType<typeof groupNodesSealed> {
  /**
   Archive, parsed once for its blocks and its seal.
   */
  const target = parseDocument({ text: targetText, },);
  return groupNodesSealed({
    sourceNodes: blocksOf({ text: sourceText, },),
    targetNodes: target.nodes,
    sourceBudget: WIDE_BUDGET,
    targetBudget: WIDE_BUDGET,
    sealed: sealedIdsOf({ target, },),
  },);
}

//endregion Sealed pages

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: groupWithNothingSealed.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'puts everything in one run when both budgets are generous, and '
            + 'covers every block on both sides exactly once',
          fn: async () => {
            /**
             Original blocks.
             */
            const sourceNodes = blocksOf({ text: SOURCE_TEXT, },);

            /**
             Translation blocks.
             */
            const targetNodes = blocksOf({ text: TARGET_TEXT, },);

            /**
             Runs under a budget nothing can exceed.
             */
            const runs = groupWithNothingSealed({
              sourceNodes,
              targetNodes,
              sourceBudget: WIDE_BUDGET,
              targetBudget: WIDE_BUDGET,
            },);

            expect(runs.length,).toBe(1,);
            expectCoversEveryBlockOnce({
              runs,
              sourceNodes,
              targetNodes,
            },);
          },
        },),

        it({
          name: 'SPLITS into several runs under a tight budget while still '
            + 'covering every block exactly once, which is the case where a '
            + 'grouping bug would drop or duplicate a block',
          fn: async () => {
            /**
             Original blocks.
             */
            const sourceNodes = blocksOf({ text: SOURCE_TEXT, },);

            /**
             Translation blocks.
             */
            const targetNodes = blocksOf({ text: TARGET_TEXT, },);

            /**
             Runs under a budget roughly one paragraph wide.
             */
            const runs = groupWithNothingSealed({
              sourceNodes,
              targetNodes,
              sourceBudget: 12,
              targetBudget: 40,
            },);

            expect(runs.length,).toBeGreaterThan(1,);
            expectCoversEveryBlockOnce({
              runs,
              sourceNodes,
              targetNodes,
            },);
          },
        },),

        it({
          name: 'closes a run when EITHER side would exceed its own budget, so a '
            + 'slice stays comparable in size on both sides even though the two '
            + 'languages differ in density: a generous source budget does not let '
            + 'the translation side run away',
          fn: async () => {
            /**
             Original blocks.
             */
            const sourceNodes = blocksOf({ text: SOURCE_TEXT, },);

            /**
             Translation blocks.
             */
            const targetNodes = blocksOf({ text: TARGET_TEXT, },);

            /**
             Runs where only the translation side is constrained.
             */
            const runs = groupWithNothingSealed({
              sourceNodes,
              targetNodes,
              sourceBudget: WIDE_BUDGET,
              targetBudget: 40,
            },);

            expect(runs.length,).toBeGreaterThan(1,);
            expectCoversEveryBlockOnce({
              runs,
              sourceNodes,
              targetNodes,
            },);
          },
        },),

        it({
          name: 'KEEPS an unpartnered block rather than dropping it. The '
            + 'translation here is missing a paragraph, and the source block it '
            + 'lacks must still land in a run: the slice text is cut from first to '
            + 'last offset, so dropping the block would not remove it from what a '
            + 'critic reads, only from the record of what the slice was built from',
          fn: async () => {
            /**
             Original blocks.
             */
            const sourceNodes = blocksOf({ text: SOURCE_TEXT, },);

            /**
             Translation missing the butterflies paragraph entirely.
             */
            const targetNodes = blocksOf({
              text: 'The cat sleeps on the windowsill.\n\n'
                + 'She wakes when the sun moves.\n\n'
                + 'In the evening she waits by the door.\n',
            },);

            /**
             Runs over the mismatched pair.
             */
            const runs = groupWithNothingSealed({
              sourceNodes,
              targetNodes,
              sourceBudget: WIDE_BUDGET,
              targetBudget: WIDE_BUDGET,
            },);

            expectCoversEveryBlockOnce({
              runs,
              sourceNodes,
              targetNodes,
            },);
          },
        },),

        it({
          name: 'covers every block even when the sides drop DIFFERENT paragraphs, '
            + 'so unpartnered blocks on both sides at once still each land '
            + 'somewhere',
          fn: async () => {
            /**
             Original missing its second paragraph.
             */
            const sourceNodes = blocksOf({
              text: '猫猫在窗台上睡觉。\n\n她追蝴蝶，很喜欢它们。\n\n晚上她在门口等着。\n',
            },);

            /**
             Translation missing its third paragraph instead.
             */
            const targetNodes = blocksOf({
              text: 'The cat sleeps on the windowsill.\n\n'
                + 'She wakes when the sun moves.\n\n'
                + 'In the evening she waits by the door.\n',
            },);

            /**
             Runs over the doubly-mismatched pair.
             */
            const runs = groupWithNothingSealed({
              sourceNodes,
              targetNodes,
              sourceBudget: WIDE_BUDGET,
              targetBudget: WIDE_BUDGET,
            },);

            expectCoversEveryBlockOnce({
              runs,
              sourceNodes,
              targetNodes,
            },);
            for (const run of runs) {
              expect(run.sourceRun.length,).toBeGreaterThan(0,);
              // No pairing was supplied here, so the scorer produced the walk and
              // no insertion may be proposed off it: the scorer cannot tell an
              // original that was merged from one that was dropped.
              expect(run.kind,).toBe('paired',);
              if (run.kind === 'paired')
                expect(run.targetRun.length,).toBeGreaterThan(0,);
            }
          },
        },),

        it({
          name: 'never emits a run empty on one side, because every later stage '
            + 'needs both sides to compare and a one-sided run is a slice nobody '
            + 'can review',
          fn: async () => {
            /**
             Original blocks.
             */
            const sourceNodes = blocksOf({ text: SOURCE_TEXT, },);

            /**
             Translation that folded four paragraphs into one.
             */
            const targetNodes = blocksOf({
              text: 'The cat sleeps on the windowsill, wakes when the sun moves, '
                + 'chases butterflies she loves, and waits by the door at evening.\n',
            },);

            for (const budget of [
              WIDE_BUDGET,
              20,
            ]) {
              /**
               Runs at this budget.
               */
              const runs = groupWithNothingSealed({
                sourceNodes,
                targetNodes,
                sourceBudget: budget,
                targetBudget: budget,
              },);

              for (const run of runs) {
                expect(run.sourceRun.length,).toBeGreaterThan(0,);
                // A MERGE IS NOT AN OMISSION. The four originals here are rendered
                // as one translation block, which the walk reports as a pairing
                // followed by continuations, so none of them is unplaced and no
                // insertion run may appear. If one did, the lane would write a
                // second rendering of a passage the page already carries.
                expect(run.kind,).toBe('paired',);
                if (run.kind === 'paired')
                  expect(run.targetRun.length,).toBeGreaterThan(0,);
              }
              expectCoversEveryBlockOnce({
                runs,
                sourceNodes,
                targetNodes,
              },);
            }
          },
        },),

        it({
          name: 'returns NO RUNS when one side has no blocks at all, the module\'s '
            + 'stated exception to coverage: nothing two-sided exists to hold the '
            + 'other side\'s blocks, and the slicing never groups such a section',
          fn: async () => {
            expect(
              groupWithNothingSealed({
                sourceNodes: blocksOf({ text: SOURCE_TEXT, },),
                targetNodes: [],
                sourceBudget: WIDE_BUDGET,
                targetBudget: WIDE_BUDGET,
              },),
            ).toStrictEqual([],);

            expect(
              groupWithNothingSealed({
                sourceNodes: [],
                targetNodes: blocksOf({ text: TARGET_TEXT, },),
                sourceBudget: WIDE_BUDGET,
                targetBudget: WIDE_BUDGET,
              },),
            ).toStrictEqual([],);
          },
        },),

        it({
          name: 'returns no runs for two empty sides rather than throwing',
          fn: async () => {
            expect(
              groupWithNothingSealed({
                sourceNodes: [],
                targetNodes: [],
                sourceBudget: WIDE_BUDGET,
                targetBudget: WIDE_BUDGET,
              },),
            ).toStrictEqual([],);
          },
        },),

        it({
          name: 'gives a block larger than the whole budget its own run rather '
            + 'than looping or dropping it, since a single unsplittable block '
            + 'cannot be made to fit',
          fn: async () => {
            /**
             Original blocks.
             */
            const sourceNodes = blocksOf({ text: SOURCE_TEXT, },);

            /**
             Translation blocks.
             */
            const targetNodes = blocksOf({ text: TARGET_TEXT, },);

            /**
             Runs under a budget no single paragraph fits inside.
             */
            const runs = groupWithNothingSealed({
              sourceNodes,
              targetNodes,
              sourceBudget: 1,
              targetBudget: 1,
            },);

            expectCoversEveryBlockOnce({
              runs,
              sourceNodes,
              targetNodes,
            },);
          },
        },),
        it({
          name: 'KEEPS a section whose every run came out one-sided, which a supplied '
            + 'pairing that pairs nothing produces once the budget splits the unpaired blocks',
          fn: async () => {
            /**
             Original blocks.
             */
            const sourceNodes = blocksOf({ text: SOURCE_TEXT, },);

            /**
             Translation blocks.
             */
            const targetNodes = blocksOf({ text: TARGET_TEXT, },);

            // NOTHING PAIRED, and a budget no block fits inside, so every run holds
            // one block and none holds both sides. The merger held those blocks
            // waiting for a two-sided run to fold them into, and none ever came, so
            // it returned nothing and the section left the document. Both sides
            // carry blocks here, so this is not the empty-side exception.
            const runs = groupWithNothingSealed({
              sourceNodes,
              targetNodes,
              sourceBudget: 1,
              targetBudget: 1,
              steps: [
                ...[ ...sourceNodes.keys(), ].map(function toSourceOnly(
                  sourceIndex,
                ): { readonly kind: 'source-only'; readonly sourceIndex: number; } {
                  return {
                    kind: 'source-only',
                    sourceIndex,
                  };
                },),
                ...[ ...targetNodes.keys(), ].map(function toTargetOnly(
                  targetIndex,
                ): { readonly kind: 'target-only'; readonly targetIndex: number; } {
                  return {
                    kind: 'target-only',
                    targetIndex,
                  };
                },),
              ],
            },);

            expectCoversEveryBlockOnce({
              runs,
              sourceNodes,
              targetNodes,
            },);
          },
        },),
        it({
          name: 'GIVES AN ORIGINAL NOTHING RENDERED ITS OWN RUN, anchored where the next rendered '
            + 'block begins. Folding it into a neighbour put its bytes inside that slice span, where '
            + 'no later stage could tell a missing passage from part of the one beside it',
          fn: async () => {
            /**
             Original blocks.
             */
            const sourceNodes = blocksOf({ text: SOURCE_TEXT, },);

            /**
             Translation missing the third paragraph entirely.
             */
            const targetNodes = blocksOf({
              text: 'The cat sleeps on the windowsill.\n\n'
                + 'She wakes when the sun moves.\n\n'
                + 'In the evening she waits by the door.\n',
            },);

            /**
             A roster pairing leaving the third original unplaced.
             */
            const runs = groupWithNothingSealed({
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
                  kind: 'paired',
                  sourceIndex: 1,
                  targetIndex: 1,
                },
                {
                  kind: 'source-only',
                  sourceIndex: 2,
                },
                {
                  kind: 'paired',
                  sourceIndex: 3,
                  targetIndex: 2,
                },
              ],
            },);

            expect(runs.map(function toKind(run,) {
              return run.kind;
            },),)
              .toStrictEqual([
                'paired',
                'insertion',
                'paired',
              ],);

            /**
             The insertion run.
             */
            const anchored = runs.find(function isInsertion(run,) {
              return run.kind === 'insertion';
            },);

            expect((anchored?.kind === 'insertion') ? anchored.targetOffset : NO_RUN,)
              .toBe(nonNullishOrThrow(targetNodes.at(2,),).startOffset,);

            expect((anchored?.sourceRun ?? []).map(function toId(node,) {
              return node.id;
            },),)
              .toStrictEqual([ nonNullishOrThrow(sourceNodes.at(2,),).id, ],);
          },
        },),

        it({
          name: 'NEVER PROPOSES AN INSERTION FOR A MERGE, since two originals rendered as one block '
            + 'arrive as a pairing plus a continuation and the second IS on the page. Writing it in '
            + 'again would put a second rendering of that passage into a memorial document',
          fn: async () => {
            /**
             Original blocks.
             */
            const sourceNodes = blocksOf({ text: SOURCE_TEXT, },);

            /**
             Translation folding the first two originals into one block.
             */
            const targetNodes = blocksOf({
              text: 'The cat sleeps on the windowsill, and wakes when the sun moves.\n\n'
                + 'She chases butterflies, and she loves them.\n\n'
                + 'In the evening she waits by the door.\n',
            },);

            /**
             A pairing whose second original continues the first one's block.
             */
            const runs = groupWithNothingSealed({
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
                {
                  kind: 'paired',
                  sourceIndex: 2,
                  targetIndex: 1,
                },
                {
                  kind: 'paired',
                  sourceIndex: 3,
                  targetIndex: 2,
                },
              ],
            },);

            expect(runs.every(function isPaired(run,) {
              return run.kind === 'paired';
            },),)
              .toBe(true,);
          },
        },),

        it({
          name: 'BEGINS A FRESH RUN AT THE STEP AFTER A DECLINE, so the declined block\'s bytes sit '
            + 'outside both spans instead of between two blocks of one run',
          fn: async () => {
            /**
             Original blocks, both placed by the pairing.
             */
            const sourceNodes = blocksOf({ text: '猫猫在窗台上睡觉。\n\n猫猫追蝴蝶。\n', },);

            /**
             Translation blocks, the middle one no original claims while every original is placed.
             */
            const targetNodes = blocksOf({
              text: 'The cat sleeps on the windowsill.\n\nA caption no original claims.\n\nThe cat chases butterflies.\n',
            },);

            /**
             Runs as grouped over a pairing that leaves the middle translation declined.
             */
            const runs = groupWithNothingSealed({
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
                  kind: 'target-only',
                  targetIndex: 1,
                },
                {
                  kind: 'paired',
                  sourceIndex: 1,
                  targetIndex: 2,
                },
              ],
            },);
            expect(runs.map(function sourceIds(run,) {
              return run.sourceRun.map(function id(node,) {
                return node.id;
              },);
            },),).toStrictEqual([
              [ nonNullishOrThrow(sourceNodes.at(0,),).id, ],
              [ nonNullishOrThrow(sourceNodes.at(1,),).id, ],
            ],);
            expect(runs.map(function targetIds(run,) {
              if (run.kind !== 'paired')
                throw new Error(`expected only paired runs after the decline, got ${run.kind}`,);
              return run.targetRun.map(function id(node,) {
                return node.id;
              },);
            },),).toStrictEqual([
              [ nonNullishOrThrow(targetNodes.at(0,),).id, ],
              [ nonNullishOrThrow(targetNodes.at(2,),).id, ],
            ],);
          },
        },),

        it({
          name: 'ANCHORS A TRAILING ORIGINAL AFTER THE LAST RENDERED BLOCK, since nothing follows it '
            + 'to sit before, and anchoring at the start of the last block instead would write the '
            + 'passage above the paragraph it comes after',
          fn: async () => {
            /**
             Original blocks.
             */
            const sourceNodes = blocksOf({ text: SOURCE_TEXT, },);

            /**
             Translation missing the last paragraph.
             */
            const targetNodes = blocksOf({
              text: 'The cat sleeps on the windowsill.\n\n'
                + 'She wakes when the sun moves.\n\n'
                + 'She chases butterflies, and she loves them.\n',
            },);

            /**
             A pairing leaving the final original unplaced.
             */
            const runs = groupWithNothingSealed({
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
                  kind: 'paired',
                  sourceIndex: 1,
                  targetIndex: 1,
                },
                {
                  kind: 'paired',
                  sourceIndex: 2,
                  targetIndex: 2,
                },
                {
                  kind: 'source-only',
                  sourceIndex: 3,
                },
              ],
            },);

            /**
             The insertion run.
             */
            const anchored = runs.find(function isInsertion(run,) {
              return run.kind === 'insertion';
            },);

            expect((anchored?.kind === 'insertion') ? anchored.targetOffset : NO_RUN,)
              .toBe(nonNullishOrThrow(targetNodes.at(-1,),).endOffset,);
          },
        },),

      ],
    },),

    describe({
      name: `${groupWithNothingSealed.name} disposing of one-sided runs`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'LEAVES no run empty on a side when held originals settle ahead of an insertion, since '
            + 'a run’s span is cut from its first node to its last and a run with no node on a side '
            + 'has no span to cut',
          fn: async () => {
            const {
              runs,
              sourceNodes,
              targetNodes,
            } = groupUnderPairing({
              sourceText: SIX_SOURCE_TEXT,
              targetText: FIVE_TARGET_TEXT,
              pairs: [
                {
                  source: 0,
                  target: 0,
                },
                {
                  source: 1,
                  target: 2,
                },
                {
                  source: 3,
                  target: 2,
                },
              ],
            },);
            expect(runs.some(function isEmptySided(run,): boolean {
              return (run.kind === 'paired')
                && ((run.sourceRun
                  .length
                  === 0)
                  || (run.targetRun
                    .length
                    === 0));
            },),).toBe(false,);
            expectCoversEveryBlockOnce({
              runs,
              sourceNodes,
              targetNodes,
            },);
          },
        },),

        it({
          name: 'KEEPS an unclaimed translation that arrives AFTER the last insertion, which has no '
            + 'two-sided run left to fold into. `declinedTargetIds` declines nothing here, because the '
            + 'pairing left originals unplaced, so the block is one review still owes a reader',
          fn: async () => {
            const {
              runs,
              sourceNodes,
              targetNodes,
            } = groupUnderPairing({
              sourceText: SIX_SOURCE_TEXT,
              targetText: THREE_TARGET_TEXT,
              pairs: [
                {
                  source: 0,
                  target: 0,
                },
                {
                  source: 1,
                  target: 0,
                },
                {
                  source: 2,
                  target: 1,
                },
                {
                  source: 3,
                  target: 1,
                },
              ],
            },);
            expectCoversEveryBlockOnce({
              runs,
              sourceNodes,
              targetNodes,
            },);
          },
        },),
      ],
    },),

    describe({
      name: `${groupNodesSealed.name} reading a supplied walk`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'GROUPS a well-formed supplied walk into a paired run and an insertion written after the '
            + 'last rendered block, with nothing sealed',
          fn: async () => {
            /**
             Original blocks.
             */
            const sourceNodes = blocksOf({ text: TWO_SOURCE_TEXT, },);

            /**
             Translation blocks.
             */
            const targetNodes = blocksOf({ text: ONE_TARGET_TEXT, },);

            expect(groupNodesSealed({
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
                },
              ],
              sealed: new Set<string>(),
            },),).toEqual({
              runs: [
                {
                  kind: 'paired',
                  sourceRun: [ nonNullishOrThrow(sourceNodes[0],), ],
                  targetRun: [ nonNullishOrThrow(targetNodes[0],), ],
                },
                {
                  kind: 'insertion',
                  sourceRun: [ nonNullishOrThrow(sourceNodes[1],), ],
                  // Where the one translation block ends, fourteen characters in.
                  targetOffset: 14,
                },
              ],
              sealedSourceIds: new Set<string>(),
              unplacedTargetIds: new Set<string>(),
            },);
          },
        },),

        it({
          name: 'REFUSES a supplied walk whose step names a translation block the section lacks, in the same '
            + 'words for a paired step alone, a paired step before an unplaced original and a '
            + 'translation-only step',
          fn: async () => {
            // THE FIRST OF THESE GROUPED TO NOTHING, SILENTLY: the missing block
            // was filtered out of its step, the run it left held an original and
            // no translation, and the merger found no settled run to fold it into.
            expect(walkRefusal({
              steps: [
                {
                  kind: 'paired',
                  sourceIndex: 0,
                  targetIndex: 3,
                },
              ],
            },),).toBe(`Error: unreachable: walk step 0 names translation block 3, and there are 1${WALK_INVARIANT}`,);
            expect(walkRefusal({
              steps: [
                {
                  kind: 'source-only',
                  sourceIndex: 0,
                },
                {
                  kind: 'paired',
                  sourceIndex: 1,
                  targetIndex: 3,
                },
              ],
            },),).toBe(`Error: unreachable: walk step 1 names translation block 3, and there are 1${WALK_INVARIANT}`,);
            expect(walkRefusal({
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
                {
                  kind: 'target-only',
                  targetIndex: 4,
                },
              ],
            },),).toBe(`Error: unreachable: walk step 2 names translation block 4, and there are 1${WALK_INVARIANT}`,);
          },
        },),

        it({
          name: 'REFUSES a supplied walk whose step names an original block the section lacks, in the same '
            + 'words for a paired step and an original-only step',
          fn: async () => {
            expect(walkRefusal({
              steps: [
                {
                  kind: 'paired',
                  sourceIndex: 5,
                  targetIndex: 0,
                },
              ],
            },),).toBe(`Error: unreachable: walk step 0 names original block 5, and there are 2${WALK_INVARIANT}`,);
            // THIS ONE GROUPED TO AN INSERTION OF NO BLOCKS, a run the slicing
            // cannot cut a span from.
            expect(walkRefusal({
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
                {
                  kind: 'source-only',
                  sourceIndex: 9,
                },
              ],
            },),).toBe(`Error: unreachable: walk step 2 names original block 9, and there are 2${WALK_INVARIANT}`,);
          },
        },),

        it({
          name: 'REFUSES a supplied walk that names no step for a block of either side, in the words of the '
            + 'refusal for a block the section lacks, an empty walk among them',
          fn: async () => {
            /**
             Why the grouper holds a walk that leaves a block out to be
             unreachable, as its refusal words it after the block and the count.
             */
            const coverageInvariant = ', though alignBlocks walks every one of these blocks and '
              + 'blockPairingToSteps gives every block of both sides a step';
            // THE FIRST OF THESE GROUPED TO NOTHING, SILENTLY: the second
            // original and the translation reached no run, and nothing said so.
            expect({
              secondOriginalUnnamed: walkRefusal({
                steps: [
                  {
                    kind: 'source-only',
                    sourceIndex: 0,
                  },
                ],
              },),
              translationUnnamed: walkRefusal({
                steps: [
                  {
                    kind: 'source-only',
                    sourceIndex: 0,
                  },
                  {
                    kind: 'source-only',
                    sourceIndex: 1,
                  },
                ],
              },),
              empty: walkRefusal({ steps: [], },),
            },).toStrictEqual({
              secondOriginalUnnamed: 'Error: unreachable: no walk step names original block 1, and there are '
                + `2${coverageInvariant}`,
              translationUnnamed: 'Error: unreachable: no walk step names translation block 0, and there are '
                + `1${coverageInvariant}`,
              empty: `Error: unreachable: no walk step names original block 0, and there are 2${coverageInvariant}`,
            },);
          },
        },),
      ],
    },),

    describe({
      name: 'groupNodesSealed on an archive page whose note seals a span',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'CARRIES a translation block ahead of the sealing note that no paired run can take as unplaced, '
            + 'with no run for it and the letter\'s original sealed',
          fn: async () => {
            /**
             What grouping the page answers.
             */
            const {
              runs,
              sealedSourceIds,
              unplacedTargetIds,
            } = groupSealedPage({
              sourceText: LETTER_SOURCE,
              targetText: GREETED_ARCHIVE,
            },);
            expect({
              runs,
              sealedSourceIds: [...sealedSourceIds,],
              unplacedTargetIds: [...unplacedTargetIds,],
            },).toStrictEqual({
              runs: [],
              sealedSourceIds: ['block/0',],
              unplacedTargetIds: ['block/0',],
            },);
          },
        },),

        it({
          name: 'PREPARES the same page through prepareDocumentPair with the greeting kept as the archive has it, '
            + 'handed to the block correction round and named in a finding by its id',
          fn: async () => {
            /**
             What preparing the page answers.
             */
            const prepared = prepareDocumentPair({
              sourceText: LETTER_SOURCE,
              targetText: GREETED_ARCHIVE,
              sealArchiveOriginal: true,
            },);
            expect({
              slices: prepared.slices,
              unclaimedTargetBlocks: prepared.unclaimedTargetBlocks,
              alignmentFindings: prepared.alignmentFindings,
            },).toStrictEqual({
              slices: [],
              unclaimedTargetBlocks: [
                {
                  location: { kind: 'aligned-pair', pairIndex: 0, },
                  blockId: 'block/0',
                  startOffset: 0,
                  endOffset: 21,
                },
              ],
              alignmentFindings: [
                'alignment archive-original (pair 0: 1 translation block and 1 original block sealed by the archive\'s '
                  + 'note, shipped as the archive has them: block/1)',
                'alignment target-unplaced (pair 0: 1 translation block no run could take, 21 characters: block/0)',
              ],
            },);
          },
        },),

        it({
          name: 'PREPARES a page whose heading ahead of the sealing note is unpaired and whose later originals follow '
            + 'the seal, keeping the heading out of every run that stands after the seal',
          fn: async () => {
            /**
             What preparing the page answers.
             */
            const prepared = prepareDocumentPair({
              sourceText: HEADED_SOURCE,
              targetText: HEADED_ARCHIVE,
              sealArchiveOriginal: true,
              blockPairings: new Map([
                [
                  0,
                  [
                    { source: 1, target: 1, },
                    { source: 3, target: 1, },
                  ],
                ],
              ],),
            },);
            expect({
              slices: prepared.slices.map(function toPlacement(slice,): unknown {
                return {
                  source: slice.source.nodes.map(function toId(node,): string {
                    return node.id;
                  },),
                  target: slice.target.nodes.map(function toId(node,): string {
                    return node.id;
                  },),
                  start: slice.target.startOffset,
                  end: slice.target.endOffset,
                };
              },),
              unclaimedTargetBlocks: prepared.unclaimedTargetBlocks,
              alignmentFindings: prepared.alignmentFindings,
            },).toStrictEqual({
              slices: [
                {
                  source: ['block/0',],
                  target: [],
                  start: 44,
                  end: 44,
                },
                {
                  source: ['block/2', 'block/3',],
                  target: [],
                  start: 70,
                  end: 70,
                },
                {
                  source: ['block/4',],
                  target: [],
                  start: 70,
                  end: 70,
                },
              ],
              unclaimedTargetBlocks: [
                {
                  location: { kind: 'aligned-pair', pairIndex: 0, },
                  blockId: 'block/0',
                  startOffset: 0,
                  endOffset: 8,
                },
              ],
              alignmentFindings: [
                'alignment archive-original (pair 0: 1 translation block and 1 original block sealed by the archive\'s '
                  + 'note, shipped as the archive has them: block/1)',
                'alignment target-unplaced (pair 0: 1 translation block no run could take, 8 characters: block/0)',
              ],
            },);
          },
        },),

        it({
          name: 'GROUPS a page whose greeting has an original of its own ahead of the sealing note into one paired '
            + 'run and takes the letter\'s original away with the seal',
          fn: async () => {
            /**
             What grouping the page answers.
             */
            const {
              runs,
              sealedSourceIds,
            } = groupSealedPage({
              sourceText: PAIRED_GREETING_SOURCE,
              targetText: GREETED_ARCHIVE,
            },);
            expect({
              runs: runs.map(function idsOfRun(run,): unknown {
                return {
                  kind: run.kind,
                  source: run.sourceRun.map(function toId(node,): string {
                    return node.id;
                  },),
                  target: (run.kind === 'insertion')
                    ? []
                    : run.targetRun.map(function toId(node,): string {
                      return node.id;
                    },),
                };
              },),
              sealedSourceIds: [...sealedSourceIds,],
            },).toStrictEqual({
              runs: [
                {
                  kind: 'paired',
                  source: ['block/0',],
                  target: ['block/0',],
                },
              ],
              sealedSourceIds: ['block/1',],
            },);
          },
        },),
      ],
    },),
  ],
},);
