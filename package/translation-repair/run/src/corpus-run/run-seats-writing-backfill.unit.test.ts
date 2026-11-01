/**
 Tests for the editor and refiner benches a reading derives: the static seats a wet provider serves, then the next
 models of the owner's role order a wet provider serves, up to the static width.

 THE CASE (corpus pass XingZ6024, 2026-11-01, build e9b966af55): NeuralWatt, Hyper, OpenRouter and Bedrock dry, and
 Synthetic, DeepSeek, Xiaomi and Anthropic wet. The refiner bench was the static three (GLM-5.3-Flash, DeepSeek V4.1
 Flash, minimax-m3), of which DeepSeek alone was reachable, so both attempts stopped with `refiners 1 of 3 reachable,
 floor 2`, while Kimi-K3 and Qwen3.8-27B, both measured refiners, were served by the wet Synthetic and sat unused. The
 owner's answer of 2026-11-01 to what a run does when a writing bench has fewer than two reachable seats was "Backfill,
 and add sonnet": fill the dark seats with the next reachable models of the role order, and seat Claude Sonnet 5.5 on the
 refiner bench as it is on the editor bench.

 EVERY EXPECTATION IS WORKED OUT FROM THE STATIC BENCH, THE ROLE ORDER AND WHO SERVES WHAT (`writing-bench-expectation`),
 never from a list of model names written into a case.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { levelCapturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';

import {
  type BudgetView,
  type MeterState,
} from '@monochromatic-dev/translation-repair-provider';
import {
  MEASURED_CHECKER_ORDER,
  reaskPoolOf,
  type RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract';
import {
  benchesOf,
  HYPER_SLOW_JUDGES,
  judgeSeatsFor,
  phaseBenches,
  readPhaseSeats,
  reachableSeats,
  RUN_MODELS,
  unreachableWritingBenches,
  WRITING_BENCH_FLOOR,
  WritingBenchUnreachableError,
} from '../../dist/final/node/index.mjs';
import {
  SEAT_BEDROCK_ONLY_VISION_UNSEATED,
  SEAT_OPENROUTER_ONLY_CHECKER,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { expectedWritingBench, } from './writing-bench-expectation.test-fixture.ts';

//region Writing bench backfill tests

/**
 The reading of corpus pass XingZ6024: Synthetic, DeepSeek, Xiaomi and Anthropic wet, the other four dry.
 */
const XINGZ6024: BudgetView = {
  neuralwatt: true,
  synthetic: false,
  hyper: true,
  bedrock: true,
  deepseek: false,
  xiaomi: false,
  anthropic: false,
  openrouter: true,
};

/**
 Only DeepSeek wet, which serves one model of the pool and so leaves every writing bench short.
 */
const DEEPSEEK_ALONE: BudgetView = {
  neuralwatt: true,
  synthetic: true,
  hyper: true,
  bedrock: true,
  deepseek: false,
  xiaomi: true,
  anthropic: true,
  openrouter: true,
};

/**
 Only Hyper wet, which serves the Hyper-slow Qwen3.8-27B and Kimi-K3 among the candidates.
 */
const HYPER_ALONE: BudgetView = {
  neuralwatt: true,
  synthetic: true,
  hyper: false,
  bedrock: true,
  deepseek: true,
  xiaomi: true,
  anthropic: true,
  openrouter: true,
};

/**
 Only Bedrock and OpenRouter wet, which serve models the pool holds out of a writing role, two with a hold of their card
 and one the owner culled.
 */
const BEDROCK_AND_OPENROUTER: BudgetView = {
  neuralwatt: true,
  synthetic: true,
  hyper: true,
  bedrock: false,
  deepseek: true,
  xiaomi: true,
  anthropic: true,
  openrouter: false,
};

/**
 Every meter answering wet, as a scripted client reports it when a case does not care what the meters said.
 */
const METERS_WET: Readonly<Record<keyof BudgetView, MeterState>> = {
  neuralwatt: 'wet',
  synthetic: 'wet',
  bedrock: 'wet',
  deepseek: 'wet',
  xiaomi: 'wet',
  anthropic: 'wet',
  hyper: 'wet',
  openrouter: 'wet',
};

/**
 No provider held out.
 */
const NO_HOLDS = {
  neuralwatt: 0,
  synthetic: 0,
  bedrock: 0,
  deepseek: 0,
  xiaomi: 0,
  anthropic: 0,
  hyper: 0,
  openrouter: 0,
};

/**
 Both writing roles, each beside the static bench the run lists for it.
 */
