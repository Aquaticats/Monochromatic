/**
 Tests that assembly's document-scale damage checks say in the log where the
 damage is (ledger L12).

 WHY. `introduced-repetition` names neither the slices nor the words, and
 `content-survival` names no slice. Both are deliberate in the findings, which
 are counted across runs and stay free of corpus wording, but nothing else
 located either, so a reader of a run had to rerun the check by hand to find
 which slices repeated a passage or lost the archive's specifics. The log now
 names the slices, with the phrase and the lost words.

 Fixtures are cat-themed invention.

 @module
 */

import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { UNATTRIBUTED_TEXT, } from '@monochromatic-dev/translation-repair-house-rule';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading';
import {
  assembleRepair,
  type ChunkRepairOutcome,
} from '../dist/final/node/index.mjs';
import { SEAT_HYPER_OPENROUTER_VISION_EDITOR, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

/**
 Archive paragraph of slice 0, carrying specific words.
 */
const FIRST = 'Mittens counted the sparrows along the garden fence every morning.';

/**
 Archive paragraph of slice 1.
 */
const SECOND = 'Later she slept beside the kitchen stove until dinner.';

/**
 Whole archive.
 */
const TARGET_TEXT = `${FIRST}\n\n${SECOND}\n`;

/**
 The two slices, as preparation stamps them.
 */
const SLICES: readonly ChunkPair[] = [FIRST, SECOND,].map(function toPair(text, sliceIndex,): ChunkPair {
  /**
   Where the paragraph starts in the archive.
   */
  const startOffset = TARGET_TEXT.indexOf(text,);
  return {
    source: {
      sliceIndex,
      text: `猫段${String(sliceIndex,)}。`,
      startOffset: sliceIndex * 4,
      endOffset: (sliceIndex * 4) + 3,
      nodes: [],
    },
    target: {
      sliceIndex,
      text,
      startOffset,
      endOffset: startOffset + text.length,
      nodes: [],
    },
  };
},);

/**
 A settled outcome for one slice.

 @param sliceIndex - slice settled

 @param repairedText - wording the lane settled on

 @returns Outcome as the lane records it

 @example
 ```ts
 const outcome = outcomeFor({ sliceIndex: 0, repairedText: FIRST, },);
 ```
 */
function outcomeFor({ sliceIndex, repairedText, }: { readonly sliceIndex: number; readonly repairedText: string; },): ChunkRepairOutcome {
  /**
   Whether the lane changed this slice, which provenance requires a heard
   critic and a selected patch for.
   */
  const changed = repairedText !== [FIRST, SECOND,][sliceIndex];
  return {
    sliceIndex,
    repairedText,
    changed,
    issues: [],
    resolvedIssueIds: [],
    candidateResolvedIssueIds: [],
    checkerReadings: {},
    recheckReadings: {},
    repairRegions: [],
    authorship: UNATTRIBUTED_TEXT,
    accuracyPatchSelected: changed,
    refined: false,
    rounds: [],
    droppedDeclaredNames: [],
    nonTranslationVotes: 0,
    nonTranslationContradicted: false,
    nonTranslationStanding: false,
    heardCritics: 1,
    heardCriticIds: changed ? [SEAT_HYPER_OPENROUTER_VISION_EDITOR,] : [],
    claimAttributions: [],
    findings: [],
    silentStages: [],
  };
}

/**
 Lines the repair assembler logs over two settled slices.

 @param outcomes - settled outcomes, in slice order

 @returns Every line logged

 @example
 ```ts
 const lines = assembledLines({ outcomes, },);
 ```
 */
function assembledLines({ outcomes, }: { readonly outcomes: readonly ChunkRepairOutcome[]; },): readonly string[] {
  /**
   Lines written, in order.
   */
  const lines: string[] = [];

  /**
   Keeps one line.

   @param message - line written
   */
  function keep(message: string,): void {
    lines.push(message,);
  }

  /**
   Logger keeping every line.
   */
  const logger: Logger = {
    debug: keep,
    error: keep,
    fatal: keep,
    flush: async () => {},
    info: keep,
    trace: keep,
    warn: keep,
  };
  assembleRepair({
    targetText: TARGET_TEXT,
    slices: SLICES,
    outcomes,
    lineStructuredSlices: new Set(),
    findings: [],
    l: logger,
  },);
  return lines;
}

await describe({
  name: 'assembly damage is located in the log (ledger L12)',
  children: [
    it({
      name: 'a repetition line NAMES the slices that carry the repeated phrase, and the phrase',
      fn: async () => {
        /**
         Slice 1 repeating slice 0's specifics.
         */
        const lines = assembledLines({
          outcomes: [
            outcomeFor({ sliceIndex: 0, repairedText: FIRST, },),
            outcomeFor({ sliceIndex: 1, repairedText: `${SECOND} Mittens counted the sparrows along the garden fence.`, },),
          ],
        },);
        expect(lines.some(function locates(line,) {
          return line.includes('introduced repetition in slices 0, 1',) && line.includes('sparrows along the garden',);
        },),).toBe(true,);
      },
    },),
    it({
      name: 'a content-loss line NAMES the slice whose archive held the lost words, and the words',
      fn: async () => {
        /**
         Slice 0 rewritten into generic prose.
         */
        const lines = assembledLines({
          outcomes: [
            outcomeFor({ sliceIndex: 0, repairedText: 'Mittens watched some birds outside each day.', },),
            outcomeFor({ sliceIndex: 1, repairedText: SECOND, },),
          ],
        },);
        expect(lines.some(function locates(line,) {
          return line.includes('content lost at slice 0',) && line.includes('sparrows',);
        },),).toBe(true,);
      },
    },),
  ],
},);
