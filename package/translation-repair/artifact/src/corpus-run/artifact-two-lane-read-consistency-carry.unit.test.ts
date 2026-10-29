/**
 Tests for reading, key by key, the two records of the sixteenth shape that the
 first round does not hold: the run-off among the finalists of a split first round and
 the carry of the chosen rendering to a receiving place. Each case breaks one
 valid record in one way and pins the whole refusal.

 What a carry says about its own ballots is the recount's question
 (`artifact-two-lane-read-consistency-carry-recount.unit.test.ts`).

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
  parseConsistencyCarry,
  parseConsistencyRunOff,
} from '../../dist/final/node/index.mjs';
import { refusalTextOf, } from './consistency-refusal.test-fixture.ts';

/**
 The count refusal's reason, which the usable count and the quorum share.
 */
const NOT_A_COUNT = 'a non-negative integer no larger than JSON carries exactly';

/**
 A run-off as the artifact records it.
 */
const RUN_OFF = {
  finalists: ['OCCURRENCE 1', 'OCCURRENCE 2',],
  ballots: [
    { seat: 'hyper-cat', reason: 'It words the original best.', choice: 'OCCURRENCE 1', },
    { seat: 'bedrock-cat', reason: 'It words the original best.', choice: 'OCCURRENCE 2', },
  ],
  usable: 2,
  quorum: 2,
};

/**
 A carry as the artifact records it: three candidates and a choice among them.
 */
