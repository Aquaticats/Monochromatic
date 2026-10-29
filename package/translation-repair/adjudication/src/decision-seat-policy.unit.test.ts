/**
 Guards the whole-package audit of 2026-09-27: the typed decision seat
 (`selectDecision`) voted on every select slate from the task, criteria,
 evidence and candidates alone. The chat seat's sheet carries the house
 rules and the community renderings block, so the decision seat read
 "Complete coverage: every proposition of the ORIGINAL is rendered" with no
 word of reader protection, the tense order, the pronoun rule or the
 precedence line, and never heard that a candidate departed from the
 community's word.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  TRANSLATE_SELECTION_TASK,
  translateSelectionCriteria,
} from '@monochromatic-dev/translation-repair-translate-floor';
import { selectDecision, } from '../dist/final/node/index.mjs';

/**
 Invented original whose kitten was outed, carrying a community term.
 */
const SOURCE = '小猫被炸柜了。';

/**
 The typed question for a two-candidate slate, one candidate lacking the
 community's rendering, as JSON.
 */
const SHOWN = JSON.stringify(selectDecision({
  task: TRANSLATE_SELECTION_TASK,
  criteria: translateSelectionCriteria({ lineStructured: false, },),
  evidence: [
    {
      label: 'ORIGINAL (Chinese)',
      text: SOURCE,
    },
  ],
  rendered: [
    'The kitten was outed.',
    'The kitten came out.',
  ],
  sourceText: SOURCE,
},),);

await describe({
  name: 'the decision seat judges under the house rules (audit of 2026-09-27)',
  children: [
    it({
      name: 'CARRIES THE HOUSE RULES and their precedence over the criteria',
      fn: async () => {
        expect(SHOWN,).toContain('House rules this corpus is written under',);
        expect(SHOWN,).toContain('Reader protection outranks completeness',);
        expect(SHOWN,).toContain('WHERE A CRITERION OR ANY OTHER RULE YOU HAVE BEEN GIVEN DISAGREES WITH A HOUSE RULE, THE HOUSE RULE WINS',);
        expect(SHOWN,).toContain('state.policy',);
      },
    },),
    it({
      name: 'NAMES A CANDIDATE LACKING the community\'s rendering, as the chat seat is told',
      fn: async () => {
        expect(SHOWN,).toContain('COMMUNITY RENDERINGS',);
        expect(SHOWN,).toContain('CANDIDATE 2 carries none of the community\'s renderings of 炸柜',);
      },
    },),
  ],
},);
