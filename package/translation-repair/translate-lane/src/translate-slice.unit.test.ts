/**
 Tests for the hop between one settled slice and the stage that settles it.

 WHAT THIS EXISTS TO CATCH, and why a test rather than a check at run time.
 The window trial compares one slice judged twice, differing only in whether the judges
 were shown the neighbouring original. `translate-stage.unit.test.ts` proves
 the stage renders that window into its sheets, and
 `translate-slice-key.unit.test.ts` proves the cache separates the two arms.
 Neither proves that {@link settleTranslateSlice} passes the window ON. A
 parameter that silently went nowhere would produce two identical arms, and
 the measurement would report a confident null after fifteen hundred calls.
 That is the failure this file makes impossible.

 IT ALSO PINS WHO SEES IT. The window is context for the judges; the
 translators are not shown it and must not be, since a translator that read
 the neighbouring original might render it and earn a coverage complaint for
 content that was never its slice.

 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import type { PreparedDocumentPair, } from '@monochromatic-dev/translation-repair-corpus-source';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading';
import {
  type ChatJsonOutcome,
  type ChatJsonRequest,
  messageText,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import { settleTranslateSlice, } from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_VISION,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Logger the lane writes its progress to.
 */
const l = tagged({ tag: 'translate-slice-test', },);

/**
 Schema name the translate stage asks translators for, which is how a request
 is told apart from a judge's ballot without reading its prose.
 */
const TRANSLATE_SCHEMA = 'translation_report';

/**
 Label the sheet gives the neighbouring sections.
 */
const SURROUNDING_LABEL = 'SURROUNDING ORIGINAL';

/**
 Neighbouring original the wide arm supplies.

 DISTINCTIVE ON PURPOSE, and present in no other fixture here, so a request
 carrying it can only have been given it through the parameter under test.
 */
const WINDOW_SENTINEL = '傍晚她回到炉火旁，炉子里的火已经快灭了。';

/**
 Original this slice renders.
 */
const SOURCE_TEXT = '猫猫在窗台上打盹，尾巴垂在暖气片旁边。';

/**
 Translation already in the archive for it.
 */
const INCUMBENT_TEXT = 'The cat is doing the sleeping on the windowsill, with tail hanging by the radiator.';

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
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
];

/**
 Slice pair the lane settles.
 */
const SLICE: ChunkPair = {
  source: {
    sliceIndex: 1,
    nodes: [],
    startOffset: 0,
    endOffset: SOURCE_TEXT.length,
    text: SOURCE_TEXT,
  },
  target: {
    sliceIndex: 1,
    nodes: [],
    startOffset: 0,
    endOffset: INCUMBENT_TEXT.length,
    text: INCUMBENT_TEXT,
  },
};

/**
 Preparation the slice belongs to.

 Minimal on purpose: what the lane reads from it is the identity context and
 the line-structure set, and this file is about neither.
 */
const PREPARED: PreparedDocumentPair = {
  sourceText: SOURCE_TEXT,
  targetText: INCUMBENT_TEXT,
  slices: [SLICE,],
  lineStructuredSliceIndices: new Set<number>(),
  declaredNames: [],
  alignmentFindings: [],
  unclaimedTargetBlocks: [],
  alignmentPairCount: 1,
};

/**
 Every request one run sent, split by the role that received it.
 */
type SentRequests = {
  /**
   Sheets the translators were sent.
   */
  readonly translator: string[];

  /**
   Sheets the judges were sent.
   */
  readonly judge: string[];

  /**
   Client recording them and answering both roles.
   */
  readonly client: ModelClient;
};

/**
 Builds a client that records what each role was sent and keeps the incumbent.

 The verdict is fixed because this file is about what the roles are SHOWN.
 Every judge abstains, which settles on the incumbent and reaches a record
 without any case having to script a winner.

 @returns Recorders and the client writing into them

 @example
 ```ts
 const recorder = recordingClient();
 ```
 */
