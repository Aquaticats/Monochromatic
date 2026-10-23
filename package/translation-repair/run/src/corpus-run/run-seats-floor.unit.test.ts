/**
 Tests for the writing-bench floor.

 THE FIFTEENTH CLASS, the owner's decision of 2026-09-08: the eighth
 hakureico pass on Bedrock alone settled a page one translator wrote and
 three judges chose, with no editor or refiner reachable, as `SETTLED`. A
 writing bench below the pair a slate needs, with no provider naming its
 return, stops the entry for a pass that has the bench.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import type { BudgetView, } from '@monochromatic-dev/translation-repair-provider';
import {
  belowFloorWritingBenches,
  benchesOf,
  judgeSeatsFor,
  phaseBenches,
  PHASES_WITHOUT_WRITER_STOP,
  reachableSeats,
  RUN_MODELS,
  RUN_TRANSLATORS,
  unreachableWritingBenches,
  WRITING_BENCH_FLOOR,
  WRITING_BENCHES,
  WritingBenchUnreachableError,
} from '../../dist/final/node/index.mjs';
import {
  SEAT_BEDROCK_ONLY_TEXT,
  SEAT_HYPER_TEXT_BEDROCK,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

//region Writing-bench floor tests

/**
 Every provider wet.
 */
const ALL_WET: BudgetView = {
  neuralwatt: false,
  synthetic: false,
  bedrock: false,
  deepseek: false,
  xiaomi: false,
  anthropic: false,
  hyper: false,
  openrouter: false,
};

/**
 The eighth hakureico pass: Bedrock alone wet.
 */
