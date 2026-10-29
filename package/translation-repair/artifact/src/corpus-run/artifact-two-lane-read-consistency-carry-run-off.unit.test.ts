/**
 Tests for reading back a carry whose choice was run off (the lead's adoption of 2026-10-15 of the owner's rule of 2026-10-11
 for this ask): the artifact keeps the run-off beside the first count, and the reader recounts both, so a run-off that does
 not follow from the first count, or a decision that does not follow from the run-off, is refused. A carry recorded
 without a run-off, as the runs of generation 17 and earlier recorded a split, is still read as it was there, and from
 generation 18 the run-off is required where the finalists rule calls for one.

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
  ARTIFACT_SCHEMA_VERSION_V17,
  ARTIFACT_SCHEMA_VERSION_V18,
  type ArtifactConsistencyCarry,
  assertCarriesRecount,
  parseConsistencyCarry,
} from '../../dist/final/node/index.mjs';
import { recountRefusalOf, } from './consistency-refusal.test-fixture.ts';

/**
 Builds one recorded choice.

 @param seat - seat that answered
 @param choice - label chosen

 @returns Recorded choice

 @example
 ```ts
 const ballot = ballotOf({ seat: 'hyper-cat', choice: 'CANDIDATE 3', },);
 ```
 */
function ballotOf({ seat, choice, }: { readonly seat: string; readonly choice: string; },): ArtifactConsistencyCarry['ballots'][number] {
  return { seat, reason: 'It words the original best.', choice, };
}

/**
 The run-off of a three-way split: all three candidates tied, and the third won two of three.
 */
const RUN_OFF: NonNullable<ArtifactConsistencyCarry['runOff']> = {
  finalists: ['CANDIDATE 1', 'CANDIDATE 2', 'CANDIDATE 3',],
  ballots: [
    ballotOf({ seat: 'hyper-cat', choice: 'CANDIDATE 3', },),
    ballotOf({ seat: 'bedrock-cat', choice: 'CANDIDATE 3', },),
    ballotOf({ seat: 'vision-cat', choice: 'CANDIDATE 2', },),
  ],
  usable: 3,
  quorum: 2,
};

/**
 A carry whose three choices split three ways and whose run-off chose the third candidate.
 */
const RUN_OFF_CARRY: ArtifactConsistencyCarry = {
  label: 'OCCURRENCE 2',
  candidates: [
    { label: 'CANDIDATE 1', kind: 'current', writers: [], text: 'The kitten naps.', },
    { label: 'CANDIDATE 2', kind: 'splice', writers: [], text: 'The cat naps.', },
    { label: 'CANDIDATE 3', kind: 'writer', writers: ['hyper-cat',], text: 'The cat dozes.', },
  ],
  ballots: [
    ballotOf({ seat: 'hyper-cat', choice: 'CANDIDATE 1', },),
    ballotOf({ seat: 'bedrock-cat', choice: 'CANDIDATE 2', },),
    ballotOf({ seat: 'vision-cat', choice: 'CANDIDATE 3', },),
  ],
  usable: 3,
  quorum: 2,
  runOff: RUN_OFF,
  decision: { kind: 'chosen', label: 'CANDIDATE 3', },
};

/**
 The generation the pass writes, and the last generation that recorded a split with no run-off.
 */
const GENERATION_EIGHTEEN = ARTIFACT_SCHEMA_VERSION_V18;

/**
 A value as the artifact stores it and a resumed run reads it back: through JSON, whose semantics are the point.

 @param value - value as built

 @returns Value read back from its JSON text

 @example
 ```ts
 const stored = asStored({ value: RUN_OFF_CARRY, },);
 ```
 */
