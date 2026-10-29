/**
 Tests that the seat reader waits only for a hold whose provider, back, would cure a short bench, and says which
 providers could when it seats a bench short.

 THE SHAPE, from the second review of the Claude route (round 3): with one provider's key dry a bench stands one
 seat short of its quorum, and a hold on a provider that serves no seat the bench lacks is waited out in full (a
 refusal's cooldown is 300,000 ms, a stated return can be hours) and the bench is seated short all the same. Here the
 fleet is the live roster of this tree, NeuralWatt and Bedrock wet and the rest dry, the Claude key included. The
 lane contest judges with the late bench (`seats.lateJudges`), which is 3 of 7 against a quorum of 4. Synthetic, Hyper
 or OpenRouter back reach the quorum; DeepSeek, Xiaomi or the Claude key back leave it at 3 of 7 (DeepSeek's one late
 seat is one Bedrock or NeuralWatt already reach, and Haiku is on the wide bench only). The Claude key is dry in
 every view here, so its own hold is the one held in the cases that name it: skipped in the lane contest, where its
 return leaves the late bench short, and waited out in the lanes phase, whose critics and panel are the wide bench and
 where its return takes the wide bench from 3 of 7 to 4 of 7 (a view where the wide bench is the only one short) or
 from 1 of 7 to 2, the floor of two voices the stages decide on; the rule is the same, and what differs is the bench the
 phase seats.

 TWO HOLDS THAT EACH CURE ALONE (round 5): a hold counts when the held set cures the bench between them and that
 provider's return adds a reachable seat to it. A rule asking that the bench stay short without the provider counted
 neither of two providers that each reach the quorum alone, and the reader seated the bench short without waiting.

 Each case reads the phase it names (the lane contest unless it says otherwise), and the wait is counted by the dryness
 reads (one read without a wait, two with). The scripted holds are 40 ms and the wait runs on a fake clock that the
 helper moves past them (`run-seats-fake-clock.test-fixture.ts`), so no case sleeps for a hold.

 Fixtures are cat-themed invention; every view and hold is scripted and no HTTP is made.

 @module
 */

import {
  describe,
  expect,
  it,
  type DisposableSandbox,
} from '@monochromatic-dev/test-harness-core/ts';

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
import { type JudgeSeatPhase, readJudgeSeats, } from '../../dist/final/node/index.mjs';
import { readOnFakeClock, } from './run-seats-fake-clock.test-fixture.ts';
import { shortWarningOf, } from './run-seats-short-warning.test-fixture.ts';

/**
 NeuralWatt and Bedrock wet and every other provider dry: the late bench is 3 of 7 reachable against a quorum of 4,
 and the wide bench 3 of 7.
 */
