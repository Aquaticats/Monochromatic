/**
 Tests the bench floors an entry's give-up of a provider must leave met (`unmetBenchFloors`): the writing floors and the
 judging quorums, counted by the seat reader's own helpers over a view in which only some providers can be asked.

 The roster is the run's real one and the view is a record the case writes, so no meter is read, no clock moves and
 no provider is asked.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import { dryExcept, } from '@monochromatic-dev/translation-repair-provider/ts/dry-except.test-fixture.ts';
import {
  BENCHES_ANY_PHASE_LEANS_ON,
  benchesOf,
  type JudgeSeatPhase,
  judgeSeatsFor,
  phaseBenches,
  shortBenches,
  unmetBenchFloors,
  unreachableWritingBenches,
  WRITING_BENCH_FLOOR,
} from '../../dist/final/node/index.mjs';

/**
 Every phase a seat reading is taken for.
 */
const PHASES: readonly JudgeSeatPhase[] = [
  'preparation',
  'pictures',
  'insertion admission',
  'lanes',
  'translate lane',
  'lane contest',
  'consolidation',
  'page consistency',
];

await describe({
  name: unmetBenchFloors.name,
  concurrency: 1,
  children: [
    it({
      name: 'FINDS NO FLOOR UNMET WITH HYPER ALONE, OR OPENROUTER ALONE: either serves seats enough for every bench any '
        + 'phase leans on',
      fn: async () => {
        expect(unmetBenchFloors({ dry: dryExcept({ wet: ['hyper',], },), },),).toEqual([],);
        expect(unmetBenchFloors({ dry: dryExcept({ wet: ['openrouter',], },), },),).toEqual([],);
      },
    },),

    it({
      name: 'NAMES THE WRITING FLOOR WHEN THE PROVIDER LEFT IS TOO SMALL TO LIFT A WRITING BENCH: DeepSeek serves one '
        + 'translator and the floor is the seat reader\'s own',
      fn: async () => {
        /**
         What stays unmet with DeepSeek alone.
         */
        const unmet = unmetBenchFloors({ dry: dryExcept({ wet: ['deepseek',], },), },);
        expect(
          unmet.filter(function isTranslatorFloor(clause,): boolean {
            return clause.startsWith('translators ',) && clause.endsWith(`, floor ${String(WRITING_BENCH_FLOOR,)}`,);
          },).length,
        ).toBe(1,);
      },
    },),

    it({
      name: 'COUNTS A JUDGING QUORUM TOO: NeuralWatt alone lifts every writing bench to its floor and leaves the judging '
        + 'benches short of their quorums, so the floors are not met',
      fn: async () => {
        /**
         What stays unmet with NeuralWatt alone.
         */
        const unmet = unmetBenchFloors({ dry: dryExcept({ wet: ['neuralwatt',], },), },);
        expect(
          unmet.filter(function isWritingFloor(clause,): boolean {
            return clause.includes(', floor ',);
          },),
        ).toEqual([],);
        expect(
          unmet.filter(function isSelectQuorum(clause,): boolean {
            return clause.startsWith('select ',) && clause.includes(', quorum ',);
          },).length,
        ).toBe(1,);
      },
    },),

    it({
      name: 'READS THE SEAT READER\'S OWN HELPERS: the clauses are the writing floors then the quorums that the helpers '
        + 'name over the same seats and the same view, for every bench any phase leans on',
      fn: async () => {
        /**
         A view with two providers that can be asked.
         */
        const dry = dryExcept({
          wet: [
            'neuralwatt',
            'deepseek',
          ],
        },);
        /**
         The benches the reader derives from that view.
         */
        const benches = benchesOf({ seats: judgeSeatsFor({ dry, },), },);
        expect(unmetBenchFloors({ dry, },),).toEqual([
          ...unreachableWritingBenches({
            benches,
            names: BENCHES_ANY_PHASE_LEANS_ON,
            dry,
          },),
          ...shortBenches({
            benches,
            names: BENCHES_ANY_PHASE_LEANS_ON,
            dry,
          },),
        ],);
      },
    },),

    it({
      name: 'COVERS EVERY BENCH EVERY PHASE LEANS ON, once each: a bench only a later phase reads counts as much as one '
        + 'the phase now seating reads',
      fn: async () => {
        for (const phase of PHASES) {
          for (const name of phaseBenches({ phase, },)) {
            expect(BENCHES_ANY_PHASE_LEANS_ON.includes(name,),).toBe(true,);
          }
        }
        expect(new Set(BENCHES_ANY_PHASE_LEANS_ON,).size,).toBe(BENCHES_ANY_PHASE_LEANS_ON.length,);
        // With nobody to ask, every one of those benches stands short of its quorum.
        /**
         What stays unmet with no provider to ask.
         */
        const unmet = unmetBenchFloors({ dry: dryExcept({ wet: [], },), },);
        for (const name of BENCHES_ANY_PHASE_LEANS_ON) {
          expect(
            unmet.some(function namesTheBench(clause,): boolean {
              return clause.startsWith(`${name} `,);
            },),
          ).toBe(true,);
        }
      },
    },),
  ],
},);