const WRITING_ROLES = [
  {
    role: 'editor',
    key: 'editors',
    staticSeats: RUN_MODELS.editorModelIds,
  },
  {
    role: 'refiner',
    key: 'refiners',
    staticSeats: RUN_MODELS.refinerModelIds,
  },
] as const;

/**
 Every dryness view of the eight meters, one per subset of the providers.

 @returns One view per subset

 @example
 ```ts
 const views = everyView();
 ```
 */
function everyView(): readonly BudgetView[] {
  /**
   Providers in the order a mask names them.
   */
  const providers = [
    'neuralwatt',
    'synthetic',
    'hyper',
    'bedrock',
    'deepseek',
    'xiaomi',
    'anthropic',
    'openrouter',
  ] as const;
  return Array.from({ length: 2 ** providers.length, }, function viewAt(
    _unused,
    mask,
  ): BudgetView {
    /**
     Whether the provider at one bit is dry in this view.

     @param bit - provider position

     @returns True when that provider is dry
     */
    function dryAt(bit: number,): boolean {
      return (mask & (1 << bit)) !== 0;
    }
    return {
      neuralwatt: dryAt(0,),
      synthetic: dryAt(1,),
      hyper: dryAt(2,),
      bedrock: dryAt(3,),
      deepseek: dryAt(4,),
      xiaomi: dryAt(5,),
      anthropic: dryAt(6,),
      openrouter: dryAt(7,),
    };
  },);
}

/**
 Builds a client that answers one dryness view with every meter wet and no hold.

 @param dry - view the client reads

 @returns Client with only the seat reader's surface

 @example
 ```ts
 const client = clientOf({ dry: XINGZ6024, },);
 ```
 */
function clientOf(
  { dry, }: { readonly dry: BudgetView; },
) {
  return {
    providerDryness: async (): Promise<BudgetView> => dry,
    providerMeterStates: async (): Promise<Readonly<Record<keyof BudgetView, MeterState>>> => METERS_WET,
    providerHolds: () => NO_HOLDS,
  };
}

/**
 The benches a reading must derive, worked out for both writing roles.

 @param dry - view of the reading

 @returns Expected bench per role beside the seats the run derived

 @example
 ```ts
 const { editors, } = benchesFor({ dry: XINGZ6024, },);
 ```
 */
function benchesFor(
  { dry, }: { readonly dry: BudgetView; },
) {
  /**
   Seats the run derives.
   */
  const seats = judgeSeatsFor({ dry, },);
  /**
   Expected bench of each role.
   */
  const expected = WRITING_ROLES.map(function expectedOf({
    role,
    staticSeats,
  },) {
    return expectedWritingBench({
      role,
      dry,
      staticSeats,
      checkerBench: seats.checkers,
      hyperSlow: HYPER_SLOW_JUDGES,
      selfCertificationPermitted: RUN_MODELS.checkerSelfCertificationPermitted,
    },);
  },);
  return {
    seats,
    derived: {
      editors: seats.repairModels.editorModelIds,
      refiners: seats.repairModels.refinerModelIds,
    },
    expected: {
      editors: expected[0]?.bench ?? [],
      refiners: expected[1]?.bench ?? [],
    },
    backfilled: {
      editors: expected[0]?.backfilled ?? [],
      refiners: expected[1]?.backfilled ?? [],
    },
  };
}

