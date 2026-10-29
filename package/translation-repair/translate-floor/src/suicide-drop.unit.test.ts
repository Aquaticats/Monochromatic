/**
 Guards class one hundred fifty (2026-09-26): an original that names a
 suicide (自杀, 自尽, 轻生) shipped in wording that dropped the attempt the
 original states and put in its place a detail the house rule keeps vague.
 A survived attempt is still an attempt: the page says she
 attempted suicide or tried to end her life, and a candidate carrying no
 wording for suicide at all is refused before any judge reads it. The floor
 asks only that the suicide be said; how vaguely the means are put stays with
 the judges. Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  validateTranslatedSlice,
  type SliceValidation,
} from '../dist/final/node/index.mjs';

/**
 Original telling of a cat's survived attempt.
 */
const ATTEMPT = '那年冬天，橘猫曾经试图自杀，休养了很久才慢慢好起来。';

/**
 Rendering that says the attempt.
 */
const SAID = 'That winter, the ginger cat attempted suicide; after a long rest she slowly recovered.';

/**
 Rendering that drops the attempt behind a vague illness.
 */
const DROPPED = 'That winter, the ginger cat fell gravely ill; after a long rest she slowly recovered.';

/**
 Verdict on a candidate no floor refuses, with no page behind it.
 */
const VALID: SliceValidation = {
  kind: 'valid',
  pageGrammar: 'absent',
};

/**
 Verdict refusing a candidate that drops the suicide an original names.

 @param named - words for suicide the original writes, joined as the finding
 joins them

 @returns Whole verdict, so a check reads every word of the finding

 @example
 ```ts
 expect(verdict,).toEqual(dropRefusal({ named: '自杀', },),);
 ```
 */
function dropRefusal({ named, }: { readonly named: string; },): SliceValidation {
  return {
    kind: 'invalid',
    findings: [
      `Your translation drops the suicide the ORIGINAL names: the ORIGINAL passage writes ${named}, and your `
        + 'translation carries no wording for suicide at all. A death by suicide is said to be a suicide, and a '
        + 'survived attempt is still an attempt: say that the person attempted suicide or tried to end their life, '
        + 'with the pronoun the page uses for them, keeping the means as vague as the house rule asks.',
    ],
  };
}

/**
 Verdict on a rendering against an original, with no page behind it.

 @param sourceText - original passage

 @param candidateText - rendering under the floor

 @returns Whole verdict

 @example
 ```ts
 verdictOf({ sourceText: ATTEMPT, candidateText: SAID, },); // VALID
 ```
 */
function verdictOf(
  {
    sourceText,
    candidateText,
  }: {
    readonly sourceText: string;
    readonly candidateText: string;
  },
): SliceValidation {
  return validateTranslatedSlice({
    sourceText,
    candidateText,
  },);
}