const CARRY = {
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
 The text of the refusal a run-off raises.

 @param value - recorded run-off

 @returns Refusal as text

 @example
 ```ts
 const text = runOffRefusal({ value: 'naps', },);
 ```
 */
function runOffRefusal({ value, }: { readonly value: unknown; },): string {
  return refusalTextOf({
    read: function readsRunOff(): unknown {
      return parseConsistencyRunOff({
        value,
        path: 'cat.runOff',
      },);
    },
  },);
}

/**
 The text of the refusal a carry raises.

 @param value - recorded carry

 @returns Refusal as text

 @example
 ```ts
 const text = carryRefusal({ value: 'naps', },);
 ```
 */
function carryRefusal({ value, }: { readonly value: unknown; },): string {
  return refusalTextOf({
    read: function readsCarry(): unknown {
      return parseConsistencyCarry({
        value,
        path: 'cat.carry',
      },);
    },
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: parseConsistencyRunOff.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS A RUN-OFF AS WRITTEN: the finalists, each choice with its seat and reason, the usable count and the quorum',
          fn: async () => {
            expect(parseConsistencyRunOff({ value: RUN_OFF, path: 'cat.runOff', },),).toEqual(RUN_OFF,);
          },
        },),
        it({
          name: 'REFUSES A RUN-OFF THAT IS NOT AN OBJECT AND A KEY IT DOES NOT HAVE, naming the path',
          fn: async () => {
            expect([
              runOffRefusal({ value: 'naps', },),
              runOffRefusal({ value: { ...RUN_OFF, winner: 'OCCURRENCE 1', }, },),
            ],).toEqual([
              'ArtifactParseError: artifact parse failed at cat.runOff: expected an object.',
              'ArtifactParseError: artifact parse failed at cat.runOff.winner: '
                + 'expected no key here beyond finalists, ballots, usable, quorum.',
            ],);
          },
        },),
        it({
          name: 'REFUSES FINALISTS THAT ARE NOT A LIST OF TEXT AND BALLOTS THAT ARE NOT A LIST OF CHOICES',
          fn: async () => {
            expect([
              runOffRefusal({ value: { ...RUN_OFF, finalists: 'OCCURRENCE 1', }, },),
              runOffRefusal({ value: { ...RUN_OFF, finalists: ['OCCURRENCE 1', 2,], }, },),
              runOffRefusal({ value: { ...RUN_OFF, ballots: 'naps', }, },),
              runOffRefusal({ value: { ...RUN_OFF, ballots: ['naps',], }, },),
              runOffRefusal({ value: { ...RUN_OFF, ballots: [{ seat: 'hyper-cat', choice: 'OCCURRENCE 1', },], }, },),
            ],).toEqual([
              'ArtifactParseError: artifact parse failed at cat.runOff.finalists: expected an array.',
              'ArtifactParseError: artifact parse failed at cat.runOff.finalists[1]: expected a string.',
              'ArtifactParseError: artifact parse failed at cat.runOff.ballots: expected an array.',
              'ArtifactParseError: artifact parse failed at cat.runOff.ballots[0]: expected an object.',
              'ArtifactParseError: artifact parse failed at cat.runOff.ballots[0].reason: expected a string.',
            ],);
          },
        },),
        it({
          name: 'REFUSES A USABLE COUNT OR A QUORUM THAT IS NOT A COUNT, whether absent, text, negative or fractional',
          fn: async () => {
            const { quorum: _quorum, ...withoutQuorum } = RUN_OFF;
            expect([
              runOffRefusal({ value: { ...RUN_OFF, usable: '2', }, },),
              runOffRefusal({ value: { ...RUN_OFF, usable: -1, }, },),
              runOffRefusal({ value: { ...RUN_OFF, quorum: 1.5, }, },),
              runOffRefusal({ value: withoutQuorum, },),
            ],).toEqual([
              'ArtifactParseError: artifact parse failed at cat.runOff.usable: expected a number.',
              `ArtifactParseError: artifact parse failed at cat.runOff.usable: expected ${NOT_A_COUNT}.`,
              `ArtifactParseError: artifact parse failed at cat.runOff.quorum: expected ${NOT_A_COUNT}.`,
              'ArtifactParseError: artifact parse failed at cat.runOff.quorum: expected a number.',
            ],);
          },
        },),
      ],
    },),
    describe({
      name: parseConsistencyCarry.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS A CARRY AS WRITTEN, a choice of a candidate, no choice for each reason, and a carry that failed',
          fn: async () => {
            const carries = [
              CARRY,
              { ...CARRY, decision: { kind: 'none', reason: 'no-majority', }, },
              { ...CARRY, decision: { kind: 'none', reason: 'no-quorum', }, },
              {
                ...CARRY,
                candidates: [CARRY.candidates[0],],
                ballots: [],
                usable: 0,
                quorum: 0,
                decision: { kind: 'none', reason: 'no-alternative', },
              },
              {
                ...CARRY,
                candidates: [],
                ballots: [],
                usable: 0,
                quorum: 0,
                decision: { kind: 'failed', detail: 'the writers could not be read', },
              },
            ];
            expect(carries.map(function read(value,): unknown {
              return parseConsistencyCarry({ value, path: 'cat.carry', },);
            },),).toEqual(carries,);
          },
        },),
        it({
          name: 'REFUSES A CARRY THAT IS NOT AN OBJECT AND A KEY IT DOES NOT HAVE, naming the path',
          fn: async () => {
            expect([
              carryRefusal({ value: 'naps', },),
              carryRefusal({ value: { ...CARRY, winner: 'CANDIDATE 3', }, },),
            ],).toEqual([
              'ArtifactParseError: artifact parse failed at cat.carry: expected an object.',
              'ArtifactParseError: artifact parse failed at cat.carry.winner: '
                + 'expected no key here beyond label, candidates, ballots, usable, quorum, runOff, decision.',
            ],);
          },
        },),
        it({
          name: 'REFUSES A CANDIDATE THAT IS NOT AN OBJECT, HAS A KEY IT DOES NOT HAVE, OR IS OF A KIND OUTSIDE THE VOCABULARY',
          fn: async () => {
            const [current,] = CARRY.candidates;
            expect([
              carryRefusal({ value: { ...CARRY, candidates: 'naps', }, },),
              carryRefusal({ value: { ...CARRY, candidates: ['naps',], }, },),
              carryRefusal({ value: { ...CARRY, candidates: [{ ...current, author: 'hyper-cat', },], }, },),
              carryRefusal({ value: { ...CARRY, candidates: [{ ...current, kind: 'guess', },], }, },),
              carryRefusal({ value: { ...CARRY, candidates: [{ ...current, writers: 'hyper-cat', },], }, },),
              carryRefusal({ value: { ...CARRY, candidates: [{ ...current, text: 4, },], }, },),
            ],).toEqual([
              'ArtifactParseError: artifact parse failed at cat.carry.candidates: expected an array.',
              'ArtifactParseError: artifact parse failed at cat.carry.candidates[0]: expected an object.',
              'ArtifactParseError: artifact parse failed at cat.carry.candidates[0].author: '
                + 'expected no key here beyond label, kind, writers, text.',
              'ArtifactParseError: artifact parse failed at cat.carry.candidates[0].kind: '
                + 'expected one of current, splice, writer.',
              'ArtifactParseError: artifact parse failed at cat.carry.candidates[0].writers: expected an array.',
              'ArtifactParseError: artifact parse failed at cat.carry.candidates[0].text: expected a string.',
            ],);
          },
        },),
        it({
          name: 'REFUSES BALLOTS THAT ARE NOT A LIST OF CHOICES AND A COUNT OR A QUORUM THAT IS NOT A COUNT',
          fn: async () => {
            expect([
              carryRefusal({ value: { ...CARRY, ballots: 'naps', }, },),
              carryRefusal({ value: { ...CARRY, ballots: [{ seat: 'hyper-cat', reason: 'x', choice: 4, },], }, },),
              carryRefusal({ value: { ...CARRY, usable: -1, }, },),
              carryRefusal({ value: { ...CARRY, quorum: '2', }, },),
            ],).toEqual([
              'ArtifactParseError: artifact parse failed at cat.carry.ballots: expected an array.',
              'ArtifactParseError: artifact parse failed at cat.carry.ballots[0].choice: expected a string.',
              `ArtifactParseError: artifact parse failed at cat.carry.usable: expected ${NOT_A_COUNT}.`,
              'ArtifactParseError: artifact parse failed at cat.carry.quorum: expected a number.',
            ],);
          },
        },),
        it({
          name: 'REFUSES A DECISION OF A KIND OUTSIDE THE VOCABULARY, WITH A KEY OF ANOTHER KIND, OR WITH A REASON OUTSIDE IT',
          fn: async () => {
            expect([
              carryRefusal({ value: { ...CARRY, decision: 'CANDIDATE 3', }, },),
              carryRefusal({ value: { ...CARRY, decision: { kind: 'guess', }, }, },),
              carryRefusal({ value: { ...CARRY, decision: { kind: 'chosen', label: 'CANDIDATE 3', reason: 'x', }, }, },),
              carryRefusal({ value: { ...CARRY, decision: { kind: 'none', reason: 'shy', }, }, },),
              carryRefusal({ value: { ...CARRY, decision: { kind: 'none', reason: 'no-majority', label: 'CANDIDATE 1', }, }, },),
              carryRefusal({ value: { ...CARRY, decision: { kind: 'failed', }, }, },),
              carryRefusal({ value: { ...CARRY, decision: { kind: 'chosen', label: 3, }, }, },),
            ],).toEqual([
              'ArtifactParseError: artifact parse failed at cat.carry.decision: expected an object.',
              'ArtifactParseError: artifact parse failed at cat.carry.decision.kind: expected one of chosen, none, failed.',
              'ArtifactParseError: artifact parse failed at cat.carry.decision.reason: expected no key here beyond kind, label.',
              'ArtifactParseError: artifact parse failed at cat.carry.decision.reason: '
                + 'expected one of no-alternative, no-quorum, no-majority.',
              'ArtifactParseError: artifact parse failed at cat.carry.decision.label: expected no key here beyond kind, reason.',
              'ArtifactParseError: artifact parse failed at cat.carry.decision.detail: expected a string.',
              'ArtifactParseError: artifact parse failed at cat.carry.decision.label: expected a string.',
            ],);
          },
        },),
      ],
    },),
  ],
},);
