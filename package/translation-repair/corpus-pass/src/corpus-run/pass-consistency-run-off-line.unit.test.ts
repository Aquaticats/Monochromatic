/**
 Tests for the one log line that reports a run-off's final count (the lead's ruling of 2026-10-15): the usable
 choices of those heard, the quorum, and the winner or the reason for none.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { ArtifactConsistencyRunOff, } from '@monochromatic-dev/translation-repair-artifact';
import { runOffResultLine, } from '../../dist/final/node/index.mjs';

/**
 A run-off of four choices, all usable.
 */
const RUN_OFF: ArtifactConsistencyRunOff = {
  finalists: ['OCCURRENCE 2', 'OCCURRENCE 3',],
  ballots: [
    { seat: 'qwen', reason: 'r', choice: 'OCCURRENCE 3', },
    { seat: 'kimi', reason: 'r', choice: 'OCCURRENCE 3', },
    { seat: 'minimax', reason: 'r', choice: 'OCCURRENCE 3', },
    { seat: 'deepseek', reason: 'r', choice: 'OCCURRENCE 2', },
  ],
  usable: 4,
  quorum: 2,
};

await describe({
  name: runOffResultLine.name,
  concurrency: DEFAULT_CONCURRENCY,
  children: [
    it({
      name: 'NAMES THE USABLE CHOICES, THE CHOICES HEARD, THE QUORUM AND THE WINNER of a run-off that chose a place',
      fn: async () => {
        expect(runOffResultLine({ runOff: RUN_OFF, winner: { kind: 'chosen', label: 'OCCURRENCE 3', }, },),).toBe(
          'the run-off counted 4 usable of 4 heard (quorum 2): OCCURRENCE 3 won',
        );
      },
    },),
    it({
      name: 'NAMES THE REASON FOR NO WINNER, and counts a choice that could not be counted as heard but not usable',
      fn: async () => {
        expect(
          runOffResultLine({
            runOff: { ...RUN_OFF, ballots: RUN_OFF.ballots.slice(0, 3,), usable: 2, },
            winner: { kind: 'none', reason: 'run-off-tied', },
          },),
        ).toBe('the run-off counted 2 usable of 3 heard (quorum 2): no winner (run-off-tied)',);
      },
    },),
  ],
},);
