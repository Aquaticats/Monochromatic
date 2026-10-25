/**
 Tests which straggler window each round of the archive block review waits
 under (ledger B713): the review round asks for whole corrected blocks, so its
 replies are writer voices and it waits the writers' window, while the
 selection round after it is a reading of text that exists and keeps the
 readers' window.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import { tagged, } from '@monochromatic-dev/module-logger/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import type { StageSettings, } from '@monochromatic-dev/translation-repair-stage';
import { runArchiveBlockReviewStage, } from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { scriptedClient, } from './scripted-review-client.test-fixture.ts';

/** Four seats, the last of which a case holds back as the straggler. */
const ROSTER = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
] as const;

/** The seat a case holds back in the review round. */
const STRAGGLER = SEAT_SYNTHETIC_TEXT_EVERYWHERE;

/**
 Judges the selection round asks of the four seats: quorum plus one spare, since the ruling that a winner needs two ballots and no
 summed weight (2026-10-25) took away the weight that sent a bench of four to its whole bench.
 */
const SELECTION_WINDOW = 3;

/**
 Windows that differ: the writers wait as long as a hang stop, every other
 round does not wait at all, so a round that reads the wrong one hears the
 straggler or cuts it by that alone.
 */
const WRITERS_WAIT_READERS_DO_NOT: StageSettings = {
  grace: {
    writerMs: HANG_STOP_MS,
    roundMs: 0,
    source: 'built-in',
  },
  contestLedger: { kind: 'unkept', },
};

/** Block under review. */
const BLOCK = 'The cat sleeps by the window and won an award.';

/** Revision every prompt seat writes. */
const QUICK_REVISION = 'The cat sleeps by the window.';

/** Revision only the straggler writes. */
const LATE_REVISION = 'The cat sleeps by the window and purrs.';

/** Logger for the window cases. */
const l = tagged({ tag: 'archive-block-review-windows-test', },);

/**
 Holds back the straggler's reply to one schema.

 @param heldSchema - schema whose reply the straggler holds back

 @returns Predicate telling whether a request is the straggler's reply to that schema

 @example
 ```ts
 const heldFor = stragglerHeldOn({ heldSchema: 'archive_block_review', },);
 ```
 */
function stragglerHeldOn(
  { heldSchema, }: { readonly heldSchema: string; },
): (request: { readonly modelId: string; readonly schema: string; },) => boolean {
  return function isStraggler({ modelId, schema, },): boolean {
    return (modelId === STRAGGLER) && (schema === heldSchema);
  };
}

/**
 Holds back the last judge the selection round asks, whichever seat the bench's rotation puts there. The selection round asks
 three of the four seats (quorum plus one), so the seat that is left out moves with the rotation; the case holds a ballot by its
 place in the order of call, which the rotation does not change, and reads the seats asked from the record.

 @param asked - sink receiving each seat a ballot was asked of, in the order of call

 @returns Predicate telling whether a request is the last ballot of the window

 @example
 ```ts
 const asked: string[] = [];
 const heldFor = lastBallotHeld({ asked, },);
 ```
 */
function lastBallotHeld(
  { asked, }: { readonly asked: string[]; },
): (request: { readonly modelId: string; readonly schema: string; },) => boolean {
  return function isLastBallot({ modelId, schema, },): boolean {
    if (schema !== 'candidate_ballot')
      return false;
    asked.push(modelId,);
    return asked.length === SELECTION_WINDOW;
  };
}

/**
 Runs the stage over the one block with the windows given.

 @param heldFor - predicate telling which requests a seat holds its reply to

 @param prompts - prompt capture sink

 @returns Stage outcome
 */
async function reviewWith(
  {
    heldFor,
    prompts,
  }: {
    readonly heldFor: (request: { readonly modelId: string; readonly schema: string; },) => boolean;
    readonly prompts: string[];
  },
): ReturnType<typeof runArchiveBlockReviewStage> {
  return await runArchiveBlockReviewStage({
    client: scriptedClient({
      prompts,
      heldFor,
      replyFor: ({ schema, modelId, },) => schema === 'archive_block_review'
        ? {
          disposition: 'revise',
          sourceQuote: '',
          replacementText: modelId === STRAGGLER ? LATE_REVISION : QUICK_REVISION,
          assessment: 'Remove the unsupported award claim.',
        }
        : {
          best: 1,
          assessment: 'Only supported details remain.',
        },
    },),
    modelIds: ROSTER,
    sourceText: '猫在窗边睡觉。',
    targetText: BLOCK,
    blockText: BLOCK,
    blockOffset: 0,
    priorFindings: [],
    signal: new AbortController().signal,
    exchangeTimeoutMs: HANG_STOP_MS,
    l,
    fanOut: 'whole-bench',
    stageSettings: WRITERS_WAIT_READERS_DO_NOT,
  },);
}

await describe({
  name: 'archive block review windows',
  children: [
    it({
      name: 'WAITS THE WRITERS\' WINDOW IN THE REVIEW ROUND (ledger B713): a reviewer\'s revision is a whole '
        + 'candidate, so the straggler that answers after quorum is heard, loses no voice and reaches the slate',
      fn: async () => {
        /** Prompts, captured to read the slate the judges were shown. */
        const prompts: string[] = [];
        /** Review over a straggler in the review round. */
        const outcome = await reviewWith({
          heldFor: stragglerHeldOn({ heldSchema: 'archive_block_review', },),
          prompts,
        },);
        expect(outcome.findings.filter(function lostInReview(finding,): boolean {
          return finding.startsWith('stage-voice-lost (archive-block-review',);
        },),).toEqual([],);
        expect(prompts.some(function slateHoldsLateRevision(prompt,): boolean {
          return prompt.includes('CURRENT ARCHIVE BLOCK',) && prompt.includes(LATE_REVISION,);
        },),).toBe(true,);
      },
    },),
    it({
      name: 'KEEPS THE READERS\' WINDOW IN THE SELECTION ROUND (ledger B713): a judge reading text that exists is '
        + 'one ballot in several, so the straggler that answers after quorum is cut and said lost',
      fn: async () => {
        /** Seats a ballot was asked of, in the order of call. */
        const asked: string[] = [];
        /** Review over a straggler in the selection round. */
        const outcome = await reviewWith({
          heldFor: lastBallotHeld({ asked, },),
          prompts: [],
        },);
        // The window asked three seats, so the last of them was the one held.
        expect(asked,).toHaveLength(SELECTION_WINDOW,);
        expect(outcome.findings,).toContain(`stage-voice-lost (select ${asked.at(-1,) ?? ''})`,);
      },
    },),
  ],
},);
