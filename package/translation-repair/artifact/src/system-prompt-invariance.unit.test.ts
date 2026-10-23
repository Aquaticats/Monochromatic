/**
 Holds every stage's system prompt to the same text on every call (R4 of the second review of the Claude route).

 The Claude body marks the system prompt of each call for the five-minute cache. A cache write costs 2.50 USD
 a million input tokens against 2.00 uncached for Sonnet, so a system prompt that embeds the words of one call is
 paid for at the write rate on every call and is never read back. The sheets keep
 the subject in the user turn; this file renders each stage for two subjects that share a shape and differ in every word
 and requires the system text to be equal.

 A stage that has a fixed set of variants (a recorded death, a refrain against a repeated sentence, the two
 perspectives of the naturalness review) is rendered once per variant, each variant against both subjects, so the
 system text may differ between variants and never between subjects. Nothing here reaches a provider.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import { systemTextOf, } from '@monochromatic-dev/translation-repair-provider';
import {
  CAT_SUBJECT,
  DOG_SUBJECT,
  FENCED_SUBJECT,
  MARKED_SUBJECT,
  type PromptMessages,
  type PromptStage,
  type PromptSubject,
} from '@monochromatic-dev/translation-repair-repair/ts/system-prompt-subject.test-fixture.ts';
import { PREPARATION_STAGES, } from '@monochromatic-dev/translation-repair-repair/ts/system-prompt-stages-preparation.test-fixture.ts';
import { SLATE_STAGES, } from '@monochromatic-dev/translation-repair-repair/ts/system-prompt-stages-slates.test-fixture.ts';
import { WRITER_STAGES, } from '@monochromatic-dev/translation-repair-repair/ts/system-prompt-stages-writers.test-fixture.ts';
import { BREAK_STAGES, } from './system-prompt-stages-breaks.test-fixture.ts';
import { CARRY_STAGES, } from './system-prompt-stages-carry.test-fixture.ts';
import { CONSISTENCY_STAGES, } from './system-prompt-stages-consistency.test-fixture.ts';
import { PAGE_STAGES, } from './system-prompt-stages-pages.test-fixture.ts';

/**
 Every stage, rendered once per variant.
 */
const STAGES: readonly PromptStage[] = [
  ...WRITER_STAGES,
  ...SLATE_STAGES,
  ...PREPARATION_STAGES,
  ...PAGE_STAGES,
  ...CONSISTENCY_STAGES,
  ...CARRY_STAGES,
  ...BREAK_STAGES,
];

/**
 The turns of a conversation that are not the system prompt.

 @param messages - conversation a builder handed back

 @returns Every user and assistant turn

 @example
 ```ts
 const turns = notSystem({ messages, },);
 ```
 */
function notSystem(
  { messages, }: { readonly messages: PromptMessages; },
): PromptMessages {
  return messages.filter(function isOtherTurn({ role, },): boolean {
    return role !== 'system';
  },);
}

/**
 Renders a stage for the cat, the dog and the fox cub and reads the system text the Claude body would cache.

 @param stage - stage and variant to render

 @returns System text for each subject

 @example
 ```ts
 const { cat, dog, } = systemTextsOf({ stage: WRITER_STAGES[0], },);
 ```
 */
function systemTextsOf(
  { stage, }: { readonly stage: PromptStage; },
): {
  readonly cat: string;
  readonly dog: string;
  readonly fenced: string;
  readonly marked: string;
} {
  return {
    cat: systemTextOf({ messages: stage.render({ subject: CAT_SUBJECT, },), },),
    dog: systemTextOf({ messages: stage.render({ subject: DOG_SUBJECT, },), },),
    fenced: systemTextOf({ messages: stage.render({ subject: FENCED_SUBJECT, },), },),
    marked: systemTextOf({ messages: stage.render({ subject: MARKED_SUBJECT, },), },),
  };
}

/**
 Renders a stage for the cat and the dog and reads every turn that is not the system prompt.

 @param stage - stage and variant to render

 @returns Other turns for each subject, as JSON

 @example
 ```ts
 const { cat, dog, } = otherTurnsOf({ stage: WRITER_STAGES[0], },);
 ```
 */
function otherTurnsOf(
  { stage, }: { readonly stage: PromptStage; },
): {
  readonly cat: string;
  readonly dog: string;
} {
  return {
    cat: JSON.stringify(notSystem({ messages: stage.render({ subject: CAT_SUBJECT, },), },),),
    dog: JSON.stringify(notSystem({ messages: stage.render({ subject: DOG_SUBJECT, },), },),),
  };
}

