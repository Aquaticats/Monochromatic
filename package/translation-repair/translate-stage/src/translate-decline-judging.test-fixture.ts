/**
 The judging a decline sentence is read in: a slate of invented candidates put through `judgeSlateWithRetry` with judges
 whose ballots a plan scripts, and the sheets each round sent, read back for the candidates they hold and the sentence
 about declining they carry (re-check 4, F1; re-check 6, M-B).

 A plan names, for each judge, the candidate text it votes for, or nothing for a decline. A judge votes by the text, not
 by a number, since a round numbers the candidates it is asked over from one and a run-off renumbers them.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import type { Candidate, } from '@monochromatic-dev/translation-repair-house-rule';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  type ChatJsonOutcome,
  type ChatJsonRequest,
  messageText,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import { replyWith, } from '@monochromatic-dev/translation-repair-provider/ts/scripted-reply-outcome.test-fixture.ts';
import {
  laneCandidate,
  type TranslateCandidateValue,
} from '@monochromatic-dev/translation-repair-translate-floor';
import { judgeSlateWithRetry, } from '../dist/final/node/index.mjs';
import {
  type DeclineMode,
  type RunoffShape,
  unmetClausesOf,
} from '@monochromatic-dev/translation-repair-adjudication/ts/decline-consequence-clauses.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import {
  SEAT_HYPER_ONLY,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

//region Translate decline judging
// The judges, the slates, the scripted client and the readers of a rendered sheet.

/**
 Logger the stage writes through, whose output is not under test.
 */
const l = tagged({ tag: 'translate-decline-judging-test', },);

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
 What `indexOf` answers for a text the sheet does not carry.
 */
const NOT_FOUND = -1;

/**
 Line that opens a candidate block in a rendered selection sheet, before the candidate's number.
 */
const CANDIDATE_OPENS_AT = '\nCANDIDATE ';

/**
 How many candidates a slate of two holds.
 */
export const PAIR = 2;

/**
 How many candidates a slate of four holds.
 */
export const QUARTET = 4;

/**
 Judges of the bench, in the order a plan names them.
 */
export const JUDGES: readonly RosterModelId[] = [
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
];

/**
 A rendering by the writer, and so a candidate no judge has a stake in.
 */
export const DOZING = 'The cat dozes on the windowsill, tail draped beside the radiator.';

/**
 A second rendering by the writer, which the second judge's own work stands beside in {@link slateOf}.
 */
export const NAPPING = 'A cat naps on the sill, its tail hanging near the heater.';

/**
 The repair lane's rendering.
 */
const CURLED ='The cat curls up on the ledge, tail tucked by the warmth.';

/**
 The translate lane's rendering.
 */
const YAWNING ='The cat yawns on the sill, tail resting against the pipe.';

/**
 A fresh rendering written by a model.

 @param rendering - rendering

 @param writer - model that wrote it

 @returns Candidate as the slate carries it

 @example
 ```ts
 const candidate = freshBy({ rendering: 'The cat dozes.', writer: SEAT_HYPER_ONLY, },);
 ```
 */
function freshBy(
  {
    rendering,
    writer,
  }: {
    readonly rendering: string;
    readonly writer: RosterModelId;
  },
): Candidate<TranslateCandidateValue> {
  return {
    rendered: rendering,
    producer: {
      kind: 'model',
      modelId: writer,
    },
    value: {
      origin: 'fresh',
      text: rendering,
    },
  };
}

/**
 The slate a case judges: four renderings, or two.

 @param width - how many candidates the slate holds, {@link QUARTET} or {@link PAIR}

 @param secondIsJudgesOwn - whether the second candidate was written by the second judge, whose vote for it then carries
 half weight, so that a plurality can stand over a vote the second judge cast for its own work

 @returns Slate's candidates in the order the producers gave them

 @example
 ```ts
 const candidates = slateOf({ width: PAIR, secondIsJudgesOwn: true, },);
 ```
 */
