/**
 Tests that the chunk selection tells its judges what a decline does
 (ledger L14(c)).

 WHY. The shared selection sheet says "the caller keeps text it already
 trusts when you decline", and for the chunk selection that holds only when
 EVERY judge declines. A decline names no candidate and counts as no vote, so
 when the judges who named one reach no decision the round is indecision, and
 the repair that landed the most operations goes on to the checkers
 (`selectChunkPatch`, `pickFallbackCandidate`). Over every artifact, 14 of
 2,476 chunk rounds had declines outnumbering the ballots that named a
 candidate, 10 of them read as indecision, and TianqiChen66621 chunk 17 was
 one (4 declined, 1 named); no round had every judge decline. The judges were
 told a cost of declining that the round did not pay.

 THE RULE ITSELF STAYS: a decline as an abstention that does not count is the
 measured policy for the consolidation gate's `neither`
 (`doc/planning/the-third-rendering.md`), and the fallback repairs because
 the panel ruled its issues real (`editor-candidates.ts`). The sheet now says
 what the round does, and both behaviours it states are pinned here.

 Fixtures are cat-themed invention.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import type { PatchOutcome, } from '@monochromatic-dev/translation-repair-page-pass';
import {
  type ChatJsonOutcome,
  type ChatJsonRequest,
  messageText,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import type { Candidate, } from '@monochromatic-dev/translation-repair-house-rule';
import { selectChunkPatch, } from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { chunkCandidate, } from '@monochromatic-dev/translation-repair-page-pass/ts/whole-chunk-candidate.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Logger for the selection under test.
 */
const l = tagged({ tag: 'chunk-decline-consequence-test', },);

/**
 Invented archive English before repair.
 */
const EXISTING_TEXT = 'The cat hates the sun.';

/**
 Judges, none of whom wrote a candidate.
 */
const JUDGES: readonly RosterModelId[] = [
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_HYPER_OPENROUTER_UNMEASURED,
];

/**
 Candidates the judges rule on.
 */
const CANDIDATES: readonly Candidate<PatchOutcome>[] = [
  chunkCandidate({ patchedText: 'The cat loves the sun.', },),
  chunkCandidate({ patchedText: 'The cat adores the sun.', },),
];

/**
 Judges' ballots and the sheet they were shown.

 @param namingCalls - places in the order of call, from zero, whose judge names candidate 1; every other judge declines. A
 judge is picked by when it was asked, not by seat, because the round asks three of the four seats (quorum plus one) and the
 seat that is left out moves with the rotation of the bench, which the sheet's text and the sweep's salt both change

 @param sheets - collector the judges' system sheets land in

 @returns Scripted client

 @example
 ```ts
 const client = judgesClient({ namingCalls: [], sheets: [], },);
 ```
 */
function judgesClient(
  {
    namingCalls,
    sheets,
  }: {
    readonly namingCalls: readonly number[];
    readonly sheets: string[];
  },
): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText unused by the selection',);
    },
    chatJson: async <ValueT,>(request: ChatJsonRequest<ValueT>,): Promise<ChatJsonOutcome<ValueT>> => {
      /**
       Place of this judge in the order of call, read before its sheet is
       recorded.
       */
      const callIndex = sheets.length;
      /**
       System sheet this judge was shown.
       */
      const system = request.messages.at(0,);
      sheets.push((system === undefined) ? '' : messageText({ message: system, },),);
      /**
       This judge's ballot.
       */
      const scripted: unknown = {
        best: namingCalls.includes(callIndex,) ? 1 : 0,
        assessment: 'scripted',
      };
      if (!request.validate(scripted,))
        throw new Error('stub ballot failed the selection guard',);
      return {
        kind: 'ok',
        value: scripted,
        rawText: JSON.stringify(scripted,),
      };
    },
    quotas: async () => {
      throw new Error('quotas unused by the selection',);
    },
  };
}

/**
 Runs one chunk selection.

 @param namingCalls - places in the order of call whose judge names candidate 1

 @returns Text that ships, and the judges' sheets

 @example
 ```ts
 const { shipped, sheets, } = await select({ namingCalls: [], },);
 ```
 */
async function select({ namingCalls, }: { readonly namingCalls: readonly number[]; },): Promise<{
  readonly shipped: string;
  readonly sheets: readonly string[];
}> {
  /**
   Judges' sheets, in order.
   */
  const sheets: string[] = [];
  const { patch, } = await selectChunkPatch({
    client: judgesClient({ namingCalls, sheets, },),
    candidates: CANDIDATES,
    judgeModelIds: JUDGES,
    sourceText: '猫猫喜欢太阳。',
    indecisionFallback: chunkCandidate({ patchedText: 'The cat likes the sun.', },),
    rejectionFallback: {
      patchedText: EXISTING_TEXT,
      applied: [],
      rejected: [],
    },
    signal: new AbortController().signal,
    perCallTimeoutMs: HANG_STOP_MS,
    l,
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
  return {
    shipped: patch.patchedText,
    sheets,
  };
}

await describe({
  name: 'the chunk selection says what a decline does (ledger L14(c))',
  children: [
    it({
      name: 'TELLS the judges the existing English is kept only when every judge declines, and what goes on '
        + 'otherwise',
      fn: async () => {
        const { sheets, } = await select({ namingCalls: [], },);
        expect({
          asked: sheets.length > 0,
          everyJudge: sheets.every(function states(sheet,) {
            return sheet.includes('only when every judge declines',);
          },),
          fallback: sheets.every(function states(sheet,) {
            return sheet.includes('the repair that landed the most edits goes on',);
          },),
          oldPromise: sheets.some(function promises(sheet,) {
            return sheet.includes('the caller keeps text it already trusts when you decline',);
          },),
        },).toEqual({
          asked: true,
          everyJudge: true,
          fallback: true,
          oldPromise: false,
        },);
      },
    },),
    it({
      name: 'KEEPS the existing English when every judge declines',
      fn: async () => {
        expect((await select({ namingCalls: [], },)).shipped,).toBe(EXISTING_TEXT,);
      },
    },),
    it({
      name: 'SENDS the fallback repair on when most judges decline and one names a candidate, as the sheet says',
      fn: async () => {
        expect((await select({ namingCalls: [0,], },)).shipped,).toBe('The cat likes the sun.',);
      },
    },),
  ],
},);
