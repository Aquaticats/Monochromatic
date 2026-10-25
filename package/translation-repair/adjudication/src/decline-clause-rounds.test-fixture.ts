/**
 The clauses of the rounds of a pair that do not ship or keep, and the table of what each round must say and must not
 promise (re-check 4, F1). The clauses of the round that ships are in `decline-clause.test-fixture.ts`; the reader
 that applies them is `decline-consequence-clauses.test-fixture.ts`.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  type DeclineClause,
  type DeclineMode,
  FIXED_ORDER,
  NONE_NAMED_WORDING_STAYS,
  NOT_A_KEEP,
  OWN_DECLINE_NAMES_NONE,
  SOME_NAMED_PREFERRED_SHIPS,
} from './decline-clause.test-fixture.ts';

//region Decline clause rounds
// The clauses only the first ask and the deferring challenge say, then the table of each round.

/**
 The round settles nothing, which is the first ask of a pair: a decline there ships no candidate and keeps no wording.

 @example
 ```ts
 const settles = SETTLES_NOTHING.meaning;
 ```
 */
const SETTLES_NOTHING: DeclineClause = {
  meaning: 'a decline settles nothing in this round',
  inOrder: [
    ['this round',],
    [
      'settles nothing',
      'decides nothing',
    ],
  ],
  forbidden: [
    'ships',
    'stays',
  ],
};

/**
 The same judges are asked again.

 @example
 ```ts
 const again = SAME_JUDGES_ASKED_AGAIN.meaning;
 ```
 */
const SAME_JUDGES_ASKED_AGAIN: DeclineClause = {
  meaning: 'the same judges are asked again',
  inOrder: [
    ['same judges',],
    [
      'asked again',
      'asked once more',
    ],
  ],
  forbidden: [],
};

/**
 A challenge whose caller defers ships only a candidate every candidate tied behind, one two or more judges named, the
 preferred of those.

 @example
 ```ts
 const tie = SHIPS_ONLY_ON_AN_EQUAL_TIE.meaning;
 ```
 */
const SHIPS_ONLY_ON_AN_EQUAL_TIE: DeclineClause = {
  meaning: 'a candidate ships only where every candidate drew equal weight and two or more judges named it',
  inOrder: [
    ['ships',],
    ['only',],
    ['every candidate',],
    [
      'equal weight',
      'the same weight',
    ],
    ['two or more judges',],
    ['preferred',],
  ],
  forbidden: [
    'stays',
    'remains',
  ],
};

/**
 Otherwise the round settles nothing and the passage goes back for new proposals and is judged again.

 @example
 ```ts
 const back = SENT_BACK_FOR_NEW_PROPOSALS.meaning;
 ```
 */
const SENT_BACK_FOR_NEW_PROPOSALS: DeclineClause = {
  meaning: 'otherwise nothing is settled and the passage goes back for new proposals and is judged again',
  inOrder: [
    [
      'settles nothing',
      'decides nothing',
    ],
    [
      'new proposals',
      'fresh proposals',
    ],
    [
      'judged again',
      'judged once more',
    ],
  ],
  forbidden: [],
};

/**
 What each round must say and what it must not promise.

 @example
 ```ts
 const {
   required,
   refused,
 } = MODES.ships;
 ```
 */
export const MODES: Readonly<Record<DeclineMode, {
  readonly required: readonly DeclineClause[];
  readonly refused: readonly DeclineClause[];
}>> = {
  ships: {
    required: [
      NOT_A_KEEP,
      OWN_DECLINE_NAMES_NONE,
      NONE_NAMED_WORDING_STAYS,
      SOME_NAMED_PREFERRED_SHIPS,
      FIXED_ORDER,
    ],
    refused: [
      SAME_JUDGES_ASKED_AGAIN,
      SENT_BACK_FOR_NEW_PROPOSALS,
    ],
  },
  'asked-again': {
    required: [
      NOT_A_KEEP,
      OWN_DECLINE_NAMES_NONE,
      SETTLES_NOTHING,
      SAME_JUDGES_ASKED_AGAIN,
    ],
    refused: [
      NONE_NAMED_WORDING_STAYS,
      SOME_NAMED_PREFERRED_SHIPS,
      SHIPS_ONLY_ON_AN_EQUAL_TIE,
      SENT_BACK_FOR_NEW_PROPOSALS,
    ],
  },
  defers: {
    required: [
      NOT_A_KEEP,
      OWN_DECLINE_NAMES_NONE,
      SHIPS_ONLY_ON_AN_EQUAL_TIE,
      FIXED_ORDER,
      SENT_BACK_FOR_NEW_PROPOSALS,
    ],
    refused: [
      NONE_NAMED_WORDING_STAYS,
      SOME_NAMED_PREFERRED_SHIPS,
      SAME_JUDGES_ASKED_AGAIN,
    ],
  },
};

//endregion Decline clause rounds