export function slateOf(
  {
    width,
    secondIsJudgesOwn,
  }: {
    readonly width: typeof PAIR | typeof QUARTET;
    readonly secondIsJudgesOwn: boolean;
  },
): readonly Candidate<TranslateCandidateValue>[] {
  /**
   The first two candidates, fresh.
   */
  const fresh = [
    freshBy({
      rendering: DOZING,
      writer: SEAT_HYPER_ONLY,
    },),
    freshBy({
      rendering: NAPPING,
      writer: secondIsJudgesOwn
        ? nonNullishOrThrow(JUDGES[1],)
        : SEAT_HYPER_ONLY,
    },),
  ];
  if (width === PAIR)
    return fresh;
  return [
    ...fresh,
    laneCandidate({
      lane: 'repair',
      text: CURLED,
    },),
    laneCandidate({
      lane: 'translate',
      text: YAWNING,
    },),
  ];
}

/**
 The text of one candidate block of a rendered selection sheet.

 A block reads its number, the opening fence, the text, and the closing fence, which equals the opening one.

 @param block - sheet from the number after the opening of a candidate block to the opening of the next

 @returns Text between the fences

 @example
 ```ts
 const text = candidateTextOf({ block: '1\n=====\nThe cat naps.\n=====\n', },); // 'The cat naps.'
 ```
 */
function candidateTextOf({ block, }: { readonly block: string; },): string {
  /**
   Lines of the block.
   */
  const lines = block.split('\n',);
  /**
   Fence the block opens with.
   */
  const fence = nonNullishOrThrow(lines[1],);
  /**
   Lines after the opening fence.
   */
  const body = lines.slice(2,);
  /**
   Lines before the closing fence.
   */
  const text = body.slice(
    0,
    body.indexOf(fence,),
  );
  return text.join('\n',);
}

/**
 The candidate texts a rendered selection sheet holds, in the order it numbers them.

 @param sheet - rendered selection sheet

 @returns Text of each fenced candidate block

 @example
 ```ts
 const texts = candidatesIn({ sheet, },);
 ```
 */
export function candidatesIn({ sheet, }: { readonly sheet: string; },): readonly string[] {
  /**
   The blocks after the first, which is the text before any candidate.
   */
  const blocks = sheet.split(CANDIDATE_OPENS_AT,)
    .slice(1,);
  return blocks.map(function textOf(block,): string {
    return candidateTextOf({ block, },);
  },);
}

/**
 Refuses a text call, which the translate stage never makes.

 @throws {@link Error} on any invocation
 */
function unexpectedText(): never {
  throw new Error('chatText unused by the translate stage',);
}

/**
 Refuses a quota read, which fixtures cannot answer.

 @throws {@link Error} on any invocation
 */
function unexpectedQuotas(): never {
  throw new Error('quotas unused by the translate stage',);
}

/**
 Client whose judges vote as a plan says, recording the sheet each was sent.

 @param sheets - where each sheet is recorded, in the order asked

 @param plan - text each judge votes for, judge it leaves out declining

 @returns Client over the candidate ballots only

 @example
 ```ts
 const client = votingClient({ sheets: [], plan: new Map([[JUDGES[0], DOZING,],],), },);
 ```
 */
function votingClient(
  {
    sheets,
    plan,
  }: {
    readonly sheets: string[];
    readonly plan: ReadonlyMap<RosterModelId, string>;
  },
): ModelClient {
  return {
    chatText: unexpectedText,
    quotas: unexpectedQuotas,
    /**
     Answers with the ballot the plan scripts for this judge, through the wire guard.
     */
    chatJson: function chatJson<ValueT,>(request: ChatJsonRequest<ValueT>,): Promise<ChatJsonOutcome<ValueT>> {
      /**
       The sheet this judge was sent.
       */
      const sheet = request.messages
        .map(function textOf(message,): string {
          return messageText({ message, },);
        },)
        .join('\n',);
      sheets.push(sheet,);
      /**
       Text this judge votes for, absent when it declines.
       */
      const named = plan.get(request.modelId,);
      /**
       Candidates this round holds, in the order it numbers them.
       */
      const held = candidatesIn({ sheet, },);
      /**
       Number of the candidate voted for, from one; zero declines every candidate, as it does for a text the round does
       not hold.
       */
      const best = (named === undefined)
        ? 0
        : (held.indexOf(named,) + 1);
      return Promise.resolve(replyWith({
        report: {
          best,
          assessment: `fixture ballot of ${request.modelId}`,
        },
        request,
      },),);
    },
  };
}

