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

import { nonNullishOrThrow, } from '@monochromatic-dev/module-or-throw/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

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
  readonly width: 2 | 4;

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
  readonly challengeIsOver: number;
};

/**
 Judge at a position of the bench.

 @param position - position, from zero

 @returns The judge

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
    width: 4,
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
    challengeIsOver: 2,
  },
  {
    shape: 'leader-and-runner-up',
    width: 4,
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
    challengeIsOver: 2,
  },
  {
    shape: 'whole-slate-tied',
    width: 2,
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
    challengeIsOver: 2,
  },
  {
    shape: 'every-judge-declined',
    width: 4,
    secondIsJudgesOwn: false,
    plan: new Map(),
    challengeIsOver: 4,
  },
  {
    shape: 'leader-alone',
    width: 4,
    secondIsJudgesOwn: false,
    plan: new Map([
      [
        judgeAt(0,),
        DOZING,
      ],
    ],),
    challengeIsOver: 4,
  },
  {
    shape: 'slate-of-two',
    width: 2,
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
    challengeIsOver: 2,
  },
];

/**
 The case that reads one shape in one kind of caller: the candidates each round was put to, and the sentence of the
 first ask and of the challenge against the clause of the shape.

 @param shapeCase - the shape and the judging that comes to it

 @param shipPastDecline - whether the caller ships past a decline in its challenge

 @returns The test case

 @example
 ```ts
 const children = [shapeIt({ shapeCase: SHAPE_CASES[0], shipPastDecline: true, },),];
 ```
 */
function shapeIt(
  {
    shapeCase,
    shipPastDecline,
  }: {
    readonly shapeCase: ShapeCase;
    readonly shipPastDecline: boolean;
  },
) {
  const { shape, } = shapeCase;
  return it({
    name: `SAYS WHAT A ROUND THAT CAME TO ${shape.toUpperCase()} DID: the challenge is put to `
      + `${String(shapeCase.challengeIsOver,)} of ${String(shapeCase.width,)} candidates, and the sentences of the `
      + `first ask and of the challenge ${shipPastDecline ? 'that ships past a decline' : 'that defers'} say so`,
    fn: async () => {
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
    },
  },);
}

await describe({
  name: judgeSlateWithRetry.name,
  children: SHAPE_CASES.flatMap(function bothCallers(shapeCase,) {
    return [
      shapeIt({
        shapeCase,
        shipPastDecline: true,
      },),
      shapeIt({
        shapeCase,
        shipPastDecline: false,
      },),
    ];
  },),
},);