await describe({
  name: 'a suicide the original passage names (class one hundred fifty)',
  children: [
    it({
      name: 'REFUSES a candidate that renders 自杀 with no wording for suicide at all',
      fn: async () => {
        expect(verdictOf({
          sourceText: ATTEMPT,
          candidateText: DROPPED,
        },),).toEqual(dropRefusal({ named: '自杀', },),);
      },
    },),
    it({
      name: 'ACCEPTS every plain way of saying it, the same for 自尽 and 轻生, and a comment\'s 自杀 asks nothing',
      fn: async () => {
        expect(verdictOf({
          sourceText: ATTEMPT,
          candidateText: SAID,
        },),).toEqual(VALID,);
        expect(verdictOf({
          sourceText: ATTEMPT,
          candidateText: 'That winter the ginger cat tried to end her own life, and after a long rest she slowly recovered.',
        },),).toEqual(VALID,);
        expect(verdictOf({
          sourceText: '老猫对小猫说：「你千万别想着自杀。」',
          candidateText: 'The old cat told the kitten, “Don’t you ever think of killing yourself.”',
        },),).toEqual(VALID,);
        expect(verdictOf({
          sourceText: '黑猫自尽了。',
          candidateText: 'The black cat took her own life.',
        },),).toEqual(VALID,);
        expect(verdictOf({
          sourceText: '她有过轻生的念头。',
          candidateText: 'She had had suicidal thoughts.',
        },),).toEqual(VALID,);
        expect(verdictOf({
          sourceText: '<!-- 自杀 --> 猫猫睡了。',
          candidateText: 'The cat slept.',
        },),).toEqual(VALID,);
      },
    },),
    it({
      name: 'ACCEPTS a life ended with a possessive and no "own" after it',
      fn: async () => {
        expect(verdictOf({
          sourceText: ATTEMPT,
          candidateText: 'That winter the ginger cat tried to end her life; after a long rest she slowly recovered.',
        },),).toEqual(VALID,);
      },
    },),
    it({
      name: 'ACCEPTS an attempt on a life, a death by one\'s own hand, and a quotation of a published work in its '
        + 'published English (ledger F-4: the replay refused "attempts on her own life" and a canonical quotation '
        + 'whose Chinese translation added 自杀)',
      fn: async () => {
        expect(verdictOf({
          sourceText: ATTEMPT,
          candidateText: 'That winter the ginger cat made an attempt on her own life; after a long rest she slowly '
            + 'recovered.',
        },),).toEqual(VALID,);
        expect(verdictOf({
          sourceText: '尽管经历了数次自杀尝试，老猫仍然每天晒太阳。',
          candidateText: 'Despite several attempts on his own life, the old cat still sunned himself every day.',
        },),).toEqual(VALID,);
        expect(verdictOf({
          sourceText: '黑猫自尽了。',
          candidateText: 'The black cat died by her own hand.',
        },),).toEqual(VALID,);
        expect(verdictOf({
          sourceText: '> 「这场游戏由清醒过渡到逃遁。我们应该沿着这条线索去理解自杀。」\n>\n> ——喵喵《猫的神话》',
          candidateText: '> One must follow and understand this game that leads from lucidity to flight.\n>\n'
            + '> — Meow Meow, *The Myth of the Cat*',
        },),).toEqual(VALID,);
      },
    },),
    it({
      name: 'ACCEPTS a quotation of a published work written on the line after a paragraph, which the parse '
        + 'reads as a block of its own, so the paragraph does not draw the quotation\'s 自杀 into the passage\'s '
        + 'own words (ledger B68)',
      fn: async () => {
        expect(verdictOf({
          sourceText: '老猫每天晒太阳。\n> 「我们应该沿着这条线索去理解自杀。」\n>\n> ——喵喵《猫的神话》',
          candidateText: 'The old cat sunned himself every day.\n> “We must follow this thread to understand it.”\n>\n'
            + '> — Meow Meow, *The Myth of the Cat*',
        },),).toEqual(VALID,);
      },
    },),
    it({
      name: 'STILL REFUSES a hand that only wrote, an unattributed quotation without the word, and names no '
        + 'pronoun the passage did not choose',
      fn: async () => {
        expect(verdictOf({
          sourceText: '橘猫自杀前，亲手写了一封信。',
          candidateText: 'Before she left, the ginger cat wrote a letter by her own hand.',
        },),).toEqual(dropRefusal({ named: '自杀', },),);
        expect(verdictOf({
          sourceText: '> 「千万别自杀。」',
          candidateText: '> “Please don’t go.”',
        },),).toEqual(dropRefusal({ named: '自杀', },),);
        expect(verdictOf({
          sourceText: '黑猫自杀了。',
          candidateText: 'The black cat passed away.',
        },),).toEqual(dropRefusal({ named: '自杀', },),);
      },
    },),
    it({
      name: 'STILL REFUSES a life verb or kill verb with nothing after it, a life verb taking another noun, and a '
        + '"by" opening the text, none of which says a suicide',
      fn: async () => {
        expect(verdictOf({
          sourceText: ATTEMPT,
          candidateText: 'That winter the ginger cat faced a choice she chose not to take.',
        },),).toEqual(dropRefusal({ named: '自杀', },),);
        expect(verdictOf({
          sourceText: ATTEMPT,
          candidateText: 'That winter the ginger cat ended the quarrel she had started.',
        },),).toEqual(dropRefusal({ named: '自杀', },),);
        expect(verdictOf({
          sourceText: ATTEMPT,
          candidateText: 'That winter the ginger cat found nothing left to kill.',
        },),).toEqual(dropRefusal({ named: '自杀', },),);
        expect(verdictOf({
          sourceText: ATTEMPT,
          candidateText: 'By that winter the ginger cat had fallen gravely ill.',
        },),).toEqual(dropRefusal({ named: '自杀', },),);
      },
    },),
    it({
      name: 'REFUSES the same drop for 自尽 and 轻生',
      fn: async () => {
        expect(verdictOf({
          sourceText: '黑猫自尽了。',
          candidateText: 'The black cat passed away.',
        },),).toEqual(dropRefusal({ named: '自尽', },),);
        expect(verdictOf({
          sourceText: '她有过轻生的念头。',
          candidateText: 'She had had dark thoughts.',
        },),).toEqual(dropRefusal({ named: '轻生', },),);
      },
    },),
  ],
},);
