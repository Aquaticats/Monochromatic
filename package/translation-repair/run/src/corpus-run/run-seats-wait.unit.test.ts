/**
 Tests for the bench-quorum reading a phase takes before it starts.

 THE THIRTEENTH CLASS: on 2026-09-07 Hyper named its return in 538 s and was
 held out for exactly that, Bedrock stayed wet, and so nothing waited. The
 translate lane started with every Hyper-only writer refused in the same
 millisecond, and every consolidation round read `quorum-not-met` at 0 ms.
 A bench that cannot reach quorum among the seats a wet provider serves is
 the phase-level twin of the all-dry case the router already waits on.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import type { BudgetView, } from '@monochromatic-dev/translation-repair-provider';
import { rosterQuorumSize, } from '@monochromatic-dev/translation-repair-stage';
import {
  benchesOf,
  judgeSeatsFor,
  phaseBenches,
  reachableSeats,
  RUN_LATE_JUDGES,
  RUN_MODELS,
  RUN_TRANSLATORS,
  RUN_WIDE_SEATS,
  shortBenches,
} from '../../dist/final/node/index.mjs';
import {
  SEAT_HYPER_ONLY,
  SEAT_HYPER_TEXT_BEDROCK,
  SEAT_HYPER_VISION,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

//region Bench quorum tests

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
 The third hakureico pass at 22:00 UTC: Synthetic's week spent, Hyper held
 out by its daily limit, OpenRouter at 0.01 USD, Bedrock alone wet.
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

/**
 XingZ6016 (2026-10-21) at its start: NeuralWatt and OpenRouter wet, Synthetic
 and Hyper dry, Bedrock refused for an expired token and so read as dry.
 */
const XINGZ6016: BudgetView = {
  neuralwatt: false,
  synthetic: true,
  bedrock: true,
  deepseek: false,
  xiaomi: false,
  anthropic: false,
  hyper: true,
  openrouter: false,
};

/**
 A view where Hyper alone is wet: NeuralWatt, Synthetic, Bedrock and OpenRouter
 dry. Hyper serves Gemma text and shows it no picture (its row reports none), so
 this view is a seat served text and shown no picture, which XingZ6016 was
 until `reader-measure` opened OpenRouter to Gemma pictures (2026-10-21).
 */
const HYPER_ALONE: BudgetView = {
  neuralwatt: true,
  synthetic: true,
  bedrock: true,
  deepseek: true,
  xiaomi: true,
  anthropic: true,
  hyper: false,
  openrouter: true,
};

/**
 Benches as the seat reader derives them from one view.

 @param dry - dryness per provider

 @returns Benches keyed by the names the phase table uses

 @example
 ```ts
 const benches = benchesUnder({ dry: ALL_WET, },);
 ```
 */
