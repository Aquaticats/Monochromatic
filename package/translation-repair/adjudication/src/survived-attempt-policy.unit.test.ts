/**
 Guards class seventy-nine (shi_Yumiaoya, 2026-09-22): a suicide attempt the
 person SURVIVED shipped as "she ended her life by taking a large amount of
 medication", two sentences before the ORIGINAL has her wake in the ICU after
 six days, on two consecutive runs, with every candidate writing it and every
 ballot approving it as reader protection. The reader-protection rule itself
 prescribed the words: its sample sentence read "the page says that she ended
 her life", so 自杀 was rendered as a death whatever the ORIGINAL went on to
 say. The rule now says that 自杀 names the act and not its outcome, and that
 where the ORIGINAL goes on to a rescue or a waking the English says an
 attempt.

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
 rewording that drops the survival case fails here.
 */
const SURVIVED_ATTEMPT =
  '自杀 names the act, not its outcome: where the ORIGINAL goes on to a rescue, a waking or a life that continues (抢救, 醒来, 幸存), '
    + 'the person survived the attempt and the page says that she tried to end her life or attempted suicide; '
    + '"ended her life" and "took her own life" assert a death, and on an attempt survived they are a factual error, not vagueness.';

/**
 Cat-themed source, since neither sheet varies with what it is given.
 */
const SOURCE_TEXT = '猫在窗台上睡觉。';

await describe({
  name: 'a survived attempt is an attempt on every sheet (class seventy-nine, shi_Yumiaoya)',
  children: [
    it({
      name: 'the house policy says 自杀 names the act and a survived attempt reads as an attempt, never a death',
      fn: async () => {
        expect(HOUSE_POLICY_BLOCK,).toContain(SURVIVED_ATTEMPT,);
      },
    },),
    it({
      name: 'the translate sheet carries the clause, so the writers stop rendering a survived attempt as a death',
      fn: async () => {
        const system = systemOf({
          messages: buildTranslateMessages({ sourceText: SOURCE_TEXT, existingText: 'The cat slept on the windowsill.', },).messages,
        },);
        expect(system,).toContain(SURVIVED_ATTEMPT,);
      },
    },),
    it({
      name: 'the select sheet carries the clause, so the judges stop approving "ended her life" as reader protection',
      fn: async () => {
        const system = systemOf({
          messages: buildCandidateSelectMessages({
            task: 'Each candidate is a rendering of the passage below.',
            criteria: ['Complete coverage: every proposition of the ORIGINAL is rendered.',],
            evidence: [{ label: 'ORIGINAL (Chinese)', text: SOURCE_TEXT, },],
            rendered: ['The cat slept on the windowsill.', 'The cat is napping on the sill.',],
          },),
        },);
        expect(system,).toContain(SURVIVED_ATTEMPT,);
      },
    },),
  ],
},);
