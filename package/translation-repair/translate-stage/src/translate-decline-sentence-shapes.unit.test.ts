/**
 Guards the sentence a judge reads about a declined round against every shape a declined round can have (re-check 6,
 M-B).

 The sentence said that when a tie leaves only some candidates at the top, or a leader only one judge named, the panel
 is asked again first over those candidates, or over the leader and its runner-up. That is true only where some
 candidates are left behind. `runoffFinalists` returns the whole slate, and `settleAbsentDecline` does not narrow, for a
 round in which every judge declined, for a leader alone in drawing a ballot, for a whole slate tied, and for a slate of
 only two, where the leader and its runner-up are the whole slate. Each case here scripts judges so the first ask comes
 to one shape, renders the real sheets of the first ask and of the challenge through `judgeSlateWithRetry`, reads how
 many candidates each round was put to, and requires the sentence to say what that shape did: narrowed, or not.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import { judgeSlateWithRetry, } from '../dist/final/node/index.mjs';
import type { RunoffShape, } from '@monochromatic-dev/translation-repair-adjudication/ts/decline-consequence-clauses.test-fixture.ts';
import {
  candidatesIn,
  DOZING,
  failuresOf,
  JUDGES,
  judgedOver,
  NAPPING,
  PAIR,
  QUARTET,
  sheetOfRound,
  slateOf,
} from './translate-decline-judging.test-fixture.ts';

/**
 One shape of declined round and the judging that comes to it.
 */
type ShapeCase = {
  /**
   What the first ask comes to.
   */
  readonly shape: RunoffShape;

  /**
   How many candidates the slate holds.
   */
  readonly width: typeof PAIR | typeof QUARTET;

  /**
   Whether the second candidate was written by the second judge, so that its vote for it weighs half.
   */
  readonly secondIsJudgesOwn: boolean;

  /**
   The text each judge votes for, the judges it leaves out declining.
   */
  readonly plan: ReadonlyMap<RosterModelId, string>;

  /**
   How many candidates the challenge is put to.
   */
  readonly challengeIsOver: typeof PAIR | typeof QUARTET;
};

/**
 Judge at a position of the bench.

 @param position - position, from zero

 @returns Judge

 @example
 ```ts
 const first = judgeAt(0,);
 ```
 */
function judgeAt(position: number,): RosterModelId {
  return nonNullishOrThrow(JUDGES[position],);
}

/**
 The shapes, each with the judging that comes to it and what the challenge is then put to.
 */
const SHAPE_CASES: readonly ShapeCase[] = [
  {
    shape: 'tied-at-top',
    width: QUARTET,
    secondIsJudgesOwn: false,
    plan: new Map([
      [
        judgeAt(0,),
        DOZING,
      ],
      [
        judgeAt(1,),
        NAPPING,
      ],
    ],),
    challengeIsOver: PAIR,
  },
  {
    shape: 'leader-and-runner-up',
    width: QUARTET,
    secondIsJudgesOwn: true,
    plan: new Map([
      [
        judgeAt(0,),
        DOZING,
      ],
      [
        judgeAt(1,),
        NAPPING,
      ],
    ],),
    challengeIsOver: PAIR,
  },
  {
    shape: 'whole-slate-tied',
    width: PAIR,
    secondIsJudgesOwn: false,
    plan: new Map([
      [
        judgeAt(0,),
        DOZING,
      ],
      [
        judgeAt(1,),
        NAPPING,
      ],
    ],),
    challengeIsOver: PAIR,
  },
  {
    shape: 'every-judge-declined',
    width: QUARTET,
    secondIsJudgesOwn: false,
    plan: new Map(),
    challengeIsOver: QUARTET,
  },
  {
    shape: 'leader-alone',
    width: QUARTET,
    secondIsJudgesOwn: false,
    plan: new Map([
      [
        judgeAt(0,),
        DOZING,
      ],
    ],),
    challengeIsOver: QUARTET,
  },
  {
    shape: 'slate-of-two',
    width: PAIR,
    secondIsJudgesOwn: true,
    plan: new Map([
      [
        judgeAt(0,),
        DOZING,
      ],
      [
        judgeAt(1,),
        NAPPING,
      ],
    ],),
    challengeIsOver: PAIR,
  },
];