/**
 Fence length a subject with no run of equals signs is fenced in.
 */
const PLAIN_FENCE = 5;

/**
 Fence length the fox cub's eleven-sign underline forces: one more than the longest run.
 */
const FORCED_FENCE = 12;

/**
 Words that open the user turn of a slate, which states the fence length.
 */
const FENCE_OPENING = 'Every block below opens and closes with a line of ';

/**
 Words that close the sentence stating the fence length.
 */
const FENCE_CLOSING = ' equals signs.';

/**
 Reads the fence length a slate's user turn states and the longest line of nothing but equals signs it holds.

 @param stage - slate stage to render

 @param subject - subject to render it for

 @returns Stated length (zero where the sentence is missing) and the longest fence line

 @example
 ```ts
 const read = fenceStatedBy({ stage, subject: CAT_SUBJECT, },);
 ```
 */
function fenceStatedBy(
  {
    stage,
    subject,
  }: {
    readonly stage: PromptStage;
    readonly subject: PromptSubject;
  },
): {
  readonly stated: number;
  readonly longestFenceLine: number;
} {
  /**
   The user turn, whole.
   */
  const user = notSystem({ messages: stage.render({ subject, },), },)
    .map(function contentOf({ content, },): string {
      if ((typeof content) !== 'string')
        return '';
      return content;
    },)
    .join('\n',);
  /**
   Where the sentence stating the length ends.
   */
  const closing = user.indexOf(FENCE_CLOSING,);
  return {
    stated: user.startsWith(FENCE_OPENING,) ? Number(user.slice(FENCE_OPENING.length, closing,),) : 0,
    longestFenceLine: Math.max(...user.split('\n',)
      .filter(function isFenceLine(line,): boolean {
        return (line !== '') && (line.replaceAll('=', '',) === '');
      },)
      .map(function lengthOf(line,): number {
        return line.length;
      },),),
  };
}

/**
 Opening mark a subject with a stray run of two opening marks is given: one more than the longest run.
 */
const FORCED_MARK_OPEN = '⟦⟦⟦';

/**
 Closing mark that goes with it.
 */
const FORCED_MARK_CLOSE = '⟧⟧⟧';

/**
 Words that open the sentence naming the marks, after the word a sheet calls its paragraphs.
 */
const MARKS_OPENING = ' the repeated wording is enclosed between ';

/**
 Reads the marks the sentence in a consistency sheet's user turn names.

 @param stage - stage to render

 @param subject - subject to render it for

 @returns Opening and the closing mark named (empty where the user turn has no such sentence)

 @example
 ```ts
 const read = marksStatedBy({ stage, subject: CAT_SUBJECT, },);
 ```
 */
function marksStatedBy(
  {
    stage,
    subject,
  }: {
    readonly stage: PromptStage;
    readonly subject: PromptSubject;
  },
): {
  readonly open: string;
  readonly close: string;
} {
  /**
   The user turn, whole.
   */
  const user = notSystem({ messages: stage.render({ subject, },), },)
    .map(function contentOf({ content, },): string {
      if ((typeof content) !== 'string')
        return '';
      return content;
    },)
    .join('\n',);
  if (!user.includes(MARKS_OPENING,))
    return { open: '', close: '', };
  /**
   The sentence from the first mark on.
   */
  const named = user.slice(user.indexOf(MARKS_OPENING,) + MARKS_OPENING.length,);
  /**
   Where the first mark ends.
   */
  const between = named.indexOf(' and ',);
  return {
    open: named.slice(0, between,),
    close: named.slice(between + ' and '.length, named.indexOf(';',),),
  };
}