function benchesUnder(
  { dry, }: { readonly dry: BudgetView; },
) {
  return benchesOf({ seats: judgeSeatsFor({ dry, },), },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: reachableSeats.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'KEEPS every seat when every provider is wet, and ONLY the seats Bedrock serves when it is '
            + 'the one wet provider',
          fn: async () => {
            expect(reachableSeats({
              seats: RUN_WIDE_SEATS,
              dry: ALL_WET,
            },),).toEqual(RUN_WIDE_SEATS,);
            const alone = reachableSeats({
              seats: RUN_WIDE_SEATS,
              dry: BEDROCK_ALONE,
            },);
            expect(alone.length,).toBeLessThan(RUN_WIDE_SEATS.length,);
            expect(alone.includes(SEAT_HYPER_TEXT_BEDROCK,),).toBe(true,);
            // The four-provider seat left every bench on 2026-09-24 (owner cull),
            // so Bedrock alone no longer reaches it through the wide seats.
            expect(RUN_WIDE_SEATS.includes(SEAT_SYNTHETIC_TEXT_EVERYWHERE,),).toBe(false,);
            expect(alone.includes(SEAT_SYNTHETIC_TEXT_EVERYWHERE,),).toBe(false,);
            expect(alone.includes(SEAT_HYPER_ONLY,),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: shortBenches.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES NOTHING when every bench can reach quorum',
          fn: async () => {
            expect(shortBenches({
              benches: benchesUnder({ dry: ALL_WET, },),
              names: phaseBenches({ phase: 'consolidation', },),
              dry: ALL_WET,
            },),).toEqual([],);
          },
        },),
        it({
          name: 'NAMES each bench that cannot reach quorum among the seats a wet provider serves, with the '
            + 'reachable count against the quorum, in the order the phase lists them',
          fn: async () => {
            const clauses = shortBenches({
              benches: benchesUnder({ dry: BEDROCK_ALONE, },),
              names: phaseBenches({ phase: 'lanes', },),
              dry: BEDROCK_ALONE,
            },);
            /**
             Wide seats Bedrock serves under that view.
             */
            const wideReachable = reachableSeats({
              seats: RUN_WIDE_SEATS,
              dry: BEDROCK_ALONE,
            },).length;
            /**
             Writers Bedrock serves under that view.
             */
            const writersReachable = reachableSeats({
              seats: RUN_TRANSLATORS,
              dry: BEDROCK_ALONE,
            },).length;
            /**
             Editors and refiners, none of which Bedrock serves.
             */
            const editors = RUN_MODELS.editorModelIds;
            const refiners = RUN_MODELS.refinerModelIds;
            expect(wideReachable,).toBeLessThan(rosterQuorumSize({ rosterSize: RUN_WIDE_SEATS.length, },),);
            expect(writersReachable,).toBeLessThan(rosterQuorumSize({ rosterSize: RUN_TRANSLATORS.length, },),);
            expect(clauses,).toEqual([
              `wide ${String(wideReachable,)} of ${String(RUN_WIDE_SEATS.length,)} reachable, quorum ${
            String(rosterQuorumSize({ rosterSize: RUN_WIDE_SEATS.length, },),)
          }`,
              `editors 0 of ${String(editors.length,)} reachable, quorum ${
            String(rosterQuorumSize({ rosterSize: editors.length, },),)
          }`,
              `refiners 0 of ${String(refiners.length,)} reachable, quorum ${
            String(rosterQuorumSize({ rosterSize: refiners.length, },),)
          }`,
              `translators ${String(writersReachable,)} of ${String(RUN_TRANSLATORS.length,)} reachable, quorum ${
            String(rosterQuorumSize({ rosterSize: RUN_TRANSLATORS.length, },),)
          }`,
            ],);
          },
        },),
        it({
          name: 'HOLDS THE READERS TO A PAIR, not to a majority: two Bedrock readers of six can corroborate a '
            + 'picture, so a Bedrock-only pictures phase is not short (2026-09-08), and one reader alone is',
          fn: async () => {
            const benches = benchesUnder({ dry: BEDROCK_ALONE, },);
            /**
             Readers Bedrock serves under that view.
             */
            const readersReachable = reachableSeats({
              seats: benches.readers,
              dry: BEDROCK_ALONE,
            },);
            expect(readersReachable.length,).toBe(2,);
            expect(shortBenches({
              benches,
              names: phaseBenches({ phase: 'pictures', },),
              dry: BEDROCK_ALONE,
            },),).toEqual([],);
            /**
             The same bench with one Bedrock reader gone: a single reachable reader.
             */
            const oneReader = benches.readers.filter(function keeps(modelId,): boolean {
              return modelId !== readersReachable[0];
            },);
            expect(shortBenches({
              benches: {
                ...benches,
                readers: oneReader,
              },
              names: phaseBenches({ phase: 'pictures', },),
              dry: BEDROCK_ALONE,
            },),).toEqual([
              `readers 1 of ${String(oneReader.length,)} reachable, quorum 2`,
            ],);
          },
        },),
        it({
          name: 'COUNTS A READER BY THE PROVIDERS THAT CAN SHOW IT A PICTURE, not by the ones that serve it text '
            + '(XingZ6016: the seat line said readers=5 while Gemma was served text by OpenRouter and shown '
            + 'pictures by Bedrock alone, which was expired; the case now reads a Hyper-alone view, where Gemma is '
            + 'served text and shown no picture)',
          fn: async () => {
            expect(reachableSeats({
              seats: [SEAT_HYPER_TEXT_BEDROCK, SEAT_HYPER_VISION,],
              dry: HYPER_ALONE,
            },),).toEqual([SEAT_HYPER_TEXT_BEDROCK, SEAT_HYPER_VISION,],);
            expect(reachableSeats({
              seats: [SEAT_HYPER_TEXT_BEDROCK, SEAT_HYPER_VISION,],
              dry: HYPER_ALONE,
              reach: 'vision',
            },),).toEqual([SEAT_HYPER_VISION,],);
            expect(shortBenches({
              benches: {
                ...benchesUnder({ dry: HYPER_ALONE, },),
                readers: [SEAT_HYPER_TEXT_BEDROCK, SEAT_HYPER_VISION,],
              },
              names: phaseBenches({ phase: 'pictures', },),
              dry: HYPER_ALONE,
            },),).toEqual(['readers 1 of 2 reachable, quorum 2',],);
          },
        },),
        it({
          name: 'COUNTS GEMMA AS SEEN BY OPENROUTER at the XingZ6016 start (Bedrock expired, OpenRouter wet), since '
            + 'reader-measure opened that route to its pictures (2026-10-21): both readers reach, the bench is not short',
          fn: async () => {
            expect(reachableSeats({
              seats: [SEAT_HYPER_TEXT_BEDROCK, SEAT_HYPER_VISION,],
              dry: XINGZ6016,
              reach: 'vision',
            },),).toEqual([SEAT_HYPER_TEXT_BEDROCK, SEAT_HYPER_VISION,],);
            expect(shortBenches({
              benches: {
                ...benchesUnder({ dry: XINGZ6016, },),
                readers: [SEAT_HYPER_TEXT_BEDROCK, SEAT_HYPER_VISION,],
              },
              names: phaseBenches({ phase: 'pictures', },),
              dry: XINGZ6016,
            },),).toEqual([],);
          },
        },),
        it({
          name: 'NAMES THE READERS A WET PROVIDER CAN SHOW A PICTURE, beside the bench the seat line counts',
          fn: async () => {
            expect(judgeSeatsFor({ dry: HYPER_ALONE, },).readersSeeing
              .includes(SEAT_HYPER_TEXT_BEDROCK,),).toBe(false,);
            expect(judgeSeatsFor({ dry: HYPER_ALONE, },).readersSeeing
              .includes(SEAT_HYPER_VISION,),).toBe(true,);
            expect(judgeSeatsFor({ dry: XINGZ6016, },).readersSeeing
              .includes(SEAT_HYPER_TEXT_BEDROCK,),).toBe(true,);
            expect(judgeSeatsFor({ dry: ALL_WET, },).readersSeeing
              .includes(SEAT_HYPER_TEXT_BEDROCK,),).toBe(true,);
          },
        },),
        it({
          name: 'READS ONLY the benches the phase leans on, so a phase that reads no pictures never waits on '
            + 'the readers',
          fn: async () => {
            expect(phaseBenches({ phase: 'pictures', },),).toEqual(['readers',],);
            expect(phaseBenches({ phase: 'translate lane', },),).toEqual([
              'translators',
              'select',
            ],);
            expect(phaseBenches({ phase: 'consolidation', },).includes('readers',),).toBe(false,);
          },
        },),
        it({
          name: 'NAMES EVERY PHASE\'S BENCHES as the table in `run-seats-wait.ts` documents them, the insertion '
            + 'admission on the roster it asks as the preparation does (ledger X12), so a phase never waits on '
            + 'a bench it does not ask nor starts past one it does',
          fn: async () => {
            /**
             Every phase a seat reading is taken for.
             */
            const phases = [
              'preparation',
              'pictures',
              'insertion admission',
              'lanes',
              'translate lane',
              'lane contest',
              'consolidation',
              'page consistency',
            ] as const;
            expect(
              Object.fromEntries(phases.map(function benchesOfPhase(phase,) {
                return [phase, phaseBenches({ phase, },),];
              },),),
            ).toEqual({
              preparation: ['roster',],
              pictures: ['readers',],
              'insertion admission': ['roster',],
              lanes: ['wide', 'editors', 'refiners', 'translators',],
              'translate lane': ['translators', 'select',],
              'lane contest': ['late',],
              consolidation: ['slate', 'late',],
              'page consistency': ['late', 'translators',],
            },);
          },
        },),
        it({
          name: 'NAMES THE WRITER BENCH BESIDE THE LATE BENCH when the page consistency phase starts with Bedrock '
            + 'alone wet, since its carry asks the translate stage\'s writers after its judges, and names nothing '
            + 'when every provider is wet',
          fn: async () => {
            /**
             Late judges Bedrock serves under that view.
             */
            const lateReachable = reachableSeats({
              seats: RUN_LATE_JUDGES,
              dry: BEDROCK_ALONE,
            },).length;
            /**
             Writers Bedrock serves under that view.
             */
            const writersReachable = reachableSeats({
              seats: RUN_TRANSLATORS,
              dry: BEDROCK_ALONE,
            },).length;
            expect(writersReachable,).toBeLessThan(rosterQuorumSize({ rosterSize: RUN_TRANSLATORS.length, },),);
            expect(shortBenches({
              benches: benchesUnder({ dry: BEDROCK_ALONE, },),
              names: phaseBenches({ phase: 'page consistency', },),
              dry: BEDROCK_ALONE,
            },),).toEqual([
              `late ${String(lateReachable,)} of ${String(RUN_LATE_JUDGES.length,)} reachable, quorum ${
            String(rosterQuorumSize({ rosterSize: RUN_LATE_JUDGES.length, },),)
          }`,
              `translators ${String(writersReachable,)} of ${String(RUN_TRANSLATORS.length,)} reachable, quorum ${
            String(rosterQuorumSize({ rosterSize: RUN_TRANSLATORS.length, },),)
          }`,
            ],);
            expect(shortBenches({
              benches: benchesUnder({ dry: ALL_WET, },),
              names: phaseBenches({ phase: 'page consistency', },),
              dry: ALL_WET,
            },),).toEqual([],);
          },
        },),
      ],
    },),
  ],
},);

//endregion Bench quorum tests
