/**
 Tests that the seat reader states, for each bench that stands short of its quorum, what the holds running on the
 providers could bring back and whether that lifts the bench to its floor and to its quorum, in the line that announces
 a wait and in the line that says none is taken, and that it waits only for a hold whose return crosses one of the two.

 THE SHAPE, from the corpus pass `XingZ6020` (2026-10-22 and 2026-10-23): NeuralWatt, DeepSeek and Xiaomi wet, Bedrock,
 Hyper and OpenRouter dry, and Synthetic refused and held out for five minutes. With Synthetic held the judge
 benches read 1 of 7 on that log. The lane contest and the consolidation judge with the late bench (7 seats, which
 holds Kimi-K3 and not Qwen3.8-27B), so the return lifts it from 1 to 2 here, over the floor of 2 though under the
 quorum of 4, where the calls are bought; the translate lane's select bench has 7 seats here since Qwen3.8-27B left the
 wide bench on 2026-10-29 (8 from the Claude key's seat joining it until then). A reader that waits for such a return waits for a floor and not a quorum; a hold whose return
 lifts a bench to neither is not waited for.

 Fixtures are cat-themed invention; every view and hold is scripted and no HTTP is made.

 @module
 */

import {
  describe,
  expect,
  it,
  type DisposableSandbox,
} from '@monochromatic-dev/test-harness-core/ts';

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
  ceilingWordsOf,
  type JudgeSeatPhase,
  readJudgeSeats,
} from '../../dist/final/node/index.mjs';
import { readOnFakeClock, } from './run-seats-fake-clock.test-fixture.ts';

//region Ceiling tests

/**
 The XingZ6020 view: NeuralWatt, DeepSeek and Xiaomi wet, and every other provider dry (Synthetic by its hold, the
 rest by their meters or the owner's switch).
 */
const SYNTHETIC_HELD: BudgetView = {
  neuralwatt: false,
  synthetic: true,
  bedrock: true,
  deepseek: false,
  xiaomi: false,
  anthropic: true,
  hyper: true,
  openrouter: true,
};

/**
 The view where the lane contest's late bench stands at 3 of 7 (Bedrock's two seats and DeepSeek's) with Synthetic dry,
 and Xiaomi, which serves no seat of any judge bench, dry as well.
 */
const BEDROCK_AND_DEEPSEEK: BudgetView = {
  neuralwatt: true,
  synthetic: true,
  bedrock: false,
  deepseek: false,
  xiaomi: true,
  anthropic: true,
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
  anthropic: true,
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
  anthropic: 'wet',
  hyper: 'wet',
  openrouter: 'wet',
};

/**
 No provider held out.
 */
const NO_HOLDS = providerRecord({ of: (): number => 0, },);

/**
 The hold the scripted provider states, waited out in slices of `POLL_MS` on a fake clock the helper moves past it, so
 no case sleeps.
 */
const SHORT_HOLD_MS = 40;

/**
 How often the reader's wait checks for abort here.
 */
const POLL_MS = 5;

/**
 What one reading logged and how many dryness reads it took.
 */
type Reading = {
  readonly lines: readonly string[];
  readonly reads: number;
};

/**
 Reads one phase's seats over a scripted view with a hold on one provider that ends with the first wait, which runs
 on a fake clock.

 @param phase - phase the reading is for

 @param view - dryness every read returns

 @param held - provider the hold is on, none when no hold runs

 @param meters - what the meters say, every one wet when absent

 @param sinon - case's own sandbox, which fakes the clock the wait sleeps on

 @returns What the reading logged and the dryness reads it took (1 without a wait, 2 with one)

 @example
 ```ts
 const { lines, reads, } = await readingOf({
   phase: 'lane contest',
   view: SYNTHETIC_HELD,
   held: 'synthetic',
   sinon: ctx.sinon,
 },);
 ```
 */
