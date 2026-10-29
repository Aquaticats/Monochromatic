/**
 Guards the lead's adoption of 2026-10-29 (open to the owner's veto): the follow-up round of a passage the judges
 declined does not ask the same judges the same slate again. XingZ6022 slices 107 and 114 asked two judges, who split
 one ballot each, the same two renderings four times (the first ask, the challenge, and the same two again in the
 follow-up), then shipped the source text under a note. The outcome rules stay (owner, 2026-10-23, "Source text +
 note"; 2026-10-25, "Two judges must name it"); only the repeat of an identical question is dropped. A follow-up whose
 renderings differ is a fresh slate and is judged as before.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type ChatJsonOutcome,
  type ChatJsonRequest,
  messageText,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import { TranslateAbsenceError, } from '@monochromatic-dev/translation-repair-translate-floor';
import { runTranslateStage, } from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { candidateCarrying, } from '@monochromatic-dev/translation-repair-test-fixture/ts/translate-ballot.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Logger for the stage under test.
 */
const l = tagged({ tag: 'translate-followup-repeated-slate-test', },);

/**
 Invented original: the cat dozed on the sill, tail by the radiator.
 */
const SOURCE_TEXT = '猫猫在窗台上打盹，尾巴垂在暖气片旁边。';

/**
 The two models that render the slice.
 */
const TRANSLATORS: readonly RosterModelId[] = [
  SEAT_SYNTHETIC_VISION_WITHHELD,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
];

/**
 The two judges, neither of whom rendered anything, so every ballot weighs the same.
 */
const JUDGES: readonly RosterModelId[] = [
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
];

/**
 What the translators render, in the first round and, in the cases saying so, in the follow-up.
 */
const FIRST_SLATE: readonly string[] = [
  'The cat dozes on the windowsill, tail draped beside the radiator.',
  'A cat naps on the sill, its tail hanging near the heater.',
];

/**
 What the translators render in a follow-up that is a fresh slate.
 */
const FRESH_SLATE: readonly string[] = [
  'The cat dozes by the window, its tail resting on the radiator.',
  'A cat naps on the ledge, tail beside the heater.',
];

/**
 What the judges voted for: the first backs the rendering carrying "dozes", the second the one carrying "naps".
 */
const NEEDLES: readonly string[] = [
  'dozes',
  'naps',
];

/**
 Calls the client served.
 */
type CallLog = {
  /**
   Translator calls.
   */
  translate: number;

  /**
   Judge ballots asked, each judge's call counted.
   */
  ballots: number;
};

/**
 Client under which the two judges split one ballot each across the two renderings.

 @param calls - where the calls are counted

 @param followup - what the translators render once shown a rejection

 @returns Client over both stages

 @example
 ```ts
 const client = splitClient({ calls, followup: FIRST_SLATE, },);
 ```
 */
function splitClient(
  {
    calls,
    followup,
  }: {
    readonly calls: CallLog;
    readonly followup: readonly string[];
  },
): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText unused by these stages',);
    },
    quotas: async () => {
      throw new Error('quotas unused by these stages',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      /**
       Stage the request belongs to, by its reply schema.
       */
      const schema = request.responseFormat?.json_schema.name ?? '';
      /**
       Sheet as the voice reads it.
       */
      const sheet = request.messages
        .map(function textOf(message,): string {
          return messageText({ message, },);
        },)
        .join('\n',);
      /**
       Reply for the stage.
       */
      const value: unknown = (function replyFor(): unknown {
        if (schema === 'translation_report') {
          calls.translate += 1;
          /**
           Rendering this translator is scripted to give, by its place in the roster.
           */
          const slate = sheet.includes('LATEST REJECTION',) ? followup : FIRST_SLATE;
          return { translation: slate[TRANSLATORS.indexOf(request.modelId as RosterModelId,)] ?? '', };
        }
        calls.ballots += 1;
        return {
          best: candidateCarrying({
            content: sheet,
            needle: NEEDLES[JUDGES.indexOf(request.modelId as RosterModelId,)] ?? '',
          },),
          assessment: 'scripted',
        };
      })();
      if (!request.validate(value,))
        throw new Error(`fixture ${schema} reply failed validation`,);
      return {
        kind: 'ok',
        value: value as ValueT,
        rawText: JSON.stringify(value,),
      };
    },
  };
}

