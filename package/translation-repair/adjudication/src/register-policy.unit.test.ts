/**
 Guards class one hundred twenty-eight (owner, 2026-09-25: "I'm not sure why
 you allowed the very informal terms like OD to go through."): the house
 policy said how to render community vocabulary and told the bench to keep
 "an abbreviation local to the person's world", but said nothing about the
 register of the English itself, so chat shorthand the ORIGINAL writes (OD,
 jk, TGT) reached the page as "ODing", "jk skirt" and "the TGT series", and
 no judge held it against a candidate. The rule now says the page is plain
 written English whatever the ORIGINAL's register, spells out shorthand, and
 reaches the writers' and the judges' sheets.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { HOUSE_POLICY_BLOCK, } from '@monochromatic-dev/translation-repair-house-rule';
import { buildTranslateMessages, } from '@monochromatic-dev/translation-repair-translate-floor';
import { buildCandidateSelectMessages, } from '../dist/final/node/index.mjs';
import { systemOf, } from '@monochromatic-dev/translation-repair-provider/ts/chat-message-reading.test-fixture.ts';

/**
 Clause the rule must carry, read off the house policy verbatim so a
 rewording that drops the shorthand case fails here.
 */
const REGISTER_RULE =
  'The page is plain written English for a general reader, whatever the register of the ORIGINAL. '
    + 'Chat shorthand and internet slang are rendered by the plain English words, never carried across and never matched by English slang: '
    + 'OD is overdose (overdosing, overdosed), never OD or ODing, except where it is the means of a death or an '
    + 'attempt, which the reader-protection rule keeps unnamed; jk 裙 is a sailor uniform; MtF is trans woman or '
    + 'trans girl, as 药娘 is; '
    + 'a work written as its initials takes its full title (TGT is The Grand Tour).';

/**
 Cat-themed source, since neither sheet varies with what it is given.
 */
const SOURCE_TEXT = '猫在窗台上睡觉。';

await describe({
  name: 'the page is plain written English on every sheet (class one hundred twenty-eight)',
  children: [
    it({
      name: 'the house policy says shorthand and slang are spelled out in plain English',
      fn: async () => {
        expect(HOUSE_POLICY_BLOCK,).toContain(REGISTER_RULE,);
      },
    },),
    it({
      name: 'the translate sheet carries the rule, so the writers spell shorthand out',
      fn: async () => {
        const system = systemOf({
          messages: buildTranslateMessages({ sourceText: SOURCE_TEXT, existingText: 'The cat slept on the windowsill.', },).messages,
        },);
        expect(system,).toContain(REGISTER_RULE,);
      },
    },),
    it({
      name: 'the select sheet carries the rule, so the judges hold shorthand against a candidate',
      fn: async () => {
        const system = systemOf({
          messages: buildCandidateSelectMessages({
            task: 'Each candidate is a rendering of the passage below.',
            criteria: ['Complete coverage: every proposition of the ORIGINAL is rendered.',],
            evidence: [{ label: 'ORIGINAL (Chinese)', text: SOURCE_TEXT, },],
            rendered: ['The cat slept on the windowsill.', 'The cat is napping on the sill.',],
          },),
        },);
        expect(system,).toContain(REGISTER_RULE,);
      },
    },),
  ],
},);