const BEDROCK_ALONE: BudgetView = {
  neuralwatt: true,
  synthetic: true,
  bedrock: false,
  deepseek: true,
  xiaomi: true,
  anthropic: true,
  hyper: true,
  openrouter: true,
};

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: unreachableWritingBenches.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES NOTHING when every provider is wet, and NOTHING for a phase that leans on no writing '
            + 'bench however dry the providers, since judges short of quorum are the thirteenth class and not '
            + 'this one',
          fn: async () => {
            expect(unreachableWritingBenches({
              benches: benchesOf({ seats: judgeSeatsFor({ dry: ALL_WET, },), },),
              names: phaseBenches({ phase: 'lanes', },),
              dry: ALL_WET,
            },),).toEqual([],);
            expect(unreachableWritingBenches({
              benches: benchesOf({ seats: judgeSeatsFor({ dry: BEDROCK_ALONE, },), },),
              names: phaseBenches({ phase: 'consolidation', },),
              dry: BEDROCK_ALONE,
            },),).toEqual([],);
            expect(unreachableWritingBenches({
              benches: benchesOf({ seats: judgeSeatsFor({ dry: BEDROCK_ALONE, },), },),
              names: phaseBenches({ phase: 'pictures', },),
              dry: BEDROCK_ALONE,
            },),).toEqual([],);
          },
        },),
        it({
          name: 'NAMES each writing bench below the pair a slate needs, with the reachable count against the '
            + 'floor, in the order the phase lists them: the eighth hakureico pass at the lanes, no editor, '
            + 'no refiner and one translator reachable on the roster of that day; since google.gemma-4-e2b '
            + 'took the translator seat (2026-09-08) Bedrock alone reaches the pair and the lanes name only '
            + 'the editors and refiners',
          fn: async () => {
            const benches = benchesOf({ seats: judgeSeatsFor({ dry: BEDROCK_ALONE, },), },);
            /**
             Translators Bedrock serves under that view: the pair the floor asks for.
             */
            const translatorsReachable = reachableSeats({
              seats: RUN_TRANSLATORS,
              dry: BEDROCK_ALONE,
            },);
            expect(translatorsReachable,).toEqual([
              SEAT_HYPER_TEXT_BEDROCK,
              SEAT_BEDROCK_ONLY_TEXT,
            ],);
            expect(reachableSeats({
              seats: RUN_MODELS.editorModelIds,
              dry: BEDROCK_ALONE,
            },),).toEqual([],);
            expect(unreachableWritingBenches({
              benches,
              names: phaseBenches({ phase: 'lanes', },),
              dry: BEDROCK_ALONE,
            },),).toEqual([
              `editors 0 of ${String(RUN_MODELS.editorModelIds.length,)} reachable, floor ${String(WRITING_BENCH_FLOOR,)}`,
              `refiners 0 of ${String(RUN_MODELS.refinerModelIds.length,)} reachable, floor ${String(WRITING_BENCH_FLOOR,)}`,
            ],);
            expect(unreachableWritingBenches({
              benches,
              names: phaseBenches({ phase: 'translate lane', },),
              dry: BEDROCK_ALONE,
            },),).toEqual([],);
            /**
             The eighth pass's translators bench, the roster of that day: one Bedrock seat.
             */
            const thatDay = RUN_TRANSLATORS.filter(function seatedBefore(modelId,): boolean {
              return modelId !== SEAT_BEDROCK_ONLY_TEXT;
            },);
            expect(unreachableWritingBenches({
              benches: {
                ...benches,
                translators: thatDay,
              },
              names: phaseBenches({ phase: 'translate lane', },),
              dry: BEDROCK_ALONE,
            },),).toEqual([
              `translators 1 of ${String(thatDay.length,)} reachable, floor ${String(WRITING_BENCH_FLOOR,)}`,
            ],);
          },
        },),
        it({
          name: 'HOLDS THE FLOOR AT A PAIR on exactly the benches that write, so a judge bench never trips it '
            + 'and a bench of two reachable writers passes',
          fn: async () => {
            expect(WRITING_BENCH_FLOOR,).toBe(2,);
            expect([...WRITING_BENCHES,].toSorted(),).toEqual([
              'editors',
              'refiners',
              'translators',
            ],);
            const benches = benchesOf({ seats: judgeSeatsFor({ dry: BEDROCK_ALONE, },), },);
            /**
             The translators bench with two Bedrock-served seats.
             */
            const pair = [
              SEAT_HYPER_TEXT_BEDROCK,
              SEAT_SYNTHETIC_TEXT_EVERYWHERE,
            ] as const;
            expect(unreachableWritingBenches({
              benches: {
                ...benches,
                translators: pair,
              },
              names: ['translators',],
              dry: BEDROCK_ALONE,
            },),).toEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: belowFloorWritingBenches.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES EACH WRITING BENCH BELOW THE FLOOR WITH ITS CLAUSE, the name beside the clause the phase prints, and '
            + 'names no judge bench however short, the same benches unreachableWritingBenches words',
          fn: async () => {
            const dry: BudgetView = { neuralwatt: true, synthetic: true, bedrock: true, deepseek: true, xiaomi: true, anthropic: true, hyper: true, openrouter: true, };
            const benches = benchesOf({ seats: judgeSeatsFor({ dry, },), },);
            const shortOfFloor = belowFloorWritingBenches({
              benches,
              names: phaseBenches({ phase: 'page consistency', },),
              dry,
            },);
            expect({
              below: shortOfFloor,
              sameClauses: shortOfFloor.map(function clauseOf({ clause, },): string {
                return clause;
              },),
              words: unreachableWritingBenches({ benches, names: phaseBenches({ phase: 'page consistency', },), dry, },),
              none: belowFloorWritingBenches({
                benches: benchesOf({ seats: judgeSeatsFor({ dry: ALL_WET, },), },),
                names: phaseBenches({ phase: 'page consistency', },),
                dry: ALL_WET,
              },),
            },).toEqual({
              below: [{ name: 'translators', clause: 'translators 0 of 8 reachable, floor 2', },],
              sameClauses: ['translators 0 of 8 reachable, floor 2',],
              words: ['translators 0 of 8 reachable, floor 2',],
              none: [],
            },);
          },
        },),
      ],
    },),
    describe({
      name: 'PHASES_WITHOUT_WRITER_STOP',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES ONLY THE PAGE CONSISTENCY PHASE, so every other phase that leans on a writing bench still stops the '
            + 'entry when the bench is below its floor',
          fn: async () => {
            expect([...PHASES_WITHOUT_WRITER_STOP,],).toEqual(['page consistency',],);
          },
        },),
      ],
    },),
    describe({
      name: WritingBenchUnreachableError.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES the phase and every clause and nothing from a provider, so the tally can print it',
          fn: async () => {
            const error = new WritingBenchUnreachableError({
              phase: 'lanes',
              clauses: [
                'editors 0 of 4 reachable, floor 2',
                'translators 1 of 7 reachable, floor 2',
              ],
            },);
            expect(error.name,).toBe('WritingBenchUnreachableError',);
            expect(error.messageNamesOnly,).toBe(true,);
            expect(error.clauses.length,).toBe(2,);
            expect(error.message,).toBe(
              'writing bench unreachable at lanes: editors 0 of 4 reachable, floor 2; translators 1 of 7 reachable, '
                + 'floor 2; no provider has named a return the pass can wait for, so the entry stops for a pass that '
                + 'has the bench',
            );
          },
        },),

        it({
          name: 'SAYS THE PASS MAY PARK FOR THE RETURN, and names the other outcome: the gate parks only where the '
            + 'return fits what the pass has left and its spent day was not already waited out, and otherwise the '
            + 'entry ends as a failure a later pass resumes',
          fn: async () => {
            const error = new WritingBenchUnreachableError({
              phase: 'lanes',
              clauses: ['editors 0 of 3 reachable, floor 2',],
              neuralwattReturnsInMs: 301_000,
              neuralwattReturnsAt: '19:15:08 UTC',
            },);
            expect(error.message,).toBe(
              'writing bench unreachable at lanes: editors 0 of 3 reachable, floor 2; NeuralWatt\'s energy returns '
                + 'at 19:15:08 UTC, which would reach the floor, so the entry stops; the pass may park for it where '
                + 'the return fits what the pass has left, and otherwise the entry ends as a failure a later pass '
                + 'resumes',
            );
          },
        },),
      ],
    },),
  ],
},);

//endregion Writing-bench floor tests
