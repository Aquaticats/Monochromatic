/**
 The page-level consistency pass's judge and the run-off among its finalists, each handed a subject, for the guard that
 holds every stage's system prompt to the same text on every call (R4 of the second review of the Claude route), over a
 repeated sentence and a refrain, and each with a recorded death.

 Fixtures are cat-themed and dog-themed invention; no corpus content appears here.

 @module
 */

import {
  buildConsistencyMessages,
  buildConsistencyRunOffMessages,
  type ConsistencyRepeat,
} from '@monochromatic-dev/translation-repair-consistency';
import type {
  PromptMessages,
  PromptStage,
  PromptSubject,
} from '@monochromatic-dev/translation-repair-repair/ts/system-prompt-subject.test-fixture.ts';
import {
  deathOf,
  refrainOf,
  sentenceOf,
  viewsOf,
} from './system-prompt-places.test-fixture.ts';

//region Consistency stages

/**
 Builds the judge's sheet for a repeat.

 @param subject - invented subject

 @param repeat - sentence or refrain the places share

 @param deaths - names a death is recorded for, none where no death is

 @returns Judge's messages

 @example
 ```ts
 const messages = judgeOf({ subject: CAT_SUBJECT, repeat: sentenceOf({ subject: CAT_SUBJECT, },), deaths: [], },);
 ```
 */
function judgeOf(
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
   The two places.
   */
  const {
    first,
    second,
  } = viewsOf({
    subject,
    repeat,
  },);
  return buildConsistencyMessages({
    subject: {
      repeat,
      occurrences: [
        first,
        second,
      ],
      identityContext: subject.identity,
      referenceContext: subject.references,
      ...deathOf({ deaths, },),
    },
  },);
}

/**
 Builds the run-off's sheet for a repeat.

 @param subject - invented subject

 @param repeat - sentence or refrain the places share

 @param deaths - names a death is recorded for, none where no death is

 @returns Run-off's messages

 @example
 ```ts
 const messages = runOffOf({ subject: CAT_SUBJECT, repeat: sentenceOf({ subject: CAT_SUBJECT, },), deaths: [], },);
 ```
 */
function runOffOf(
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
   The two places.
   */
  const {
    first,
    second,
  } = viewsOf({
    subject,
    repeat,
  },);
  return buildConsistencyRunOffMessages({
    subject: {
      repeat,
      occurrences: [
        first,
        second,
      ],
      identityContext: subject.identity,
      referenceContext: subject.references,
      ...deathOf({ deaths, },),
    },
    finalists: [
      'OCCURRENCE 1',
      'OCCURRENCE 2',
    ],
    renderings: [
      {
        kind: 'located',
        quote: subject.repairQuote,
        core: subject.repairQuote,
        start: 0,
      },
      {
        kind: 'located',
        quote: subject.archiveQuote,
        core: subject.archiveQuote,
        start: subject.repairQuote
          .length,
      },
    ],
  },);
}

/**
 The consistency pass's stages, each rendered from a subject.
 */
export const CONSISTENCY_STAGES: readonly PromptStage[] = [
  {
    name: 'consistency, repeated sentence',
    render: function renderConsistencySentence({ subject, },) {
      return judgeOf({
        subject,
        repeat: sentenceOf({ subject, },),
        deaths: [],
      },);
    },
  },
  {
    name: 'consistency, refrain',
    render: function renderConsistencyRefrain({ subject, },) {
      return judgeOf({
        subject,
        repeat: refrainOf({ subject, },),
        deaths: [],
      },);
    },
  },
  {
    name: 'consistency, repeated sentence, death recorded',
    render: function renderConsistencySentenceDeath({ subject, },) {
      return judgeOf({
        subject,
        repeat: sentenceOf({ subject, },),
        deaths: [subject.name,],
      },);
    },
  },
  {
    name: 'consistency, refrain, death recorded',
    render: function renderConsistencyRefrainDeath({ subject, },) {
      return judgeOf({
        subject,
        repeat: refrainOf({ subject, },),
        deaths: [subject.name,],
      },);
    },
  },
  {
    name: 'consistency run-off, repeated sentence',
    render: function renderRunOffSentence({ subject, },) {
      return runOffOf({
        subject,
        repeat: sentenceOf({ subject, },),
        deaths: [],
      },);
    },
  },
  {
    name: 'consistency run-off, refrain',
    render: function renderRunOffRefrain({ subject, },) {
      return runOffOf({
        subject,
        repeat: refrainOf({ subject, },),
        deaths: [],
      },);
    },
  },
  {
    name: 'consistency run-off, repeated sentence, death recorded',
    render: function renderRunOffSentenceDeath({ subject, },) {
      return runOffOf({
        subject,
        repeat: sentenceOf({ subject, },),
        deaths: [subject.name,],
      },);
    },
  },
];

//endregion Consistency stages
