/**
 Guards class one hundred eighty-three (TianqiChen66614, 2026-09-27): a quote
 whose clause the ORIGINAL gives to 我 shipped with the clause handed to
 "they" ("when I leave you all, they will end with the memories of
 kigurumi"), memories ending with memories, because the English already on
 the page had moved the subject the same way and the bench kept it. The first
 fix keyed a rendering-glossary entry on the sentence's own words; the owner
 pointed out that a sentence fragment is not a dictionary term, so the entry
 is gone and the house policy states the general rule: a clause keeps the
 subject the ORIGINAL writes.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { RENDERING_GLOSSARY, } from '@monochromatic-dev/translation-repair-rendering-audit';
import { HOUSE_POLICY_BLOCK, } from '@monochromatic-dev/translation-repair-house-rule';
import { buildTranslateMessages, } from '@monochromatic-dev/translation-repair-translate-floor';
import { buildCandidateSelectMessages, } from '../dist/final/node/index.mjs';
import { systemOf, } from '@monochromatic-dev/translation-repair-provider/ts/chat-message-reading.test-fixture.ts';

/**
 Clause the rule must carry, read off the house policy verbatim so a
 rewording that drops the subject rule fails here.
 */
const KEPT_SUBJECT =
  'A clause keeps the subject the ORIGINAL writes: where 我 (or a named person) does something, the English makes '
    + 'I (or that person) the one who does it, and never hands the verb to another subject (they, the memories, the page) to make '
    + 'the sentence smoother; the English already on the page moving a subject is a mistranslation to correct, not a '
    + 'reading to keep.';

/**
 Cat-themed source, since neither sheet varies with what it is given.
 */
const SOURCE_TEXT = '我会记得这只猫。';

await describe({
  name: 'a clause keeps the subject the ORIGINAL writes (class one hundred eighty-three, TianqiChen666)',
  children: [
    it({
      name: 'the house policy states the subject rule',
      fn: async () => {
        expect(HOUSE_POLICY_BLOCK,).toContain(KEPT_SUBJECT,);
      },
    },),
    it({
      name: 'the translate sheet carries the rule, so the writers keep 我 as the subject',
      fn: async () => {
        const system = systemOf({
          messages: buildTranslateMessages({ sourceText: SOURCE_TEXT, existingText: 'The cat will be remembered.', },)
            .messages,
        },);
        expect(system,).toContain(KEPT_SUBJECT,);
      },
    },),
    it({
      name: 'the select sheet carries the rule, so the judges stop keeping a moved subject',
      fn: async () => {
        const system = systemOf({
          messages: buildCandidateSelectMessages({
            task: 'Each candidate is a rendering of the passage below.',
            criteria: ['Complete coverage: every proposition of the ORIGINAL is rendered.',],
            evidence: [{ label: 'ORIGINAL (Chinese)', text: SOURCE_TEXT, },],
            rendered: ['I will remember this cat.', 'The cat will be remembered.',],
          },),
        },);
        expect(system,).toContain(KEPT_SUBJECT,);
      },
    },),
    it({
      name: 'LEAVES the rendering glossary to words and set phrases: no entry keyed on one sentence\'s words',
      fn: async () => {
        expect(RENDERING_GLOSSARY.some(function isSentencePatch(entry,): boolean {
          return entry.term === 'kigurumi的记忆结束';
        },),).toBe(false,);
      },
    },),
  ],
},);