function recordingClient(): SentRequests {
  /**
   Sheets the translators were sent.
   */
  const translator: string[] = [];

  /**
   Sheets the judges were sent.
   */
  const judge: string[] = [];

  return {
    translator,
    judge,
    client: {
      chatText: async () => {
        throw new Error('chatText unused by the translate lane',);
      },
      quotas: async () => {
        throw new Error('quotas unused by the translate lane',);
      },
      chatJson: async <ValueT,>(
        request: ChatJsonRequest<ValueT>,
      ): Promise<ChatJsonOutcome<ValueT>> => {
        /**
         Whole sheet this role received, in call order.
         */
        const content = request.messages
          .map(function toContent(message,) {
            return messageText({ message, },);
          },)
          .join('\n',);

        /**
         Schema the caller asked for, which names the role.
         */
        const schema = request.responseFormat
          ?.json_schema
          .name;
        if (schema === TRANSLATE_SCHEMA) {
          translator.push(content,);

          /**
           Rendering this translator returns.
           */
          const value: unknown = { translation: 'The cat dozes on the windowsill, tail beside the radiator.', };
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

        judge.push(content,);

        /**
         Ballot declining every candidate, which leaves the incumbent standing.
         */
        const ballot: unknown = {
          best: 0,
          assessment: 'fixture',
        };
        if (!request.validate(ballot,)) {
          return {
            kind: 'schema-mismatch',
            rawText: JSON.stringify(ballot,),
            detail: 'reply failed the wire guard',
          };
        }
        return {
          kind: 'ok',
          value: ballot as ValueT,
          rawText: JSON.stringify(ballot,),
        };
      },
    },
  };
}

/**
 Settles the slice once and hands back what each role was sent.

 @param neighbouringSourceText - wider window, absent for the narrow arm

 @returns Sheets both roles received

 @example
 ```ts
 const arm = await settleWith({},);
 ```
 */
async function settleWith(
  { neighbouringSourceText, }: { readonly neighbouringSourceText?: string; },
): Promise<{
  readonly translator: readonly string[];
  readonly judge: readonly string[];
}> {
  /**
   Recorder capturing this arm.
   */
  const recorder = recordingClient();

  await settleTranslateSlice({
    client: recorder.client,
    slice: SLICE,
    prepared: PREPARED,
    models: {
      translatorModelIds: TRANSLATORS,
      judgeModelIds: JUDGES,
    },
    ...((neighbouringSourceText === undefined)
      ? {}
      : { neighbouringSourceText, }),
    signal: AbortSignal.timeout(HANG_STOP_MS,),
    perCallTimeoutMs: HANG_STOP_MS,
    l,
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);

  return {
    translator: recorder.translator,
    judge: recorder.judge,
  };
}

await describe({
  name: settleTranslateSlice.name,
  children: [
    it({
      name: 'POSITIVE CONTROL: a narrow settlement reaches both roles, so an assertion that some '
        + 'sheet lacks the window is reading sheets that were actually sent rather than an empty '
        + 'recording from a run that failed before it called anyone',
      fn: async () => {
        const arm = await settleWith({},);
        expect(arm.translator
          .length,).toBeGreaterThan(0,);
        expect(arm.judge
          .length,).toBeGreaterThan(0,);
      },
    },),
    it({
      name: 'FORWARDS the window to the judges, which is the hop nothing else tests: the stage '
        + 'renders it and the cache key separates it, but a parameter that stopped here would '
        + 'make both arms of the window trial identical and report a null after fifteen hundred calls',
      fn: async () => {
        const arm = await settleWith({ neighbouringSourceText: WINDOW_SENTINEL, },);
        expect(arm.judge
          .filter(function carriesWindow(sheet,) {
            return sheet.includes(WINDOW_SENTINEL,);
          },)
          .length,).toBe(arm.judge
            .length,);
      },
    },),
    it({
      name: 'sends the window to NO translator, since a translator shown the neighbouring original '
        + 'might render it and be marked down for covering content that was never its slice',
      fn: async () => {
        const arm = await settleWith({ neighbouringSourceText: WINDOW_SENTINEL, },);
        expect(arm.translator
          .filter(function carriesWindow(sheet,) {
            return sheet.includes(WINDOW_SENTINEL,);
          },)
          .length,).toBe(0,);
      },
    },),
    it({
      name: 'reaches EVERY role without the window when no caller supplies one, so every '
        + 'measurement taken before this parameter existed still describes what production sends',
      fn: async () => {
        const arm = await settleWith({},);
        expect([...arm.translator,
          ...arm.judge,]
          .filter(function carriesLabel(sheet,) {
            return sheet.includes(SURROUNDING_LABEL,);
          },)
          .length,).toBe(0,);
      },
    },),
  ],
},);
