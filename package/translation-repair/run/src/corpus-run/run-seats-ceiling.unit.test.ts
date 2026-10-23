/**
 Tests that the seat reader states, for each bench that stands short of its quorum, what the holds running on the
 providers could bring back and whether that lifts the bench to its quorum, in the line that announces a wait and in the
 line that says none is taken.

 THE SHAPE, from the corpus pass `XingZ6020` (2026-10-22 and 2026-10-23): NeuralWatt, DeepSeek and Xiaomi wet, Bedrock,
 Hyper and OpenRouter dry, and Synthetic refused and held out for five minutes. The judge benches read 1 of 7 with
 Synthetic held and 3 of 7 with every provider wet, against a quorum of 4: no return lifts them, and the pass waited for
 the hold 228 times all the same (112 of them in the translate lane, 52 in the lane contest, 59 in the consolidation).
 The wait itself is not taken in the phases that lean on judge benches alone, since the hold cures none of them (the
 cure rule of ledger B827 builds on commit `a019032c6d`); what the log said was that a hold would not help and nothing of
 the ceiling the benches stood at. The translate lane also leans on the translators, which Synthetic's return does lift
 from 2 of 8 to their quorum, so that wait is the right one and is announced with the benches it can and cannot lift.

 Fixtures are cat-themed invention; every view and hold is scripted and no HTTP is made.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import type { BudgetView, } from '@monochromatic-dev/translation-repair-provider';
import {
  type ProviderName,
  providerRecord,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  levelCapturingLogger,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import {
  metersAllWet,
} from '@monochromatic-dev/translation-repair-provider/ts/meter-states.test-fixture.ts';
import {
  type JudgeSeatPhase,
  readJudgeSeats,
} from '../../dist/final/node/index.mjs';

//region Ceiling tests

/**
 The XingZ6020 view: NeuralWatt, DeepSeek and Xiaomi wet, and every other provider dry (Synthetic by its hold, the rest by
 their meters or the owner's switch).
 */
const SYNTHETIC_HELD: BudgetView = {
  neuralwatt: false,
  synthetic: true,
  bedrock: true,
  deepseek: false,
  xiaomi: false,
  hyper: true,
  openrouter: true,
};

/**
 No provider held out.
 */
const NO_HOLDS = providerRecord({ of: (): number => 0, },);

/**
 A hold short enough for a case to wait out on the real clock.
 */
const SHORT_HOLD_MS = 40;

/**
 What one reading logged and how many dryness reads it took.
 */
type Reading = {
  readonly lines: readonly string[];
  readonly reads: number;
};

/**
 Reads one phase's seats over a scripted view with a hold on one provider that ends with the first wait.

 @param phase - phase the reading is for

 @param view - dryness every read returns

 @param held - provider the hold is on, none when no hold runs

 @returns What the reading logged and the dryness reads it took (1 without a wait, 2 with one)

 @example
 ```ts
 const { lines, reads, } = await readingOf({ phase: 'lane contest', view: SYNTHETIC_HELD, held: 'synthetic', },);
 ```
 */
async function readingOf(
  {
    phase,
    view,
    held,
  }: {
    readonly phase: JudgeSeatPhase;
    readonly view: BudgetView;
    readonly held?: ProviderName;
  },
): Promise<Reading> {
  /**
   Lines the reading wrote.
   */
  const lines: string[] = [];
  /**
   Reads taken so far, and how often the holds were asked (they end with the first wait).
   */
  const counted = { reads: 0, asks: 0, };
  await readJudgeSeats({
    client: {
      providerDryness: async () => {
        counted.reads += 1;
        return view;
      },
      providerMeterStates: metersAllWet,
      providerHolds: () => {
        counted.asks += 1;
        return ((counted.asks === 1) && (held !== undefined))
          ? providerRecord({
            of: function holdOf(provider,): number {
              return (provider === held) ? SHORT_HOLD_MS : 0;
            },
          },)
          : NO_HOLDS;
      },
    },
    phase,
    signal: new AbortController().signal,
    l: levelCapturingLogger({ lines, },),
    pollMs: 5,
  },);
  return { lines, reads: counted.reads, };
}

/**
 The first warning of a reading that is about a short bench, whether it announces a wait or says none is taken.

 @param lines - what the reading logged

 @returns Line, empty where the reading wrote none (a case that expects one then fails on its content)

 @example
 ```ts
 const warning = shortBenchWarningOf({ lines, },);
 ```
 */