const LATE_ONE_SHORT: BudgetView = {
  neuralwatt: false,
  synthetic: true,
  bedrock: false,
  deepseek: true,
  xiaomi: true,
  anthropic: true,
  hyper: true,
  openrouter: true,
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
 The hold the scripted providers state, waited out in slices of `POLL_MS` on a fake clock the helper moves past it, so
 no case sleeps.
 */
const SHORT_HOLD_MS = 40;

/**
 How often the reader's wait checks for abort here.
 */
const POLL_MS = 5;

/**
 NeuralWatt wet and every other provider dry: the late bench stands three seats short of its quorum of 4. Hyper back
 alone reaches the quorum, and so do Synthetic and Bedrock back together, while Synthetic alone or Bedrock alone leaves
 it short: with the three held, each one is unnecessary given the others, yet every one of them adds a seat to the bench
 the held set cures. The lanes phase also reads the wide bench over this view, at 1 of 7 reachable against a floor of
 2 and a quorum of 4: the floor-lift case uses it for that bench, where the Claude key back makes the wide bench 2.
 */
const LATE_THREE_SHORT: BudgetView = {
  neuralwatt: false,
  synthetic: true,
  bedrock: true,
  deepseek: true,
  xiaomi: true,
  anthropic: true,
  hyper: true,
  openrouter: true,
};

/**
 NeuralWatt, Bedrock, DeepSeek and Xiaomi wet and the rest dry: the wide bench is 3 of 7 reachable against a
 quorum of 4, and the editors, refiners and translators the lanes phase also leans on are each at their quorum, so
 the wide bench is the only one short. The Claude key back makes the wide bench 4 of 7.
 */
const WIDE_ONLY_SHORT: BudgetView = {
  neuralwatt: false,
  synthetic: true,
  bedrock: false,
  deepseek: false,
  xiaomi: false,
  anthropic: true,
  hyper: true,
  openrouter: true,
};

/**
 NeuralWatt, DeepSeek and Xiaomi wet and the rest dry: the readers bench is 1 of 6 reachable against a pair. Synthetic,
 Bedrock, Hyper or OpenRouter back reach the pair, and the Claude key back adds no reader.
 */
const READERS_ONE_SHORT: BudgetView = {
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
 Reads one phase's seats over a scripted view under holds that all end with the first wait, counting the dryness reads.
 The wait runs on a fake clock.

 @param held - providers the holds are on, each for the short hold

 @param lines - receives what the reading logs

 @param view - dryness every read returns (the late bench one seat short by default)

 @param phase - phase the seats are read for (the lane contest by default)

 @param sinon - case's own sandbox, which fakes the clock the wait sleeps on

 @returns Dryness reads the reading took: 1 without a wait, 2 with one

 @example
 ```ts
 const reads = await readsUnderHolds({ held: ['deepseek',], lines: [], sinon: ctx.sinon, },);
 ```
 */
async function readsUnderHolds(
  {
    held,
    lines,
    view = LATE_ONE_SHORT,
    phase = 'lane contest',
    sinon,
  }: {
    readonly held: readonly ProviderName[];
    readonly lines: string[];
    readonly view?: BudgetView;
    readonly phase?: JudgeSeatPhase;
    readonly sinon: DisposableSandbox;
  },
): Promise<number> {
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
          providerMeterStates: metersAllWet,
          providerHolds: () => {
            counted.asks += 1;
            return (counted.asks === 1)
              ? providerRecord({
                of: function holdOf(provider,): number {
                  return held.includes(provider,) ? SHORT_HOLD_MS : 0;
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
  return counted.reads;
}

/**
 Reads the lane contest's seats over the one-short view under one hold, counting the dryness reads.

 @param held - provider the hold is on

 @param lines - receives what the reading logs

 @param sinon - case's own sandbox, which fakes the clock the wait sleeps on

 @returns Dryness reads the reading took: 1 without a wait, 2 with one

 @example
 ```ts
 const reads = await readsUnderHold({ held: 'deepseek', lines: [], sinon: ctx.sinon, },);
 ```
 */
async function readsUnderHold(
  {
    held,
    lines,
    sinon,
  }: {
    readonly held: ProviderName;
    readonly lines: string[];
    readonly sinon: DisposableSandbox;
  },
): Promise<number> {
  return await readsUnderHolds({
    held: [held,],
    lines,
    sinon,
  },);
}

/**
 The warning a reading wrote when it seated a short bench without waiting.

 @param lines - what the reading logged

 @param phase - phase the seats were read for (the lane contest by default)

 @returns Line, empty where the reading wrote no such warning (a case that expects one then fails on its content)

 @example
 ```ts
 const warning = noWaitWarningOf({ lines, },);
 ```
 */
function noWaitWarningOf(
  {
    lines,
    phase = 'lane contest',
  }: {
    readonly lines: readonly string[];
    readonly phase?: JudgeSeatPhase;
  },
): string {
  return shortWarningOf({ lines, phase, },);
}

await describe({
  name: 'the seat reader and a hold whose provider cannot cure the short bench',
  concurrency: 1,
  children: [
    it({
      name: 'WAITS OUT A HOLD ON A PROVIDER WHOSE RETURN CURES THE SHORTFALL (the control): with Synthetic, Hyper or '
        + 'OpenRouter back the late bench reaches 4 or more seats against a quorum of 4, so the wait buys the quorum',
      fn: async (ctx) => {
        expect(await readsUnderHold({ held: 'synthetic', lines: [], sinon: ctx.sinon, },),).toBe(2,);
        expect(await readsUnderHold({ held: 'hyper', lines: [], sinon: ctx.sinon, },),).toBe(2,);
        expect(await readsUnderHold({ held: 'openrouter', lines: [], sinon: ctx.sinon, },),).toBe(2,);
      },
    },),

    it({
      name: 'DOES NOT WAIT OUT A HOLD ON A PROVIDER WHOSE RETURN LEAVES THE BENCH SHORT: with DeepSeek or Xiaomi back '
        + 'the late bench is still 3 of 7, so the wait buys nothing',
      fn: async (ctx) => {
        expect(await readsUnderHold({ held: 'deepseek', lines: [], sinon: ctx.sinon, },),).toBe(1,);
        expect(await readsUnderHold({ held: 'xiaomi', lines: [], sinon: ctx.sinon, },),).toBe(1,);
      },
    },),

    it({
      name: 'WAITS WHEN TWO HOLDS EACH CURE THE BENCH ALONE: Synthetic and Hyper both held, and either back makes the '
        + 'late bench 4 of 7 or more, so the wait is for the shorter of them',
      fn: async (ctx) => {
        expect(await readsUnderHolds({ held: ['synthetic', 'hyper',], lines: [], sinon: ctx.sinon, },),).toBe(2,);
      },
    },),

    it({
      name: 'DOES NOT WAIT OUT A HOLD ON THE CLAUDE KEY IN THE LANE CONTEST, WHICH JUDGES WITH THE LATE BENCH: the '
        + 'Claude key back makes the wide bench 4 of 7 but the late bench stays 3 of 7 against a quorum of 4 '
        + '(Haiku sits on the wide bench only), so the wait buys nothing and the line says the hold would not help',
      fn: async (ctx) => {
        const lines: string[] = [];
        expect(await readsUnderHold({ held: 'anthropic', lines, sinon: ctx.sinon, },),).toBe(1,);
        expect(noWaitWarningOf({ lines, },),).toContain(
          'held: anthropic (1 s); it would lift no short bench over its floor or to its quorum',
        );
        expect(noWaitWarningOf({ lines, },),).toContain(' late 3 of 7 reachable, quorum 4',);
      },
    },),

    it({
      name: 'DOES NOT WAIT OUT A HOLD ON THE CLAUDE KEY IN THE PAGE CONSISTENCY PASS, WHICH ASKS THE LATE BENCH: the '
        + 'wide bench is 3 of 7 and the Claude key back makes it 4 of 7, but the late bench the pass asks stays '
        + '3 of 7 against a quorum of 4, so the wait buys nothing',
      fn: async (ctx) => {
        const lines: string[] = [];
        expect(
          await readsUnderHolds({
            held: ['anthropic',],
            lines,
            view: WIDE_ONLY_SHORT,
            phase: 'page consistency',
            sinon: ctx.sinon,
          },),
        ).toBe(1,);
        expect(noWaitWarningOf({ lines, phase: 'page consistency', },),).toContain(' late 3 of 7 reachable, quorum 4',);
      },
    },),

    it({
      name: 'WAITS OUT A HOLD ON THE CLAUDE KEY IN A PHASE WHOSE JUDGING BENCH IS THE WIDE ONE (the control for the '
        + 'Claude key as the held provider): the lanes phase seats critics and panel from the wide bench, which is 3 '
        + 'of 7 and with the Claude key back 4 of 7 against a quorum of 4, so the wait buys the quorum and the line '
        + 'announcing it prints the Claude key\'s hold',
      fn: async (ctx) => {
        const lines: string[] = [];
        expect(
          await readsUnderHolds({
            held: ['anthropic',],
            lines,
            view: WIDE_ONLY_SHORT,
            phase: 'lanes',
            sinon: ctx.sinon,
          },),
        ).toBe(2,);
        /**
         The line announcing the wait, which prints each provider's hold as the wait counts it.
         */
        const waiting = lines.find(function isWaiting(line,): boolean {
          return line.includes('waiting ',);
        },);
        expect(waiting,).toContain('anthropic 40ms',);
      },
    },),

    it({
      name: 'WAITS OUT A HOLD ON THE CLAUDE KEY WHEN ITS RETURN LIFTS THE WIDE BENCH OVER ITS FLOOR THOUGH NOT TO ITS '
        + 'QUORUM: in the lanes phase, with only NeuralWatt wet, the wide bench reads 1 of 7 and the Claude key back '
        + 'makes it 2, the floor of 2 voices the stages decide on, so the wait buys a decision and not the quorum',
      fn: async (ctx) => {
        expect(
          await readsUnderHolds({
            held: ['anthropic',],
            lines: [],
            view: LATE_THREE_SHORT,
            phase: 'lanes',
            sinon: ctx.sinon,
          },),
        ).toBe(2,);
      },
    },),

    it({
      name: 'DOES NOT WAIT OUT A HOLD ON THE CLAUDE KEY WHEN ITS RETURN LEAVES THE BENCH SHORT: with only '
        + 'NeuralWatt wet the late bench is three seats short of its quorum of 4, the Claude key back adds no '
        + 'late seat and the bench is still short, so the wait buys nothing and the line says the hold would not help',
      fn: async (ctx) => {
        const lines: string[] = [];
        expect(
          await readsUnderHolds({ held: ['anthropic',], lines, view: LATE_THREE_SHORT, sinon: ctx.sinon, },),
        ).toBe(1,);
        const warning = noWaitWarningOf({ lines, },);
        expect(warning,).toContain(
          'held: anthropic (1 s); it would lift no short bench over its floor or to its quorum',
        );
      },
    },),

    it({
      name: 'WAITS WHEN SYNTHETIC AND OPENROUTER ARE BOTH HELD, each of which cures the late bench alone',
      fn: async (ctx) => {
        expect(await readsUnderHolds({ held: ['synthetic', 'openrouter',], lines: [], sinon: ctx.sinon, },),).toBe(2,);
      },
    },),

    it({
      name: 'WAITS WHEN THREE HOLDS EACH LEAVE THE OTHERS ENOUGH: NeuralWatt wet leaves the late bench three seats '
        + 'short, Hyper cures it alone and Synthetic with Bedrock cure it together, so with all three held none is '
        + 'necessary and every one adds a seat',
      fn: async (ctx) => {
        expect(
          await readsUnderHolds({
            held: ['synthetic', 'bedrock', 'hyper',],
            lines: [],
            view: LATE_THREE_SHORT,
            sinon: ctx.sinon,
          },),
        ).toBe(2,);
      },
    },),

    it({
      name: 'WAITS FOR SYNTHETIC ONLY WHEN DEEPSEEK IS HELD BESIDE IT: DeepSeek back adds no late seat, so its hold is '
        + 'not one the wait counts, and the line names Synthetic as the hold waited for',
      fn: async (ctx) => {
        const lines: string[] = [];
        expect(await readsUnderHolds({ held: ['synthetic', 'deepseek',], lines, sinon: ctx.sinon, },),).toBe(2,);
        /**
         The line announcing the wait, which prints each provider's hold as the wait counts it.
         */
        const waiting = lines.find(function isWaiting(line,): boolean {
          return line.includes('waiting ',);
        },);
        expect(waiting,).toContain('synthetic 40ms',);
        expect(waiting,).toContain('deepseek 0ms',);
      },
    },),

    it({
      name: 'WAITS FOR SYNTHETIC ONLY WHEN THE CLAUDE KEY IS HELD BESIDE IT IN THE PICTURES PHASE: with NeuralWatt, '
        + 'DeepSeek and Xiaomi wet the readers are 1 of 6 against a pair, Synthetic back cures them and the Claude '
        + 'key back adds no reader (Haiku and the other Claude models transcribe no pictures), so the hold the wait '
        + 'counts is Synthetic\'s alone',
      fn: async (ctx) => {
        const lines: string[] = [];
        expect(
          await readsUnderHolds({
            held: ['synthetic', 'anthropic',],
            lines,
            view: READERS_ONE_SHORT,
            phase: 'pictures',
            sinon: ctx.sinon,
          },),
        ).toBe(2,);
        /**
         The line announcing the wait, which prints each provider's hold as the wait counts it.
         */
        const waiting = lines.find(function isWaiting(line,): boolean {
          return line.includes('waiting ',);
        },);
        expect(waiting,).toContain('synthetic 40ms',);
        expect(waiting,).toContain('anthropic 0ms',);
      },
    },),

    it({
      name: 'NAMES THE BENCH, THE COUNTS AND THE PROVIDERS THAT COULD CURE IT when it seats a bench short after the '
        + 'waits: the Synthetic hold is waited out, the view does not change, and the warning says late 3 of 7, '
        + 'quorum 4, and that Synthetic, Hyper or OpenRouter back would reach it (the Claude key back would not)',
      fn: async (ctx) => {
        const lines: string[] = [];
        await readsUnderHold({ held: 'synthetic', lines, sinon: ctx.sinon, },);
        /**
         The warning the reading wrote once the waits were over.
         */
        const warning = lines.find(function isSeatedShort(line,): boolean {
          return line.startsWith('warn JUDGE SEATS phase=lane contest seated short',);
        },);
        expect(warning,).toContain('late 3 of 7 reachable, quorum 4',);
        expect(warning,).toContain('synthetic, hyper, openrouter',);
        expect(warning,).not.toContain('deepseek',);
        expect(warning,).not.toContain('anthropic',);
      },
    },),

    it({
      name: 'NAMES THE PROVIDERS THAT COULD CURE IT when no wait was taken either: the DeepSeek hold is not waited for '
        + 'and the short bench is seated short with the same warning',
      fn: async (ctx) => {
        const lines: string[] = [];
        await readsUnderHold({ held: 'deepseek', lines, sinon: ctx.sinon, },);
        const warning = noWaitWarningOf({ lines, },);
        expect(warning,).toContain('synthetic, hyper, openrouter',);
      },
    },),

    it({
      name: 'NAMES THE RUNNING HOLDS IT DID NOT WAIT FOR and why, in place of saying no provider named a return: the '
        + 'DeepSeek hold is listed with its length and "would lift no short bench over its floor or to its quorum" is '
        + 'the reason',
      fn: async (ctx) => {
        const lines: string[] = [];
        await readsUnderHolds({ held: ['deepseek',], lines, sinon: ctx.sinon, },);
        const warning = noWaitWarningOf({ lines, },);
        expect(warning,).toContain(
          'held: deepseek (1 s); it would lift no short bench over its floor or to its quorum',
        );
        expect(warning,).not.toContain('no provider has named a return',);
      },
    },),

    it({
      name: 'NAMES BOTH SKIPPED HOLDS WITH "neither": DeepSeek and Xiaomi held, and neither would lift '
        + 'the late bench over its floor or to its quorum',
      fn: async (ctx) => {
        const lines: string[] = [];
        await readsUnderHolds({ held: ['deepseek', 'xiaomi',], lines, sinon: ctx.sinon, },);
        const warning = noWaitWarningOf({ lines, },);
        expect(warning,).toContain(
          'held: deepseek (1 s), xiaomi (1 s); neither would lift a short bench over its floor or to its quorum',
        );
      },
    },),

    it({
      name: 'SAYS NO NEURALWATT RETURN ONLY WHEN NEURALWATT IS DRY WITH NONE INSIDE WHAT THE PASS HAS LEFT: with '
        + 'NeuralWatt wet the clause is absent, and with NeuralWatt dry and no hold on it the clause '
        + 'follows the skipped holds',
      fn: async (ctx) => {
        /**
         The warning of a reading over a view, with DeepSeek held.
         */
        async function warningOver({ view, }: { readonly view: BudgetView; },): Promise<string> {
          const lines: string[] = [];
          await readsUnderHolds({
            held: ['deepseek',],
            lines,
            view,
            sinon: ctx.sinon,
          },);
          return noWaitWarningOf({ lines, },);
        }
        expect(await warningOver({ view: LATE_ONE_SHORT, },),).not.toContain('no NeuralWatt return',);
        expect(await warningOver({ view: { ...LATE_ONE_SHORT, neuralwatt: true, }, },),).toContain(
          '; no NeuralWatt return inside what the pass has left,',
        );
      },
    },),

    it({
      name: 'KEEPS "no provider has named a return the pass can wait for" WHEN NO HOLD RUNS AT ALL: nothing is '
        + 'skipped, so there is no hold to name',
      fn: async (ctx) => {
        const lines: string[] = [];
        await readsUnderHolds({ held: [], lines, sinon: ctx.sinon, },);
        const warning = noWaitWarningOf({ lines, },);
        expect(warning,).toContain('no provider has named a return the pass can wait for',);
        expect(warning,).not.toContain('held:',);
      },
    },),
  ],
},);
