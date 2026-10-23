/**
 Tests that the seat reader states, for each bench that stands short of its quorum, what the holds running on the
 providers could bring back and whether that lifts the bench to its floor and to its quorum, in the line that announces
 a wait and in the line that says none is taken, and that it waits only for a hold whose return crosses one of the two.

 THE SHAPE, from the corpus pass `XingZ6020` (2026-10-22 and 2026-10-23): NeuralWatt, DeepSeek and Xiaomi wet, Bedrock,
 Hyper and OpenRouter dry, and Synthetic refused and held out for five minutes. The judge benches read 1 of 7 with
 Synthetic held and 3 of 7 with every provider wet, against a quorum of 4 and a floor of 2 voices. The return lifts the
 wide bench over the floor, where the lane contest and the consolidation buy calls, though not to the quorum, so the
 reader waits for it; a hold whose return lifts a bench to neither is not waited for.

 Fixtures are cat-themed invention; every view and hold is scripted and no HTTP is made.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import type {
  BudgetView,
  MeterState,
} from '@monochromatic-dev/translation-repair-provider';
import {
  type ProviderName,
  providerRecord,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  levelCapturingLogger,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
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
 The view where the lane contest's wide bench stands at 3 of 7 (Bedrock's two seats and DeepSeek's) with Synthetic dry,
 and Xiaomi, which serves no seat of any judge bench, dry as well.
 */
const BEDROCK_AND_DEEPSEEK: BudgetView = {
  neuralwatt: true,
  synthetic: true,
  bedrock: false,
  deepseek: false,
  xiaomi: true,
  hyper: true,
  openrouter: true,
};

/**
 Bedrock alone wet: the translate lane's translators read 2 of 8 and NeuralWatt's two seats would lift them to 4.
 */
const BEDROCK_ALONE: BudgetView = {
  neuralwatt: true,
  synthetic: true,
  bedrock: false,
  deepseek: true,
  xiaomi: true,
  hyper: true,
  openrouter: true,
};

/**
 What each meter says when none is asked about.
 */
