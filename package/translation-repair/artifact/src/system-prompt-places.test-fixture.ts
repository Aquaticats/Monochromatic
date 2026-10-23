/**
 The places two sheets of the page-level consistency pass show, built from a subject, for the guard that holds every
 stage's system prompt to the same text on every call (R4 of the second review of the Claude route).

 Fixtures are cat-themed and dog-themed invention; no corpus content appears here.

 @module
 */

import type {
  CarryChosenPlace,
  ConsistencyOccurrenceView,
  ConsistencyRepeat,
} from '@monochromatic-dev/translation-repair-consistency';
import type { PromptSubject, } from '@monochromatic-dev/translation-repair-repair/ts/system-prompt-subject.test-fixture.ts';

//region Consistency places

/**
 The words that open the second place's paragraph in the original.
 */
const SECOND_PLACE_LEAD = '傍晚的时候，';

/**
 The repeated sentence of a subject: the words its two places share.

 @param subject - invented subject

 @returns Repeat

 @example
 ```ts
 const repeat = sentenceOf({ subject: CAT_SUBJECT, },);
 ```
 */
export function sentenceOf(
  { subject, }: { readonly subject: PromptSubject; },
): ConsistencyRepeat {
  return {
    kind: 'maximal',
    wording: subject.wording,
  };
}

/**
 The refrain of a subject: the frame its two places share.

 @param subject - invented subject

 @returns Repeat

 @example
 ```ts
 const repeat = refrainOf({ subject: CAT_SUBJECT, },);
 ```
 */
export function refrainOf(
  { subject, }: { readonly subject: PromptSubject; },
): ConsistencyRepeat {
  return {
    kind: 'refrain',
    prefix: subject.prefix,
    suffix: '的小宝贝',
  };
}

/**
 The two places of a subject's repeat as the consistency sheet shows them.

 @param subject - invented subject

 @param repeat - sentence or refrain the places share

 @returns First and the second place

 @example
 ```ts
 const { first, second, } = viewsOf({ subject: CAT_SUBJECT, repeat: sentenceOf({ subject: CAT_SUBJECT, },), },);
 ```
 */
export function viewsOf(
  {
    subject,
    repeat,
  }: {
    readonly subject: PromptSubject;
    readonly repeat: ConsistencyRepeat;
  },
): {
  readonly first: ConsistencyOccurrenceView;
  readonly second: ConsistencyOccurrenceView;
} {
  /**
   The words the places share, in the original.
   */
  const shared = (repeat.kind === 'maximal') ? repeat.wording : `${repeat.prefix}是胖胖${repeat.suffix}`;
  /**
   Where a refrain's slot is read, none for a repeated sentence.
   */
  const slot = (repeat.kind === 'refrain') ? { slotSource: '胖胖', } : {};
  return {
    first: {
      sliceIndex: 0,
      paragraph: `${shared}。`,
      markStart: 0,
      markEnd: shared.length,
      ...slot,
      english: {
        kind: 'present',
        text: subject.repair,
      },
    },
    second: {
      sliceIndex: 2,
      paragraph: `${SECOND_PLACE_LEAD}${shared}，${subject.neighbourSource}`,
      markStart: SECOND_PLACE_LEAD.length,
      markEnd: SECOND_PLACE_LEAD.length + shared.length,
      ...slot,
      english: {
        kind: 'present',
        text: `In the evening, ${subject.archive}`,
      },
    },
  };
}

/**
 The chosen place of a subject's repeat, as the carry sheets show it.

 @param subject - invented subject

 @param view - place that was chosen

 @returns Place with its words and its paragraph

 @example
 ```ts
 const chosen = chosenOf({ subject: CAT_SUBJECT, view, },);
 ```
 */
export function chosenOf(
  {
    subject,
    view,
  }: {
    readonly subject: PromptSubject;
    readonly view: ConsistencyOccurrenceView;
  },
): CarryChosenPlace {
  return {
    view,
    span: subject.repairQuote,
    paragraph: subject.repair,
  };
}

/**
 What a recorded death adds to a sheet's subject.

 @param deaths - names the death is recorded for, none where no death is

 @returns Flag and the names, nothing where no death is recorded

 @example
 ```ts
 const added = deathOf({ deaths: ['Mittens',], },);
 ```
 */
export function deathOf(
  { deaths, }: { readonly deaths: readonly string[]; },
): {
  readonly lifeEnded?: true;
  readonly subjectNames?: readonly string[];
} {
  return (deaths.length === 0)
    ? {}
    : {
      lifeEnded: true,
      subjectNames: deaths,
    };
}

//endregion Consistency places
