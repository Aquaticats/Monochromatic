/**
 Tests for one editor roster's arm over one slice.

 `runArm` is the editor stage plus one reduction, and the reduction is where
 the comparison's two conventions live: an arm that shipped the translation
 untouched reads as blank, and the producer is read off the stage's own answer
 rather than off a slate index. Both are checked here on a scripted client.

 Fixtures are cat-themed invention, the same two-sentence chunk the editor
 stage's composite case uses.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import type {
  AdjudicatedIssue,
  EditableEnvelope,
} from '@monochromatic-dev/translation-repair-shared-contract';
import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure';
import {
  runArm,
  type WidthProbeInput,
} from '../../dist/final/node/index.mjs';
import {
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { replyWith, } from '@monochromatic-dev/translation-repair-provider/ts/scripted-reply-outcome.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Logger for the arms under test.
 */
const l = tagged({ tag: 'editor-width-arm-test', },);

/**
 First sentence, whose only defect is one mistranslated word.
 */
const SENTENCE_ONE = 'The cat is doing the sleeping on the windowsill.';

/**
 Second sentence, with a different single-word defect.
 */
const SENTENCE_TWO = 'The dog is doing the barking in the yard.';

/**
 Translation the arms repair.
 */
const TARGET_TEXT = `${SENTENCE_ONE} ${SENTENCE_TWO}`;

/**
 Original the edits answer to.
 */
const SOURCE_TEXT = '猫猫在窗台上睡觉。狗狗在院子里叫。';

/**
 One envelope per sentence, in document order.
 */
const ENVELOPES: readonly EditableEnvelope[] = [
  {
    envelopeId: 'envelope/cat',
    startOffset: 0,
    endOffset: SENTENCE_ONE.length,
    baseText: SENTENCE_ONE,
    baseHash: hashContent({ content: SENTENCE_ONE, },),
    issueIds: ['adjudicated/cat-tense',],
  },
  {
    envelopeId: 'envelope/dog',
    startOffset: SENTENCE_ONE.length + 1,
    endOffset: SENTENCE_ONE.length + 1 + SENTENCE_TWO.length,
    baseText: SENTENCE_TWO,
    baseHash: hashContent({ content: SENTENCE_TWO, },),
    issueIds: ['adjudicated/dog-tense',],
  },
];

/**
 Accepted issues, one per envelope, neither quoting anything.
 */
const ISSUES: readonly AdjudicatedIssue[] = [
  {
    issueId: 'adjudicated/cat-tense',
    status: 'accepted' as const,
    severity: 'major' as const,
    claims: [],
    tallies: {},
  },
  {
    issueId: 'adjudicated/dog-tense',
    status: 'accepted' as const,
    severity: 'major' as const,
    claims: [],
    tallies: {},
  },
];

/**
 Slice the arms run over.
 */
const INPUT: WidthProbeInput = {
  entryId: 'whiskers',
  sliceIndex: 0,
  sourceText: SOURCE_TEXT,
  targetText: TARGET_TEXT,
  issues: ISSUES,
  envelopes: ENVELOPES,
  findings: [],
};

/**
 Editors seated on the arm.
 */
const EDITORS = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_WITHHELD,
] as const;

/**
 Panel with no stake in either editor's output.
 */
const JUDGES = [
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
] as const;

/**
 Client answering each seat by model id.

 @param answers - reply per model id; a model with no entry is a fixture bug

 @returns Client honoring that script

 @example
 ```ts
 const client = scriptedClient({ answers: { [EDITORS[0]]: { edits: [], }, }, },);
 ```
 */
function scriptedClient(
  { answers, }: { readonly answers: Readonly<Record<string, unknown>>; },
): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText unused',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      if (!Object.hasOwn(answers, request.modelId,))
        throw new Error(`no scripted answer for ${request.modelId}`,);
      return replyWith({
        report: answers[request.modelId],
        request,
      },);
    },
    quotas: async () => {
      throw new Error('quotas unused',);
    },
  };
}

await describe({
  name: runArm.name,
  children: [
    it({
      name: 'reads an arm whose editors proposed nothing as BLANK with no producer, so two such arms compare '
        + 'as nothing shipped rather than as agreement about a repair',
      fn: async () => {
        const arm = await runArm({
          client: scriptedClient({
            answers: {
              [EDITORS[0]]: { edits: [], },
              [EDITORS[1]]: { edits: [], },
            },
          },),
          input: INPUT,
          editorModelIds: EDITORS,
          judgeModelIds: JUDGES,
          signal: new AbortController().signal,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);

        expect(arm.text,).toBe('',);
        expect(arm.producers,).toEqual([],);
        expect(arm.heard,).toBe(EDITORS.length,);
        expect(arm.patch.patchedText,).toBe(TARGET_TEXT,);
      },
    },),

    it({
      name: 'ships the composite of two editors\' per-envelope winners as the arm\'s text and names both as '
        + 'its producers, read off the stage\'s own answer',
      fn: async () => {
        const arm = await runArm({
          client: scriptedClient({
            answers: {
              [EDITORS[0]]: {
                edits: [
                  {
                    region: 1,
                    newText: 'The cat is doing the napping on the windowsill.',
                  },
                ],
              },
              [EDITORS[1]]: {
                edits: [
                  {
                    region: 2,
                    newText: 'The dog is doing the howling in the yard.',
                  },
                ],
              },
              [JUDGES[0]]: {
                best: 3,
                assessment: 'the composite is the only whole-chunk candidate that repairs both sentences',
              },
              [JUDGES[1]]: {
                best: 3,
                assessment: 'the composite is the only whole-chunk candidate that repairs both sentences',
              },
            },
          },),
          input: INPUT,
          editorModelIds: EDITORS,
          judgeModelIds: JUDGES,
          signal: new AbortController().signal,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);

        expect(arm.text,).toBe(
          'The cat is doing the napping on the windowsill. The dog is doing the howling in the yard.',
        );
        expect(arm.heard,).toBe(EDITORS.length,);
        expect([...arm.producers,].toSorted(),).toEqual([...EDITORS,].toSorted(),);
      },
    },),
  ],
},);
