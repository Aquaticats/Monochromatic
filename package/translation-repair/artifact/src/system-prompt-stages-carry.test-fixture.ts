/**
 The page-level consistency pass's carry sheets, each handed a subject, for the guard that holds every stage's system
 prompt to the same text on every call (R4 of the second review of the Claude route): the writer that carries the
 chosen rendering and the judge that chooses among the carried candidates, over a repeated sentence and a refrain, and
 each with a recorded death.

 Fixtures are cat-themed and dog-themed invention; no corpus content appears here.

 @module
 */

import {
  buildCarryChoiceMessages,
  buildCarryWriterMessages,
  type ConsistencyRepeat,
} from '@monochromatic-dev/translation-repair-consistency';
import type {
  PromptMessages,
  PromptStage,
  PromptSubject,
} from '@monochromatic-dev/translation-repair-repair/ts/system-prompt-subject.test-fixture.ts';
import {
  chosenOf,
  deathOf,
  refrainOf,
  sentenceOf,
  viewsOf,
} from './system-prompt-places.test-fixture.ts';

//region Carry stages

/**
 Builds the writer's sheet for a repeat.

 @param subject - invented subject

 @param repeat - sentence or refrain the places share

 @param deaths - names a death is recorded for, none where no death is

 @returns Writer's messages

 @example
 ```ts
 const messages = writerOf({ subject: CAT_SUBJECT, repeat: sentenceOf({ subject: CAT_SUBJECT, },), deaths: [], },);
 ```
 */
function writerOf(
  {
    subject,
    repeat,
    deaths,
  }: {
    readonly subject: PromptSubject;
    readonly repeat: ConsistencyRepeat;
    readonly deaths: readonly string[];
  },
): PromptMessages {
  /**
   The two places, the first chosen and the second receiving.
   */
  const {
    first,
    second,
  } = viewsOf({
    subject,
    repeat,
  },);
  return buildCarryWriterMessages({
    subject: {
      repeat,
      chosen: chosenOf({
        subject,
        view: first,
      },),
      receiving: [{
        view: second,
        span: subject.archiveQuote,
      },],
      paragraph: subject.archive,
      keeping: [subject.name,],
      before: subject.neighbourSource,
      after: subject.verseSource,
      lineStructured: false,
      identityContext: subject.identity,
      referenceContext: subject.references,
      ...deathOf({ deaths, },),
    },
  },);
}

/**
 Builds the choice's sheet for a repeat.

 @param subject - invented subject

 @param repeat - sentence or refrain the places share

 @param runOff - whether the choice is the run-off among finalists

 @param deaths - names a death is recorded for, none where no death is

 @returns Choice's messages

 @example
 ```ts
 const messages = choiceOf({
   subject: CAT_SUBJECT,
   repeat: sentenceOf({ subject: CAT_SUBJECT, },),
   runOff: false,
   deaths: [],
 },);
 ```
 */
function choiceOf(
  {
    subject,
    repeat,
    runOff,
    deaths,
  }: {
    readonly subject: PromptSubject;
    readonly repeat: ConsistencyRepeat;
    readonly runOff: boolean;
    readonly deaths: readonly string[];
  },
): PromptMessages {
  /**
   The two places, the first chosen and the second receiving.
   */
  const {
    first,
    second,
  } = viewsOf({
    subject,
    repeat,
  },);
  return buildCarryChoiceMessages({
    ...(runOff ? { runOff: true, } : {}),
    subject: {
      repeat,
      receiving: [{
        view: second,
        span: subject.archiveQuote,
      },],
      chosen: chosenOf({
        subject,
        view: first,
      },),
      keeping: [subject.name,],
      candidates: [
        {
          label: 'CANDIDATE 1',
          current: true,
          text: subject.archive,
        },
        {
          label: 'CANDIDATE 2',
          current: false,
          text: subject.repair,
        },
      ],
      identityContext: subject.identity,
      referenceContext: subject.references,
      ...deathOf({ deaths, },),
    },
  },);
}

/**
 The carry sheets, each rendered from a subject.
 */
export const CARRY_STAGES: readonly PromptStage[] = [
  {
    name: 'carry writer, repeated sentence',
    render: function renderWriterSentence({ subject, },) {
      return writerOf({
        subject,
        repeat: sentenceOf({ subject, },),
        deaths: [],
      },);
    },
  },
  {
    name: 'carry writer, refrain',
    render: function renderWriterRefrain({ subject, },) {
      return writerOf({
        subject,
        repeat: refrainOf({ subject, },),
        deaths: [],
      },);
    },
  },
  {
    name: 'carry writer, repeated sentence, death recorded',
    render: function renderWriterSentenceDeath({ subject, },) {
      return writerOf({
        subject,
        repeat: sentenceOf({ subject, },),
        deaths: [subject.name,],
      },);
    },
  },
  {
    name: 'carry choice, repeated sentence',
    render: function renderChoiceSentence({ subject, },) {
      return choiceOf({
        subject,
        repeat: sentenceOf({ subject, },),
        runOff: false,
        deaths: [],
      },);
    },
  },
  {
    name: 'carry choice, refrain',
    render: function renderChoiceRefrain({ subject, },) {
      return choiceOf({
        subject,
        repeat: refrainOf({ subject, },),
        runOff: false,
        deaths: [],
      },);
    },
  },
  {
    name: 'carry choice run-off, repeated sentence',
    render: function renderChoiceRunOffSentence({ subject, },) {
      return choiceOf({
        subject,
        repeat: sentenceOf({ subject, },),
        runOff: true,
        deaths: [],
      },);
    },
  },
  {
    name: 'carry choice, repeated sentence, death recorded',
    render: function renderChoiceSentenceDeath({ subject, },) {
      return choiceOf({
        subject,
        repeat: sentenceOf({ subject, },),
        runOff: false,
        deaths: [subject.name,],
      },);
    },
  },
];

//endregion Carry stages
