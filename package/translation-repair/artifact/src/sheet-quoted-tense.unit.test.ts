/**
 Guards the TianqiChen666 finding of 2026-10-15: her last message, written in her own present, shipped in the past on two
 runs, because the house tense rule listed what the person "said" among the life told in the past and gave quoted words no
 exception, and every sheet that repeats the rule in its own words moved a present-tense line to the past without one either.

 Every rendered sheet (`rendered-sheets.test-fixture.ts`) that carries the tense rule shows the quoted-words sentences, and
 every sheet that tells a producer or a judge to move a line of the life to the past says that quoted words are not such a
 line. The deterministic tense candidates already blank quoted speech (`present-tense-text.ts`); that is unchanged.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  HOUSE_POLICY_BLOCK,
  JUDGE_POLICY_BLOCK,
  POLISH_GATE_HOUSE_RULES,
} from '@monochromatic-dev/translation-repair-house-rule';
import { ENVELOPE_SELECTION_CRITERIA, } from '@monochromatic-dev/translation-repair-repair';
import { CONTEST_POLICY, } from '@monochromatic-dev/translation-repair-lane-contest';
import { renderedSheets, } from './rendered-sheets.test-fixture.ts';

//region Fixtures

/**
 The sentences the house rule carries about quoted words, as the rule now reads.
 */
const HOUSE_RULE_SENTENCES =
  'Words quoted as someone wrote or said them (a letter, a message, a quoted line, a blockquote of their words) keep the tense the '
    + 'speaker used, because they are the speaker\'s own voice and the past belongs to the narration around them. A quotation moved '
    + 'into the past is a defect to hold against a rendering, and keeping its present is never a tense inconsistency.';

/**
 The one sentence every other sheet carries beside an instruction to move a line of the life to the past.
 */
const REMINDER =
  'Words quoted as someone wrote or said them keep the speaker\'s own tense, so moving a quotation to the past is a defect and '
    + 'keeping its present is not a tense inconsistency.';

/**
 Phrases that open the house tense rule in a sheet.
 */
const RULE_MARKER = 'Tense is chosen once and held.';

/**
 Phrases with which a sheet tells a reader to move a line of the life to the past, or calls that move a correction.
 */
const MOVE_MARKERS: readonly string[] = [
  'ONE TENSE, IN THIS ORDER',
  'a present-tense line of the life to the past',
  'tells the life in the present',
  'a present-tense line about their life',
  'moves the narrative of a life that has ended into the past tense',
  'a line of the life moved from the present to the past',
];

/**
 The list of house corrections of form, which names the past tense as one.
 */
const FORM_LIST_MARKER = 'the past tense for the life of a person who has died';

/**
 What the list of house corrections of form says about quoted words.
 */
const FORM_LIST_EXCEPTION = 'the past tense for the life of a person who has died (never for quoted words)';

/**
 The words with which the house tense rule puts the narrative of a life, what the person said among it, in the past. A sheet that
 repeats the rule in its own words (the carry writer's one tense line) is not this sentence.
 */
const LIFE_IN_THE_PAST_MARKER = 'the narrative of a life (what the person did, said, made, felt) is told in the past tense';

/**
 The exception the house tense rule states in the sentence that puts the life in the past (n166, Z11).
 */
const EXCEPTION_BESIDE_THE_PAST =
  'except for the person\'s words quoted as they wrote or said them, which keep the speaker\'s own tense because they are the '
    + 'speaker\'s own voice';

/**
 The words with which the judge tail says no rule holds a life in the present.
 */
const NO_RULE_MARKER = 'there is no rule holding';

/**
 The exception the judge tail states in the sentence that says no rule holds a life in the present (n166, Z11).
 */
const EXCEPTION_BESIDE_NO_RULE =
  'and the person\'s words quoted as they wrote or said them are no part of that narration, since they keep the speaker\'s own tense';

/**
 The sentences of a text that hold a marker, a sentence running to the next full stop and a space.

 @param text - text of a sheet
 @param marker - words the sentence holds

 @returns Sentences, none when the marker stands nowhere

 @example
 ```ts
 const held = sentencesHolding({ text: 'The cat naps. The dog barks.', marker: 'dog', },); // ['The dog barks.']
 ```
 */
