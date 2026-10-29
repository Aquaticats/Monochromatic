/**
 Tests for reading what was asked of the bench about one recorded group and
 what it answered: a bench not asked, a group whose handling failed, and a
 bench asked, with its ballots, counts and winner. Each is read key by key, so
 every case breaks one valid record in one way and pins the whole refusal.

 What the verdict says about the group's places and ballots is the coherence
 check's question (`artifact-two-lane-read-consistency-coherence.unit.test.ts`).

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { parseConsistencyVerdict, } from '../../dist/final/node/index.mjs';
import { refusalTextOf, } from './consistency-refusal.test-fixture.ts';

/**
 The count refusal's reason, which the usable count and the quorum share.
 */
const NOT_A_COUNT = 'a non-negative integer no larger than JSON carries exactly';

/**
 A bench asked and answered, one ballot, the first place winning.
 */
const ASKED = {
  kind: 'asked',
  ballots: [
    {
      seat: 'hyper-cat',
      reason: 'The first place words it best.',
      quotes: [{ label: 'OCCURRENCE 1', quote: 'The cat naps.', },],
      choice: 'OCCURRENCE 1',
      keep: [],
    },
  ],
  usable: 1,
  quorum: 1,
  winner: {
    kind: 'chosen',
    label: 'OCCURRENCE 1',
  },
};

/**
 The text of the refusal a verdict raises.

 @param value - recorded verdict

 @returns Refusal as text

 @example
 ```ts
 const text = verdictRefusal({ value: 'naps', },);
 ```
 */
function verdictRefusal(
  {
    value,
    generation = 15,
  }: {
    readonly value: unknown;
    readonly generation?: 15 | 16;
  },
): string {
  return refusalTextOf({
    read: function readsVerdict(): unknown {
      return parseConsistencyVerdict({
        value,
        path: 'cat.verdict',
        generation,
      },);
    },
  },);
}

/**
 A run-off as the artifact records it: two places tied for first and two judges chose between them.
 */