await describe({
  name: 'writing bench per reading',
  concurrency: 1,
  children: [
    describe({
      name: judgeSeatsFor.name,
      concurrency: 1,
      children: [
        it({
          name: 'SEATS EACH WRITING BENCH AT OR ABOVE THE FLOOR in the XingZ6024 reading, from the static seats a wet '
            + 'provider serves and then the role order, where the refiner bench held one reachable seat of three',
          fn: async () => {
            const {
              derived,
              expected,
              backfilled,
            } = benchesFor({ dry: XINGZ6024, },);
            for (const { key, } of WRITING_ROLES) {
              expect(derived[key],).toEqual(expected[key],);
              expect(reachableSeats({
                seats: derived[key],
                dry: XINGZ6024,
              },)
                .length,).toBeGreaterThanOrEqual(WRITING_BENCH_FLOOR,);
              // THE BACKFILL WAS NEEDED AND TAKEN: this reading serves fewer static seats than the width.
              expect(backfilled[key].length,).toBeGreaterThan(0,);
            }
          },
        },),
        it({
          name: 'DERIVES BOTH WRITING BENCHES IN EVERY READING OF THE EIGHT METERS as the static seats a wet provider serves, '
            + 'the role order, then the static seats nobody serves, never wider than the static bench and never repeating a seat',
          fn: async () => {
            for (const dry of everyView()) {
              const {
                derived,
                expected,
              } = benchesFor({ dry, },);
              for (const { staticSeats, key, } of WRITING_ROLES) {
                expect(derived[key],).toEqual(expected[key],);
                expect(derived[key].length,).toBeLessThanOrEqual(staticSeats.length,);
                expect(new Set(derived[key],).size,).toBe(derived[key].length,);
              }
            }
          },
        },),
        it({
          name: 'NEVER SEATS A MODEL THE ROLE\'S POOL HOLDS OUT, nor one the owner culled, in any reading: only static seats and '
            + 'models of the role pool sit on a writing bench',
          fn: async () => {
            for (const dry of everyView()) {
              const { derived, } = benchesFor({ dry, },);
              for (const { staticSeats, role, key, } of WRITING_ROLES) {
                for (const modelId of derived[key]) {
                  expect(
                    staticSeats.includes(modelId,) || reaskPoolOf({ role, },).includes(modelId,),
                  ).toBe(true,);
                }
              }
            }
          },
        },),
        it({
          name: 'SEATS NO MODEL WITH A HOLD OF ITS CARD or one only OpenRouter would buy when only Bedrock and OpenRouter '
            + 'are wet, and still fills the dark seats from the role order',
          fn: async () => {
            const {
              derived,
              expected,
              backfilled,
            } = benchesFor({ dry: BEDROCK_AND_OPENROUTER, },);
            for (const { key, } of WRITING_ROLES) {
              expect(derived[key],).toEqual(expected[key],);
              expect(backfilled[key].length,).toBeGreaterThan(0,);
              // A CULLED MODEL, A MODEL NOBODY MEASURED AS A WRITER AND A MODEL NOBODY MEASURED AS A JUDGE, all served here.
              for (const held of [
                SEAT_SYNTHETIC_TEXT_EVERYWHERE,
                SEAT_OPENROUTER_ONLY_CHECKER,
                SEAT_BEDROCK_ONLY_VISION_UNSEATED,
              ])
                expect(derived[key],).not.toContain(held,);
            }
          },
        },),
        it({
          name: 'KEEPS A MODEL THE RUN WITHHOLDS WHILE HYPER WOULD SERVE IT OFF THE BACKFILL: with only Hyper wet the '
            + 'Hyper-slow Qwen3.8-27B is skipped and the next model of the role order takes the place',
          fn: async () => {
            const {
              derived,
              expected,
              backfilled,
            } = benchesFor({ dry: HYPER_ALONE, },);
            expect(derived.editors,).toEqual(expected.editors,);
            expect(derived.refiners,).toEqual(expected.refiners,);
            expect(HYPER_SLOW_JUDGES.has(SEAT_SYNTHETIC_VISION_NO_OPENROUTER,),).toBe(true,);
            expect(derived.editors,).not.toContain(SEAT_SYNTHETIC_VISION_NO_OPENROUTER,);
            expect(derived.refiners,).not.toContain(SEAT_SYNTHETIC_VISION_NO_OPENROUTER,);
            // THE NEXT MODEL OF THE ORDER STANDS IN: a skipped candidate is not a lost place.
            expect(backfilled.editors,).toContain(SEAT_SYNTHETIC_VISION_WITHHELD,);
            expect(derived.editors,).toContain(SEAT_SYNTHETIC_VISION_WITHHELD,);
          },
        },),
        it({
          name: 'SEATS A CANDIDATE THAT IS ALSO A CHECKER ONLY WHERE THE CHECKER BENCH OF THE SAME READING HOLDS IT, and only '
            + 'while the contract permits a checker to confirm text it wrote (Kimi-K3 in the XingZ6024 reading)',
          fn: async () => {
            expect(RUN_MODELS.checkerSelfCertificationPermitted,).toBe(true,);
            const { seats, derived, } = benchesFor({ dry: XINGZ6024, },);
            expect(MEASURED_CHECKER_ORDER,).toContain(SEAT_SYNTHETIC_VISION_WITHHELD,);
            expect(seats.checkers,).toContain(SEAT_SYNTHETIC_VISION_WITHHELD,);
            expect(derived.refiners,).toContain(SEAT_SYNTHETIC_VISION_WITHHELD,);
            for (const dry of everyView()) {
              const { seats: reading, derived, } = benchesFor({ dry, },);
              for (const modelId of [...derived.editors, ...derived.refiners,]) {
                const isStatic = RUN_MODELS.editorModelIds.includes(modelId,)
                  || RUN_MODELS.refinerModelIds.includes(modelId,);
                if (MEASURED_CHECKER_ORDER.includes(modelId,) && (!isStatic))
                  expect(reading.checkers,).toContain(modelId,);
              }
            }
          },
        },),
        it({
          name: 'REACHES EVERY READER OF THE BENCH: the roster the repair lane is handed, the benches the floor reads and the '
            + 'stage rosters all carry the derived bench, and the floor counts it',
          fn: async () => {
            const {
              seats,
              derived,
            } = benchesFor({ dry: XINGZ6024, },);
            expect(seats.repairModels.editorModelIds,).toEqual(derived.editors,);
            expect(seats.repairModels.refinerModelIds,).toEqual(derived.refiners,);
            const benches = benchesOf({ seats, },);
            expect(benches.editors,).toEqual(derived.editors,);
            expect(benches.refiners,).toEqual(derived.refiners,);
            expect(unreachableWritingBenches({
              benches,
              names: phaseBenches({ phase: 'lanes', },),
              dry: XINGZ6024,
            },),).toEqual([],);
          },
        },),
      ],
    },),
    describe({
      name: readPhaseSeats.name,
      concurrency: 1,
      children: [
        it({
          name: 'DOES NOT STOP THE LANES in the XingZ6024 reading, which stopped both attempts of the pass on `refiners 1 of 3 '
            + 'reachable, floor 2`, and names the seats it backfilled on the JUDGE SEATS line',
          fn: async () => {
            const lines: string[] = [];
            const { seats, unreachableWriters, } = await readPhaseSeats({
              client: clientOf({ dry: XINGZ6024, },),
              phase: 'lanes',
              signal: new AbortController().signal,
              l: levelCapturingLogger({ lines, },),
              pollMs: 5,
            },);
            const { backfilled, } = benchesFor({ dry: XINGZ6024, },);
            expect(unreachableWriters,).toEqual([],);
            expect(seats.repairModels.refinerModelIds,).toEqual(benchesFor({ dry: XINGZ6024, },).expected.refiners,);
            /**
             The line the reading wrote.
             */
            const seatsLine = lines.find(function isSeatsLine(line,): boolean {
              return line.startsWith('info JUDGE SEATS phase=lanes',);
            },);
            expect(seatsLine,).toBeDefined();
            expect(seatsLine,).toContain(
              `backfilled-by-role-order=editors:${backfilled.editors.join(',',)}|refiners:${backfilled.refiners.join(',',)} `,
            );
            expect(seatsLine,).toContain(`editors=${String(seats.repairModels.editorModelIds.length,)} `,);
            expect(seatsLine,).toContain(`refiners=${String(seats.repairModels.refinerModelIds.length,)} `,);
          },
        },),
        it({
          name: 'STILL STOPS THE LANES WHEN EVERY BACKFILL CANDIDATE IS DARK (only DeepSeek wet), and the stop line says the '
            + 'role order gave nothing',
          fn: async () => {
            const lines: string[] = [];
            const { seats, expected, } = benchesFor({ dry: DEEPSEEK_ALONE, },);
            /**
             Why the lanes stop, from the benches the run derived.
             */
            const clauses = ['editors', 'refiners',].map(function clauseOf(name,): string {
              /**
               Bench of that name.
               */
              const bench = (name === 'editors') ? expected.editors : expected.refiners;
              return `${name} ${String(reachableSeats({ seats: bench, dry: DEEPSEEK_ALONE, },).length,)} of ${
                String(bench.length,)
              } reachable, floor ${String(WRITING_BENCH_FLOOR,)}`;
            },);
            let refusal: unknown;
            try {
              await readPhaseSeats({
                client: clientOf({ dry: DEEPSEEK_ALONE, },),
                phase: 'lanes',
                signal: new AbortController().signal,
                l: levelCapturingLogger({ lines, },),
                pollMs: 5,
              },);
            }
            catch (error) {
              refusal = error;
            }
            expect(refusal,).toBeInstanceOf(WritingBenchUnreachableError,);
            expect(seats.repairModels.editorModelIds,).toEqual(expected.editors,);
            expect(lines,).toContain(
              `error JUDGE SEATS phase=lanes writing bench unreachable: ${clauses.join('; ',)}; `
                + 'backfilled by role order: none; stopping the entry',
            );
          },
        },),
      ],
    },),
  ],
},);

//endregion Writing bench backfill tests