/**
 Judges a withheld passage's slate through the retry with the judges voting as a plan says.

 @param sheets - where each sheet is recorded

 @param candidates - slate

 @param plan - text each judge votes for; the default has every judge decline

 @param shipPastDecline - whether the caller ships past a decline, left to the judging's default when absent

 @returns What the retry raised, or settled on

 @example
 ```ts
 const outcome = await judgedOver({ sheets: [], candidates: slateOf({ width: QUARTET, secondIsJudgesOwn: false, },), },);
 ```
 */
export async function judgedOver(
  {
    sheets,
    candidates,
    plan = new Map<RosterModelId, string>(),
    shipPastDecline,
  }: {
    readonly sheets: string[];
    readonly candidates: readonly Candidate<TranslateCandidateValue>[];
    readonly plan?: ReadonlyMap<RosterModelId, string>;
    readonly shipPastDecline?: boolean;
  },
): Promise<unknown> {
  try {
    return await judgeSlateWithRetry({
      judging: {
        client: votingClient({
          sheets,
          plan,
        },),
        produced: {
          candidates,
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
    l.info(`the judging raised ${String(error,)}, which the case reads`,);
    return error;
  }
}

/**
 The sentence about declining in one sheet, as a judge reads it.

 @param sheet - rendered selection sheet

 @returns Text between the line that calls declining a real answer and the line that asks for the reply

 @throws {@link Error} when the sheet carries neither line, or the reply line stands before the sentence

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
  if ((opens === NOT_FOUND) || (closes < opens))
    throw new Error('the sheet does not carry the sentence about declining between its two lines',);
  return sheet.slice(
    opens,
    closes,
  );
}

/**
 First sheet of one round, which stands for it since the judging sends the same sheet to every seat of a round.

 @param sheets - every sheet the judging sent, one per judge in each round

 @param round - zero for the first ask, one for the challenge

 @returns Round's sheet

 @example
 ```ts
 const sheet = sheetOfRound({ sheets, round: 1, },);
 ```
 */
export function sheetOfRound(
  {
    sheets,
    round,
  }: {
    readonly sheets: readonly string[];
    readonly round: number;
  },
): string {
  return nonNullishOrThrow(sheets[round * JUDGES.length],);
}

/**
 What a round's sheet fails, read by the clauses of the round it is told in.

 @param sheets - every sheet the judging sent, one per judge in each round

 @param mode - round the sentence is told in

 @param round - zero for the first ask, one for the challenge

 @param shape - what the declined round came to, when the case names it

 @returns Failures of the clause reader for that round

 @example
 ```ts
 const failures = failuresOf({ sheets, mode: 'asked-again', round: 0, },);
 ```
 */
export function failuresOf(
  {
    sheets,
    mode,
    round,
    shape,
  }: {
    readonly sheets: readonly string[];
    readonly mode: DeclineMode;
    readonly round: number;
    readonly shape?: RunoffShape;
  },
): readonly string[] {
  return unmetClausesOf({
    text: sentenceOf({
      sheet: sheetOfRound({
        sheets,
        round,
      },),
    },),
    mode,
    // Conditional spread keeps the shape absent where the case names none.
    ...((shape === undefined) ? {} : { shape, }),
  },);
}

//endregion Translate decline judging
