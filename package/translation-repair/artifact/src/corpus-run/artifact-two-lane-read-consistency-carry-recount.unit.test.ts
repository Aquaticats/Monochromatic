/**
 Tests for recounting a recorded carry from its own ballots: candidates labelled
 in order, the current English once, the splice at most once, each seat heard
 once, the usable count and the decision a strict majority reaching the quorum
 gives; and the outcomes a carry's decision allows its place to end in.

 What a carry says about the group's places and outcomes is the coherence check's
 question (`artifact-two-lane-read-consistency-span.unit.test.ts`).

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  ARTIFACT_SCHEMA_VERSION_V18,
  type ArtifactConsistencyCarry,
  assertCarriesRecount,
  outcomesAfter,
} from '../../dist/final/node/index.mjs';
import {
  recountRefusalOf,
  refusalTextOf,
} from './consistency-refusal.test-fixture.ts';

/**
 A carry as the artifact records it: three candidates, two choices of the third.
 */
const CARRY: ArtifactConsistencyCarry = {
  label: 'OCCURRENCE 2',
  candidates: [
    { label: 'CANDIDATE 1', kind: 'current', writers: [], text: 'The kitten naps.', },
    { label: 'CANDIDATE 2', kind: 'splice', writers: [], text: 'The cat naps.', },
    { label: 'CANDIDATE 3', kind: 'writer', writers: ['hyper-cat',], text: 'The cat dozes.', },
  ],
  ballots: [
    { seat: 'hyper-cat', reason: 'It words the original best.', choice: 'CANDIDATE 3', },
    { seat: 'bedrock-cat', reason: 'It words the original best.', choice: 'CANDIDATE 3', },
  ],
  usable: 2,
  quorum: 2,
  decision: { kind: 'chosen', label: 'CANDIDATE 3', },
};

/**
 The current English as the carry offered it.
 */
const CURRENT = nonNullishOrThrow(CARRY.candidates[0],);

/**
 The splice as the carry offered it.
 */
const SPLICE = nonNullishOrThrow(CARRY.candidates[1],);

/**
 The writer's paragraph as the carry offered it.
 */
const WRITER = nonNullishOrThrow(CARRY.candidates[2],);

/**
 The first seat's choice.
 */
const FIRST_BALLOT = nonNullishOrThrow(CARRY.ballots[0],);

/**
 The second seat's choice.
 */
const SECOND_BALLOT = nonNullishOrThrow(CARRY.ballots[1],);

/**
 A carry that put no question to the bench because only the current English was on offer.
 */