const RUN_OFF = {
  finalists: ['OCCURRENCE 1', 'OCCURRENCE 2',],
  ballots: [
    { seat: 'hyper-cat', reason: 'The first place words it best.', choice: 'OCCURRENCE 1', },
    { seat: 'bedrock-cat', reason: 'The first place words it best.', choice: 'OCCURRENCE 1', },
  ],
  usable: 2,
  quorum: 2,
};

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: parseConsistencyVerdict.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS A BENCH NOT ASKED FOR EACH REASON IT WAS NOT, and a group whose handling failed, as written',
          fn: async () => {
            /**
             Every verdict of the two kinds that carry no ballots.
             */
            const quiet = [
              { kind: 'not-asked', reason: 'too-few-placed', },
              { kind: 'not-asked', reason: 'no-english', },
              { kind: 'not-asked', reason: 'consistent', },
              { kind: 'failed', detail: 'refused by ConsistencyInvariantError', },
            ];
            expect(quiet.map(function read(value,): unknown {
              return parseConsistencyVerdict({
                value,
                path: 'cat.verdict',
                generation: 15,
              },);
            },),).toEqual(quiet,);
          },
        },),
        it({
          name: 'READS A BENCH ASKED WITH ITS BALLOTS, ITS COUNTS AND A WINNER, and a bench asked that has no winner for each reason',
          fn: async () => {
            /**
             The asked verdict under each winner the bench can end with.
             */
            const verdicts = [
              ASKED,
              {
                ...ASKED,
                winner: { kind: 'none', reason: 'no-quorum', },
              },
              {
                ...ASKED,
                winner: { kind: 'none', reason: 'no-majority', },
              },
              {
                ...ASKED,
                winner: { kind: 'none', reason: 'chosen-unlocated', },
              },
              {
                ...ASKED,
                ballots: [],
                usable: 0,
              },
            ];
            expect(verdicts.map(function read(value,): unknown {
              return parseConsistencyVerdict({
                value,
                path: 'cat.verdict',
                generation: 15,
              },);
            },),).toEqual(verdicts,);
          },
        },),
        it({
          name: 'READS A BENCH ASKED OF GENERATION SIXTEEN WITH ITS RUN-OFF, and the reasons only a run-off gives, as written',
          fn: async () => {
            const verdicts = [
              { ...ASKED, runOff: RUN_OFF, },
              { ...ASKED, winner: { kind: 'none', reason: 'run-off-tied', }, runOff: RUN_OFF, },
              { ...ASKED, winner: { kind: 'none', reason: 'run-off-no-quorum', }, runOff: { ...RUN_OFF, quorum: 3, }, },
              ASKED,
            ];
            expect(verdicts.map(function read(value,): unknown {
              return parseConsistencyVerdict({
                value,
                path: 'cat.verdict',
                generation: 16,
              },);
            },),).toEqual(verdicts,);
          },
        },),
        it({
          name: 'REFUSES A RUN-OFF AND THE REASONS ONLY A RUN-OFF GIVES IN GENERATION FIFTEEN, which runs off no tie, and a '
            + 'run-off whose shape is wrong in generation sixteen',
          fn: async () => {
            expect([
              verdictRefusal({ value: { ...ASKED, runOff: RUN_OFF, }, generation: 15, },),
              verdictRefusal({ value: { ...ASKED, winner: { kind: 'none', reason: 'run-off-tied', }, }, generation: 15, },),
              verdictRefusal({ value: { ...ASKED, runOff: { ...RUN_OFF, finalists: 'OCCURRENCE 1', }, }, generation: 16, },),
              verdictRefusal({ value: { ...ASKED, runOff: { ...RUN_OFF, extra: 1, }, }, generation: 16, },),
            ],).toEqual([
              'ArtifactParseError: artifact parse failed at cat.verdict.runOff: '
                + 'expected no key here beyond kind, ballots, usable, quorum, winner.',
              'ArtifactParseError: artifact parse failed at cat.verdict.winner.reason: '
                + 'expected a reason generation fifteen gives, which runs off no tie.',
              'ArtifactParseError: artifact parse failed at cat.verdict.runOff.finalists: expected an array.',
              'ArtifactParseError: artifact parse failed at cat.verdict.runOff.extra: '
                + 'expected no key here beyond finalists, ballots, usable, quorum.',
            ],);
          },
        },),
        it({
          name: 'REFUSES A VERDICT THAT IS NOT AN OBJECT AND A KIND OUTSIDE THE VOCABULARY, naming the path',
          fn: async () => {
            expect([
              verdictRefusal({ value: 'naps', },),
              verdictRefusal({ value: { kind: 'asleep', }, },),
              verdictRefusal({ value: { kind: 4, }, },),
            ],).toEqual([
              'ArtifactParseError: artifact parse failed at cat.verdict: expected an object.',
              'ArtifactParseError: artifact parse failed at cat.verdict.kind: expected one of not-asked, failed, asked.',
              'ArtifactParseError: artifact parse failed at cat.verdict.kind: expected a string.',
            ],);
          },
        },),
        it({
          name: 'REFUSES A BENCH NOT ASKED THAT CARRIES A KEY OF ANOTHER KIND OR A REASON OUTSIDE THE VOCABULARY',
          fn: async () => {
            expect([
              verdictRefusal({ value: { kind: 'not-asked', reason: 'consistent', detail: 'x', }, },),
              verdictRefusal({ value: { kind: 'not-asked', reason: 'shy', }, },),
              verdictRefusal({ value: { kind: 'not-asked', }, },),
            ],).toEqual([
              'ArtifactParseError: artifact parse failed at cat.verdict.detail: expected no key here beyond kind, reason.',
              'ArtifactParseError: artifact parse failed at cat.verdict.reason: '
                + 'expected one of too-few-placed, no-english, consistent.',
              'ArtifactParseError: artifact parse failed at cat.verdict.reason: expected a string.',
            ],);
          },
        },),
        it({
          name: 'REFUSES A FAILED GROUP THAT CARRIES A KEY OF ANOTHER KIND OR A DETAIL THAT IS NOT TEXT',
          fn: async () => {
            expect([
              verdictRefusal({ value: { kind: 'failed', detail: 'x', reason: 'consistent', }, },),
              verdictRefusal({ value: { kind: 'failed', detail: 4, }, },),
              verdictRefusal({ value: { kind: 'failed', }, },),
            ],).toEqual([
              'ArtifactParseError: artifact parse failed at cat.verdict.reason: expected no key here beyond kind, detail.',
              'ArtifactParseError: artifact parse failed at cat.verdict.detail: expected a string.',
              'ArtifactParseError: artifact parse failed at cat.verdict.detail: expected a string.',
            ],);
          },
        },),
        it({
          name: 'REFUSES AN ASKED BENCH WITH A KEY IT DOES NOT HAVE AND BALLOTS THAT ARE NOT A LIST OF OBJECTS',
          fn: async () => {
            expect([
              verdictRefusal({ value: { ...ASKED, detail: 'x', }, },),
              verdictRefusal({ value: { ...ASKED, ballots: 'naps', }, },),
              verdictRefusal({ value: { ...ASKED, ballots: ['naps',], }, },),
              verdictRefusal({
                value: {
                  ...ASKED,
                  ballots: [{ ...ASKED.ballots[0], weight: 1, },],
                },
              },),
            ],).toEqual([
              'ArtifactParseError: artifact parse failed at cat.verdict.detail: '
                + 'expected no key here beyond kind, ballots, usable, quorum, winner.',
              'ArtifactParseError: artifact parse failed at cat.verdict.ballots: expected an array.',
              'ArtifactParseError: artifact parse failed at cat.verdict.ballots[0]: expected an object.',
              'ArtifactParseError: artifact parse failed at cat.verdict.ballots[0].weight: '
                + 'expected no key here beyond seat, reason, quotes, choice, keep.',
            ],);
          },
        },),
        it({
          name: 'REFUSES A USABLE COUNT OR A QUORUM THAT IS NOT A COUNT, whether absent, text, negative or fractional',
          fn: async () => {
            expect([
              verdictRefusal({ value: { ...ASKED, usable: '1', }, },),
              verdictRefusal({ value: { ...ASKED, usable: -1, }, },),
              verdictRefusal({ value: { ...ASKED, quorum: 1.5, }, },),
              verdictRefusal({ value: { kind: 'asked', ballots: [], usable: 0, winner: ASKED.winner, }, },),
            ],).toEqual([
              'ArtifactParseError: artifact parse failed at cat.verdict.usable: expected a number.',
              `ArtifactParseError: artifact parse failed at cat.verdict.usable: expected ${NOT_A_COUNT}.`,
              `ArtifactParseError: artifact parse failed at cat.verdict.quorum: expected ${NOT_A_COUNT}.`,
              'ArtifactParseError: artifact parse failed at cat.verdict.quorum: expected a number.',
            ],);
          },
        },),
        it({
          name: 'REFUSES A WINNER THAT IS NOT AN OBJECT AND A WINNER KIND OUTSIDE THE VOCABULARY',
          fn: async () => {
            expect([
              verdictRefusal({ value: { ...ASKED, winner: 'OCCURRENCE 1', }, },),
              verdictRefusal({ value: { ...ASKED, winner: { kind: 'tie', }, }, },),
              verdictRefusal({ value: { ...ASKED, winner: { kind: 2, }, }, },),
            ],).toEqual([
              'ArtifactParseError: artifact parse failed at cat.verdict.winner: expected an object.',
              'ArtifactParseError: artifact parse failed at cat.verdict.winner.kind: expected one of chosen, none.',
              'ArtifactParseError: artifact parse failed at cat.verdict.winner.kind: expected a string.',
            ],);
          },
        },),
        it({
          name: 'REFUSES A CHOSEN WINNER THAT CARRIES A REASON OR NO LABEL TEXT, and a winner of none that carries a label '
            + 'or a reason outside the vocabulary',
          fn: async () => {
            expect([
              verdictRefusal({
                value: { ...ASKED, winner: { kind: 'chosen', label: 'OCCURRENCE 1', reason: 'no-quorum', }, },
              },),
              verdictRefusal({ value: { ...ASKED, winner: { kind: 'chosen', label: 1, }, }, },),
              verdictRefusal({
                value: { ...ASKED, winner: { kind: 'none', reason: 'no-quorum', label: 'OCCURRENCE 1', }, },
              },),
              verdictRefusal({ value: { ...ASKED, winner: { kind: 'none', reason: 'shy', }, }, },),
            ],).toEqual([
              'ArtifactParseError: artifact parse failed at cat.verdict.winner.reason: '
                + 'expected no key here beyond kind, label.',
              'ArtifactParseError: artifact parse failed at cat.verdict.winner.label: expected a string.',
              'ArtifactParseError: artifact parse failed at cat.verdict.winner.label: '
                + 'expected no key here beyond kind, reason.',
              'ArtifactParseError: artifact parse failed at cat.verdict.winner.reason: '
                + 'expected one of no-quorum, no-majority, chosen-unlocated, run-off-no-quorum, run-off-tied.',
            ],);
          },
        },),
      ],
    },),
  ],
},);
