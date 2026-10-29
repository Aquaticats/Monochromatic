/**
 Tests that the editors and both editor selections are told that saying
 something twice is adding content, unless the ORIGINAL says it again at that
 place (ledger L14(b)).

 WHY. The audit found editors writing neighbouring text into a region and the
 chunk judges choosing it. Over every artifact, 56 of 6,222 chunk candidates
 carried a sentence more often than the slice's archive English did, the
 judges selected such a candidate 3 times, and Carena0442 slice 14 on a build
 of 2026-09-02 shipped two such sentences through a repair-lane win. The
 editor sheet forbade copying the NEARBY blocks but said nothing about the
 translation outside a region, and the selections' faithfulness test named
 added content without saying a repeat is some.

 CONDITIONED ON THE ORIGINAL, because the editor sheet also says to translate
 ALL of an omission: a refrain the archive rendered once is an omission, and a
 rule against repetition as such would forbid filling it.

 The editor sheet shows the whole TRANSLATION and the whole ORIGINAL, so the
 rule asks for nothing the editor cannot check (rendered 2026-09-28).

 Fixtures are cat-themed invention.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure';
import {
  type ChatJsonOutcome,
  type ChatJsonRequest,
  messageText,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import {
  buildEditorMessages,
  CHUNK_SELECTION_CRITERIA,
  ENVELOPE_SELECTION_CRITERIA,
  selectChunkPatch,
} from '../dist/final/node/index.mjs';
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
const l = tagged({ tag: 'repeated-content-rule-test', },);

/**
 The editors' rule, as the sheet states it.
 */
const EDITOR_RULE = 'goes into it only when the ORIGINAL says it again at that place';

/**
 The selections' faithfulness line, as both criteria state it.
 */
const JUDGE_RULE = 'Saying something twice is adding content unless the ORIGINAL says it again at that place';

/**
 Invented archive English with one region.
 */
const TARGET_TEXT = 'Mittens woke at dawn.\n\nThe cat hates the sun.\n';

/**
 System sheets the chunk judges are shown.

 @returns Every judge's system sheet

 @example
 ```ts
 const sheets = await chunkJudgeSheets();
 ```
 */
async function chunkJudgeSheets(): Promise<readonly string[]> {
  /**
   Judges' sheets, in order.
   */
  const sheets: string[] = [];
  /**
   Judges answering with a decline, which ends the round without a winner.
   */
  const client: ModelClient = {
    chatText: async () => {
      throw new Error('chatText unused by the selection',);
    },
    chatJson: async <ValueT,>(request: ChatJsonRequest<ValueT>,): Promise<ChatJsonOutcome<ValueT>> => {
      /**
       System sheet this judge was shown.
       */
      const system = request.messages.at(0,);
      sheets.push((system === undefined) ? '' : messageText({ message: system, },),);
      /**
       A decline.
       */
      const scripted: unknown = { best: 0, assessment: 'scripted', };
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
  await selectChunkPatch({
    client,
    candidates: [
      chunkCandidate({ patchedText: 'Mittens woke at dawn.\n\nThe cat loves the sun.\n', },),
      chunkCandidate({ patchedText: 'Mittens woke at dawn.\n\nThe cat adores the sun.\n', },),
    ],
    judgeModelIds: [
      SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
      SEAT_SYNTHETIC_VISION_WITHHELD,
      SEAT_SYNTHETIC_TEXT_EVERYWHERE,
      SEAT_HYPER_OPENROUTER_UNMEASURED,
    ],
    sourceText: '咪咪天亮就醒了。\n\n猫猫喜欢太阳。\n',
    indecisionFallback: chunkCandidate({ patchedText: 'Mittens woke at dawn.\n\nThe cat likes the sun.\n', },),
    rejectionFallback: {
      patchedText: TARGET_TEXT,
      applied: [],
      rejected: [],
    },
    signal: new AbortController().signal,
    perCallTimeoutMs: HANG_STOP_MS,
    l,
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
  return sheets;
}

await describe({
  name: 'saying something twice is adding content unless the ORIGINAL says it again (ledger L14(b))',
  children: [
    it({
      name: 'the editor sheet STATES the rule',
      fn: async () => {
        /**
         Region over the second paragraph.
         */
        const startOffset = TARGET_TEXT.indexOf('The cat',);
        /**
         Every message of the editor sheet, joined.
         */
        const sheet = buildEditorMessages({
          sourceText: '咪咪天亮就醒了。\n\n猫猫喜欢太阳。\n',
          targetText: TARGET_TEXT,
          envelopes: [
            {
              envelopeId: 'envelope/sun',
              startOffset,
              endOffset: TARGET_TEXT.length - 1,
              baseText: TARGET_TEXT.slice(startOffset, -1,),
              baseHash: hashContent({ content: TARGET_TEXT.slice(startOffset, -1,), },),
              issueIds: [],
            },
          ],
          issues: [],
        },).messages
          .map(function textOf(message,) {
            return messageText({ message, },);
          },)
          .join('\n',);
        expect(sheet.includes(EDITOR_RULE,),).toBe(true,);
      },
    },),
    it({
      name: 'both selections\' faithfulness tests STATE the rule',
      fn: async () => {
        expect({
          envelope: ENVELOPE_SELECTION_CRITERIA.some(function states(criterion,) {
            return criterion.includes(JUDGE_RULE,);
          },),
          chunk: CHUNK_SELECTION_CRITERIA.some(function states(criterion,) {
            return criterion.includes(JUDGE_RULE,);
          },),
        },).toEqual({
          envelope: true,
          chunk: true,
        },);
      },
    },),
    it({
      name: 'the chunk judges\' rendered sheet CARRIES the rule',
      fn: async () => {
        /**
         Every chunk judge's sheet.
         */
        const sheets = await chunkJudgeSheets();
        expect({
          asked: sheets.length > 0,
          stated: sheets.every(function states(sheet,) {
            return sheet.includes(JUDGE_RULE,);
          },),
        },).toEqual({
          asked: true,
          stated: true,
        },);
      },
    },),
  ],
},);
