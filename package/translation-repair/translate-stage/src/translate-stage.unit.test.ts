/**
 Tests for the translate lane: several models render one slice from its
 original, the translation already in the archive stands among them, and
 judges choose.

 What these lock down is mostly what the stage does when something is MISSING,
 because that is the whole reason the lane exists. A slice with no translation
 must still produce one; a translator that answers with nothing must not put an
 empty candidate on the ballot; a lost voice must be named rather than reduce
 quietly to a smaller slate.

 Judges are scripted BY THE TEXT they see rather than by candidate number, on
 purpose: the stage rotates the slate per slice so the incumbent does not sit
 in one position, and a test that pinned index 1 would be asserting the
 rotation rather than the decision.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  type IncumbentKind,
  TRANSLATE_LINE_STRUCTURE_CRITERION,
  TranslateAbsenceError,
  validateTranslatedSlice,
} from '@monochromatic-dev/translation-repair-translate-floor';
import {
  type ChatJsonOutcome,
  type ChatJsonRequest,
  messageText,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import { firstRoundWindow, } from '@monochromatic-dev/translation-repair-stage';
import {
  runTranslateStage,
  unflooredFinding,
} from '../dist/final/node/index.mjs';
import { capturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import {
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_VISION,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { candidateCarrying, } from '@monochromatic-dev/translation-repair-test-fixture/ts/translate-ballot.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Logger for the stage under test.
 */
const l = tagged({ tag: 'translate-stage-test', },);

/**
 Original slice every case renders.
 */
const SOURCE_TEXT = '猫猫在窗台上打盹，尾巴垂在暖气片旁边。';

/**
 Translation already in the archive, awkward but present.
 */
const INCUMBENT_TEXT = 'The cat is doing the sleeping on the windowsill, with tail hanging by the radiator.';

/**
 Original with an expression the strict grammar never closes, so the floor
 can compare nothing written for it.
 */
const UNREADABLE_SOURCE = '猫猫在{窗台上打盹。';

/**
 Models that render the slice.
 */
const TRANSLATORS: readonly RosterModelId[] = [
  SEAT_SYNTHETIC_VISION_WITHHELD,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_HYPER_VISION,
];

/**
 Whole roster the judges are drawn from, translators included.
 */
const JUDGES: readonly RosterModelId[] = [
  ...TRANSLATORS,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
];

/**
 Judges one healthy select round asks: quorum plus one, not the bench
 (`stage-fanout-window.ts`), so a round on six judges is four calls.
 */
const JUDGE_WINDOW = firstRoundWindow({ benchSize: JUDGES.length, },);

/**
 What each model returns when asked to translate.

 A model absent from the map answers with prose wrapped around its JSON, which
 fails the wire guard and costs the stage that voice.
 */
type TranslateScript = Readonly<Record<string, string>>;

/**
 Calls the stage made, by stage name.
 */
type CallLog = {
  translate: number;
  select: number;
};

/**
 Client serving both stages of the lane from a script.

 @param translations - what each translator returns

 @param followupTranslations - alternate renderings after exact rejection evidence

 @param needle - text the judges vote for, absent when they should abstain

 @param needleAfterRetry - text the judges vote for once the panel has been
 asked a second time, so a case can script a panel that declines and then
 agrees; without it the panel answers the same way every round

 @param calls - shared call log the cases assert on

 @param judgeSheets - every judge sheet this run produced, so a case can read
 what judges were actually told rather than what the prompt builder is
 believed to say

 @returns Client honoring the script

 @example
 ```ts
 const client = laneClient({ translations, needle: 'dozes', calls, },);
 ```
 */
function laneClient(
  {
    translations,
    followupTranslations,
    needle,
    needleAfterRetry,
    calls,
    judgeSheets,
    producerPrompts,
  }: {
    readonly translations: TranslateScript;
    readonly followupTranslations?: readonly TranslateScript[];
    readonly needle: string;
    readonly needleAfterRetry?: string;
    readonly calls: CallLog;
    readonly judgeSheets: string[];
    readonly producerPrompts: string[];
  },
): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText unused by the translate lane',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      /**
       Schema the caller asked for, which names the stage.
       */
      const schema = request.responseFormat
        ?.json_schema
        .name;
      if (schema === 'translation_report') {
        /**
         Zero-based production round before this call is counted.
         */
        const productionRound = Math.floor(calls.translate / TRANSLATORS.length,);
        calls.translate += 1;
        producerPrompts.push(JSON.stringify({
          modelId: request.modelId,
          messages: request.messages,
        },),);

        /**
         Whether this request is grounded in latest rejected slate.
         */
        const isFollowup = request.messages.some(function carriesRejection(message,): boolean {
          return messageText({ message, },).includes('LATEST REJECTION',);
        },);
        /**
         Script selected by initial or repair responsibility.
         */
        const selectedTranslations = (isFollowup && (followupTranslations !== undefined))
          ? (followupTranslations[productionRound - 1] ?? translations)
          : translations;
        /**
         Rendering this translator was scripted to return, absent when it was
         scripted to answer unusably.
         */
        const scripted = selectedTranslations[request.modelId];
        if (scripted === undefined) {
          return {
            kind: 'schema-mismatch',
            rawText: 'Here is my translation:\n{"translation": "..."}',
            detail: 'prose around the JSON',
          };
        }

        /**
         Wire reply carrying it.
         */
        const value: unknown = { translation: scripted, };
        // EXACTLY WHAT THE REAL CLIENT DOES with a reply that fails the
        // caller's guard, rather than throwing at the script: a model is free
        // to answer with a blank translation, and what the lane must do about
        // it is the thing under test.
        if (!request.validate(value,)) {
          return {
            kind: 'schema-mismatch',
            rawText: JSON.stringify(value,),
            detail: 'reply failed the wire guard',
          };
        }
        return {
          kind: 'ok',
          value,
          rawText: JSON.stringify(value,),
        };
      }
      calls.select += 1;

      /**
       Judge sheet as this judge received it.
       */
      const content = request.messages
        .map(function toContent(message,) {
          return messageText({ message, },);
        },)
        .join('\n',);

      judgeSheets.push(content,);

      /**
       Text this round votes for, which changes once the panel has been asked
       again. `calls.select` counts individual judges, so a whole first round
       is one per judge the window asked.
       */
      const roundNeedle = ((needleAfterRetry !== undefined) && (calls.select > JUDGE_WINDOW))
        ? needleAfterRetry
        : needle;

      /**
       Ballot naming the candidate carrying the needle.
       */
      const ballot: unknown = {
        best: (roundNeedle === '') ? 0 : candidateCarrying({
          content,
          needle: roundNeedle,
        },),
        assessment: 'scripted',
      };
      if (!request.validate(ballot,))
        throw new Error('scripted ballot failed the wire guard',);
      return {
        kind: 'ok',
        value: ballot,
        rawText: JSON.stringify(ballot,),
      };
    },
    quotas: async () => {
      throw new Error('quotas unused by the translate lane',);
    },
  };
}