const METERS_WET: Readonly<Record<keyof BudgetView, MeterState>> = {
  neuralwatt: 'wet',
  synthetic: 'wet',
  bedrock: 'wet',
  deepseek: 'wet',
  xiaomi: 'wet',
  hyper: 'wet',
  openrouter: 'wet',
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

 @param meters - what the meters say, every one wet when absent

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
    meters = METERS_WET,
  }: {
    readonly phase: JudgeSeatPhase;
    readonly view: BudgetView;
    readonly held?: ProviderName;
    readonly meters?: Readonly<Record<keyof BudgetView, MeterState>>;
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
      providerMeterStates: () => Promise.resolve(meters,),
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
      name: 'WAITS IN THE LANE CONTEST AND THE CONSOLIDATION WITH SYNTHETIC HELD, the XingZ6020 shape: the return lifts '
        + 'the wide bench from 1 to 3, over the floor of 2 though under the quorum of 4, so the reader waits for it '
        + '(the case that stood here read no wait, against the quorum alone)',
      fn: async () => {
        expect((await readingOf({ phase: 'lane contest', view: SYNTHETIC_HELD, held: 'synthetic', },)).reads,).toBe(2,);
        expect((await readingOf({ phase: 'consolidation', view: SYNTHETIC_HELD, held: 'synthetic', },)).reads,).toBe(2,);
      },
    },),

    it({
      name: 'WAITS IN THE TRANSLATE LANE WITH SYNTHETIC HELD, the XingZ6020 shape: the hold ends in a return that lifts '
        + 'the translators to their quorum',
      fn: async () => {
        expect((await readingOf({ phase: 'translate lane', view: SYNTHETIC_HELD, held: 'synthetic', },)).reads,).toBe(2,);
      },
    },),

    it({
      name: 'STATES BOTH READINGS OF A BENCH THE RETURN LIFTS OVER ITS FLOOR BUT NOT TO ITS QUORUM, the case the '
        + 'quorum-only rule misjudged: wide reads 1 of 7, Synthetic back it reads 3, floor 2, quorum 4',
      fn: async () => {
        const warning = shortBenchWarningOf({
          lines: (await readingOf({ phase: 'lane contest', view: SYNTHETIC_HELD, held: 'synthetic', },)).lines,
        },);
        expect(warning,).toContain('wide 1 of 7 reachable, quorum 4',);
        expect(warning,).toContain(
          'wide ceiling 3 of 7 with synthetic back, floor 2 and quorum 4, '
            + 'so the return lifts it to the floor but not to the quorum',
        );
        expect(warning,).toContain('waiting 40ms for the shortest hold to end',);
      },
    },),

    it({
      name: 'TAKES NO WAIT FOR A HOLD WHOSE RETURN CROSSES NEITHER THRESHOLD: the wide bench reads 3 of 7, over its '
        + 'floor, and Xiaomi, which serves no seat of it, cannot lift it to the quorum',
      fn: async () => {
        const { lines, reads, } = await readingOf({
          phase: 'lane contest',
          view: BEDROCK_AND_DEEPSEEK,
          held: 'xiaomi',
        },);
        expect(reads,).toBe(1,);
        expect(shortBenchWarningOf({ lines, },),).toContain(
          'wide ceiling 3 of 7 with xiaomi back, floor 2 and quorum 4, so no return lifts it to the quorum',
        );
      },
    },),

    it({
      name: 'WAITS FOR A HOLD THAT LIFTS A BENCH FROM OVER ITS FLOOR TO ITS QUORUM: the wide bench reads 3 of 7 and '
        + 'Synthetic back it reads 5',
      fn: async () => {
        const { lines, reads, } = await readingOf({
          phase: 'lane contest',
          view: BEDROCK_AND_DEEPSEEK,
          held: 'synthetic',
        },);
        expect(reads,).toBe(2,);
        expect(shortBenchWarningOf({ lines, },),).toContain(
          'wide ceiling 5 of 7 with synthetic back, floor 2 and quorum 4, so the return lifts it to the quorum',
        );
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
      name: 'STATES EACH BENCH OF THE CONSOLIDATION: the slate and the wide bench both read 1 of 7 and the return lifts '
        + 'each over the floor, the slate to 2 of 7 (Qwen sits on the wide bench only, ledger B815) and the wide bench '
        + 'to 3 of 7',
      fn: async () => {
        const warning = shortBenchWarningOf({
          lines: (await readingOf({ phase: 'consolidation', view: SYNTHETIC_HELD, held: 'synthetic', },)).lines,
        },);
        expect(warning,).toContain(
          'slate ceiling 2 of 7 with synthetic back, floor 2 and quorum 4, '
            + 'so the return lifts it to the floor but not to the quorum',
        );
        expect(warning,).toContain(
          'wide ceiling 3 of 7 with synthetic back, floor 2 and quorum 4, '
            + 'so the return lifts it to the floor but not to the quorum',
        );
      },
    },),

    it({
      name: 'STATES WHICH BENCH THE WAIT CAN LIFT AND WHICH IT CANNOT in the line that announces it: the translate '
        + 'lane waits for the translators, whose return lifts them to their quorum, and says the select bench stays '
        + 'under its quorum at 3 of 7 after lifting over the floor',
      fn: async () => {
        const warning = shortBenchWarningOf({
          lines: (await readingOf({ phase: 'translate lane', view: SYNTHETIC_HELD, held: 'synthetic', },)).lines,
        },);
        expect(warning,).toContain('waiting 40ms for the shortest hold to end',);
        expect(warning,).toContain('translators ceiling',);
        expect(warning,).toContain('so the return lifts it to the quorum',);
        expect(warning,).toContain('select ceiling 3 of 7 with synthetic back, floor 2 and quorum 4, so the return',);
      },
    },),

    it({
      name: 'STATES NO CEILING WHEN NO HOLD RUNS, but says a bench under its floor is under it: the ceiling of a short '
        + 'bench is then what it reads, which its clause already says, and the clause did not say that a stage under '
        + 'the floor buys no call',
      fn: async () => {
        const warning = shortBenchWarningOf({
          lines: (await readingOf({ phase: 'lane contest', view: SYNTHETIC_HELD, },)).lines,
        },);
        expect(warning,).toContain('wide 1 of 7 reachable, quorum 4',);
        expect(warning,).toContain('wide reads under its floor of 2 and no hold runs that could lift it',);
        expect(warning,).toContain('no provider has named a return the pass can wait for',);
        expect(warning,).not.toContain('ceiling',);
      },
    },),

    it({
      name: 'SAYS NOTHING OF THE FLOOR OF A BENCH OVER IT WHEN NO HOLD RUNS: the wide bench reads 3 of 7 and only its '
        + 'clause is said',
      fn: async () => {
        const warning = shortBenchWarningOf({
          lines: (await readingOf({ phase: 'lane contest', view: BEDROCK_AND_DEEPSEEK, },)).lines,
        },);
        expect(warning,).toContain('wide 3 of 7 reachable, quorum 4',);
        expect(warning,).not.toContain('floor',);
      },
    },),

    it({
      name: 'DOES NOT COUNT A PROVIDER DRY BY ITS METER AS COMING BACK: NeuralWatt is held and its meter reads dry, the '
        + 'translators read 2 of 8, and its two seats would lift them to their quorum of 4 only if its hold ended in a '
        + 'return, which a dry meter keeps dry, so no wait is taken and the line brings back no seat',
      fn: async () => {
        const { lines, reads, } = await readingOf({
          phase: 'translate lane',
          view: BEDROCK_ALONE,
          held: 'neuralwatt',
          meters: { ...METERS_WET, neuralwatt: 'dry', },
        },);
        expect(reads,).toBe(1,);
        const warning = shortBenchWarningOf({ lines, },);
        expect(warning,).toContain('translators ceiling 2 of 8',);
        expect(warning,).toContain('no return lifts it to the quorum',);
        expect(warning,).toContain('neuralwatt reads dry by its meter',);
        expect(warning,).not.toContain('hf:zai-org/GLM-5.3-Flash',);
      },
    },),

    it({
      name: 'COUNTS A HELD PROVIDER WHOSE METER READS WET OR UNREADABLE AS COMING BACK: the same view and hold wait, '
        + 'and the line names the seats of the return',
      fn: async () => {
        const wet = await readingOf({ phase: 'translate lane', view: BEDROCK_ALONE, held: 'neuralwatt', },);
        expect(wet.reads,).toBe(2,);
        expect(shortBenchWarningOf({ lines: wet.lines, },),).toContain('hf:zai-org/GLM-5.3-Flash',);
        const unreadable = await readingOf({
          phase: 'translate lane',
          view: BEDROCK_ALONE,
          held: 'neuralwatt',
          meters: { ...METERS_WET, neuralwatt: 'unreadable', },
        },);
        expect(unreadable.reads,).toBe(2,);
      },
    },),
  ],
},);

//endregion Ceiling tests
