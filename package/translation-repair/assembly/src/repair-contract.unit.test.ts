/**
 Tests for the producer-roster independence guard.

 `assertJudgeableProducerRoster` refuses a roster that could not decide a
 round however its judges voted, and NOTHING ELSE. By the user ruling of
 2026-08-15 self-judging is allowed at reduced weight, so a roster where every
 model both produces and judges is legal; what is not legal is a roster too
 small to draw the ballots a winner needs, since one judge contributes at
 most one ballot. No weight is compared since the owner's rulings of
 2026-10-25 ("floor of 3 - don't set a floor.", "Every weight floor").

 The cases that used to assert an independence requirement are inverted here
 on purpose: they now assert that the same rosters are ACCEPTED. An earlier
 version of this file recorded the opposite policy, and reading them side by
 side is the clearest statement of what changed.

 `assertJudgeableEditorRoster` delegates here and is covered through the
 editor ensemble, so the arithmetic branches already run. What was never
 exercised is the `role` parameter and the second caller that uses it: the
 naturalness lane passes `refiner`, and if `role` were ever dropped a refiner
 roster failure would report itself as an editor failure and send whoever
 reads it to the wrong configuration.

 Model ids are real catalog entries because `RosterModelId` is a closed
 union.

 @module
 */

import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { MIN_SELECTION_BALLOTS, } from '@monochromatic-dev/translation-repair-house-rule';
import {
  assertJudgeableEditorRoster,
  assertJudgeableProducerRoster,
  ProducerRosterError,
} from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_VISION,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

/**
 Model that produces candidates in most cases in this file.
 */
const PRODUCER_ONE = SEAT_HYPER_OPENROUTER_VISION_EDITOR;

/**
 Second producer, for repeat and overlap cases.
 */
const PRODUCER_TWO = SEAT_SYNTHETIC_VISION_WITHHELD;

/**
 Judge with no stake in either producer's output.
 */
const JUDGE_ONE = SEAT_SYNTHETIC_VISION_NO_OPENROUTER;

/**
 Second disinterested judge, so a roster can meet the vote minimum.
 */
const JUDGE_TWO = SEAT_SYNTHETIC_TEXT_EVERYWHERE;

/**
 Third disinterested judge, for rosters that must exceed the minimum.
 */
const JUDGE_THREE = SEAT_HYPER_VISION;

/**
 Attempts a one-producer roster judged by that producer and one disinterested
 judge, which draws one full ballot and one half ballot: two answering judges.

 @example
 ```ts
 expect(acceptAuthorAndOne,).not.toThrow();
 ```
 */
function acceptAuthorAndOne(): void {
  assertJudgeableProducerRoster({
    producerModelIds: [PRODUCER_ONE,],
    judgeModelIds: [
      PRODUCER_ONE,
      JUDGE_ONE,
    ],
    role: 'editor',
  },);
}

/**
 Attempts a one-producer roster judged by one model alone, which can never
 draw the two ballots a winner needs.

 @throws ProducerRosterError naming the judges the roster seats and the ballots it falls short of

 @example
 ```ts
 expect(refuseOneJudge,).toThrow(ProducerRosterError,);
 ```
 */