/**
 Runs the lane over the fixture slice.

 @param translations - what each translator returns

 @param followupTranslations - alternate outputs for stage-local repair

 @param needle - text the judges vote for, empty to make them decline

 @param incumbentText - translation as it stands

 @param sourceText - original passage, defaulting to shared fixture

 @param messages - log the stage's lines are kept in, absent to log as usual

 @returns Stage result plus the call log

 @example
 ```ts
 const { result, } = await runLane({ translations, needle: 'dozes', },);
 ```
 */
async function runLane(
  {
    translations,
    followupTranslations,
    needle,
    needleAfterRetry,
    incumbentText,
    incumbentKind = 'present',
    sourceText = SOURCE_TEXT,
    neighbouringSourceText,
    lineStructured = false,
    messages,
  }: {
    readonly translations: TranslateScript;
    readonly followupTranslations?: readonly TranslateScript[];
    readonly needle: string;
    readonly needleAfterRetry?: string;
    readonly incumbentText: string;
    readonly incumbentKind?: IncumbentKind;
    readonly sourceText?: string;
    readonly neighbouringSourceText?: string;
    readonly messages?: string[];

    /**
     Whether the enclosing chunk is governed by the verse rule, which decides
     what BOTH halves of this round are told.
     */
    readonly lineStructured?: boolean;
  },
) {
  /**
   Calls each stage made.
   */
  const calls: CallLog = {
    translate: 0,
    select: 0,
  };

  /**
   Judge sheets this round produced, so a case can read what judges were told.
   */
  const judgeSheets: string[] = [];
  /**
   Exact model-plus-message identities for producer calls.
   */
  const producerPrompts: string[] = [];

  /**
   What the lane decided for this slice.
   */
  const result = await runTranslateStage({
    client: laneClient({
      translations,
      ...((followupTranslations === undefined) ? {} : { followupTranslations, }),
      needle,
      ...((needleAfterRetry === undefined) ? {} : { needleAfterRetry, }),
      calls,
      judgeSheets,
      producerPrompts,
    },),
    translatorModelIds: TRANSLATORS,
    judgeModelIds: JUDGES,
    sourceText,
    incumbentText,
    incumbentKind,
    ...((neighbouringSourceText === undefined) ? {} : { neighbouringSourceText, }),
    lineStructured,
    signal: new AbortController().signal,
    perCallTimeoutMs: HANG_STOP_MS,
    l: (messages === undefined) ? l : capturingLogger({ messages, },),
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
  return {
    result,
    calls,
    judgeSheets,
    producerPrompts,
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: runTranslateStage.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'translates a slice that has NO translation at all, which the '
            + 'editor stage cannot reach: an absent passage files no defect, so '
            + 'the defect-driven loop never sees it',
          fn: async () => {
            const { result, } = await runLane({
              translations: {
                [SEAT_SYNTHETIC_VISION_WITHHELD]: 'The cat dozes on the windowsill, tail draped beside the radiator.',
                [SEAT_HYPER_OPENROUTER_VISION_EDITOR]: 'A cat naps on the sill, its tail hanging near the heater.',
                [SEAT_HYPER_VISION]: 'The cat sleeps on the ledge, tail beside the radiator.',
              },
              needle: 'dozes',
              incumbentText: '',
            },);
            expect(result.origin,).toBe('fresh',);
            expect(result.decision,).toBe('judged',);
            expect(result.text,).toBe(
              'The cat dozes on the windowsill, tail draped beside the radiator.',
            );
            // Three renderings and nothing else: an empty incumbent is not offered,
            // since "leave it untranslated" is not a candidate.
            expect(result.candidateCount,).toBe(3,);
          },
        },),

        it({
          name: 'stands the existing translation among the candidates and reports '
            + 'it as KEPT when judges prefer it, which is the measurement the '
            + 'whole lane exists to produce',
          fn: async () => {
            const { result, } = await runLane({
              translations: {
                [SEAT_SYNTHETIC_VISION_WITHHELD]: 'The cat dozes on the windowsill, tail draped beside the radiator.',
                [SEAT_HYPER_OPENROUTER_VISION_EDITOR]: 'A cat naps on the sill, its tail hanging near the heater.',
                [SEAT_HYPER_VISION]: 'The cat sleeps on the ledge, tail beside the radiator.',
              },
              needle: 'is doing the sleeping',
              incumbentText: INCUMBENT_TEXT,
            },);
            expect(result.origin,).toBe('incumbent',);
            expect(result.decision,).toBe('judged',);
            expect(result.text,).toBe(INCUMBENT_TEXT,);
            expect(result.candidateCount,).toBe(4,);
            // The window's judges, translators among them. Nobody wrote the
            // incumbent, so every ballot for it carries full weight.
            expect(result.voteWeight,).toBe(JUDGE_WINDOW,);
          },
        },),

        it({
          name: 'EXCLUDES archive fallback missing source destination and settles source-complete rendering',
          fn: async () => {
            const sourceText = '[The cat slept](https://example.test/cat-record).';
            const incumbentText = 'The cat slept.';
            const fresh = '[The cat slept](https://example.test/cat-record).';
            const { result, } = await runLane({
              translations: {
                [SEAT_SYNTHETIC_VISION_WITHHELD]: fresh,
                [SEAT_HYPER_OPENROUTER_VISION_EDITOR]: fresh,
                [SEAT_HYPER_VISION]: fresh,
              },
              needle: 'cat-record',
              sourceText,
              incumbentText,
            },);

            expect(result.origin,).toBe('fresh');
            expect(result.text,).toBe(fresh);
            expect(result.findings,).toContain('translate incumbent excluded by deterministic source floor');
            expect(result.candidateCount,).toBe(1);
          },
        },),
        it({
          name: 'ASKS NOBODY where the floor can compare nothing, an original the strict grammar cannot read: the '
            + 'archive stands, and the finding and the log name the floor\'s own reason (ledger B43)',
          fn: async () => {
            /**
             The floor's account of why it compared nothing, as the stage asks it.
             */
            const verdict = validateTranslatedSlice({
              sourceText: UNREADABLE_SOURCE,
              candidateText: INCUMBENT_TEXT,
              pageText: INCUMBENT_TEXT,
              lineStructured: false,
              declared: [],
              disputedWordings: [],
            },);
            if (verdict.kind !== 'unknown')
              throw new Error(`the fixture's original must be one no grammar reads, and the floor said ${verdict.kind}`,);
            const messages: string[] = [];
            const {
              result,
              calls,
            } = await runLane({
              translations: {
                [SEAT_SYNTHETIC_VISION_WITHHELD]: 'The cat dozes on the windowsill.',
                [SEAT_HYPER_OPENROUTER_VISION_EDITOR]: 'The cat dozes on the windowsill.',
                [SEAT_HYPER_VISION]: 'The cat dozes on the windowsill.',
              },
              needle: 'dozes',
              sourceText: UNREADABLE_SOURCE,
              incumbentText: INCUMBENT_TEXT,
              messages,
            },);

            expect(calls,).toEqual({
              translate: 0,
              select: 0,
            },);
            expect(result,).toEqual({
              text: INCUMBENT_TEXT,
              origin: 'incumbent',
              producer: {
                kind: 'incumbent',
                matched: [],
              },
              decision: 'unfloored',
              voteWeight: 0,
              tally: {
                judgesAvailable: 0,
                ballots: 0,
                abstentions: 0,
                selfVotes: 0,
              },
              ballots: [],
              heardTranslators: 0,
              candidateCount: 0,
              findings: [unflooredFinding({ stage: 'translate', detail: verdict.detail, },),],
              slate: [],
              selectedIndex: 0,
              shippedIndex: 0,
              perCandidate: [],
            },);
            expect(messages.some(function namesTheReason(line,): boolean {
              return line.includes(verdict.detail,) && line.includes('the archive stands',);
            },),).toBe(true,);
          },
        },),
        it({
          name: 'LEAVES AN ABSENT PASSAGE UNFILLED where the floor can compare nothing, with no call, rather than '
            + 'writing in a rendering nobody could check (ledger B43)',
          fn: async () => {
            /**
             Calls each stage made.
             */
            const calls = {
              translate: 0,
              select: 0,
            };
            /**
             What the stage raised.
             */
            let raised: unknown;
            try {
              await runTranslateStage({
                client: laneClient({
                  translations: {
                    [SEAT_SYNTHETIC_VISION_WITHHELD]: 'The cat dozes on the windowsill.',
                    [SEAT_HYPER_OPENROUTER_VISION_EDITOR]: 'The cat dozes on the windowsill.',
                    [SEAT_HYPER_VISION]: 'The cat dozes on the windowsill.',
                  },
                  needle: 'dozes',
                  calls,
                  judgeSheets: [],
                  producerPrompts: [],
                },),
                translatorModelIds: TRANSLATORS,
                judgeModelIds: JUDGES,
                sourceText: UNREADABLE_SOURCE,
                incumbentText: '',
                incumbentKind: 'absent',
                lineStructured: false,
                signal: new AbortController().signal,
                perCallTimeoutMs: HANG_STOP_MS,
                l,
                stageSettings: SCRIPTED_STAGE_SETTINGS,
              },);
            }
            catch (error) {
              raised = error;
            }
            if (!(raised instanceof TranslateAbsenceError))
              throw new Error('expected the stage to leave the passage unfilled',);

            expect(calls,).toEqual({
              translate: 0,
              select: 0,
            },);
            /**
             The floor's account of why it compared nothing on this slice.
             */
            const verdict = validateTranslatedSlice({
              sourceText: UNREADABLE_SOURCE,
              candidateText: 'The cat dozes on the windowsill.',
              pageText: '',
              lineStructured: false,
              declared: [],
              disputedWordings: [],
            },);
            if (verdict.kind !== 'unknown')
              throw new Error(`the fixture's original must be one no grammar reads, and the floor said ${verdict.kind}`,);
            expect(raised.reason,).toBe('unfloored',);
            expect(raised.findings,).toEqual([unflooredFinding({ stage: 'translate', detail: verdict.detail, },),],);
          },
        },),
        it({
          name: 'NAMES a translator whose reply arrives wrapped in prose and ships '
            + 'from the voices that remain, since a smaller slate that says '
            + 'nothing about why reads exactly like a slate nobody had more to '
            + 'offer for',
          fn: async () => {
            const { result, } = await runLane({
              translations: {
                // Kimi is absent from the script, so its reply arrives wrapped
                // in prose and fails the wire guard.
                [SEAT_HYPER_OPENROUTER_VISION_EDITOR]: 'A cat naps on the sill, its tail hanging near the heater.',
                [SEAT_HYPER_VISION]: 'The cat sleeps on the ledge, tail beside the radiator.',
              },
              needle: 'naps on the sill',
              incumbentText: INCUMBENT_TEXT,
            },);
            expect(result.heardTranslators,).toBe(2,);
            expect(result.findings,).toContain(
              'stage-voice-lost (translate hf:moonshotai/Kimi-K3)',
            );
            expect(result.origin,).toBe('fresh',);
            expect(result.text,).toBe('A cat naps on the sill, its tail hanging near the heater.',);
          },
        },),

        it({
          name: 'RE-ASKS a translator that answered with EMPTY text rather than counting it as heard, since '
            + 'a reply saying nothing is not a reply: the roster gets another round out of that model, and '
            + 'the loss is named if it stays blank',
          fn: async () => {
            const { result, } = await runLane({
              translations: {
                [SEAT_SYNTHETIC_VISION_WITHHELD]: '   \n  ',
                [SEAT_HYPER_OPENROUTER_VISION_EDITOR]: 'A cat naps on the sill, its tail hanging near the heater.',
                [SEAT_HYPER_VISION]: 'The cat sleeps on the ledge, tail beside the radiator.',
              },
              needle: 'naps on the sill',
              incumbentText: INCUMBENT_TEXT,
            },);
            // NOT heard, which is the change: the blank used to arrive as a voice
            // and be filtered off the ballot afterwards, so the model was recorded
            // as having answered and never re-asked.
            expect(result.heardTranslators,).toBe(2,);
            expect(result.findings,).toContain(
              'stage-voice-lost (translate hf:moonshotai/Kimi-K3)',
            );
            expect(result.candidateCount,).toBe(3,);
          },
        },),

        it({
          name: 'KEEPS the existing translation when judges decline TWICE, and records '
            + 'that as a decline rather than a win. A tie, a lost round and an '
            + 'empty slate all ship the incumbent too, and counting those as wins '
            + 'would report the archive as vindicated by the rounds that examined '
            + 'nothing',
          fn: async () => {
            const { result, calls, } = await runLane({
              translations: {
                [SEAT_SYNTHETIC_VISION_WITHHELD]: 'The cat dozes on the windowsill, tail draped beside the radiator.',
                [SEAT_HYPER_OPENROUTER_VISION_EDITOR]: 'A cat naps on the sill, its tail hanging near the heater.',
                [SEAT_HYPER_VISION]: 'The cat sleeps on the ledge, tail beside the radiator.',
              },
              needle: '',
              incumbentText: INCUMBENT_TEXT,
            },);
            expect(result.decision,).toBe('no-candidate-backed',);
            expect(result.origin,).toBe('incumbent',);
            expect(result.text,).toBe(INCUMBENT_TEXT,);
            expect(result.voteWeight,).toBe(0,);
            expect(result.findings,).toContain('translate-declined (rejection)',);
            // The panel was asked twice about the same candidates, which is what
            // separates a settled decline from a momentary one.
            expect(result.findings,).toContain('translate-declined-retried',);
            expect(calls.select,).toBe(JUDGE_WINDOW * 2,);
          },
        },),

        it({
          name: 'ACCEPTS a candidate the panel backs on the SECOND ask, which is the '
            + 'whole reason the retry is bought: a panel that declines once has not '
            + 'necessarily settled anything, and shipping the incumbent there would '
            + 'discard a rendering the judges did in the end prefer',
          fn: async () => {
            const { result, calls, } = await runLane({
              translations: {
                [SEAT_SYNTHETIC_VISION_WITHHELD]: 'The cat dozes on the windowsill, tail draped beside the radiator.',
                [SEAT_HYPER_OPENROUTER_VISION_EDITOR]: 'A cat naps on the sill, its tail hanging near the heater.',
                [SEAT_HYPER_VISION]: 'The cat sleeps on the ledge, tail beside the radiator.',
              },
              needle: '',
              needleAfterRetry: 'dozes',
              incumbentText: INCUMBENT_TEXT,
            },);
            expect(result.decision,).toBe('judged',);
            expect(result.origin,).not
              .toBe('incumbent',);
            expect(result.findings,).toContain('translate-declined-retried',);
            expect(calls.select,).toBe(JUDGE_WINDOW * 2,);
          },
        },),

        it({
          name: 'BUYS NO SECOND ROUND when the first one decided, since the retry '
            + 'exists to separate a momentary decline from a settled one and a '
            + 'decision is neither',
          fn: async () => {
            const { result, calls, } = await runLane({
              translations: {
                [SEAT_SYNTHETIC_VISION_WITHHELD]: 'The cat dozes on the windowsill, tail draped beside the radiator.',
                [SEAT_HYPER_OPENROUTER_VISION_EDITOR]: 'A cat naps on the sill, its tail hanging near the heater.',
                [SEAT_HYPER_VISION]: 'The cat sleeps on the ledge, tail beside the radiator.',
              },
              needle: 'dozes',
              incumbentText: INCUMBENT_TEXT,
            },);
            expect(result.decision,).toBe('judged',);
            expect(result.findings,).not
              .toContain('translate-declined-retried',);
            expect(calls.select,).toBe(JUDGE_WINDOW,);
          },
        },),


        it({
          name: 'ships UNJUDGED when every translator reproduced the existing '
            + 'translation, since nothing could change whatever the judges said, '
            + 'and names the models that matched it: a text several models arrive '
            + 'at independently is not the same evidence as one nobody examined',
          fn: async () => {
            const { result, calls, } = await runLane({
              translations: {
                [SEAT_SYNTHETIC_VISION_WITHHELD]: INCUMBENT_TEXT,
                [SEAT_HYPER_OPENROUTER_VISION_EDITOR]: INCUMBENT_TEXT,
                [SEAT_HYPER_VISION]: INCUMBENT_TEXT,
              },
              needle: 'is doing the sleeping',
              incumbentText: INCUMBENT_TEXT,
            },);
            expect(result.decision,).toBe('sole-candidate',);
            expect(result.origin,).toBe('incumbent',);
            expect(result.candidateCount,).toBe(1,);
            // No judge was asked, which is the cost this exit saves.
            expect(calls.select,).toBe(0,);
            expect(result.findings,).toContain(
              'translate-matched-incumbent (hf:moonshotai/Kimi-K3)',
            );
          },
        },),

        it({
          name: 'TELLS THE JUDGES what declining actually costs, which differs by slice: the shared sheet '
            + 'promises that the caller keeps text it already trusts, and at a slice with no translation '
            + 'that promise is false and buys a missing passage with the caution it asks for',
          fn: async () => {
            /** Round over a slice the archive HAS translated. */
            const present = await runLane({
              translations: {
                [SEAT_SYNTHETIC_VISION_WITHHELD]: 'The cat dozes on the windowsill, tail draped beside the radiator.',
                [SEAT_HYPER_OPENROUTER_VISION_EDITOR]: 'A cat naps on the sill, its tail hanging near the heater.',
                [SEAT_HYPER_VISION]: 'The cat sleeps on the ledge, tail beside the radiator.',
              },
              needle: 'dozes',
              incumbentText: INCUMBENT_TEXT,
            },);

            /** Round over one it has not. */
            const absent = await runLane({
              translations: {
                [SEAT_SYNTHETIC_VISION_WITHHELD]: 'The cat dozes on the windowsill, tail draped beside the radiator.',
                [SEAT_HYPER_OPENROUTER_VISION_EDITOR]: 'A cat naps on the sill, its tail hanging near the heater.',
                [SEAT_HYPER_VISION]: 'The cat sleeps on the ledge, tail beside the radiator.',
              },
              needle: 'dozes',
              incumbentText: '',
              incumbentKind: 'absent',
            },);
            expect(present.judgeSheets
              .length,).toBeGreaterThan(0,);
            expect(absent.judgeSheets
              .length,).toBeGreaterThan(0,);
            expect(present.judgeSheets
              .every(function keepsTrustedText(sheet: string,): boolean {
                return sheet.includes('keeps text it already trusts',);
              },),).toBe(true,);
            expect(absent.judgeSheets
              .every(function saysUntranslated(sheet: string,): boolean {
                return sheet.includes('leaves it untranslated',);
              },),).toBe(true,);
            expect(absent.judgeSheets
              .some(function keepsTrustedText(sheet: string,): boolean {
                return sheet.includes('keeps text it already trusts',);
              },),).toBe(false,);
          },
        },),

        it({
          name: 'FILLS a slice the archive never translated, which is the ordinary absent-mode round: the '
            + 'incumbent is not on the ballot because there is none, and the judges choose among the '
            + 'renderings alone',
          fn: async () => {
            const { result, } = await runLane({
              translations: {
                [SEAT_SYNTHETIC_VISION_WITHHELD]: 'The cat dozes on the windowsill, tail draped beside the radiator.',
                [SEAT_HYPER_OPENROUTER_VISION_EDITOR]: 'A cat naps on the sill, its tail hanging near the heater.',
                [SEAT_HYPER_VISION]: 'The cat sleeps on the ledge, tail beside the radiator.',
              },
              needle: 'dozes',
              incumbentText: '',
              incumbentKind: 'absent',
            },);
            expect(result.origin,).toBe('fresh',);
            expect(result.decision,).toBe('judged',);
            expect(result.candidateCount,).toBe(3,);
          },
        },),

        it({
          name: 'PAUSES operationally rather than settling when absent passage hears no translator voice',
          fn: async () => {
            await expect(runLane({
              // Nobody is scripted, so every reply arrives wrapped in prose and
              // fails the wire guard: no voice, no candidate, and no incumbent to
              // stand in for them.
              translations: {},
              needle: 'dozes',
              incumbentText: '',
              incumbentKind: 'absent',
            },),).rejects.toThrow('translation repair interrupted: provider-unavailable',);
          },
        },),

        it({
          name: 'SETTLES a declined absent passage as absence after the single follow-up round',
          fn: async () => {
            // Fixed depth two: the initial round, one follow-up carrying the
            // rejection evidence, then the absence is settled evidence for the
            // slice attempt to record as unfilled, never a paused entry.
            await expect(runLane({
              translations: {
                [SEAT_SYNTHETIC_VISION_WITHHELD]: 'The cat dozes on the windowsill, tail draped beside the radiator.',
                [SEAT_HYPER_OPENROUTER_VISION_EDITOR]: 'A cat naps on the sill, its tail hanging near the heater.',
                [SEAT_HYPER_VISION]: 'The cat sleeps on the ledge, tail beside the radiator.',
              },
              needle: '',
              incumbentText: '',
              incumbentKind: 'absent',
            },),).rejects.toThrow(TranslateAbsenceError,);
          },
        },),

        it({
          name: 'FILLS an absent passage by producing from latest exact rejected slate instead of ending on decline',
          fn: async () => {
            const { result, calls, } = await runLane({
              translations: {
                [SEAT_SYNTHETIC_VISION_WITHHELD]: 'A cat rests at the window.',
                [SEAT_HYPER_OPENROUTER_VISION_EDITOR]: 'A cat rests beside the heater.',
                [SEAT_HYPER_VISION]: 'The cat is near a window.',
              },
              followupTranslations: [{
                [SEAT_SYNTHETIC_VISION_WITHHELD]: 'The repaired cat dozes on the windowsill, tail beside the radiator.',
                [SEAT_HYPER_OPENROUTER_VISION_EDITOR]: 'A repaired cat naps on the sill, its tail beside the heater.',
                [SEAT_HYPER_VISION]: 'The repaired cat sleeps by the radiator.',
              },],
              needle: 'repaired',
              incumbentText: '',
              incumbentKind: 'absent',
            },);

            expect(result.origin,).toBe('fresh',);
            expect(result.text,).toContain('repaired',);
            expect(calls.translate,).toBe(TRANSLATORS.length * 2,);
            expect(calls.select,).toBe(JUDGE_WINDOW * 3,);
          },
        },),

        it({
          name: 'STOPS at depth two even when a third round would have succeeded',
          fn: async () => {
            // The needle appears only in the SECOND scripted follow-up, which a
            // fixed-depth stage never asks for: any regression back to a loop
            // would find it and turn this throw into a success.
            await expect(runLane({
              translations: {
                [SEAT_SYNTHETIC_VISION_WITHHELD]: 'The first cat rests.',
                [SEAT_HYPER_OPENROUTER_VISION_EDITOR]: 'The first cat waits.',
                [SEAT_HYPER_VISION]: 'The first cat sits.',
              },
              followupTranslations: [
                {
                  [SEAT_SYNTHETIC_VISION_WITHHELD]: 'The second cat rests.',
                  [SEAT_HYPER_OPENROUTER_VISION_EDITOR]: 'The second cat waits.',
                  [SEAT_HYPER_VISION]: 'The second cat sits.',
                },
                {
                  [SEAT_SYNTHETIC_VISION_WITHHELD]: 'The final repaired cat dozes.',
                  [SEAT_HYPER_OPENROUTER_VISION_EDITOR]: 'The final repaired cat naps.',
                  [SEAT_HYPER_VISION]: 'The final repaired cat sleeps.',
                },
              ],
              needle: 'final repaired',
              incumbentText: '',
              incumbentKind: 'absent',
            },),).rejects.toThrow(TranslateAbsenceError,);
          },
        },),

        it({
          name: 'carries the evidence on the settled absence rather than losing it with the exception, so a '
            + 'run recording a passage it could not fill can say which translators were heard and what '
            + 'the judges counted',
          fn: async () => {
            /** Refusal the depth-two rounds raised. */
            let raised: unknown;
            try {
              await runLane({
                translations: {
                  [SEAT_SYNTHETIC_VISION_WITHHELD]: 'The cat dozes on the windowsill, tail draped beside the radiator.',
                  [SEAT_HYPER_OPENROUTER_VISION_EDITOR]: 'A cat naps on the sill, its tail hanging near the heater.',
                  [SEAT_HYPER_VISION]: 'The cat sleeps on the ledge, tail beside the radiator.',
                },
                needle: '',
                incumbentText: '',
                incumbentKind: 'absent',
              },);
            }
            catch (error) {
              raised = error;
            }
            expect(raised instanceof TranslateAbsenceError,).toBe(true,);
            if (!(raised instanceof TranslateAbsenceError))
              throw new Error('expected settled translate absence',);
            expect(raised.findings,).toContain('translate-declined (rejection)',);
            expect(raised.findings,).toContain('translate-declined-retried',);
            expect(raised.findings
              .some(function namesTheSlate(finding: string,): boolean {
                return finding.startsWith('translate-candidates',);
              },),).toBe(true,);
          },
        },),

        it({
          name: 'CARRIES THE FIRST ROUND\'S EVIDENCE beside the follow-up\'s and names the follow-up, on a settled '
            + 'absence and on a follow-up that fills, so the record says the first slate was declined and why '
            + '(ledger B41)',
          fn: async () => {
            /**
             What one declined round records: the slate, the decline, the same
             panel asked again, and its second decline, each ask with the slate it
             judged.
             */
            const declinedRound = [
              'translate incumbent excluded by deterministic source floor',
              'translate-candidates (3/3 heard, 3 distinct, 0 collapsed)',
              'translate-declined (rejection)',
              'translate-declined-retried',
              'translate incumbent excluded by deterministic source floor',
              'translate-candidates (3/3 heard, 3 distinct, 0 collapsed)',
              'translate-declined (rejection)',
            ];
            /**
             What names the follow-up round and why it was asked.
             */
            const followup = 'translate-followup-round (after no-candidate-backed, 3 rejected candidates)';
            /**
             What the depth-two rounds raised when both declined.
             */
            const raised = await (async function attempt(): Promise<unknown> {
              try {
                await runLane({
                  translations: {
                    [SEAT_SYNTHETIC_VISION_WITHHELD]: 'The cat dozes on the windowsill, tail draped beside the radiator.',
                    [SEAT_HYPER_OPENROUTER_VISION_EDITOR]: 'A cat naps on the sill, its tail hanging near the heater.',
                    [SEAT_HYPER_VISION]: 'The cat sleeps on the ledge, tail beside the radiator.',
                  },
                  // A follow-up slate that differs, since one that repeats the first is not judged again
                  // (`translate-followup-repeated-slate.unit.test.ts`).
                  followupTranslations: [{
                    [SEAT_SYNTHETIC_VISION_WITHHELD]: 'The cat dozes by the window, its tail on the radiator.',
                    [SEAT_HYPER_OPENROUTER_VISION_EDITOR]: 'A cat naps on the ledge, tail beside the heater.',
                    [SEAT_HYPER_VISION]: 'The cat sleeps near the glass, tail beside the radiator.',
                  },],
                  needle: '',
                  incumbentText: '',
                  incumbentKind: 'absent',
                },);
                return 'settled';
              }
              catch (error) {
                return error;
              }
            })();
            if (!(raised instanceof TranslateAbsenceError))
              throw new Error('expected settled translate absence',);
            expect(raised.findings,).toEqual([
              ...declinedRound,
              followup,
              ...declinedRound,
            ],);

            const { result, } = await runLane({
              translations: {
                [SEAT_SYNTHETIC_VISION_WITHHELD]: 'A cat rests at the window.',
                [SEAT_HYPER_OPENROUTER_VISION_EDITOR]: 'A cat rests beside the heater.',
                [SEAT_HYPER_VISION]: 'The cat is near a window.',
              },
              followupTranslations: [{
                [SEAT_SYNTHETIC_VISION_WITHHELD]: 'The repaired cat dozes on the windowsill, tail beside the radiator.',
                [SEAT_HYPER_OPENROUTER_VISION_EDITOR]: 'A repaired cat naps on the sill, its tail beside the heater.',
                [SEAT_HYPER_VISION]: 'The repaired cat sleeps by the radiator.',
              },],
              needle: 'repaired',
              incumbentText: '',
              incumbentKind: 'absent',
            },);
            expect(result.text,).toContain('repaired',);
            // The follow-up is judged at its first ask, so what follows the marker
            // is that round's own record.
            expect(result.findings.slice(
              0,
              declinedRound.length + 1,
            ),).toEqual([
              ...declinedRound,
              followup,
            ],);
          },
        },),

        it({
          name: 'ACCEPTS a candidate backed on the second ask at a passage with NO '
            + 'existing translation, which is the door the other retry cases never '
            + 'open: a decline there leaves by an exception rather than a return, so '
            + 'a retry wired only to the returning door would refuse this passage '
            + 'while filling the identical one that happens to have an incumbent',
          fn: async () => {
            const { result, calls, } = await runLane({
              translations: {
                [SEAT_SYNTHETIC_VISION_WITHHELD]: 'The cat dozes on the windowsill, tail draped beside the radiator.',
                [SEAT_HYPER_OPENROUTER_VISION_EDITOR]: 'A cat naps on the sill, its tail hanging near the heater.',
                [SEAT_HYPER_VISION]: 'The cat sleeps on the ledge, tail beside the radiator.',
              },
              needle: '',
              needleAfterRetry: 'dozes',
              incumbentText: '',
              incumbentKind: 'absent',
            },);
            expect(result.decision,).toBe('judged',);
            expect(result.origin,).not
              .toBe('incumbent',);
            expect(result.text,).toContain('dozes',);
            // The first round's evidence survives the exception it left by, which is
            // the part a returning round gets for free and this one does not.
            expect(result.findings,).toContain('translate-declined-retried',);
            expect(calls.select,).toBe(JUDGE_WINDOW * 2,);
          },
        },),
      ],
    },),

    describe({
      name: `${runTranslateStage.name} window`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'renders NO surrounding block when a caller does not ask for one, in EVERY judge '
            + 'sheet, so every measurement taken before this parameter existed still describes the '
            + 'sheet production sends',
          fn: async () => {
            const { judgeSheets, } = await runLane({
              translations: {
                [SEAT_SYNTHETIC_VISION_WITHHELD]: 'The cat dozes on the windowsill, tail draped beside the radiator.',
                [SEAT_HYPER_OPENROUTER_VISION_EDITOR]: 'A cat naps on the sill, its tail hanging near the heater.',
                [SEAT_HYPER_VISION]: 'The cat sleeps on the ledge, tail beside the radiator.',
              },
              needle: 'dozes',
              incumbentText: INCUMBENT_TEXT,
            },);
            expect(judgeSheets.length,).toBeGreaterThan(0,);
            expect(judgeSheets
              .filter(function carriesLabel(sheet,) {
                return sheet.includes('SURROUNDING ORIGINAL',);
              },)
              .length,).toBe(0,);
          },
        },),
        it({
          name: 'renders the surrounding block AND its context-only caveat when a caller supplies '
            + 'one, which is what lets a flagged slice be judged twice differing in exactly one '
            + 'thing, as the window trial needs',
          fn: async () => {
            const { judgeSheets, } = await runLane({
              translations: {
                [SEAT_SYNTHETIC_VISION_WITHHELD]: 'The cat dozes on the windowsill, tail draped beside the radiator.',
                [SEAT_HYPER_OPENROUTER_VISION_EDITOR]: 'A cat naps on the sill, its tail hanging near the heater.',
                [SEAT_HYPER_VISION]: 'The cat sleeps on the ledge, tail beside the radiator.',
              },
              needle: 'dozes',
              incumbentText: INCUMBENT_TEXT,
              neighbouringSourceText: '她看着外面的鸟。\n',
            },);
            expect(judgeSheets
              .filter(function carriesContext(sheet,) {
                if (!sheet.includes('SURROUNDING ORIGINAL',))
                  return false;
                if (!sheet.includes('她看着外面的鸟。',))
                  return false;
                return sheet.includes('not expected to render this',);
              },)
              .length,).toBe(judgeSheets.length,);
          },
        },),

        it({
          name: 'FORWARDS THE VERSE RULE TO ITS JUDGES, not only to its translators. This stage has always '
            + 'handed the flag to `produceTranslateSlate` and, until 2026-08-22, to nothing else, so a slate '
            + 'written to unmerge a passage was decided by a panel told a shape the ORIGINAL lacks is no fault. '
            + 'The lane cases in `translate-judge.unit.test.ts` call the judging half directly and would pass '
            + 'whether or not this handover existed, which is why the pair is repeated here',
          fn: async () => {
            const { judgeSheets, } = await runLane({
              translations: {
                [SEAT_SYNTHETIC_VISION_WITHHELD]: 'The cat dozes on the windowsill, tail draped beside the radiator.',
                [SEAT_HYPER_OPENROUTER_VISION_EDITOR]: 'A cat naps on the sill, its tail hanging near the heater.',
                [SEAT_HYPER_VISION]: 'The cat sleeps on the ledge, tail beside the radiator.',
              },
              needle: 'dozes',
              incumbentText: 'The cat is doing a nap upon the windowsill.',
              lineStructured: true,
            },);

            expect(judgeSheets.length,).toBeGreaterThan(0,);
            expect(
              judgeSheets.every(function carriesCriterion(sheet,): boolean {
                return sheet.includes(TRANSLATE_LINE_STRUCTURE_CRITERION,);
              },),
            ).toBe(true,);
          },
        },),

        it({
          name: 'LEAVES IT OUT OF AN UNGOVERNED ROUND, which is the positive control the "FORWARDS THE VERSE '
            + 'RULE TO ITS JUDGES" case needs: '
            + 'a sheet carrying the criterion for every slice would satisfy that one exactly as well and would '
            + 'mean the flag was never read on the way through',
          fn: async () => {
            const { judgeSheets, } = await runLane({
              translations: {
                [SEAT_SYNTHETIC_VISION_WITHHELD]: 'The cat dozes on the windowsill, tail draped beside the radiator.',
                [SEAT_HYPER_OPENROUTER_VISION_EDITOR]: 'A cat naps on the sill, its tail hanging near the heater.',
                [SEAT_HYPER_VISION]: 'The cat sleeps on the ledge, tail beside the radiator.',
              },
              needle: 'dozes',
              incumbentText: 'The cat is doing a nap upon the windowsill.',
              lineStructured: false,
            },);

            expect(judgeSheets.length,).toBeGreaterThan(0,);
            expect(
              judgeSheets.some(function carriesCriterion(sheet,): boolean {
                return sheet.includes(TRANSLATE_LINE_STRUCTURE_CRITERION,);
              },),
            ).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