async function readingOf(
  {
    phase,
    view,
    held,
    meters = METERS_WET,
    sinon,
  }: {
    readonly phase: JudgeSeatPhase;
    readonly view: BudgetView;
    readonly held?: ProviderName;
    readonly meters?: Readonly<Record<keyof BudgetView, MeterState>>;
    readonly sinon: DisposableSandbox;
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
  await readOnFakeClock({
    sinon,
    holdMs: SHORT_HOLD_MS,
    pollMs: POLL_MS,
    reading: async function readSeats({ signal, },): Promise<unknown> {
      return await readJudgeSeats({
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
        signal,
        l: levelCapturingLogger({ lines, },),
        pollMs: POLL_MS,
      },);
    },
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
      name: 'WAITS IN THE LANE CONTEST AND THE CONSOLIDATION WITH SYNTHETIC HELD, the XingZ6020 shape: the return '
        + 'lifts the late bench from 1 to 2, over the floor of 2 though under the quorum of 4, so the reader waits for '
        + 'it (the case that stood here read no wait, against the quorum alone)',
      fn: async (ctx) => {
        expect(
          (await readingOf({
            phase: 'lane contest',
            view: SYNTHETIC_HELD,
            held: 'synthetic',
            sinon: ctx.sinon,
          },)).reads,
        ).toBe(2,);
        expect(
          (await readingOf({
            phase: 'consolidation',
            view: SYNTHETIC_HELD,
            held: 'synthetic',
            sinon: ctx.sinon,
          },)).reads,
        ).toBe(2,);
      },
    },),

    it({
      name: 'WAITS IN THE TRANSLATE LANE WITH SYNTHETIC HELD, the XingZ6020 shape: the hold ends in a return that '
        + 'lifts the translators to their quorum',
      fn: async (ctx) => {
        expect(
          (await readingOf({
            phase: 'translate lane',
            view: SYNTHETIC_HELD,
            held: 'synthetic',
            sinon: ctx.sinon,
          },)).reads,
        ).toBe(2,);
      },
    },),

    it({
      name: 'STATES BOTH READINGS OF A BENCH THE RETURN LIFTS OVER ITS FLOOR BUT NOT TO ITS QUORUM, the case the '
        + 'quorum-only rule misjudged: late reads 1 of 7, Synthetic back it reads 2, floor 2, quorum 4',
      fn: async (ctx) => {
        const warning = shortBenchWarningOf({
          lines: (await readingOf({
            phase: 'lane contest',
            view: SYNTHETIC_HELD,
            held: 'synthetic',
            sinon: ctx.sinon,
          },)).lines,
        },);
        expect(warning,).toContain('late 1 of 7 reachable, quorum 4',);
        expect(warning,).toContain(
          'late ceiling 2 of 7 with synthetic back, floor 2 and quorum 4, '
            + 'so the return lifts it to the floor but not to the quorum',
        );
        expect(warning,).toContain('waiting 40ms for the shortest hold to end',);
      },
    },),

    it({
      name: 'TAKES NO WAIT FOR A HOLD WHOSE RETURN CROSSES NEITHER THRESHOLD: the late bench reads 3 of 7, over its '
        + 'floor, and Xiaomi, which serves no seat of it, cannot lift it to the quorum',
      fn: async (ctx) => {
        const { lines, reads, } = await readingOf({
          phase: 'lane contest',
          view: BEDROCK_AND_DEEPSEEK,
          held: 'xiaomi',
          sinon: ctx.sinon,
        },);
        expect(reads,).toBe(1,);
        expect(shortBenchWarningOf({ lines, },),).toContain(
          'late ceiling 3 of 7 with xiaomi back, floor 2 and quorum 4, so no return lifts it to the quorum',
        );
      },
    },),

    it({
      name: 'WAITS FOR A HOLD THAT LIFTS A BENCH FROM OVER ITS FLOOR TO ITS QUORUM: the late bench reads 3 of 7 and '
        + 'Synthetic back it reads 4',
      fn: async (ctx) => {
        const { lines, reads, } = await readingOf({
          phase: 'lane contest',
          view: BEDROCK_AND_DEEPSEEK,
          held: 'synthetic',
          sinon: ctx.sinon,
        },);
        expect(reads,).toBe(2,);
        expect(shortBenchWarningOf({ lines, },),).toContain(
          'late ceiling 4 of 7 with synthetic back, floor 2 and quorum 4, so the return lifts it to the quorum',
        );
      },
    },),

    it({
      name: 'NAMES THE SEATS A RETURN WOULD BRING BACK, by their model ids, from the bench the phase seats: the '
        + 'late bench of the lane contest has the one Synthetic seat Kimi-K3 and not Qwen3.8-27B, which sits on the '
        + 'translators of the translate lane and on no judge bench since 2026-10-29',
      fn: async (ctx) => {
        /**
         The warning of the lane contest, whose late bench holds no Qwen seat.
         */
        const contest = shortBenchWarningOf({
          lines: (await readingOf({
            phase: 'lane contest',
            view: SYNTHETIC_HELD,
            held: 'synthetic',
            sinon: ctx.sinon,
          },)).lines,
        },);
        /**
         The warning of the translate lane, whose translators hold both and whose select bench holds Kimi-K3 alone.
         */
        const translate = shortBenchWarningOf({
          lines: (await readingOf({
            phase: 'translate lane',
            view: SYNTHETIC_HELD,
            held: 'synthetic',
            sinon: ctx.sinon,
          },)).lines,
        },);
        expect(contest,).toContain('hf:moonshotai/Kimi-K3',);
        expect(contest,).not.toContain('hf:Qwen/Qwen3.8-27B',);
        expect(translate,).toContain('hf:moonshotai/Kimi-K3',);
        expect(translate,).toContain('hf:Qwen/Qwen3.8-27B',);
      },
    },),

    it({
      name: 'STATES EACH BENCH OF THE CONSOLIDATION: the slate and the late bench both read 1 of 7, and the return '
        + 'lifts each over the floor to 2 of 7 (Qwen sits on no judge bench, ledger B815 and B1030, and the late bench '
        + 'gates the consolidation)',
      fn: async (ctx) => {
        const warning = shortBenchWarningOf({
          lines: (await readingOf({
            phase: 'consolidation',
            view: SYNTHETIC_HELD,
            held: 'synthetic',
            sinon: ctx.sinon,
          },)).lines,
        },);
        expect(warning,).toContain(
          'slate ceiling 2 of 7 with synthetic back, floor 2 and quorum 4, '
            + 'so the return lifts it to the floor but not to the quorum',
        );
        expect(warning,).toContain(
          'late ceiling 2 of 7 with synthetic back, floor 2 and quorum 4, '
            + 'so the return lifts it to the floor but not to the quorum',
        );
      },
    },),

    it({
      name: 'STATES WHICH BENCH THE WAIT CAN LIFT AND WHICH IT CANNOT in the line that announces it: the translate '
        + 'lane waits for the translators, whose return lifts them to their quorum, and says the select bench stays '
        + 'under its quorum at 2 of 7 after lifting over the floor',
      fn: async (ctx) => {
        const warning = shortBenchWarningOf({
          lines: (await readingOf({
            phase: 'translate lane',
            view: SYNTHETIC_HELD,
            held: 'synthetic',
            sinon: ctx.sinon,
          },)).lines,
        },);
        expect(warning,).toContain('waiting 40ms for the shortest hold to end',);
        expect(warning,).toContain('translators ceiling',);
        expect(warning,).toContain('so the return lifts it to the quorum',);
        expect(warning,).toContain('select ceiling 2 of 7 with synthetic back, floor 2 and quorum 4, so the return',);
      },
    },),

    it({
      name: 'STATES NO CEILING WHEN NO HOLD RUNS, but says a bench under its floor is under it: the ceiling of a short '
        + 'bench is then what it reads, which its clause already says, and the clause did not say that a stage under '
        + 'the floor buys no call',
      fn: async (ctx) => {
        const warning = shortBenchWarningOf({
          lines: (await readingOf({ phase: 'lane contest', view: SYNTHETIC_HELD, sinon: ctx.sinon, },)).lines,
        },);
        expect(warning,).toContain('late 1 of 7 reachable, quorum 4',);
        expect(warning,).toContain('late reads under its floor of 2 and no hold runs that could lift it',);
        expect(warning,).toContain('no provider has named a return the pass can wait for',);
        expect(warning,).not.toContain('ceiling',);
      },
    },),

    it({
      name: 'SAYS NOTHING OF THE FLOOR OF A BENCH OVER IT WHEN NO HOLD RUNS: the late bench reads 3 of 7 and only its '
        + 'clause is said',
      fn: async (ctx) => {
        const warning = shortBenchWarningOf({
          lines: (await readingOf({ phase: 'lane contest', view: BEDROCK_AND_DEEPSEEK, sinon: ctx.sinon, },)).lines,
        },);
        expect(warning,).toContain('late 3 of 7 reachable, quorum 4',);
        expect(warning,).not.toContain('floor',);
      },
    },),

    it({
      name: 'DOES NOT COUNT A PROVIDER DRY BY ITS METER AS COMING BACK: NeuralWatt is held by a refusal and its meter '
        + 'reads dry, the translators read 2 of 8, and its two seats would lift them to their quorum of 4 only if '
        + 'the hold ending brought it back, which a dry meter does not, so no wait is taken and the line brings back '
        + 'no seat',
      fn: async (ctx) => {
        const { lines, reads, } = await readingOf({
          phase: 'translate lane',
          view: BEDROCK_ALONE,
          held: 'neuralwatt',
          meters: { ...METERS_WET, neuralwatt: 'dry', },
          sinon: ctx.sinon,
        },);
        expect(reads,).toBe(1,);
        const warning = shortBenchWarningOf({ lines, },);
        expect(warning,).toContain('translators ceiling 2 of 8',);
        expect(warning,).toContain('no return lifts it to the quorum',);
        expect(warning,).toContain(
          'neuralwatt reads dry by its meter or by a payment refusal that stands until the meter moves, which a hold '
            + 'ending does not change',
        );
        expect(warning,).not.toContain('hf:zai-org/GLM-5.3-Flash',);
      },
    },),

    it({
      name: 'COUNTS A HELD PROVIDER WHOSE METER READS WET OR UNREADABLE AS COMING BACK: the same view and hold wait, '
        + 'and the line names the seats of the return',
      fn: async (ctx) => {
        const wet = await readingOf({
          phase: 'translate lane',
          view: BEDROCK_ALONE,
          held: 'neuralwatt',
          sinon: ctx.sinon,
        },);
        expect(wet.reads,).toBe(2,);
        expect(shortBenchWarningOf({ lines: wet.lines, },),).toContain('hf:zai-org/GLM-5.3-Flash',);
        const unreadable = await readingOf({
          phase: 'translate lane',
          view: BEDROCK_ALONE,
          held: 'neuralwatt',
          meters: { ...METERS_WET, neuralwatt: 'unreadable', },
          sinon: ctx.sinon,
        },);
        expect(unreadable.reads,).toBe(2,);
      },
    },),

    it({
      name: 'SAYS THE RETURN BRINGS BACK NO SEAT where the held provider serves none of the bench: Xiaomi held, the '
        + 'late bench reads 3 of 7, and the line closes on the words that name no seat',
      fn: async (ctx) => {
        const warning = shortBenchWarningOf({
          lines: (await readingOf({
            phase: 'lane contest',
            view: BEDROCK_AND_DEEPSEEK,
            held: 'xiaomi',
            sinon: ctx.sinon,
          },)).lines,
        },);
        expect(warning,).toContain('(the return brings back no seat)',);
      },
    },),

    it({
      name: 'STATES THE DIGITS OF THE TRANSLATORS CEILING: with Synthetic back the translators read 4 of 8 against a '
        + 'floor of 2 and a quorum of 4, which is the quorum exactly',
      fn: async (ctx) => {
        const warning = shortBenchWarningOf({
          lines: (await readingOf({
            phase: 'translate lane',
            view: SYNTHETIC_HELD,
            held: 'synthetic',
            sinon: ctx.sinon,
          },)).lines,
        },);
        expect(warning,).toContain(
          'translators ceiling 4 of 8 with synthetic back, floor 2 and quorum 4, so the return lifts it to the quorum',
        );
      },
    },),

    it({
      name: 'ceilingWordsOf STATES ONE CLAUSE PER SHORT BENCH IN THE ORDER THE BENCHES ARE NAMED, and none for a bench '
        + 'that reads its quorum: with Synthetic held the wide and select benches read 1 of 7 and 2 of 7 back, and '
        + 'with Bedrock, Hyper and OpenRouter wet the wide bench reads its quorum and has no clause',
      fn: async () => {
        const clauses = ceilingWordsOf({
          running: providerRecord({ of: (provider) => (provider === 'synthetic') ? SHORT_HOLD_MS : 0, },),
          dry: SYNTHETIC_HELD,
          meterDry: new Set(),
          names: ['wide', 'select',],
        },);
        expect(clauses.length,).toBe(2,);
        expect(clauses[0],).toBe(
          'wide ceiling 2 of 7 with synthetic back, floor 2 and quorum 4, so the return lifts it to the floor but '
            + 'not to the quorum (the return brings back hf:moonshotai/Kimi-K3)',
        );
        expect(clauses[1],).toContain('select ceiling 2 of 7 with synthetic back',);
        expect(ceilingWordsOf({
          running: providerRecord({ of: (provider) => (provider === 'synthetic') ? SHORT_HOLD_MS : 0, },),
          dry: { ...SYNTHETIC_HELD, bedrock: false, hyper: false, openrouter: false, },
          meterDry: new Set(),
          names: ['wide',],
        },),).toEqual([],);
      },
    },),

    it({
      name: 'ceilingWordsOf LEAVES A METER-DRY HELD PROVIDER OUT OF THE RETURN and says so, and says nothing of the '
        + 'ceiling for a bench over its floor when no hold runs',
      fn: async () => {
        const clauses = ceilingWordsOf({
          running: providerRecord({ of: (provider) => (provider === 'neuralwatt') ? SHORT_HOLD_MS : 0, },),
          dry: BEDROCK_ALONE,
          meterDry: new Set(['neuralwatt',],),
          names: ['translators',],
        },);
        expect(clauses,).toEqual([
          'translators ceiling 2 of 8 with no held provider back, floor 2 and quorum 4, so no return lifts it to the '
            + 'quorum (the return brings back no seat; neuralwatt reads dry by its meter or by a payment refusal that '
            + 'stands until the meter moves, which a hold ending does not change)',
        ],);
        expect(ceilingWordsOf({
          running: NO_HOLDS,
          dry: BEDROCK_AND_DEEPSEEK,
          meterDry: new Set(),
          names: ['wide',],
        },),).toEqual([],);
      },
    },),
  ],
},);

//endregion Ceiling tests
