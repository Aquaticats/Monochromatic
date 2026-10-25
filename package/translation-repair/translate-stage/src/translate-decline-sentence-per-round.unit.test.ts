/**
 Guards the sentence a judge reads about what a decline does, as the translate stage renders it round by round over
 wording that cannot ship (re-check 4, F1).

 The sentence once promised, in every round, what only the last round of a pair does. The first ask of a pair
 (`initial-selection`) ships nothing and keeps nothing on a decline: `settleAbsentDecline` raises, and
 `judgeSlateWithRetry` asks the same panel again. The challenge ships the preferred candidate two or more judges named,
 or keeps the wording, only where its caller ships past a decline; the translate lane's first call does not, and
 re-produces with the judges' reasons instead (`translate-stage-repair.ts`). These cases render the real sheets through
 `judgeSlateWithRetry` with every judge declining, and read the sentence between the line that calls declining a real
 answer and the line that asks for the reply, by its clauses (`decline-consequence-clauses.test-fixture.ts`).

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import { tagged, } from '@monochromatic-dev/module-logger/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/module-or-throw/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import type { Candidate, } from '@monochromatic-dev/translation-repair-house-rule';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  type ChatJsonOutcome,
  type ChatJsonRequest,
  messageText,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import {
  laneCandidate,
  TranslateAbsenceError,
  type TranslateCandidateValue,
} from '@monochromatic-dev/translation-repair-translate-floor';
import { judgeSlateWithRetry, } from '../dist/final/node/index.mjs';
import {
  type DeclineMode,
  unmetClausesOf,
} from '@monochromatic-dev/translation-repair-adjudication/ts/decline-consequence-clauses.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Logger the stage writes through, whose output is not under test.
 */
const l = tagged({ tag: 'translate-decline-sentence-per-round-test', },);

/**
 Invented original: the cat dozed on the sill, tail by the radiator.
 */
const SOURCE = '猫猫在窗台上打盹，尾巴垂在暖气片旁边。';

/**
 Archive wording the deterministic rule refused.
 */
const ARCHIVE = 'The cat is doing the sleeping on the windowsill, a tail hanging by the radiator, purring for you.';

/**
 Line that opens the sentence about declining in a rendered selection sheet.
 */
const SENTENCE_OPENS_AT = 'Declining is a real answer';

/**
 Line that closes the sentence about declining in a rendered selection sheet.
 */
const SENTENCE_CLOSES_AT = 'Reply with ONLY a JSON object';

/**
 Judges, who all decline.
 */
const JUDGES: readonly RosterModelId[] = [
  'hf:cat/Cat-C',
  'hf:cat/Cat-F',
  'hf:cat/Cat-G',
].map(function toId(id,) {
  return id as unknown as RosterModelId;
},);

/**
 Writer of the fresh renderings.
 */
const WRITER = 'hf:cat/Cat-A' as unknown as RosterModelId;

/**
 A fresh rendering written by a model.

 @param rendering - the rendering

 @returns Candidate as the slate carries it

 @example
 ```ts
 const candidate = freshByWriter({ rendering: 'The cat dozes.', },);
 ```
 */
function freshByWriter({ rendering, }: { readonly rendering: string; },): Candidate<TranslateCandidateValue> {
  return {
    rendered: rendering,
    producer: {
      kind: 'model',
      modelId: WRITER,
    },
    value: {
      origin: 'fresh',
      text: rendering,
    },
  };
}

/**
 Client whose judges all decline, recording the sheet each was sent.

 @param sheets - where each sheet is recorded, in the order asked

 @returns Client over the candidate ballots only

 @example
 ```ts
 const client = decliningClient({ sheets: [], },);
 ```
 */
function decliningClient({ sheets, }: { readonly sheets: string[]; },): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText unused by the translate stage',);
    },
    quotas: async () => {
      throw new Error('quotas unused by the translate stage',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      sheets.push(request.messages
        .map(function textOf(message,): string {
          return messageText({ message, },);
        },)
        .join('\n',),);
      /**
       Ballot declining every candidate.
       */
      const ballot: unknown = {
        best: 0,
        assessment: `fixture rejection of ${request.modelId}`,
      };
      if (!request.validate(ballot,))
        throw new Error('the fixture ballot failed the wire guard',);
      return {
        kind: 'ok',
        value: ballot as ValueT,
        rawText: JSON.stringify(ballot,),
      };
    },
  };
}

/**
 Judges a withheld passage's slate through the retry with every judge declining.

 @param sheets - where each sheet is recorded
 @param shipPastDecline - whether the caller ships past a decline, left to the judging's default when absent

 @returns What the retry raised, or settled on

 @example
 ```ts
 const outcome = await judgedDeclining({ sheets: [], shipPastDecline: false, },);
 ```
 */