function shortBenchWarningOf({ lines, }: { readonly lines: readonly string[]; },): string {
  return lines.find(function isShortBench(line,): boolean {
    return line.startsWith('warn JUDGE SEATS',) && line.includes('short of quorum',);
  },) ?? '';
}

await describe({
  name: 'the seat reader and the ceiling of a short bench under a hold',
  concurrency: 1,
  children: [
    it({
      name: 'TAKES NO WAIT IN THE LANE CONTEST AND THE CONSOLIDATION WITH SYNTHETIC HELD, the XingZ6020 shape: the hold '
        + 'cures no judge bench, so the phase runs on what is reachable at once',
      fn: async () => {
        expect((await readingOf({ phase: 'lane contest', view: SYNTHETIC_HELD, held: 'synthetic', },)).reads,).toBe(1,);
        expect((await readingOf({ phase: 'consolidation', view: SYNTHETIC_HELD, held: 'synthetic', },)).reads,).toBe(1,);
      },
    },),

    it({
      name: 'WAITS IN THE TRANSLATE LANE WITH SYNTHETIC HELD, the XingZ6020 shape: the hold ends in a return that lifts '
        + 'the translators to their quorum, so that wait is the right one',
      fn: async () => {
        expect((await readingOf({ phase: 'translate lane', view: SYNTHETIC_HELD, held: 'synthetic', },)).reads,).toBe(2,);
      },
    },),

    it({
      name: 'STATES THE CEILING OF EACH SHORT BENCH IN THE LINE THAT TAKES NO WAIT: wide reads 1 of 7 against a quorum of '
        + '4, and with Synthetic back it would read 3 of 7, which no return lifts',
      fn: async () => {
        const warning = shortBenchWarningOf({
          lines: (await readingOf({ phase: 'lane contest', view: SYNTHETIC_HELD, held: 'synthetic', },)).lines,
        },);
        expect(warning,).toContain('wide 1 of 7 reachable, quorum 4',);
        expect(warning,).toContain('wide ceiling 3 of 7 with synthetic back, quorum 4, so no return lifts it',);
        expect(warning,).toContain('so the phase runs on what is reachable',);
      },
    },),

    it({
      name: 'NAMES THE SEATS A RETURN WOULD BRING BACK, by their model ids: the two Synthetic seats of the wide bench',
      fn: async () => {
        const warning = shortBenchWarningOf({
          lines: (await readingOf({ phase: 'lane contest', view: SYNTHETIC_HELD, held: 'synthetic', },)).lines,
        },);
        expect(warning,).toContain('hf:moonshotai/Kimi-K3',);
        expect(warning,).toContain('hf:Qwen/Qwen3.8-27B',);
      },
    },),

    it({
      name: 'STATES EACH BENCH OF THE CONSOLIDATION: the slate and the wide bench both read 1 of 7 and neither can be '
        + 'lifted by the hold',
      fn: async () => {
        const warning = shortBenchWarningOf({
          lines: (await readingOf({ phase: 'consolidation', view: SYNTHETIC_HELD, held: 'synthetic', },)).lines,
        },);
        expect(warning,).toContain('slate ceiling 3 of 7 with synthetic back, quorum 4, so no return lifts it',);
        expect(warning,).toContain('wide ceiling 3 of 7 with synthetic back, quorum 4, so no return lifts it',);
      },
    },),

    it({
      name: 'STATES WHICH BENCH THE WAIT CAN LIFT AND WHICH IT CANNOT in the line that announces it: the translate '
        + 'lane waits for the translators, whose return lifts them, and says the select bench stays at 3 of 7',
      fn: async () => {
        const warning = shortBenchWarningOf({
          lines: (await readingOf({ phase: 'translate lane', view: SYNTHETIC_HELD, held: 'synthetic', },)).lines,
        },);
        expect(warning,).toContain('waiting 40ms for the shortest hold to end',);
        expect(warning,).toContain('translators ceiling',);
        expect(warning,).toContain('so the return lifts it',);
        expect(warning,).toContain('select ceiling 3 of 7 with synthetic back, quorum 4, so no return lifts it',);
      },
    },),

    it({
      name: 'SAYS NO HOLD RUNS WHEN NONE DOES: the ceiling of a short bench is what it reads, with no seat to bring back',
      fn: async () => {
        const warning = shortBenchWarningOf({
          lines: (await readingOf({ phase: 'lane contest', view: SYNTHETIC_HELD, },)).lines,
        },);
        expect(warning,).toContain('wide ceiling 1 of 7 with no hold running, quorum 4, so no return lifts it',);
      },
    },),
  ],
},);

//endregion Ceiling tests
