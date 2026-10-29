/**
 Guards ledger S13 and H6: the absolute naturalness review reads the house
 rules, and on prose it reads each paragraph as it renders.

 Its findings are required corrections, yet the sheet carried no house rule:
 it told reviewers to preserve "deliberate source-language kinship terms",
 against the house rule translating a term with an everyday English
 equivalent, and a reviewer measuring against no reader-protection rule could
 file a protected vagueness as a defect to fix. It was also shown the polish
 wrapped at its semantic boundaries, the layout the polish gate stopped
 showing its judges after they weighed line breaks (classes one hundred
 fifty-two and fifty-three).

 Fixtures are cat-themed invention.

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
import { buildAbsoluteNaturalnessReviewMessages, } from '@monochromatic-dev/translation-repair-archive';
import { polishConsolidation, } from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Invented-size roster for every polish role.
 */
const ROSTER = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
] as const;

/**
 Literal base the polish replaces.
 */
const BASE = 'She viewed rainy days proactively and spent many a cozy afternoon with the other cats, while doing her best to stay curious and close to the cats around her.';

/**
 The refiner's rewrite, one line; the round wraps it before its gate.
 */
const POLISHED = 'She kept a cheerful outlook on rainy days and spent many cozy afternoons with the other cats, doing her best to stay curious and close to those around her.';

/**
 System half of a review sheet over a one-line candidate.
 */
const reviewSystem = buildAbsoluteNaturalnessReviewMessages({
  subject: {
    sourceText: '猫睡了。',
    candidateText: 'The cat slept.',
    paragraphs: ['The cat slept.',],
    lineStructured: false,
  },
},).at(0,)?.content ?? '';

/**
 Client proposing the rewrite, recording each review sheet's user half.

 @param reviewSheets - where each review sheet is recorded

 @returns Scripted client
 */
function recordingClient({ reviewSheets, }: { readonly reviewSheets: string[]; },): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText unused by structured polish stages',);
    },
    chatJson: async <ValueT,>(request: ChatJsonRequest<ValueT>,): Promise<ChatJsonOutcome<ValueT>> => {
      /**
       Stage the request belongs to, by its reply schema.
       */
      const schema = request.responseFormat?.json_schema.name ?? '';
      /**
       User half of the sheet, as text.
       */
      const content = request.messages.at(1,)?.content ?? '';
      if (schema === 'absolute_naturalness_review')
        reviewSheets.push(((typeof content) === 'string') ? content : JSON.stringify(content,),);
      /**
       Reply for the stage.
       */
      const value: unknown = (schema === 'refine_report')
        ? { rewrites: [{ paragraph: 1, newText: POLISHED, },], }
        : (schema === 'candidate_ballot')
          ? { best: 1, assessment: 'more idiomatic', }
          : (schema === 'consolidation_polish_gate')
            ? { verdict: 'polished', unsupported: [], dropped: [], assessment: 'equally faithful', }
            : (schema === 'absolute_naturalness_review')
              ? { isAcceptable: true, findings: [], assessment: 'publication-ready', }
              : {};
      if (!request.validate(value,))
        throw new Error(`synthetic ${schema} reply failed validation`,);
      return {
        kind: 'ok',
        value,
        rawText: JSON.stringify(value,),
      };
    },
    quotas: async () => {
      throw new Error('quotas unused by polish stages',);
    },
  };
}

await describe({
  name: 'the naturalness review sheet (ledger S13, H6)',
  children: [
    it({
      name: 'READS the house rules for measuring sheets, whose findings are required fixes',
      fn: async () => {
        expect(reviewSystem,).toContain('House rules this corpus is written under',);
        expect(reviewSystem,).toContain('WHERE A RULE YOU HAVE BEEN GIVEN AND A HOUSE RULE DISAGREE',);
      },
    },),
    it({
      name: 'NEVER TELLS the reviewer to preserve source-language kinship terms the house rules translate',
      fn: async () => {
        expect(reviewSystem.includes('source-language kinship terms',),).toBe(false,);
      },
    },),
    it({
      name: 'COUNTS a departure from a house rule of form as a material defect',
      fn: async () => {
        expect(reviewSystem,).toContain('A departure from a house rule of form',);
      },
    },),
    it({
      name: 'SHOWS a prose polish to the reviewer one line a paragraph, as it renders',
      fn: async () => {
        /**
         Every review sheet the polish sent.
         */
        const reviewSheets: string[] = [];
        await polishConsolidation({
          client: recordingClient({ reviewSheets, },),
          sourceText: '她以乐观的态度看待雨天，和其他猫一起度过了许多惬意的下午。',
          archiveText: BASE,
          baseText: BASE,
          lineStructured: false,
          sliceIndex: 0,
          config: {
            refinerModelIds: [ROSTER[0],],
            judgeModelIds: ROSTER,
            gateModelIds: ROSTER,
            declaredNames: [],
            definitions: '',
          },
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          perCallTimeoutMs: HANG_STOP_MS,
          l: tagged({ tag: 'naturalness-review-sheet-test', },),
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(reviewSheets.length,).toBeGreaterThan(0,);
        expect(reviewSheets.every(function showsFolded(sheet,): boolean {
          return sheet.includes(`\n${POLISHED}\n`,);
        },),).toBe(true,);
      },
    },),
  ],
},);