/**
 Runs the stage over a passage the archive never translated and records what it asked.

 @param followup - what the translators render in the follow-up round

 @param incumbentText - translation as it stands, blank for an absent passage

 @param incumbentKind - whether the archive holds any wording

 @returns What the stage settled or raised, and the calls it made

 @example
 ```ts
 const { raised, calls, } = await runOver({ followup: FIRST_SLATE, incumbentText: '', incumbentKind: 'absent', },);
 ```
 */
async function runOver(
  {
    followup,
    incumbentText,
    incumbentKind,
  }: {
    readonly followup: readonly string[];
    readonly incumbentText: string;
    readonly incumbentKind: 'absent' | 'present';
  },
): Promise<{
  readonly raised?: unknown;
  readonly calls: CallLog;
}> {
  /**
   Calls made.
   */
  const calls: CallLog = {
    translate: 0,
    ballots: 0,
  };
  try {
    await runTranslateStage({
      client: splitClient({
        calls,
        followup,
      },),
      translatorModelIds: TRANSLATORS,
      judgeModelIds: JUDGES,
      sourceText: SOURCE_TEXT,
      incumbentText,
      incumbentKind,
      lineStructured: false,
      signal: AbortSignal.timeout(HANG_STOP_MS,),
      perCallTimeoutMs: HANG_STOP_MS,
      l,
      stageSettings: SCRIPTED_STAGE_SETTINGS,
    },);
    return { calls, };
  }
  catch (error) {
    return {
      raised: error,
      calls,
    };
  }
}

await describe({
  name: '',
  children: [
    describe({
      name: 'a follow-up that repeats the slate the same two judges split on (lead\'s adoption, 2026-10-29)',
      children: [
        it({
          name: 'DOES NOT ASK THE SAME JUDGES THE SAME SLATE AGAIN: the first ask and its challenge are the only '
            + 'rounds the repeated slate is judged in, the writers are still asked for the follow-up, and the '
            + 'absence is settled with the repeat named',
          fn: async () => {
            const { raised, calls, } = await runOver({
              followup: FIRST_SLATE,
              incumbentText: '',
              incumbentKind: 'absent',
            },);
            expect(raised instanceof TranslateAbsenceError,).toBe(true,);
            expect({
              translate: calls.translate,
              ballots: calls.ballots,
              named: (raised instanceof TranslateAbsenceError)
                && raised.findings.some(function namesIt(finding,): boolean {
                  return finding.startsWith('translate-followup-slate-repeated',);
                },),
            },).toEqual({
              translate: TRANSLATORS.length * 2,
              ballots: JUDGES.length * 2,
              named: true,
            },);
          },
        },),
        it({
          name: 'JUDGES A FRESH FOLLOW-UP SLATE as before: the first ask, the challenge, and both again',
          fn: async () => {
            const { raised, calls, } = await runOver({
              followup: FRESH_SLATE,
              incumbentText: '',
              incumbentKind: 'absent',
            },);
            expect(raised instanceof TranslateAbsenceError,).toBe(true,);
            expect({
              translate: calls.translate,
              ballots: calls.ballots,
            },).toEqual({
              translate: TRANSLATORS.length * 2,
              ballots: JUDGES.length * 4,
            },);
          },
        },),
        it({
          name: 'ASKS AGAIN OVER WORDING THAT CANNOT SHIP, where the follow-up round ships a candidate two judges '
            + 'named or keeps the archive past a decline, so a repeated slate there is still a question with an '
            + 'answer of its own (owner, 2026-09-27 and 2026-10-25)',
          fn: async () => {
            const { calls, } = await runOver({
              followup: FIRST_SLATE,
              // The original left in Han, which the untranslated floor refuses.
              incumbentText: SOURCE_TEXT,
              incumbentKind: 'present',
            },);
            expect(calls.ballots,).toBe(JUDGES.length * 4,);
          },
        },),
      ],
    },),
  ],
},);