/**
 Reads one shape of declined round for one kind of caller: the candidates each round was put to, and the sentences of
 the first ask and of the challenge against the clause of the shape.

 @param shapeCase - shape and judging that comes to it

 @param shipPastDecline - whether the caller ships past a decline in its challenge

 @example
 ```ts
 await readsShapeFor({ shapeCase: SHAPE_CASES[0], shipPastDecline: true, },);
 ```
 */
async function readsShapeFor(
  {
    shapeCase,
    shipPastDecline,
  }: {
    readonly shapeCase: ShapeCase;
    readonly shipPastDecline: boolean;
  },
): Promise<void> {
  const { shape, } = shapeCase;
  /**
   Every sheet the judging sent.
   */
  const sheets: string[] = [];
  await judgedOver({
    sheets,
    candidates: slateOf({
      width: shapeCase.width,
      secondIsJudgesOwn: shapeCase.secondIsJudgesOwn,
    },),
    plan: shapeCase.plan,
    shipPastDecline,
  },);
  expect([
    0,
    1,
  ].map(function candidatesAskedIn(round,): number {
    return candidatesIn({
      sheet: sheetOfRound({
        sheets,
        round,
      },),
    },).length;
  },),).toEqual([
    shapeCase.width,
    shapeCase.challengeIsOver,
  ],);
  expect(failuresOf({
    sheets,
    mode: 'asked-again',
    round: 0,
    shape,
  },),).toEqual([],);
  expect(failuresOf({
    sheets,
    mode: shipPastDecline ? 'ships' : 'defers',
    round: 1,
    shape,
  },),).toEqual([],);
}

/**
 Reads one shape of declined round for a caller that ships past a decline and for one that defers.

 @param shape - what the first ask comes to

 @example
 ```ts
 await readsShape({ shape: 'slate-of-two', },);
 ```
 */
async function readsShape({ shape, }: { readonly shape: RunoffShape; },): Promise<void> {
  /**
   The shape and the judging that comes to it.
   */
  const shapeCase = nonNullishOrThrow(SHAPE_CASES.find(function named(candidate,): boolean {
    return candidate.shape === shape;
  },),);
  await Promise.all([
    readsShapeFor({
      shapeCase,
      shipPastDecline: true,
    },),
    readsShapeFor({
      shapeCase,
      shipPastDecline: false,
    },),
  ],);
}

await describe({
  name: judgeSlateWithRetry.name,
  children: [
    it({
      name: 'SAYS WHAT A TIE AT THE TOP THAT LEAVES SOME CANDIDATES BEHIND DID: the challenge is put to 2 of 4 '
        + 'candidates, and the sentences of the first ask and of both kinds of challenge say it narrows',
      fn: async () => {
        await readsShape({ shape: 'tied-at-top', },);
      },
    },),

    it({
      name: 'SAYS WHAT A LEADER ONE JUDGE NAMED BESIDE A RUNNER-UP WITH SOME CANDIDATES BEHIND DID: the challenge is '
        + 'put to 2 of 4 candidates, and the sentences of the first ask and of both kinds of challenge say it narrows',
      fn: async () => {
        await readsShape({ shape: 'leader-and-runner-up', },);
      },
    },),

    it({
      name: 'SAYS WHAT A WHOLE SLATE TIED DID: the challenge is put to 2 of 2 candidates, and the sentences of the '
        + 'first ask and of both kinds of challenge say it narrows nothing',
      fn: async () => {
        await readsShape({ shape: 'whole-slate-tied', },);
      },
    },),

    it({
      name: 'SAYS WHAT A ROUND IN WHICH EVERY JUDGE DECLINED DID: the challenge is put to 4 of 4 candidates, and the '
        + 'sentences of the first ask and of both kinds of challenge say it narrows nothing',
      fn: async () => {
        await readsShape({ shape: 'every-judge-declined', },);
      },
    },),

    it({
      name: 'SAYS WHAT A LEADER ALONE IN DRAWING A BALLOT DID: the challenge is put to 4 of 4 candidates, and the '
        + 'sentences of the first ask and of both kinds of challenge say it narrows nothing (re-check 6, M-B)',
      fn: async () => {
        await readsShape({ shape: 'leader-alone', },);
      },
    },),

    it({
      name: 'SAYS WHAT A LEADER BESIDE ITS RUNNER-UP IN A SLATE OF TWO DID: the challenge is put to 2 of 2 candidates, '
        + 'and the sentences of the first ask and of both kinds of challenge say it narrows nothing (re-check 6, M-B)',
      fn: async () => {
        await readsShape({ shape: 'slate-of-two', },);
      },
    },),
  ],
},);
