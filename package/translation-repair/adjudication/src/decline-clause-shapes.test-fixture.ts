/**
 The clauses of the decline sentence about the shape of the round that was declined: the shapes in which a run-off
 narrows the question, and the shapes in which it cannot (re-check 6, M-B). The shapes are those `runoffFinalists` in
 `translate-stage/src/translate-runoff.ts` returns and `settleAbsentDecline` in
 `translate-stage/src/translate-runoff-tie.ts` acts on. The general clauses of the three rounds are in
 `decline-clause.test-fixture.ts` and `decline-clause-rounds.test-fixture.ts`; the reader that applies them is
 `decline-consequence-clauses.test-fixture.ts`.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  type DeclineClause,
  type DeclineMode,
  LEFT_BEHIND,
} from './decline-clause.test-fixture.ts';

//region Decline clause shapes
// One clause for what the sentence says of each shape a declined round can have.

/**
 A declined round's shape, by what the next ask is over: a tie at the top that leaves some candidates behind, or a
 leader and its runner-up with some behind, narrow the question; the whole slate tied, every judge declining, a leader
 alone in drawing a ballot, and a slate of only two narrow nothing, since there is nothing to narrow.
 */
export type RunoffShape =
  | 'tied-at-top'
    | 'leader-and-runner-up'
    | 'whole-slate-tied'
    | 'every-judge-declined'
    | 'leader-alone'
    | 'slate-of-two';

/**
 The shapes in which a run-off narrows the question.
 */
export const NARROWING_SHAPES: readonly RunoffShape[] = [
  'tied-at-top',
  'leader-and-runner-up',
];

/**
 The shapes in which it does not, the next ask being over the whole slate.
 */
export const WHOLE_SLATE_SHAPES: readonly RunoffShape[] = [
  'whole-slate-tied',
  'every-judge-declined',
  'leader-alone',
  'slate-of-two',
];

/**
 A challenge asks no narrower question where its round has nothing to narrow, and the sentence may not say that the
 panel is asked again there or say an outcome. It may not use the word behind either: the run-off part of the sentence
 uses it for the candidates a narrowing would drop, and a lone leader of a wide slate leaves other candidates that drew
 nothing, so a part that said no candidate is left behind would give the word a second meaning (re-check 7, nit 1).

 @example
 ```ts
 const none = NOTHING_TO_NARROW_ASKS_NO_QUESTION.meaning;
 ```
 */
export const NOTHING_TO_NARROW_ASKS_NO_QUESTION: DeclineClause = {
  meaning: 'a round that has nothing to narrow asks no such question, and does not say behind',
  inOrder: [
    [
      'nothing to narrow',
      'nothing left to narrow',
    ],
    [
      'no such question',
      'no narrower question',
    ],
  ],
  forbidden: [
    'asked again',
    'ships',
    'stays',
    'remains',
    'behind',
    'left out',
  ],
};

/**
 The first ask is over the whole slate where there is nothing to narrow, and the sentence may not say an outcome.

 @example
 ```ts
 const whole = OTHERWISE_THE_WHOLE_SLATE.meaning;
 ```
 */
export const OTHERWISE_THE_WHOLE_SLATE: DeclineClause = {
  meaning: 'where there is nothing to narrow the panel is asked again over the whole slate',
  inOrder: [
    ['otherwise',],
    ['whole slate',],
  ],
  forbidden: [
    'ships',
    'stays',
    'remains',
  ],
};

/**
 How a sentence names each shape that narrows nothing, in lower case.
 */
const SHAPE_WORDS: Readonly<Record<Exclude<RunoffShape, 'tied-at-top' | 'leader-and-runner-up'>, readonly string[]>> = {
  'whole-slate-tied': ['whole slate tied',],
  'every-judge-declined': ['every judge declin',],
  'leader-alone': ['alone in drawing a ballot',],
  'slate-of-two': [
    'slate of only two',
    'slate of two',
  ],
};

/**
 The clause a sentence must carry for one shape of declined round, in the round it is told in.

 A shape that narrows the question is read from the run-off clause's own half; a shape that does not is read from the
 clause that says there is nothing to narrow and names the shape, in a challenge, and from the clause that says the
 whole slate is asked otherwise, in the first ask.

 @param shape - what the declined round came to

 @param mode - round the sentence is told in

 @returns Clause the sentence must say for that shape

 @example
 ```ts
 const clause = shapeClauseOf({ shape: 'slate-of-two', mode: 'ships', },);
 ```
 */
export function shapeClauseOf(
  {
    shape,
    mode,
  }: {
    readonly shape: RunoffShape;
    readonly mode: DeclineMode;
  },
): DeclineClause {
  if (shape === 'tied-at-top')
    return {
      meaning: 'a tie at the top that leaves some candidates behind narrows the question',
      inOrder: [
        [
          'tied at the top',
          'tied for the lead',
        ],
        LEFT_BEHIND,
      ],
      forbidden: [],
    };
  if (shape === 'leader-and-runner-up')
    return {
      meaning: 'a leader one judge named, beside a runner-up with some candidates behind, narrows the question',
      inOrder: [
        ['one judge',],
        ['runner-up',],
        LEFT_BEHIND,
      ],
      forbidden: [],
    };
  if (mode === 'asked-again')
    return OTHERWISE_THE_WHOLE_SLATE;
  return {
    meaning: `a round that has nothing to narrow asks no such question, naming the shape ${shape}`,
    inOrder: [
      [
        'nothing to narrow',
        'nothing left to narrow',
      ],
      SHAPE_WORDS[shape],
      [
        'no such question',
        'no narrower question',
      ],
    ],
    forbidden: NOTHING_TO_NARROW_ASKS_NO_QUESTION.forbidden,
  };
}

//endregion Decline clause shapes