async function judgedDeclining(
  {
    sheets,
    shipPastDecline,
  }: {
    readonly sheets: string[];
    readonly shipPastDecline?: boolean;
  },
): Promise<unknown> {
  try {
    return await judgeSlateWithRetry({
      judging: {
        client: decliningClient({ sheets, },),
        produced: {
          candidates: [
            freshByWriter({ rendering: 'The cat dozes on the windowsill, tail draped beside the radiator.', },),
            freshByWriter({ rendering: 'A cat naps on the sill, its tail hanging near the heater.', },),
            laneCandidate({
              lane: 'repair',
              text: 'The cat curls up on the ledge, tail tucked by the warmth.',
            },),
            laneCandidate({
              lane: 'translate',
              text: 'The cat yawns on the sill, tail resting against the pipe.',
            },),
          ],
          heardTranslators: 2,
          findings: [],
        },
        judgeModelIds: JUDGES,
        sourceText: SOURCE,
        incumbentText: ARCHIVE,
        incumbentKind: 'absent',
        lineStructured: false,
        withheldStanding: true,
        ...((shipPastDecline === undefined) ? {} : { shipPastDecline, }),
        fanOut: 'whole-bench',
        signal: AbortSignal.timeout(HANG_STOP_MS,),
        perCallTimeoutMs: HANG_STOP_MS,
        l,
        stageSettings: SCRIPTED_STAGE_SETTINGS,
      },
    },);
  }
  catch (error) {
    return error;
  }
}

/**
 The sentence about declining in one sheet, as a judge reads it.

 @param sheet - rendered selection sheet

 @returns The text between the line that calls declining a real answer and the line that asks for the reply

 @example
 ```ts
 const sentence = sentenceOf({ sheet, },);
 ```
 */
function sentenceOf({ sheet, }: { readonly sheet: string; },): string {
  /**
   Where the sentence opens.
   */
  const opens = sheet.indexOf(SENTENCE_OPENS_AT,);
  /**
   Where the line that asks for the reply stands.
   */
  const closes = sheet.indexOf(SENTENCE_CLOSES_AT,);
  expect(opens,).toBeGreaterThanOrEqual(0,);
  expect(closes,).toBeGreaterThan(opens,);
  return sheet.slice(
    opens,
    closes,
  );
}

/**
 What a round's sheets fail, for the first sheet of each round.

 @param sheets - every sheet the judging sent, three to a round
 @param mode - the round the sentence is told in
 @param round - zero for the first ask, one for the challenge

 @returns The failures of the clause reader for that round

 @example
 ```ts
 const failures = failuresOf({ sheets, mode: 'asked-again', round: 0, },);
 ```
 */
function failuresOf(
  {
    sheets,
    mode,
    round,
  }: {
    readonly sheets: readonly string[];
    readonly mode: DeclineMode;
    readonly round: number;
  },
): readonly string[] {
  return unmetClausesOf({
    // The judging sends the same sheet to every seat of a round, so the first of the round stands for it.
    text: sentenceOf({ sheet: nonNullishOrThrow(sheets[round * JUDGES.length],), },),
    mode,
  },);
}

await describe({
  name: judgeSlateWithRetry.name,
  children: [
    it({
      name: 'TELLS THE FIRST ASK THAT A DECLINE SETTLES NOTHING and the same judges are asked again, where the '
        + 'caller ships past a decline in its challenge: it promises neither the ship nor the stay',
      fn: async () => {
        /**
         Every sheet the judging sent.
         */
        const sheets: string[] = [];
        await judgedDeclining({ sheets, },);
        expect(sheets.length,).toBe(JUDGES.length * 2,);
        expect(failuresOf({
          sheets,
          mode: 'asked-again',
          round: 0,
        },),).toEqual([],);
      },
    },),

    it({
      name: 'TELLS THE CHALLENGE THAT SHIPS PAST A DECLINE the two outcomes, the ship of the preferred candidate two '
        + 'or more judges named and the stay of the wording, the control for the other two rounds',
      fn: async () => {
        /**
         Every sheet the judging sent.
         */
        const sheets: string[] = [];
        await judgedDeclining({ sheets, },);
        expect(failuresOf({
          sheets,
          mode: 'ships',
          round: 1,
        },),).toEqual([],);
      },
    },),

    it({
      name: 'TELLS THE FIRST ASK THAT A DECLINE SETTLES NOTHING where the caller does not ship past a decline either '
        + '(the translate lane\'s first call): the first ask never ships or keeps, whatever its caller does later',
      fn: async () => {
        /**
         Every sheet the judging sent.
         */
        const sheets: string[] = [];
        await judgedDeclining({
          sheets,
          shipPastDecline: false,
        },);
        expect(failuresOf({
          sheets,
          mode: 'asked-again',
          round: 0,
        },),).toEqual([],);
      },
    },),

    it({
      name: 'TELLS THE CHALLENGE OF A CALLER THAT DOES NOT SHIP PAST A DECLINE that nothing ships but a level tie two '
        + 'or more judges named, and that the passage goes back for new proposals (the translate lane\'s first '
        + 'call, which raises the absence its follow-up round answers)',
      fn: async () => {
        /**
         Every sheet the judging sent.
         */
        const sheets: string[] = [];
        /**
         What the judging came to.
         */
        const outcome = await judgedDeclining({
          sheets,
          shipPastDecline: false,
        },);
        expect(outcome instanceof TranslateAbsenceError,).toBe(true,);
        expect(failuresOf({
          sheets,
          mode: 'defers',
          round: 1,
        },),).toEqual([],);
      },
    },),
  ],
},);