function asStored({ value, }: { readonly value: unknown; },): unknown {
  // oxlint-disable-next-line unicorn/prefer-structured-clone -- JSON semantics are the point: the artifact is written as JSON, so a key whose value is undefined must drop and a field JSON cannot hold must fail here, both of which structuredClone would hide.
  return JSON.parse(JSON.stringify(value,),);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: parseConsistencyCarry.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS THE RUN-OFF OF A CARRY BESIDE ITS FIRST COUNT, and a carry without one as before',
          fn: async () => {
            expect(parseConsistencyCarry({ value: asStored({ value: RUN_OFF_CARRY, },), path: 'cat', },),).toEqual(
              RUN_OFF_CARRY,
            );
            const { runOff: _runOff, ...without } = RUN_OFF_CARRY;
            expect(
              parseConsistencyCarry({
                value: asStored({ value: { ...without, decision: { kind: 'none', reason: 'no-majority', }, }, },),
                path: 'cat',
              },),
            ).toEqual({ ...without, decision: { kind: 'none', reason: 'no-majority', }, },);
          },
        },),
      ],
    },),
    describe({
      name: assertCarriesRecount.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ACCEPTS A CARRY WHOSE RUN-OFF FOLLOWS FROM THE TIE OF ITS FIRST COUNT AND WHOSE DECISION FOLLOWS FROM THE RUN-OFF: '
            + 'a choice, a second split, a run-off that missed its quorum, and a tie of two leaders',
          fn: async () => {
            const carries: readonly ArtifactConsistencyCarry[] = [
              RUN_OFF_CARRY,
              {
                ...RUN_OFF_CARRY,
                runOff: {
                  ...RUN_OFF,
                  ballots: [
                    ballotOf({ seat: 'hyper-cat', choice: 'CANDIDATE 1', },),
                    ballotOf({ seat: 'bedrock-cat', choice: 'CANDIDATE 2', },),
                    ballotOf({ seat: 'vision-cat', choice: 'CANDIDATE 3', },),
                  ],
                },
                decision: { kind: 'none', reason: 'no-majority', },
              },
              {
                ...RUN_OFF_CARRY,
                runOff: {
                  ...RUN_OFF,
                  ballots: [ballotOf({ seat: 'hyper-cat', choice: 'CANDIDATE 3', },),],
                  usable: 1,
                },
                decision: { kind: 'none', reason: 'no-quorum', },
              },
              {
                ...RUN_OFF_CARRY,
                ballots: [
                  ballotOf({ seat: 'hyper-cat', choice: 'CANDIDATE 1', },),
                  ballotOf({ seat: 'bedrock-cat', choice: 'CANDIDATE 3', },),
                ],
                usable: 2,
                runOff: {
                  finalists: ['CANDIDATE 1', 'CANDIDATE 3',],
                  ballots: [
                    ballotOf({ seat: 'hyper-cat', choice: 'CANDIDATE 1', },),
                    ballotOf({ seat: 'bedrock-cat', choice: 'CANDIDATE 1', },),
                  ],
                  usable: 2,
                  quorum: 2,
                },
                decision: { kind: 'chosen', label: 'CANDIDATE 1', },
              },
            ];
            // Each carry is read without a refusal: the recount returns for every one.
            for (const carry of carries) {
              expect(assertCarriesRecount({ carries: [carry,], path: 'cat.carries', generation: GENERATION_EIGHTEEN, },),)
                .toBeUndefined();
            }
          },
        },),
        it({
          name: 'REQUIRES THE RUN-OFF FROM GENERATION EIGHTEEN WHERE THE FINALISTS RULE CALLS FOR ONE, and reads a split recorded '
            + 'with no run-off as written in generation seventeen: a three-way split and a lone leader without a majority are '
            + 'refused at eighteen and accepted at seventeen, and a first count that missed its quorum needs none at either',
          fn: async () => {
            const { runOff: _runOff, ...without } = RUN_OFF_CARRY;
            /**
             The three-way split of the run-off carry, recorded with no run-off.
             */
            const threeWay: ArtifactConsistencyCarry = { ...without, decision: { kind: 'none', reason: 'no-majority', }, };
            /**
             Four choices that give one candidate two and each of the others one, recorded with no run-off.
             */
            const loneLeader: ArtifactConsistencyCarry = {
              ...threeWay,
              ballots: [
                ballotOf({ seat: 'hyper-cat', choice: 'CANDIDATE 1', },),
                ballotOf({ seat: 'bedrock-cat', choice: 'CANDIDATE 1', },),
                ballotOf({ seat: 'vision-cat', choice: 'CANDIDATE 2', },),
                ballotOf({ seat: 'synthetic-cat', choice: 'CANDIDATE 3', },),
              ],
              usable: 4,
            };
            /**
             A first count that missed its quorum, so there is no split to run off.
             */
            const noQuorum: ArtifactConsistencyCarry = {
              ...threeWay,
              ballots: [ballotOf({ seat: 'hyper-cat', choice: 'CANDIDATE 1', },),],
              usable: 1,
              decision: { kind: 'none', reason: 'no-quorum', },
            };
            /**
             The refusal of a missing run-off.
             */
            const missing = 'ArtifactParseError: artifact parse failed at cat.carries[0].runOff: expected a run-off among the finalists of '
              + 'the first ballots, since they reached their quorum and gave no candidate a strict majority.';
            expect({
              seventeen: [threeWay, loneLeader, noQuorum,].map(function readAtSeventeen(carry,): unknown {
                return assertCarriesRecount({
                  carries: [carry,],
                  path: 'cat.carries',
                  generation: ARTIFACT_SCHEMA_VERSION_V17,
                },);
              },),
              eighteen: [threeWay, loneLeader,].map(function refusedAtEighteen(carry,): string {
                return recountRefusalOf({ carry, generation: GENERATION_EIGHTEEN, },);
              },),
              eighteenNoQuorum: assertCarriesRecount({
                carries: [noQuorum,],
                path: 'cat.carries',
                generation: GENERATION_EIGHTEEN,
              },),
            },).toEqual({
              seventeen: [undefined, undefined, undefined,],
              eighteen: [missing, missing,],
              eighteenNoQuorum: undefined,
            },);
          },
        },),
        it({
          name: 'REFUSES A RUN-OFF THAT DOES NOT FOLLOW FROM THE FIRST COUNT: finalists that are not the tied leaders, a run-off '
            + 'beside a strict majority, and one beside a first count that missed its quorum',
          fn: async () => {
            const wrongFinalists = recountRefusalOf({
              carry: { ...RUN_OFF_CARRY, runOff: { ...RUN_OFF, finalists: ['CANDIDATE 1', 'CANDIDATE 3',], }, },
            },);
            const besideMajority = recountRefusalOf({
              carry: {
                ...RUN_OFF_CARRY,
                ballots: [
                  ballotOf({ seat: 'hyper-cat', choice: 'CANDIDATE 3', },),
                  ballotOf({ seat: 'bedrock-cat', choice: 'CANDIDATE 3', },),
                  ballotOf({ seat: 'vision-cat', choice: 'CANDIDATE 2', },),
                ],
              },
            },);
            const besideNoQuorum = recountRefusalOf({
              carry: { ...RUN_OFF_CARRY, ballots: [ballotOf({ seat: 'hyper-cat', choice: 'CANDIDATE 1', },),], usable: 1, },
            },);
            // Each is refused at the very check that names its fault, not at a later one that would refuse it for another.
            expect([wrongFinalists, besideMajority, besideNoQuorum,],).toEqual([
              'ArtifactParseError: artifact parse failed at cat.carries[0].runOff.finalists: '
                + 'expected the finalists of the first ballots, in sheet order, and at least two.',
              'ArtifactParseError: artifact parse failed at cat.carries[0].runOff: '
                + 'expected a run-off only beside first ballots that reached their quorum and gave no candidate a strict majority.',
              'ArtifactParseError: artifact parse failed at cat.carries[0].runOff: '
                + 'expected a run-off only beside first ballots that reached their quorum and gave no candidate a strict majority.',
            ],);
          },
        },),
        it({
          name: 'REFUSES FINALISTS THAT ARE MORE THAN THE TIED LEADERS WHEN THE LABELS IT NAMES ARE THE LEADERS, and finalists that '
            + 'are as many as the tied leaders and other labels, each at the finalists, whichever check would have refused it',
          fn: async () => {
            /**
             A carry whose first ballots tie the first and the third candidate, so those are the finalists.
             */
            const tie: ArtifactConsistencyCarry = {
              ...RUN_OFF_CARRY,
              ballots: [
                ballotOf({ seat: 'hyper-cat', choice: 'CANDIDATE 1', },),
                ballotOf({ seat: 'bedrock-cat', choice: 'CANDIDATE 3', },),
              ],
              usable: 2,
            };
            /**
             A run-off over the given finalists whose ballots and decision are what they give, so only the finalists can be refused.
             */
            function runOffOver(
              { finalists, last, }: { readonly finalists: readonly string[]; readonly last: string; },
            ): ArtifactConsistencyCarry {
              return {
                ...tie,
                runOff: {
                  finalists,
                  ballots: [
                    ballotOf({ seat: 'hyper-cat', choice: 'CANDIDATE 1', },),
                    ballotOf({ seat: 'bedrock-cat', choice: 'CANDIDATE 1', },),
                    ballotOf({ seat: 'vision-cat', choice: last, },),
                  ],
                  usable: 3,
                  quorum: 2,
                },
                decision: { kind: 'chosen', label: 'CANDIDATE 1', },
              };
            }
            /**
             The refusal of finalists that are not the tied leaders.
             */
            const wrongFinalists = 'ArtifactParseError: artifact parse failed at cat.carries[0].runOff.finalists: '
              + 'expected the finalists of the first ballots, in sheet order, and at least two.';
            expect({
              accepted: assertCarriesRecount({
                carries: [runOffOver({ finalists: ['CANDIDATE 1', 'CANDIDATE 3',], last: 'CANDIDATE 3', },),],
                path: 'cat.carries',
                generation: GENERATION_EIGHTEEN,
              },),
              longer: recountRefusalOf({
                carry: runOffOver({ finalists: ['CANDIDATE 1', 'CANDIDATE 3', 'CANDIDATE 2',], last: 'CANDIDATE 3', },),
              },),
              otherLabels: recountRefusalOf({
                carry: runOffOver({ finalists: ['CANDIDATE 1', 'CANDIDATE 2',], last: 'CANDIDATE 2', },),
              },),
              shorter: recountRefusalOf({
                carry: runOffOver({ finalists: ['CANDIDATE 1',], last: 'CANDIDATE 1', },),
              },),
            },).toEqual({
              accepted: undefined,
              longer: wrongFinalists,
              otherLabels: wrongFinalists,
              shorter: wrongFinalists,
            },);
          },
        },),
        it({
          name: 'REFUSES A RUN-OFF QUORUM BELOW ONE, since the bench was asked among the finalists, though the choices it counts '
            + 'give the decision recorded',
          fn: async () => {
            expect(recountRefusalOf({
              carry: { ...RUN_OFF_CARRY, runOff: { ...RUN_OFF, quorum: 0, }, },
            },),).toBe(
              'ArtifactParseError: artifact parse failed at cat.carries[0].runOff.quorum: '
                + 'expected a quorum of one at least, since the bench was asked among the finalists.',
            );
          },
        },),
        it({
          name: 'COUNTS THE RUN-OFF AGAINST THE FINALISTS ONLY, so a choice that names a candidate which was not a finalist is not '
            + 'counted, and a usable count that counts it is refused',
          fn: async () => {
            /**
             A carry whose first ballots tie the first and the third candidate, and whose run-off heard two choices of the second
             candidate, which was not a finalist, and one of the first.
             */
            const strayingRunOff: NonNullable<ArtifactConsistencyCarry['runOff']> = {
              finalists: ['CANDIDATE 1', 'CANDIDATE 3',],
              ballots: [
                ballotOf({ seat: 'hyper-cat', choice: 'CANDIDATE 2', },),
                ballotOf({ seat: 'bedrock-cat', choice: 'CANDIDATE 2', },),
                ballotOf({ seat: 'vision-cat', choice: 'CANDIDATE 1', },),
              ],
              usable: 1,
              quorum: 2,
            };
            const straying: ArtifactConsistencyCarry = {
              ...RUN_OFF_CARRY,
              ballots: [
                ballotOf({ seat: 'hyper-cat', choice: 'CANDIDATE 1', },),
                ballotOf({ seat: 'bedrock-cat', choice: 'CANDIDATE 3', },),
              ],
              usable: 2,
              runOff: strayingRunOff,
              decision: { kind: 'none', reason: 'no-quorum', },
            };
            expect({
              accepted: assertCarriesRecount({ carries: [straying,], path: 'cat.carries', generation: GENERATION_EIGHTEEN, },),
              countedAll: recountRefusalOf({
                carry: { ...straying, runOff: { ...strayingRunOff, usable: 3, }, decision: { kind: 'chosen', label: 'CANDIDATE 2', }, },
              },),
            },).toEqual({
              accepted: undefined,
              countedAll: 'ArtifactParseError: artifact parse failed at cat.carries[0].runOff.usable: '
                + 'expected the number of run-off ballots that can be counted against the finalists.',
            },);
          },
        },),
        it({
          name: 'REFUSES A COUNT OR A DECISION THAT DOES NOT FOLLOW FROM THE RUN-OFF\'S OWN CHOICES: a usable count that is not '
            + 'the choices of the finalists, a decision that is not what they give, and a seat heard twice',
          fn: async () => {
            const wrongUsable = recountRefusalOf({
              carry: { ...RUN_OFF_CARRY, runOff: { ...RUN_OFF, usable: 2, }, },
            },);
            const wrongDecision = recountRefusalOf({
              carry: { ...RUN_OFF_CARRY, decision: { kind: 'chosen', label: 'CANDIDATE 2', }, },
            },);
            const twiceHeard = recountRefusalOf({
              carry: {
                ...RUN_OFF_CARRY,
                runOff: {
                  ...RUN_OFF,
                  ballots: [
                    ballotOf({ seat: 'hyper-cat', choice: 'CANDIDATE 3', },),
                    ballotOf({ seat: 'hyper-cat', choice: 'CANDIDATE 3', },),
                  ],
                  usable: 2,
                },
              },
            },);
            expect([
              wrongUsable.includes('cat.carries[0].runOff.usable',),
              wrongDecision.includes('cat.carries[0].decision',),
              twiceHeard.includes('cat.carries[0].runOff.ballots',),
            ],).toEqual([true, true, true,],);
          },
        },),
      ],
    },),
  ],
},);