const ALONE: ArtifactConsistencyCarry = {
  label: 'OCCURRENCE 2',
  candidates: [CURRENT,],
  ballots: [],
  usable: 0,
  quorum: 0,
  decision: { kind: 'none', reason: 'no-alternative', },
};

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: assertCarriesRecount.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ACCEPTS A CARRY WHOSE COUNT AND DECISION ARE WHAT ITS BALLOTS GIVE: a choice, no quorum, no '
            + 'alternative, a failure, and a splice absent',
          fn: async () => {
            const carries: readonly ArtifactConsistencyCarry[] = [
              CARRY,
              {
                ...CARRY,
                ballots: [FIRST_BALLOT,],
                usable: 1,
                decision: { kind: 'none', reason: 'no-quorum', },
              },
              ALONE,
              {
                ...ALONE,
                candidates: [],
                decision: { kind: 'failed', detail: 'the writers could not be read', },
              },
              {
                ...CARRY,
                candidates: [CURRENT, { ...WRITER, label: 'CANDIDATE 2', },],
                ballots: [
                  { ...FIRST_BALLOT, choice: 'CANDIDATE 2', },
                  { ...SECOND_BALLOT, choice: 'CANDIDATE 2', },
                ],
                decision: { kind: 'chosen', label: 'CANDIDATE 2', },
              },
            ];
            expect(carries.map(function recounted(carry,): unknown {
              return assertCarriesRecount({
                carries: [carry,],
                path: 'cat.carries',
                generation: ARTIFACT_SCHEMA_VERSION_V18,
              },);
            },),).toEqual(carries.map(function accepted(): unknown {
              return undefined;
            },),);
          },
        },),
        it({
          name: 'REFUSES CANDIDATES NOT LABELLED IN ORDER, A WRITER WITHOUT A SEAT, AND A SEAT IN TWO CANDIDATES',
          fn: async () => {
            const carries: readonly ArtifactConsistencyCarry[] = [
              { ...CARRY, candidates: [CURRENT, { ...SPLICE, label: 'CANDIDATE 3', }, WRITER,], },
              { ...CARRY, candidates: [CURRENT, SPLICE, { ...WRITER, writers: [], },], },
              { ...CARRY, candidates: [{ ...CURRENT, writers: ['hyper-cat',], }, SPLICE, WRITER,], },
            ];
            expect(carries.map(function refusedFor(carry,): string {
              return recountRefusalOf({ carry, },);
            },),).toEqual([
              'ArtifactParseError: artifact parse failed at cat.carries[0].candidates[1].label: '
                + 'expected the label CANDIDATE 2, since the candidates are labelled in order.',
              'ArtifactParseError: artifact parse failed at cat.carries[0].candidates[2].writers: '
                + 'expected a seat at least, since a writer candidate is a writer\'s paragraph.',
              'ArtifactParseError: artifact parse failed at cat.carries[0].candidates[2].writers: '
                + 'expected a seat in one candidate only, since a seat gives one reply.',
            ],);
          },
        },),
        it({
          name: 'REFUSES CANDIDATES THAT HOLD THE CURRENT ENGLISH TWICE OR NOT AT ALL, OR THE SPLICE TWICE',
          fn: async () => {
            const carries: readonly ArtifactConsistencyCarry[] = [
              { ...CARRY, candidates: [CURRENT, { ...SPLICE, kind: 'current', }, WRITER,], },
              { ...CARRY, candidates: [{ ...CURRENT, kind: 'splice', }, SPLICE, WRITER,], },
              { ...CARRY, candidates: [CURRENT, SPLICE, { ...WRITER, kind: 'splice', },], },
            ];
            expect(carries.map(function refusedFor(carry,): string {
              return recountRefusalOf({ carry, },);
            },),).toEqual([
              'ArtifactParseError: artifact parse failed at cat.carries[0].candidates: '
                + 'expected the current English once and the splice at most once.',
              'ArtifactParseError: artifact parse failed at cat.carries[0].candidates: '
                + 'expected the current English once and the splice at most once.',
              'ArtifactParseError: artifact parse failed at cat.carries[0].candidates: '
                + 'expected the current English once and the splice at most once.',
            ],);
          },
        },),
        it({
          name: 'REFUSES BALLOTS, A COUNT OR A QUORUM BESIDE A CARRY THE BENCH WAS NOT ASKED, and one that is not the current '
            + 'English alone',
          fn: async () => {
            const carries: readonly ArtifactConsistencyCarry[] = [
              { ...ALONE, ballots: CARRY.ballots, },
              { ...ALONE, usable: 1, },
              { ...ALONE, quorum: 2, },
              { ...ALONE, candidates: CARRY.candidates, },
            ];
            expect(carries.map(function refusedFor(carry,): string {
              return recountRefusalOf({ carry, },);
            },),).toEqual([
              'ArtifactParseError: artifact parse failed at cat.carries[0]: '
                + 'expected no ballots, no usable count and no quorum, since the bench was not asked among the candidates.',
              'ArtifactParseError: artifact parse failed at cat.carries[0]: '
                + 'expected no ballots, no usable count and no quorum, since the bench was not asked among the candidates.',
              'ArtifactParseError: artifact parse failed at cat.carries[0]: '
                + 'expected no ballots, no usable count and no quorum, since the bench was not asked among the candidates.',
              'ArtifactParseError: artifact parse failed at cat.carries[0].candidates: '
                + 'expected the current English alone, since no alternative was on offer.',
            ],);
          },
        },),
        it({
          name: 'REFUSES A RUN-OFF BESIDE A CARRY THE BENCH WAS NOT ASKED, since a run-off follows a first round that reached its '
            + 'quorum and a carry with no alternative was never asked',
          fn: async () => {
            const carries: readonly ArtifactConsistencyCarry[] = [
              {
                ...ALONE,
                runOff: { finalists: ['CANDIDATE 1', 'CANDIDATE 2',], ballots: [], usable: 0, quorum: 1, },
              },
            ];
            expect(carries.map(function refusedFor(carry,): string {
              return recountRefusalOf({ carry, },);
            },),).toEqual([
              'ArtifactParseError: artifact parse failed at cat.carries[0].runOff: '
                + 'expected no run-off, since the bench was not asked among the candidates.',
            ],);
          },
        },),
        it({
          name: 'REFUSES A CARRY THE BENCH WAS ASKED WITH FEWER THAN TWO CANDIDATES OR A QUORUM BELOW ONE, a seat heard twice, '
            + 'a usable count that is not the ballots\' and a decision that is not their majority',
          fn: async () => {
            const carries: readonly ArtifactConsistencyCarry[] = [
              { ...CARRY, candidates: [CURRENT,], decision: { kind: 'chosen', label: 'CANDIDATE 1', }, },
              { ...CARRY, quorum: 0, },
              { ...CARRY, ballots: [FIRST_BALLOT, { ...SECOND_BALLOT, seat: 'hyper-cat', },], },
              { ...CARRY, usable: 3, },
              { ...CARRY, decision: { kind: 'chosen', label: 'CANDIDATE 2', }, },
              { ...CARRY, decision: { kind: 'none', reason: 'no-majority', }, },
              { ...CARRY, ballots: [FIRST_BALLOT, { ...SECOND_BALLOT, choice: 'CANDIDATE 9', },], usable: 1, },
            ];
            expect(carries.map(function refusedFor(carry,): string {
              return recountRefusalOf({ carry, },);
            },),).toEqual([
              'ArtifactParseError: artifact parse failed at cat.carries[0]: '
                + 'expected two candidates at least and a quorum of one at least, since the bench was asked among the candidates.',
              'ArtifactParseError: artifact parse failed at cat.carries[0]: '
                + 'expected two candidates at least and a quorum of one at least, since the bench was asked among the candidates.',
              'ArtifactParseError: artifact parse failed at cat.carries[0].ballots[1].seat: '
                + 'expected a seat heard once, since a gather hears each seat once.',
              'ArtifactParseError: artifact parse failed at cat.carries[0].usable: '
                + 'expected the number of ballots that can be counted against the candidates.',
              'ArtifactParseError: artifact parse failed at cat.carries[0].decision: '
                + 'expected the decision a strict majority of the usable ballots, reaching the quorum, give.',
              'ArtifactParseError: artifact parse failed at cat.carries[0].decision: '
                + 'expected the decision a strict majority of the usable ballots, reaching the quorum, give.',
              'ArtifactParseError: artifact parse failed at cat.carries[0].decision: '
                + 'expected the decision a strict majority of the usable ballots, reaching the quorum, give.',
            ],);
          },
        },),
        it({
          name: 'NAMES THE PATH OF EACH CARRY OF A GROUP, counted from zero',
          fn: async () => {
            const refusal = refusalTextOf({
              read: function recountsBoth(): unknown {
                return assertCarriesRecount({
                  carries: [CARRY, { ...CARRY, usable: 3, },],
                  path: 'cat.groups[0].carries',
                  generation: ARTIFACT_SCHEMA_VERSION_V18,
                },);
              },
            },);
            expect(refusal,).toBe(
              'ArtifactParseError: artifact parse failed at cat.groups[0].carries[1].usable: '
                + 'expected the number of ballots that can be counted against the candidates.',
            );
          },
        },),
      ],
    },),
    describe({
      name: outcomesAfter.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ALLOWS KEPT-CURRENT FOR A CARRY THAT CHOSE THE CURRENT ENGLISH, CHOSE NOTHING OR FAILED, and the outcomes a '
            + 'splice or a writer\'s paragraph can end in otherwise',
          fn: async () => {
            const keptCurrent = ['kept-current',];
            const replaces = ['replaced', 'kept-floor', 'overlapped',];
            expect([
              outcomesAfter({ carry: { ...CARRY, decision: { kind: 'chosen', label: 'CANDIDATE 1', }, }, },),
              outcomesAfter({ carry: { ...CARRY, decision: { kind: 'none', reason: 'no-majority', }, }, },),
              outcomesAfter({ carry: ALONE, },),
              outcomesAfter({ carry: { ...ALONE, decision: { kind: 'failed', detail: 'x', }, }, },),
              outcomesAfter({ carry: CARRY, },),
              outcomesAfter({ carry: { ...CARRY, decision: { kind: 'chosen', label: 'CANDIDATE 2', }, }, },),
              outcomesAfter({ carry: { ...CARRY, decision: { kind: 'chosen', label: 'CANDIDATE 9', }, }, },),
            ],).toEqual([keptCurrent, keptCurrent, keptCurrent, keptCurrent, replaces, replaces, replaces,],);
          },
        },),
        it({
          name: 'ALLOWS ONLY KEPT-CURRENT FOR A PLACE CODE CANNOT MOVE WHEN THE SPLICE WAS CHOSEN, and the outcomes a writer\'s '
            + 'paragraph can end in, since that rewrites the place too',
          fn: async () => {
            const keptCurrent = ['kept-current',];
            const replaces = ['replaced', 'kept-floor', 'overlapped',];
            expect([
              outcomesAfter({ carry: { ...CARRY, decision: { kind: 'chosen', label: 'CANDIDATE 2', }, }, spliceMovesIt: false, },),
              outcomesAfter({ carry: CARRY, spliceMovesIt: false, },),
              outcomesAfter({ carry: { ...CARRY, decision: { kind: 'chosen', label: 'CANDIDATE 1', }, }, spliceMovesIt: false, },),
              outcomesAfter({ carry: { ...CARRY, decision: { kind: 'chosen', label: 'CANDIDATE 2', }, }, spliceMovesIt: true, },),
            ],).toEqual([keptCurrent, replaces, keptCurrent, replaces,],);
          },
        },),
      ],
    },),
  ],
},);