function sentencesHolding({ text, marker, }: { readonly text: string; readonly marker: string; },): readonly string[] {
  return text
    .split('. ',)
    .filter(function holds(sentence,): boolean {
      return sentence.includes(marker,);
    },);
}

/**
 Sheets that carry a given marker, by stage name.
 */
function sheetsCarrying({ marker, }: { readonly marker: string; },): readonly { readonly name: string; readonly text: string; }[] {
  return renderedSheets()
    .filter(function carries({ text, },): boolean {
      return text.includes(marker,);
    },);
}

//endregion Fixtures

await describe({
  name: 'quoted words keep the speaker tense in every sheet',
  concurrency: 1,
  children: [
    describe({
      name: 'the house rule',
      children: [
        it({
          name: 'IS CARRIED by at least one rendered sheet, so the next tests are not vacuous',
          fn: async () => {
            expect(sheetsCarrying({ marker: RULE_MARKER, },).length > 0,).toBe(true,);
            expect(HOUSE_POLICY_BLOCK,).toContain(RULE_MARKER,);
          },
        },),

        it({
          name: 'SHOWS the quoted-words sentences in every rendered sheet that carries the rule',
          fn: async () => {
            /**
             Names of the sheets that carry the rule and lack the sentences.
             */
            const lacking = sheetsCarrying({ marker: RULE_MARKER, },)
              .filter(function lacks({ text, },): boolean {
                return !text.includes(HOUSE_RULE_SENTENCES,);
              },)
              .map(function nameOf({ name, },): string {
                return name;
              },);
            expect(lacking,).toEqual([],);
          },
        },),

        it({
          name: 'STATES the exception in the very sentence that puts the narrative of a life, the person\'s sayings included, in the past '
            + '(n166, Z11)',
          fn: async () => {
            /**
             The sheets that carry the rule, which must each hold that sentence once.
             */
            const carrying = sheetsCarrying({ marker: LIFE_IN_THE_PAST_MARKER, },);
            expect(carrying.length > 0,).toBe(true,);
            /**
             Names of the sheets where the sentence that puts the life in the past lacks the exception.
             */
            const lacking = carrying
              .filter(function lacks({ text, },): boolean {
                return sentencesHolding({ text, marker: LIFE_IN_THE_PAST_MARKER, },)
                  .some(function withoutException(sentence,): boolean {
                    return !sentence.includes(EXCEPTION_BESIDE_THE_PAST,);
                  },);
              },)
              .map(function nameOf({ name, },): string {
                return name;
              },);
            expect(lacking,).toEqual([],);
            expect(HOUSE_POLICY_BLOCK,).toContain(EXCEPTION_BESIDE_THE_PAST,);
          },
        },),

        it({
          name: 'STATES the exception in the very sentence of the judge tail that says no rule holds a life in the present (n166, Z11)',
          fn: async () => {
            /**
             The sheets that carry the judge tail.
             */
            const carrying = sheetsCarrying({ marker: NO_RULE_MARKER, },);
            expect(carrying.length > 0,).toBe(true,);
            /**
             Names of the sheets where that sentence lacks the exception.
             */
            const lacking = carrying
              .filter(function lacks({ text, },): boolean {
                return sentencesHolding({ text, marker: NO_RULE_MARKER, },)
                  .some(function withoutException(sentence,): boolean {
                    return !sentence.includes(EXCEPTION_BESIDE_NO_RULE,);
                  },);
              },)
              .map(function nameOf({ name, },): string {
                return name;
              },);
            expect(lacking,).toEqual([],);
            expect(JUDGE_POLICY_BLOCK,).toContain(EXCEPTION_BESIDE_NO_RULE,);
          },
        },),

        it({
          name: 'SHOWS the quoted-words sentences in the block every tier inherits',
          fn: async () => {
            expect(HOUSE_POLICY_BLOCK,).toContain(HOUSE_RULE_SENTENCES,);
            expect(JUDGE_POLICY_BLOCK,).toContain(HOUSE_RULE_SENTENCES,);
            expect(CONTEST_POLICY,).toContain(HOUSE_RULE_SENTENCES,);
          },
        },),
      ],
    },),

    describe({
      name: 'a sheet that tells the reader to move a line of the life to the past',
      children: [
        it({
          name: 'SAYS that quoted words are no such line, in every rendered sheet that carries such an instruction',
          fn: async () => {
            /**
             Names of the sheets with an instruction to move a line to the past and no sentence about quoted words.
             */
            const lacking = renderedSheets()
              .filter(function moves({ text, },): boolean {
                return MOVE_MARKERS.some(function carries(marker,): boolean {
                  return text.includes(marker,);
                },);
              },)
              .filter(function lacks({ text, },): boolean {
                return !(text.includes(REMINDER,) || text.includes(HOUSE_RULE_SENTENCES,));
              },)
              .map(function nameOf({ name, },): string {
                return name;
              },);
            expect(lacking,).toEqual([],);
          },
        },),

        it({
          name: 'SAYS it in the one-sentence reminder, beside every instruction of its own that the house rule block does not '
            + 'carry, whether or not the sheet also carries the house rule',
          fn: async () => {
            /**
             Markers of an instruction written in a sheet's own words: the house rule block does not hold them.
             */
            const ownWords = MOVE_MARKERS.filter(function isOwn(marker,): boolean {
              return !HOUSE_POLICY_BLOCK.includes(marker,);
            },);
            /**
             Names of the sheets with an instruction of their own words and no reminder.
             */
            const lacking = renderedSheets()
              .filter(function moves({ text, },): boolean {
                return ownWords.some(function carries(marker,): boolean {
                  return text.includes(marker,);
                },);
              },)
              .filter(function lacks({ text, },): boolean {
                return !text.includes(REMINDER,);
              },)
              .map(function nameOf({ name, },): string {
                return name;
              },);
            expect(ownWords.length > 0,).toBe(true,);
            expect(lacking,).toEqual([],);
          },
        },),

        it({
          name: 'SAYS it in the tense paragraph the polish gate adds to its sheet, besides the judge block the gate also carries, '
            + 'which no rendered sheet of the fixture shows',
          fn: async () => {
            expect(POLISH_GATE_HOUSE_RULES,).toContain(JUDGE_POLICY_BLOCK,);
            expect(POLISH_GATE_HOUSE_RULES.split(REMINDER,).length - 1,).toBe(2,);
          },
        },),

        it({
          name: 'IS FOUND in at least one rendered sheet per marker, so the test of every sheet carrying a marker is not vacuous',
          fn: async () => {
            /**
             Markers no rendered sheet carries.
             */
            const unused = MOVE_MARKERS.filter(function carriedByNone(marker,): boolean {
              return sheetsCarrying({ marker, },).length === 0;
            },);
            // Two markers belong to sheets the fixture renders under another flag; the rest must be found.
            expect((MOVE_MARKERS.length - unused.length) >= 3,).toBe(true,);
          },
        },),

        it({
          name: 'KEEPS the selection criterion about tense in step with the rule',
          fn: async () => {
            expect(ENVELOPE_SELECTION_CRITERIA.join(' ',),).toContain(REMINDER,);
          },
        },),
      ],
    },),

    describe({
      name: 'the list of house corrections of form',
      children: [
        it({
          name: 'NAMES the past tense as a correction for the life, never for quoted words, in every sheet that lists it',
          fn: async () => {
            /**
             Names of the sheets that list the past tense and give no exception.
             */
            const lacking = sheetsCarrying({ marker: FORM_LIST_MARKER, },)
              .filter(function lacks({ text, },): boolean {
                return !text.includes(FORM_LIST_EXCEPTION,);
              },)
              .map(function nameOf({ name, },): string {
                return name;
              },);
            expect(lacking,).toEqual([],);
            expect(sheetsCarrying({ marker: FORM_LIST_MARKER, },).length > 0,).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
