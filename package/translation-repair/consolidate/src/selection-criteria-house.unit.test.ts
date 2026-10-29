/**
 Guards the criteria half of ledger S14: the editor and refine selections
 rank candidates by criteria that agree with the house rules, and every sheet
 naming the house rules of form names the same list.

 The editor's per-envelope criterion "Fits the surrounding text in register and
 tense" made the surrounding English the tense authority, so a repair moving
 a life into the past lost to one matching a present-tense neighbour. The
 refine criteria "Says exactly what the CURRENT text says" and "Reads more
 naturally than the CURRENT text by a clear margin" refused a candidate whose
 only change was a house correction. Neither editor criterion named the page
 apparatus or the declared-name exemption the translate slate reads. And the
 polish gate, the refiner and the naturalness review each listed the house
 rules of form in their own words.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure';
import {
  buildRefineMessages,
  buildRefineSelectionContext,
  CHUNK_SELECTION_CRITERIA,
  ENVELOPE_SELECTION_CRITERIA,
  type RefineStageMode,
} from '@monochromatic-dev/translation-repair-repair';
import { buildAbsoluteNaturalnessReviewMessages, } from '@monochromatic-dev/translation-repair-archive';
import { buildConsolidationPolishGateMessages, } from '../dist/final/node/index.mjs';

/**
 A phrase of the shared house-form list every sheet must carry verbatim.
 */
const FORM_LIST = 'singular they for a TA, English for a word left in Han, chat shorthand spelled out';

/**
 What a house correction is, on a sheet that asks whether meaning changed.
 */
const KEEPS_MEANING = 'is not a change to what the text says';

/**
 One of each refine mode.
 */
const refineModes: readonly RefineStageMode[] = [
  { kind: 'comparative', },
  {
    kind: 'objection-correction',
    groups: [{ origin: 'consolidation gate', objections: ['The base drops the tabby\'s name.',], },],
  },
];

/**
 Criteria of each refine mode, joined.
 */
const refineCriteria = refineModes.map(function criteriaOf(mode,): string {
  return buildRefineSelectionContext({
    mode,
    sourceText: '猫睡了。',
    repairedText: 'The cat sleeps.',
  },).criteria.join('\n',);
},);

await describe({
  name: 'selection criteria agree with the house rules (ledger S14)',
  children: [
    it({
      name: 'NEVER MAKES the surrounding English the tense authority for an envelope',
      fn: async () => {
        expect(ENVELOPE_SELECTION_CRITERIA.join('\n',).includes('Fits the surrounding text in register and tense',),)
          .toBe(false,);
        expect(ENVELOPE_SELECTION_CRITERIA.join('\n',),).toContain('follows the house tense rule',);
      },
    },),
    it({
      name: 'NAMES the page apparatus and the declared-name exemption in both editor faithfulness criteria',
      fn: async () => {
        for (const criteria of [ENVELOPE_SELECTION_CRITERIA, CHUNK_SELECTION_CRITERIA,]) {
          expect(criteria.join('\n',),).toContain('WHAT THE EXISTING TRANSLATION CARRIES',);
          expect(criteria.join('\n',),).toContain('When a name is used TO REFER TO a person',);
        }
      },
    },),
    it({
      name: 'TELLS every refine selection a house correction keeps the meaning',
      fn: async () => {
        for (const criteria of refineCriteria)
          expect(criteria,).toContain(KEEPS_MEANING,);
      },
    },),
    it({
      name: 'LISTS the house rules of form in one wording on the polish gate, the refiner and the review',
      fn: async () => {
        /**
         Polish gate system text.
         */
        const gate = buildConsolidationPolishGateMessages({
          subject: {
            sourceText: '猫睡了。',
            archiveText: 'The cat sleeps.',
            baseText: 'The cat sleeps.',
            polishedText: 'The cat slept.',
            lineStructured: false,
            mode: { kind: 'comparative', },
          },
        },).at(0,)?.content ?? '';
        /**
         Refiner system text.
         */
        const refiner = buildRefineMessages({
          sourceText: '猫睡了。',
          envelopes: [{
            envelopeId: 'paragraph/0',
            startOffset: 0,
            endOffset: 'The cat sleeps.'.length,
            baseText: 'The cat sleeps.',
            baseHash: hashContent({ content: 'The cat sleeps.', },),
            issueIds: [],
          },],
        },).messages.at(0,)?.content ?? '';
        /**
         Naturalness review system text.
         */
        const review = buildAbsoluteNaturalnessReviewMessages({
          subject: {
            sourceText: '猫睡了。',
            candidateText: 'The cat sleeps.',
            paragraphs: ['The cat sleeps.',],
            lineStructured: false,
          },
        },).at(0,)?.content ?? '';
        for (const sheet of [gate, refiner, review,])
          expect(sheet,).toContain(FORM_LIST,);
      },
    },),
  ],
},);
