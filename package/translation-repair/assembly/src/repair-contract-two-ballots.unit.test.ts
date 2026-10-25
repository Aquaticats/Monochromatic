/**
 Tests for the producer-roster guard once the selection minimum is a count of two ballots and no longer a weight.

 THE OWNER'S RULINGS OF 2026-10-25, quoted where the rule is stated
 (`doc/design-commitments-votes-repairs-and-run-offs.md`): "floor of 3 - don't set a floor." and "Every weight
 floor". A round is decided by two or more answering judges, and a producer's vote for its own text is one answering
 judge though it weighs half toward which candidate leads. A roster is therefore refused only when it cannot seat two
 judges; before the change it was refused when the best candidate could draw less than a weight of two, which refused
 two authors judging only each other and one author judged by itself and one other model.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  assertJudgeableProducerRoster,
  ProducerRosterError,
} from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

/**
 First author, also a judge in most cases here.
 */
const PRODUCER_ONE = SEAT_HYPER_OPENROUTER_VISION_EDITOR;

/**
 Second author.
 */
const PRODUCER_TWO = SEAT_SYNTHETIC_VISION_WITHHELD;

/**
 Judge with no stake in either author's output.
 */
const JUDGE_ONE = SEAT_SYNTHETIC_VISION_NO_OPENROUTER;

await describe({
  name: assertJudgeableProducerRoster.name,
  children: [
    it({
      name: 'ACCEPTS two authors judging only each other, since either one\'s text can draw two answering judges: '
        + 'the other author in full and itself at half',
      fn: async () => {
        expect(function acceptTwoAuthors() {
          assertJudgeableProducerRoster({
            producerModelIds: [PRODUCER_ONE, PRODUCER_TWO,],
            judgeModelIds: [PRODUCER_ONE, PRODUCER_TWO,],
            role: 'editor',
          },);
        },).not.toThrow();
      },
    },),

    it({
      name: 'ACCEPTS one author judged by itself and one other model, which can name its text with two judges',
      fn: async () => {
        expect(function acceptAuthorAndOne() {
          assertJudgeableProducerRoster({
            producerModelIds: [PRODUCER_ONE,],
            judgeModelIds: [PRODUCER_ONE, JUDGE_ONE,],
            role: 'editor',
          },);
        },).not.toThrow();
      },
    },),

    it({
      name: 'REFUSES one judge alone, the positive control: one judge alone never decides, whoever wrote the text',
      fn: async () => {
        expect(function refuseOneJudge() {
          assertJudgeableProducerRoster({
            producerModelIds: [PRODUCER_ONE,],
            judgeModelIds: [JUDGE_ONE,],
            role: 'editor',
          },);
        },).toThrow(ProducerRosterError,);
      },
    },),
  ],
},);
