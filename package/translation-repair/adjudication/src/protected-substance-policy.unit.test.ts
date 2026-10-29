/**
 Guards class one hundred twenty-two (shi_Yumiaoya23, 2026-09-25): the
 reader-protection rule was on every sheet and the page still said she took
 "three bottles of some kind of sedative-hypnotic" before she died, and that
 she went up to "the rooftop of a tall building" to end her life. The judges
 read a drug class as vague because it named no drug, read the rule as
 covering only the medication that was the means, and marked down the one
 candidate that wrote "a large amount of medication" as over-vague. The owner
 answered on 2026-09-25 that not showing suicide methods is expected. The
 rule now says that a class of drug is a substance and a count is a dose
 whether or not the drug is the means, and that a place which is itself the
 means is a method.

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
 Clause naming a drug class and a count as substance and dose, read off the
 house policy verbatim so a rewording that drops either fails here.
 */
const SUBSTANCE_CLASS =
  'A class of drug (a sedative, a sleeping pill, an antidepressant, a painkiller) is a substance and a count '
    + '(three bottles, a whole box, forty pills) is a dose, named or not: "three bottles of a sedative" names both. '
    + 'This holds for any medication or drug the page ties to a death or an attempt, not only the one it names as the means; '
    + 'the page says at most that she had taken medication.';

/**
 Clause naming a place that is itself the means as a method.
 */
const PLACE_AS_MEANS =
  'A place that is itself the means (a rooftop, a high floor, a bridge, a railway line, deep water) is a method too: '
    + 'the page keeps where it happened when that tells a reader nothing to copy (a city, a hotel, a hospital), '
    + 'and otherwise says that she ended her life, or attempted to, without saying where she went to do it.';

/**
 Cat-themed source, since neither sheet varies with what it is given.
 */
const SOURCE_TEXT = '猫在窗台上睡觉。';

await describe({
  name: 'a drug class, a count and a place that is the means stay vague (class one hundred twenty-two)',
  children: [
    it({
      name: 'the house policy names a drug class and a count as substance and dose, whether or not the drug is the means',
      fn: async () => {
        expect(HOUSE_POLICY_BLOCK,).toContain(SUBSTANCE_CLASS,);
      },
    },),
    it({
      name: 'the house policy names a place that is itself the means as a method',
      fn: async () => {
        expect(HOUSE_POLICY_BLOCK,).toContain(PLACE_AS_MEANS,);
      },
    },),
    it({
      name: 'the translate sheet carries both clauses, so the writers stop naming the class, the count and the rooftop',
      fn: async () => {
        /**
         Translate writer's standing rules.
         */
        const system = systemOf({
          messages: buildTranslateMessages({ sourceText: SOURCE_TEXT, existingText: 'The cat slept on the windowsill.', },).messages,
        },);
        expect(system,).toContain(SUBSTANCE_CLASS,);
        expect(system,).toContain(PLACE_AS_MEANS,);
      },
    },),
    it({
      name: 'the select sheet carries both clauses, so the judges stop reading a drug class as vague',
      fn: async () => {
        /**
         Select judge's standing rules.
         */
        const system = systemOf({
          messages: buildCandidateSelectMessages({
            task: 'Each candidate is a rendering of the passage below.',
            criteria: ['Complete coverage: every proposition of the ORIGINAL is rendered.',],
            evidence: [{ label: 'ORIGINAL (Chinese)', text: SOURCE_TEXT, },],
            rendered: ['The cat slept on the windowsill.', 'The cat is napping on the sill.',],
          },),
        },);
        expect(system,).toContain(SUBSTANCE_CLASS,);
        expect(system,).toContain(PLACE_AS_MEANS,);
      },
    },),
  ],
},);
