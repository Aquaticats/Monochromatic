/**
 Guards class one hundred twenty-four (shi_Yumiaoya24, 2026-09-25): the
 reader-protection rule opened "When a death was by suicide, the specific
 method is deliberately kept vague", so the judges read a survived attempt as
 outside it and chose "she swallowed a large amount of medication in the early
 hours of January 16 and attempted to end her life", one ballot writing
 "survived the attempt, so no method named", another calling the method "the
 appropriate way to handle the specific method mention in the original". The
 owner wrote on 2026-09-25 that dropping it is expected. The rule now covers a
 death and an attempt alike.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { HOUSE_POLICY_BLOCK, } from '@monochromatic-dev/translation-repair-house-rule';
import { buildCandidateSelectMessages, } from '../dist/final/node/index.mjs';

/**
 Opening that reaches an attempt, read off the house policy verbatim.
 */
const DEATH_OR_ATTEMPT =
  'When a death or an attempt was by suicide, the specific method is deliberately kept vague, whether she died or survived';

/**
 Clause saying a survived attempt keeps its method vague too.
 */
const ATTEMPT_METHOD =
  'A survived attempt is no exception: the page says that she attempted suicide or tried to end her life, never by what means '
    + '("swallowed a large amount of medication" names the means).';

await describe({
  name: 'a survived attempt keeps its method vague (class one hundred twenty-four)',
  children: [
    it({
      name: 'the house policy opens on a death or an attempt, not a death alone',
      fn: async () => {
        expect(HOUSE_POLICY_BLOCK,).toContain(DEATH_OR_ATTEMPT,);
        expect(HOUSE_POLICY_BLOCK,).not.toContain('When a death was by suicide,',);
      },
    },),
    it({
      name: 'the house policy says a survived attempt names no means',
      fn: async () => {
        expect(HOUSE_POLICY_BLOCK,).toContain(ATTEMPT_METHOD,);
      },
    },),
    it({
      name: 'the select sheet carries both, so the judges stop reading an attempt as outside the rule',
      fn: async () => {
        /**
         Select judge's standing rules.
         */
        const system = buildCandidateSelectMessages({
          task: 'Each candidate is a rendering of the passage below.',
          criteria: ['Complete coverage: every proposition of the ORIGINAL is rendered.',],
          evidence: [{ label: 'ORIGINAL (Chinese)', text: '猫在窗台上睡觉。', },],
          rendered: ['The cat slept on the windowsill.', 'The cat is napping on the sill.',],
        },)
          .filter(function isSystem(message,): boolean {
            return message.role === 'system';
          },)
          .map(function toContent(message,): string {
            return message.content;
          },)
          .join('\n',);
        expect(system,).toContain(DEATH_OR_ATTEMPT,);
        expect(system,).toContain(ATTEMPT_METHOD,);
      },
    },),
  ],
},);
