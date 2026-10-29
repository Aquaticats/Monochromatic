/**
 Tests for which candidates a carry choice runs off (the lead's adoption of 2026-10-15 of the owner's rule of 2026-10-11
 for this ask, open to the owner's veto): a choice that reached its quorum and gave no strict majority runs off among the
 candidates tied for first, and, when one candidate leads alone, among that candidate and the candidates with the
 second-highest count, as the first round runs off among places.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  carryRunOffFinalistsOf,
  type ConsistencyChoiceVoice,
  runOffChoiceFinalistsOf,
} from '../dist/final/node/index.mjs';

/**
 Labels a choice among five candidates showed.
 */
const LABELS = ['CANDIDATE 1', 'CANDIDATE 2', 'CANDIDATE 3', 'CANDIDATE 4', 'CANDIDATE 5',] as const;

/**
 Builds the heard choices of seats that wrote the given labels.

 @param choices - what each seat wrote, in seat order

 @returns Heard choices

 @example
 ```ts
 const voices = votesFor({ choices: ['CANDIDATE 1', '2',], },);
 ```
 */
function votesFor({ choices, }: { readonly choices: readonly string[]; },): readonly ConsistencyChoiceVoice[] {
  return choices.map(function voiceOf(
    choice,
    position,
  ): ConsistencyChoiceVoice {
    return { seat: `seat-${String(position,)}`, reason: 'It reads best.', choice, };
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: runOffChoiceFinalistsOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES THE LABELS TIED FOR FIRST, in the order of the sheet, when no label has more than half of the usable choices',
          fn: async () => {
            expect([
              runOffChoiceFinalistsOf({ voices: votesFor({ choices: ['CANDIDATE 3', 'CANDIDATE 5', 'CANDIDATE 1',], },), labels: LABELS, },),
              runOffChoiceFinalistsOf({
                voices: votesFor({ choices: ['CANDIDATE 2', 'CANDIDATE 1', 'CANDIDATE 2', 'CANDIDATE 1',], },),
                labels: LABELS,
              },),
            ],).toEqual([
              ['CANDIDATE 1', 'CANDIDATE 3', 'CANDIDATE 5',],
              ['CANDIDATE 1', 'CANDIDATE 2',],
            ],);
          },
        },),
        it({
          name: 'ADDS EVERY LABEL WITH THE SECOND-HIGHEST COUNT TO A LONE LEADER that fell short of a majority, in the order of '
            + 'the sheet',
          fn: async () => {
            expect([
              runOffChoiceFinalistsOf({
                voices: votesFor({ choices: ['CANDIDATE 2', 'CANDIDATE 1', 'CANDIDATE 3', 'CANDIDATE 2',], },),
                labels: LABELS,
              },),
              runOffChoiceFinalistsOf({
                voices: votesFor({ choices: ['CANDIDATE 1', 'CANDIDATE 1', 'CANDIDATE 2', 'CANDIDATE 3', 'CANDIDATE 4',], },),
                labels: LABELS,
              },),
              runOffChoiceFinalistsOf({
                voices: votesFor({
                  choices: ['CANDIDATE 1', 'CANDIDATE 1', 'CANDIDATE 1', 'CANDIDATE 2', 'CANDIDATE 2', 'CANDIDATE 3', 'CANDIDATE 4',],
                },),
                labels: LABELS,
              },),
            ],).toEqual([
              ['CANDIDATE 1', 'CANDIDATE 2', 'CANDIDATE 3',],
              ['CANDIDATE 1', 'CANDIDATE 2', 'CANDIDATE 3', 'CANDIDATE 4',],
              ['CANDIDATE 1', 'CANDIDATE 2',],
            ],);
          },
        },),
        it({
          name: 'NAMES NONE WHEN A LABEL HAS A STRICT MAJORITY OR WHEN NOTHING COUNTED',
          fn: async () => {
            expect([
              runOffChoiceFinalistsOf({ voices: votesFor({ choices: ['CANDIDATE 1', 'CANDIDATE 1', 'CANDIDATE 2',], },), labels: LABELS, },),
              runOffChoiceFinalistsOf({
                voices: votesFor({ choices: ['CANDIDATE 1', 'CANDIDATE 1', 'CANDIDATE 1', 'CANDIDATE 2', 'CANDIDATE 3',], },),
                labels: LABELS,
              },),
              runOffChoiceFinalistsOf({ voices: votesFor({ choices: ['CANDIDATE 9', 'nothing',], },), labels: LABELS, },),
            ],).toEqual([[], [], [],],);
          },
        },),
        it({
          name: 'READS THE FORMS OF A LABEL AS THE LABEL, so a tie is a tie whichever way the seats wrote it',
          fn: async () => {
            expect(runOffChoiceFinalistsOf({ voices: votesFor({ choices: ['Candidate 1', '2', 'candidate 1', ' 2',], },), labels: LABELS, },),)
              .toEqual(['CANDIDATE 1', 'CANDIDATE 2',],);
          },
        },),
      ],
    },),
    describe({
      name: carryRunOffFinalistsOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'RUNS OFF THE TIED LEADERS OF A CHOICE THAT REACHED ITS QUORUM, with the quorum the first ask closed on as the floor',
          fn: async () => {
            expect(
              carryRunOffFinalistsOf({
                voices: votesFor({ choices: ['CANDIDATE 3', 'CANDIDATE 5', 'CANDIDATE 1',], },),
                labels: LABELS,
                quorum: 2,
              },),
            ).toEqual(['CANDIDATE 1', 'CANDIDATE 3', 'CANDIDATE 5',],);
          },
        },),
        it({
          name: 'RUNS OFF A LONE LEADER WITH THE CANDIDATES NEXT BEHIND IT when the choices reached their quorum and the leader '
            + 'has no strict majority',
          fn: async () => {
            expect([
              carryRunOffFinalistsOf({
                voices: votesFor({ choices: ['CANDIDATE 1', 'CANDIDATE 1', 'CANDIDATE 2', 'CANDIDATE 3',], },),
                labels: LABELS,
                quorum: 2,
              },),
              carryRunOffFinalistsOf({
                voices: votesFor({ choices: ['CANDIDATE 4', 'CANDIDATE 1', 'CANDIDATE 4', 'CANDIDATE 5', 'CANDIDATE 2',], },),
                labels: LABELS,
                quorum: 4,
              },),
            ],).toEqual([
              ['CANDIDATE 1', 'CANDIDATE 2', 'CANDIDATE 3',],
              ['CANDIDATE 1', 'CANDIDATE 2', 'CANDIDATE 4', 'CANDIDATE 5',],
            ],);
          },
        },),
        it({
          name: 'RUNS OFF NOTHING BELOW THE QUORUM, a lone leader included, OR WHEN A CANDIDATE WON BY A STRICT MAJORITY',
          fn: async () => {
            expect([
              carryRunOffFinalistsOf({ voices: votesFor({ choices: ['CANDIDATE 3', 'CANDIDATE 5',], },), labels: LABELS, quorum: 3, },),
              carryRunOffFinalistsOf({
                voices: votesFor({ choices: ['CANDIDATE 1', 'CANDIDATE 1', 'CANDIDATE 2', 'CANDIDATE 3',], },),
                labels: LABELS,
                quorum: 5,
              },),
              carryRunOffFinalistsOf({
                voices: votesFor({ choices: ['CANDIDATE 3', 'CANDIDATE 3', 'CANDIDATE 5',], },),
                labels: LABELS,
                quorum: 2,
              },),
            ],).toEqual([[], [], [],],);
          },
        },),
      ],
    },),
  ],
},);