await describe({
  name: 'the system prompt of every stage',
  concurrency: 1,
  children: [
    ...STAGES.map(function caseOf(stage,) {
      return it({
        name: `IS THE SAME TEXT FOR FOUR SUBJECTS, ONE FORCING A LONGER FENCE AND ONE LONGER MARKS, IN THE `
          + `${stage.name.toUpperCase()} SHEET (R4): the cache breakpoint on `
          + 'the system block is written once and read on every later call, never rewritten for each subject',
        fn: async () => {
          const { cat, dog, fenced, marked, } = systemTextsOf({ stage, },);
          expect(cat.length,).toBeGreaterThan(0,);
          expect(dog,).toBe(cat,);
          // A SUBJECT WHOSE TEXTS FORCE A LONGER FENCE gets the same system text: the fence and its length are stated
          // in the user turn, so a text holding a run of equals signs writes no second cache entry.
          expect(fenced,).toBe(cat,);
          // A SUBJECT WHOSE TEXTS HOLD A STRAY RUN OF THE CONSISTENCY MARKS gets the same system text: the marks are
          // stated in the user turn too.
          expect(marked,).toBe(cat,);
          // THE SUBJECTS REACH THE BUILDER: the turns that are not the system prompt differ, so an equal system
          // text is not the result of a render that ignored its subject.
          expect(otherTurnsOf({ stage, },).dog,).not.toBe(otherTurnsOf({ stage, },).cat,);
        },
      },);
    },),

    it({
      name: 'FAILS FOR A SYSTEM TEXT THAT EMBEDS THE SUBJECT (the positive control of R4): the comparison made for every '
        + 'stage can tell two subjects apart, so an equal pair is evidence and not a probe that reads nothing',
      fn: async () => {
        const { cat, dog, } = systemTextsOf({
          stage: {
            name: 'control',
            render: ({ subject, },) => [{ role: 'system', content: `Judge ${subject.source}`, },],
          },
        },);
        expect(dog,).not.toBe(cat,);
      },
    },),

    ...STAGES.filter(function isSlate({ name, },): boolean {
      return name.includes(' slate',);
    },)
      .map(function fenceCaseOf(stage,) {
        return it({
          name: `STATES IN THE USER TURN THE FENCE LENGTH THE ${stage.name.toUpperCase()} BLOCKS USE (R4): the `
            + `sentence opens the user turn with the true length, including for a text holding a run of equals signs`,
          fn: async () => {
            expect(fenceStatedBy({
              stage,
              subject: CAT_SUBJECT,
            },),).toEqual({ stated: PLAIN_FENCE, longestFenceLine: PLAIN_FENCE, },);
            expect(fenceStatedBy({ stage, subject: FENCED_SUBJECT, },),).toEqual({
              stated: FORCED_FENCE,
              longestFenceLine: FORCED_FENCE,
            },);
          },
        },);
      },),

    ...STAGES.filter(function isMarked({ name, },): boolean {
      return name.startsWith('consistency',) || name.startsWith('carry',);
    },)
      .map(function marksCaseOf(stage,) {
        return it({
          name: `STATES IN THE USER TURN THE MARKS THE ${stage.name.toUpperCase()} PARAGRAPHS CARRY (R4): the `
            + `sentence naming the marks is in the user turn with the true marks, including for a text holding a `
            + `stray run of them`,
          fn: async () => {
            expect(marksStatedBy({ stage, subject: CAT_SUBJECT, },),).toEqual({ open: '⟦', close: '⟧', },);
            expect(marksStatedBy({
              stage,
              subject: MARKED_SUBJECT,
            },),).toEqual({ open: FORCED_MARK_OPEN, close: FORCED_MARK_CLOSE, },);
          },
        },);
      },),

    ...STAGES.filter(function hasBreaks({ name, },): boolean {
      return name.endsWith('source only with breaks',) || (name === 'translate challenge slate');
    },)
      .map(function breaksCaseOf(stage,) {
        return it({
          name: `KEEPS THE BREAK COUNTS OUT OF THE SYSTEM TEXT OF THE ${stage.name.toUpperCase()} SHEET AND IN THE `
            + `USER TURN (R4): the fixed instruction is cached, and the measured counts of the call follow the text `
            + `they measure`,
          fn: async () => {
            /**
             Words that open every measured line of the facts.
             */
            const counted = 'explicit breaks by top-level block: ';
            for (const [subject, count,] of [
              [CAT_SUBJECT, '[1]',],
              [DOG_SUBJECT, '[2]',],
            ] as const) {
              /**
               The conversation this stage builds for the subject.
               */
              const messages = stage.render({ subject, },);
              expect(systemTextOf({ messages, },),).toContain('RENDERED LINE STRUCTURE (formatting evidence',);
              expect(systemTextOf({ messages, },),).not.toContain(counted,);
              /**
               The turns that are not the system prompt, as the model reads them.
               */
              const others = notSystem({ messages, },);
              expect(JSON.stringify(others,),).toContain(`ORIGINAL ${counted}${count}`,);
            }
          },
        },);
      },),

    it({
      name: 'NAMES EVERY STAGE ONCE: a variant renamed twice would be rendered twice and read once',
      fn: async () => {
        const names = STAGES.map(function nameOf({ name, },): string {
          return name;
        },);
        expect(new Set(names,).size,).toBe(names.length,);
      },
    },),
  ],
},);
