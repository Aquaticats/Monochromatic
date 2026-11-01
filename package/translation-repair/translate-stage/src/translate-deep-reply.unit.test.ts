/**
 Tests that one translator's reply nested too deeply to read costs the stage
 that voice and nothing else.

 THE SHAPE UNDER TEST. A model reply is text from one voice. Before the nesting
 bound, a reply of thousands of nested quotation markers overflowed the stack
 inside a parser the repair turn reads the candidate through, and the
 `RangeError` that came out ended the whole translate stage, so the voices that
 wrote ordinary renderings lost their work with it. Each case here has one voice
 write such a reply and two write ordinary ones.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  type ChatJsonOutcome,
  type ChatJsonRequest,
  messageText,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import { runTranslateStage, } from '../dist/final/node/index.mjs';
import { capturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import {
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_HYPER_VISION,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { candidateCarrying, } from '@monochromatic-dev/translation-repair-test-fixture/ts/translate-ballot.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

//region Deep reply tests

/**
 Original slice every case renders.
 */
const SOURCE_TEXT = '猫猫在窗台上打盹，尾巴垂在暖气片旁边。';

/**
 Translation already in the archive.
 */
const INCUMBENT_TEXT = 'The cat is doing the sleeping on the windowsill, with tail hanging by the radiator.';

/**
 The voice that writes the deep reply.
 */
const DEEP_VOICE = SEAT_SYNTHETIC_VISION_WITHHELD;

/**
 What the floor tells the deep voice, and the stage records, about its reply.
 */
const DEEP_REFUSAL = 'Your translation could not be parsed as Markdown: MdxParseError: MDX body refused to parse '
  + 'because it is nested too deeply to read: its container markers pass the bound of 256 at line 1, column 257; '
  + 'corpus documents compile as MDX upstream, so failure signals corruption or an unsupported construct.';

/**
 Models that render the slice.
 */
const TRANSLATORS: readonly RosterModelId[] = [
  DEEP_VOICE,
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
 Client answering each translator from a script, the repair turn with a reply
 no wire guard accepts, and every judge with a ballot for one rendering.

 @param deepReply - what the deep voice writes, the first time and when asked again

 @param needle - text the judges vote for

 @returns Client honoring the script

 @example
 ```ts
 const client = deepVoiceClient({ deepReply: '>'.repeat(300,), needle: 'naps', },);
 ```
 */
function deepVoiceClient(
  {
    deepReply,
    needle,
  }: {
    readonly deepReply: string;
    readonly needle: string;
  },
): ModelClient {
  /**
   What each ordinary translator writes.
   */
  const ordinary: Readonly<Record<string, string>> = {
    [SEAT_HYPER_OPENROUTER_VISION_EDITOR]: 'A cat naps on the sill, its tail hanging near the heater.',
    [SEAT_HYPER_VISION]: 'The cat sleeps on the ledge, tail beside the radiator.',
  };
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
      if (schema === 'translation_repair_report')
        return {
          kind: 'schema-mismatch',
          rawText: 'no repair',
          detail: 'scripted to leave the repair turn unanswered',
        };
      if (schema === 'translation_report') {
        /**
         Wire reply this translator was scripted to write.
         */
        const value: unknown = { translation: (request.modelId === DEEP_VOICE) ? deepReply : ordinary[request.modelId], };
        if (!request.validate(value,))
          throw new Error('scripted reply failed the wire guard',);
        return {
          kind: 'ok',
          value,
          rawText: JSON.stringify(value,),
        };
      }
      /**
       Judge sheet as this judge received it.
       */
      const content = request.messages
        .map(function toContent(message,) {
          return messageText({ message, },);
        },)
        .join('\n',);
      /**
       Ballot naming the candidate carrying the needle.
       */
      const ballot: unknown = {
        best: candidateCarrying({
          content,
          needle,
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
 Runs the translate stage over the fixture slice with one deep voice.

 @param deepReply - what the deep voice writes

 @returns What the stage decided and every line it logged

 @example
 ```ts
 const { result, } = await runDeepVoiceStage({ deepReply: '>'.repeat(300,), },);
 ```
 */
async function runDeepVoiceStage(
  { deepReply, }: { readonly deepReply: string; },
) {
  /**
   Lines the stage logged.
   */
  const messages: string[] = [];
  /**
   What the stage decided for the slice.
   */
  const result = await runTranslateStage({
    client: deepVoiceClient({
      deepReply,
      needle: 'naps on the sill',
    },),
    translatorModelIds: TRANSLATORS,
    judgeModelIds: JUDGES,
    sourceText: SOURCE_TEXT,
    incumbentText: INCUMBENT_TEXT,
    incumbentKind: 'present',
    lineStructured: false,
    signal: new AbortController().signal,
    perCallTimeoutMs: HANG_STOP_MS,
    l: capturingLogger({ messages, },),
    // The case names the one self-vote its findings record, so every judge is asked: a window of quorum plus one judges, rotated
    // by the sheet text, may leave that judge out.
    fanOut: 'whole-bench',
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
  return {
    result,
    messages,
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
          name: 'FINISHES on the voices that wrote ordinary renderings when one voice wrote 16,000 nested quotation '
            + 'markers, and names that voice\'s refusal',
          fn: async () => {
            const { result, } = await runDeepVoiceStage({ deepReply: `${'>'.repeat(16_000,)} cat`, },);

            expect(result.origin,).toBe('fresh',);
            expect(result.text,).toBe('A cat naps on the sill, its tail hanging near the heater.',);
            expect(result.decision,).toBe('judged',);
            expect(result.findings,).toEqual([
              `translate-invalid (${DEEP_VOICE}): ${DEEP_REFUSAL}`,
              `translate-repair-unheard (${DEEP_VOICE})`,
              `translate-candidate-refused (${DEEP_VOICE}): ${DEEP_REFUSAL}`,
              'translate-candidates (3/3 heard, 3 distinct, 0 collapsed)',
              `select-self-vote (${SEAT_HYPER_OPENROUTER_VISION_EDITOR})`,
            ],);
          },
        },),
        it({
          name: 'FINISHES the same way on a reply one marker past the bound, which no stack decided',
          fn: async () => {
            const { result, } = await runDeepVoiceStage({ deepReply: `${'>'.repeat(257,)} cat`, },);

            expect(result.origin,).toBe('fresh',);
            expect(result.text,).toBe('A cat naps on the sill, its tail hanging near the heater.',);
          },
        },),
      ],
    },),
  ],
},);

//endregion Deep reply tests