function refuseOneJudge(): void {
  assertJudgeableProducerRoster({
    producerModelIds: [PRODUCER_TWO,],
    judgeModelIds: [JUDGE_ONE,],
    role: 'editor',
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: assertJudgeableProducerRoster.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'accepts a roster leaving exactly the minimum disinterested '
            + 'judges, since the minimum is a floor rather than something to '
            + 'exceed',
          fn: async () => {
            expect(function acceptMinimum() {
              assertJudgeableProducerRoster({
                producerModelIds: [PRODUCER_ONE,],
                judgeModelIds: [
                  PRODUCER_ONE,
                  JUDGE_ONE,
                  JUDGE_TWO,
                ],
                role: 'editor',
              },);
            },).not.toThrow();
          },
        },),

        it({
          name: 'ACCEPTS a roster whose judges are mostly producers, which the '
            + 'old guard refused. This is the ruling of 2026-08-15 in one case: a '
            + 'model grading its own work is a discounted opinion, not a '
            + 'forbidden one, so a provider degraded for the day cannot make a '
            + 'legal roster illegal',
          fn: async () => {
            expect(function acceptMostlyProducers() {
              assertJudgeableProducerRoster({
                producerModelIds: [
                  PRODUCER_ONE,
                  PRODUCER_TWO,
                ],
                judgeModelIds: [
                  PRODUCER_ONE,
                  PRODUCER_TWO,
                  JUDGE_ONE,
                ],
                role: 'editor',
              },);
            },).not.toThrow();
          },
        },),

        it({
          name: 'ACCEPTS a roster where EVERY judge produced, which is the widest '
            + 'case the ruling allows and the one a full-roster bench needs. '
            + 'Nothing here says such a round will decide anything: if every judge '
            + 'backs its own candidate each draws half a vote and the incumbent '
            + 'survives, which is the weights doing the work this guard used to',
          fn: async () => {
            expect(function acceptFullOverlap() {
              assertJudgeableProducerRoster({
                producerModelIds: [
                  PRODUCER_ONE,
                  PRODUCER_TWO,
                  JUDGE_ONE,
                  JUDGE_TWO,
                ],
                judgeModelIds: [
                  PRODUCER_ONE,
                  PRODUCER_TWO,
                  JUDGE_ONE,
                  JUDGE_TWO,
                ],
                role: 'editor',
              },);
            },).not.toThrow();
          },
        },),

        it({
          name: 'accepts THREE authors judging only each other, because a candidate '
            + 'one of them wrote draws half a vote from its author and a full one '
            + 'from each of the other two. An earlier version of this guard measured '
            + 'the collapse case instead, treating all three as stakeholders in one '
            + 'candidate, and refused a bench that decides comfortably whenever they '
            + 'disagree',
          fn: async () => {
            expect(function acceptThreeAuthors() {
              assertJudgeableProducerRoster({
                producerModelIds: [
                  PRODUCER_ONE,
                  PRODUCER_TWO,
                  JUDGE_ONE,
                ],
                judgeModelIds: [
                  PRODUCER_ONE,
                  PRODUCER_TWO,
                  JUDGE_ONE,
                ],
                role: 'editor',
              },);
            },).not.toThrow();
          },
        },),

        it({
          name: 'ACCEPTS two authors judging only each other, which the weight arithmetic refused '
            + 'until 2026-10-25: whichever of them wrote a candidate is discounted on it, '
            + 'but its text still draws two answering judges and two ballots decide',
          fn: async () => {
            expect(function acceptTwoAuthorsJudgingEachOther() {
              assertJudgeableProducerRoster({
                producerModelIds: [
                  PRODUCER_ONE,
                  PRODUCER_TWO,
                ],
                judgeModelIds: [
                  PRODUCER_ONE,
                  PRODUCER_TWO,
                ],
                role: 'editor',
              },);
            },).not.toThrow();
          },
        },),

        it({
          name: 'counts DISTINCT judges, so one model listed twice is one voice: '
            + 'a duplicated judge id would otherwise count as two answering judges with '
            + 'a single model deciding the stage, which is the exact outcome this '
            + 'guard exists to prevent',
          fn: async () => {
            expect(function refuseDuplicatedJudge() {
              assertJudgeableProducerRoster({
                producerModelIds: [PRODUCER_ONE,],
                judgeModelIds: [
                  PRODUCER_ONE,
                  JUDGE_ONE,
                  JUDGE_ONE,
                ],
                role: 'editor',
              },);
            },).toThrow(ProducerRosterError,);
          },
        },),

        it({
          name: 'refuses a producer roster with a repeat, since the same model '
            + 'twice is one voice pretending to be an ensemble',
          fn: async () => {
            expect(function refuseRepeatedProducer() {
              assertJudgeableProducerRoster({
                producerModelIds: [
                  PRODUCER_ONE,
                  PRODUCER_ONE,
                ],
                judgeModelIds: [
                  JUDGE_ONE,
                  JUDGE_TWO,
                  JUDGE_THREE,
                ],
                role: 'editor',
              },);
            },).toThrow(ProducerRosterError,);
          },
        },),

        it({
          name: 'refuses an EMPTY producer roster, because a stage with no '
            + 'producers has nothing to judge and would otherwise pass on the '
            + 'technicality that no judge has a stake',
          fn: async () => {
            expect(function refuseEmptyProducers() {
              assertJudgeableProducerRoster({
                producerModelIds: [],
                judgeModelIds: [
                  JUDGE_ONE,
                  JUDGE_TWO,
                  JUDGE_THREE,
                ],
                role: 'editor',
              },);
            },).toThrow(ProducerRosterError,);
          },
        },),

        it({
          name: 'NAMES THE ROLE in the failure, so the naturalness lane reports a '
            + 'refiner roster rather than an editor one and whoever reads it opens '
            + 'the right configuration',
          fn: async () => {
            /**
             What refuseRefinerRoster raised, read for its class as well as its wording.
             */
            const refusalOfRefuseRefinerRoster = caught(function refuseRefinerRoster() {
              assertJudgeableProducerRoster({
                producerModelIds: [PRODUCER_ONE,],
                judgeModelIds: [JUDGE_ONE,],
                role: 'refiner',
              },);
            },);

            expect(refusalOfRefuseRefinerRoster,).toBeInstanceOf(ProducerRosterError,);
            expect((refusalOfRefuseRefinerRoster as Error).message,).toContain('refiner',);
          },
        },),

        it({
          name: 'names both rosters in the failure, so the message is actionable '
            + 'without reopening the run configuration to find out who was on them',
          fn: async () => {
            expect(function refuseAndReport() {
              assertJudgeableProducerRoster({
                producerModelIds: [PRODUCER_ONE,],
                judgeModelIds: [JUDGE_ONE,],
                role: 'editor',
              },);
            },).toThrow(PRODUCER_ONE,);
          },
        },),

        it({
          name: 'seats judges by the BALLOTS A WINNER NEEDS rather than by a written '
            + 'count, and refuses one seat below it. One judge can contribute at '
            + 'most one ballot, so a roster below that floor declines '
            + 'every round while looking like a pipeline that simply found '
            + 'nothing to change',
          fn: async () => {
            /**
             Judges available to draw from, sliced against the constant so this
             case follows the floor if it ever moves.
             */
            const pool = [
              JUDGE_ONE,
              JUDGE_TWO,
              JUDGE_THREE,
            ] as const;

            /**
             Seats the rule requires: one per ballot a winner needs.
             */
            const seats = MIN_SELECTION_BALLOTS;

            expect(function refuseBelowSeats() {
              assertJudgeableProducerRoster({
                producerModelIds: [PRODUCER_ONE,],
                judgeModelIds: pool.slice(
                  0,
                  seats - 1,
                ),
                role: 'editor',
              },);
            },).toThrow(ProducerRosterError,);

            expect(function acceptAtSeats() {
              assertJudgeableProducerRoster({
                producerModelIds: [PRODUCER_ONE,],
                judgeModelIds: pool.slice(
                  0,
                  seats,
                ),
                role: 'editor',
              },);
            },).not.toThrow();
          },
        },),

        it({
          name: 'ACCEPTS one producer judged by itself and one other model, which '
            + 'the weight arithmetic refused until 2026-10-25: its text can draw two '
            + 'answering judges, the other in full and the author at half',
          fn: async () => {
            expect(acceptAuthorAndOne,).not.toThrow();
          },
        },),

        it({
          name: 'names the COUNT in the refusal of one judge alone, since the remedy differs by '
            + 'fault: a roster short of judges needs another judge, while a '
            + 'repeated id needs one removed, and a message covering both sends '
            + 'whoever reads it to the wrong configuration',
          fn: async () => {
            /**
             What refuseOneJudge raised, read for its class as well as its wording.
             */
            const refusalOfRefuseOneJudge = caught(refuseOneJudge,);

            expect(refusalOfRefuseOneJudge,).toBeInstanceOf(ProducerRosterError,);
            expect((refusalOfRefuseOneJudge as Error).message,).toContain('judges seated: 1; ballots a winner needs: 2',);
          },
        },),
      ],
    },),

    describe({
      name: assertJudgeableEditorRoster.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'reports itself as the EDITOR roster, which is what makes the '
            + 'shared guard readable at two call sites: the same arithmetic fails '
            + 'under two different names',
          fn: async () => {
            /**
             What refuseEditorRoster raised, read for its class as well as its wording.
             */
            const refusalOfRefuseEditorRoster = caught(function refuseEditorRoster() {
              assertJudgeableEditorRoster({
                editorModelIds: [PRODUCER_ONE,],
                judgeModelIds: [JUDGE_ONE,],
              },);
            },);

            expect(refusalOfRefuseEditorRoster,).toBeInstanceOf(ProducerRosterError,);
            expect((refusalOfRefuseEditorRoster as Error).message,).toContain('editor',);
          },
        },),

        it({
          name: 'accepts the same roster the shared guard accepts, so delegation '
            + 'has not narrowed what the editor stage allows',
          fn: async () => {
            expect(function acceptEditorRoster() {
              assertJudgeableEditorRoster({
                editorModelIds: [PRODUCER_ONE,],
                judgeModelIds: [
                  PRODUCER_ONE,
                  JUDGE_ONE,
                  JUDGE_TWO,
                ],
              },);
            },).not.toThrow();
          },
        },),
      ],
    },),
  ],
},);
