/**
 The clauses of the sentence that tells a judge what a decline does over wording that cannot ship, when the round is
 the one that ships or keeps (re-check 3, N1 and N7; re-check 4, F1 and F2; re-check 5, M-B, M-D and N-a), and the
 clauses all three rounds share about the run-off. The clauses that belong to the other rounds of a pair are in
 `decline-clause-rounds.test-fixture.ts`; the reader that applies them is `decline-consequence-clauses.test-fixture.ts`.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

//region Decline clause
// One entry per thing the sentence must say, in the order a judge meets them.

/**
 Words that turn an outcome the other way, which an outcome clause refuses so that "never stays" and "does not ship"
 are not read as the stay and the ship (re-check 5, N-a). This is the whole of the negation the reader refuses: a
 negation put in other words ("fails to ship", "is unable to stay") is not read, and each such shape wants its own
 refused phrase.
 */
export const NEGATIONS: readonly string[] = [
  'never',
  'does not',
  'will not',
  'cannot',
  'no longer',
];

/**
 The ways a sentence says that no narrower question is left, which the outcomes of a challenge wait on: the challenge
 asks the panel again over the leaders while a tie narrows the question, and ships or keeps only once none does.
 */
export const NO_NARROWER_QUESTION_LEFT: readonly string[] = [
  'no such question is left',
  'no narrower question is left',
  'nothing is left to narrow',
];

/**
 The round a sentence is told in: the first ask of a pair, a challenge whose caller ships past a decline, and a
 challenge whose caller defers to a later production.
 */
export type DeclineMode = 'asked-again' | 'ships' | 'defers';

/**
 One thing the sentence must say.

 @example
 ```ts
 const clause: DeclineClause = { meaning: 'a decline names no candidate', inOrder: [['own decline',], ['no candidate',],], forbidden: [], };
 ```
 */
export type DeclineClause = {
  /**
   What the clause tells a judge, as it is named in a failure.
   */
  readonly meaning: string;

  /**
   The slots in the order a judge reads them, each the ways the idea can be put, in lower case.
   */
  readonly inOrder: readonly (readonly string[])[];

  /**
   Phrases the part must not carry, in lower case: each is the other outcome or the other meaning the clause would
   otherwise let through, so a part that says both is not a part that says this.
   */
  readonly forbidden: readonly string[];
};

/**
 Declining alone does not keep the wording, and the part that says so may not say that it keeps.

 THIS IS WHAT KEEPS A JUDGE FROM READING ITS DECLINE AS THE WAY TO THE STAY.

 @example
 ```ts
 const lead = NOT_A_KEEP.meaning;
 ```
 */
export const NOT_A_KEEP: DeclineClause = {
  meaning: 'declining does not by itself keep the wording in place',
  inOrder: [
    ['declin',],
    [
      'does not',
      'will not',
      'is not',
    ],
    [
      'by itself',
      'alone',
      'on its own',
      'quiet',
    ],
    ['keep',],
  ],
  forbidden: [
    'keeps',
    'keeping',
    'does keep',
    'will keep',
  ],
};

/**
 The same decline names no candidate, so it adds to no candidate's count.

 @example
 ```ts
 const own = OWN_DECLINE_NAMES_NONE.meaning;
 ```
 */
export const OWN_DECLINE_NAMES_NONE: DeclineClause = {
  meaning: 'a judge\'s own decline names no candidate',
  inOrder: [
    ['own decline',],
    [
      'no candidate',
      'not name a candidate',
      'names none',
    ],
  ],
  forbidden: [],
};

/**
 None named by two or more judges leaves the wording in place and the page reports it, and may not say the preferred
 ships.

 @example
 ```ts
 const stay = NONE_NAMED_WORDING_STAYS.meaning;
 ```
 */
export const NONE_NAMED_WORDING_STAYS: DeclineClause = {
  meaning: 'no candidate named by two or more judges: the wording in place stays and the page reports it',
  inOrder: [
    ['no candidate',],
    [
      'named by two or more judges',
      'named by two judges or more',
    ],
    [
      'stays',
      'remains',
    ],
    ['report',],
  ],
  forbidden: [
    'preferred',
    'ships',
    'not stay',
    ...NEGATIONS,
  ],
};

/**
 One or more named by two or more judges ships the preferred of those, and may not say none was named, that the wording
 stays, or that the ship waits on an equal weight.

 @example
 ```ts
 const ship = SOME_NAMED_PREFERRED_SHIPS.meaning;
 ```
 */
export const SOME_NAMED_PREFERRED_SHIPS: DeclineClause = {
  meaning: 'one or more candidates named by two or more judges: the preferred of those ships',
  inOrder: [
    [
      'named by two or more judges',
      'named by two judges or more',
    ],
    ['preferred',],
    [
      'ships',
      'ship',
    ],
  ],
  forbidden: [
    'no candidate',
    'none',
    'stays',
    'remains',
    'only',
    'every candidate',
    'not ship',
    ...NEGATIONS,
  ],
};

/**
 The fixed order, the behaviour of `rankedByPreference` in `translate-stage/src/translate-runoff-tie.ts`.

 @example
 ```ts
 const order = FIXED_ORDER.meaning;
 ```
 */
export const FIXED_ORDER: DeclineClause = {
  meaning: 'the preferred ships by a fixed order, the repair lane first, then the translate lane, then the slate',
  inOrder: [
    ['fixed order',],
    ['repair lane',],
    ['translate lane',],
    ['first on the slate',],
  ],
  forbidden: [],
};

/**
 The same stay, once no narrower question is left: a challenge asks the panel again over the leaders first.

 @example
 ```ts
 const stay = NONE_NAMED_STAYS_ONCE_NARROWED.meaning;
 ```
 */
export const NONE_NAMED_STAYS_ONCE_NARROWED: DeclineClause = {
  meaning: 'once no narrower question is left, with no candidate named by two or more judges '
    + 'the wording in place stays',
  inOrder: [
    NO_NARROWER_QUESTION_LEFT,
    ...NONE_NAMED_WORDING_STAYS.inOrder,
  ],
  forbidden: NONE_NAMED_WORDING_STAYS.forbidden,
};

/**
 The same ship, once no narrower question is left.

 @example
 ```ts
 const ship = SOME_NAMED_SHIPS_ONCE_NARROWED.meaning;
 ```
 */
export const SOME_NAMED_SHIPS_ONCE_NARROWED: DeclineClause = {
  meaning: 'once no narrower question is left, with one or more candidates named by two or more judges '
    + 'the preferred ships',
  inOrder: [
    NO_NARROWER_QUESTION_LEFT,
    ...SOME_NAMED_PREFERRED_SHIPS.inOrder,
  ],
  forbidden: SOME_NAMED_PREFERRED_SHIPS.forbidden,
};

/**
 The panel is asked again, and a judge is not told it will be the same members: the window of seats a round asks
 starts where an FNV hash of the sheet puts it (`stage-fanout-window.ts`), and the sheet of a challenge differs from
 the sheet of the first ask (re-check 5, M-D).

 @example
 ```ts
 const again = PANEL_ASKED_AGAIN.meaning;
 ```
 */
export const PANEL_ASKED_AGAIN: DeclineClause = {
  meaning: 'the panel is asked again, though a round may seat other members of it',
  inOrder: [
    ['panel',],
    [
      'asked again',
      'asked once more',
    ],
    [
      'other members',
      'different members',
    ],
  ],
  forbidden: [],
};

/**
 A tie at the top that backs only some candidates, or a leader only one judge named, sends those candidates, or the
 leader and its runner-up, to the panel first, for as long as that narrows the question (`runoffFinalists` in
 `translate-stage/src/translate-runoff.ts`, `judgeSlateWithRetry` in `translate-stage/src/translate-retry.ts`;
 re-check 5, M-B). The part that says it may not say an outcome.

 @example
 ```ts
 const runoff = RUNS_OFF_OVER_THE_LEADERS.meaning;
 ```
 */
export const RUNS_OFF_OVER_THE_LEADERS: DeclineClause = {
  meaning: 'a tie at the top backing only some candidates, or a leader one judge named, narrows the question to the '
    + 'leaders, for as long as it narrows',
  inOrder: [
    [
      'tied at the top',
      'tied for the lead',
    ],
    ['one judge',],
    ['runner-up',],
    ['for as long as',],
    ['narrows',],
  ],
  forbidden: [
    'ships',
    'ship ',
    'stays',
    'remains',
  ],
};

//endregion Decline clause
