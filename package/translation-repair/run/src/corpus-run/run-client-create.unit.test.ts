/**
 Tests the run client the packages export (`run-client-create.ts`): which provider an entry's give-up of a refusing
 provider waits for, over the run client the passes build, with the seat reader's bench floors it hands the budgets.

 An entry gives a provider up when it refuses right after a cooldown the entry waited out. The give-up is set aside, and
 the cooldown waited out as before, only when waiting buys something: the given-up provider is the only writing provider
 not dry, or, with that provider back, no bench floor the entry needs would be left unmet while without it at least
 one is (the writing floors and the judging quorums, counted the way the seat reader counts them). Otherwise the
 give-up holds: its return would not make the entry finishable, so no further wait is taken for it.

 Every case builds the run client itself (`runClientFrom`) over a scripted transport on a fake clock: Synthetic
 answers its quota with a body the budgets cannot read (so the meter counts spendable) and every chat call with HTTP
 429, as in XingZ6020, and the providers beside it read wet. The case of condition (b) has Hyper refuse in the same
 way, since no set of providers beside a refusing Synthetic is lifted to every floor by its return (Bedrock is off, and
 never a live alternative). The cases ask a seat only Synthetic and Hyper serve, then read the seats of the translate
 lane as the next slice would, so the wiring from the client to the budgets and the seat reader is part of what they
 run. No case waits, and no request leaves the process.

 An ask is awaited to its end and never cut short by a count of event-loop turns: it reads NeuralWatt's ledger file
 whenever NeuralWatt reads wet, which finishes when the host's thread pool says and not after a fixed number of turns,
 and the fake clock must not move while that read is out, or the hold the refusal sets after it is stamped on a clock
 that is about to jump past it. Only the case where no writing provider reads wet lets the clock move during the ask,
 since the router then holds the ask itself through the cooldown (`routerHoldsTheAsk`).

 Fixtures are cat-themed invention; every reply is a script.

 @module
 */

import { syncBuiltinESMExports, } from 'node:module';
import { join, } from 'node:path';
import timersPromises from 'node:timers/promises';

import {
  describe,
  type DisposableSandbox,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { NEURALWATT_LEDGER_PATH_VAR, } from '@monochromatic-dev/translation-repair-provider';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import type {
  TransportExchange,
  TransportReply,
} from '@monochromatic-dev/translation-repair-provider-stream';
import { TEST_MAX_HOLD_MS, } from '@monochromatic-dev/translation-repair-provider/ts/budget-hold.test-fixture.ts';
import { consoleLinesDuring, } from '@monochromatic-dev/translation-repair-provider/ts/neuralwatt-ledger.test-fixture.ts';
import { SEAT_SYNTHETIC_VISION_NO_OPENROUTER, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { levelCapturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import {
  readPhaseSeats,
  type RunClient,
  runClientFrom,
} from '../../dist/final/node/index.mjs';
import { waitsIn, } from './run-seats-waits.test-fixture.ts';

/**
 Abort signal every read here carries.
 */
const SIGNAL: AbortSignal = new AbortController().signal;

/**
 The refusal cooldown the budgets set by default, which a refusal on a meter that cannot be read waits out.
 */
const COOLDOWN_MS = 300_000;

/**
 How often the seat reader's wait checks for abort, the real one-second slice.
 */
const POLL_MS = 1_000;

/**
 How many slices ask Synthetic, each right after the cooldown the slice before it waited out.
 */
const SLICES = 3;

/**
 A seat only Synthetic and Hyper serve, so no provider read wet in these cases (NeuralWatt, DeepSeek) takes a call for
 it, and a Hyper that reads wet answers it with an error the cases do not read.
 */
const SYNTHETIC_SEAT: RosterModelId = SEAT_SYNTHETIC_VISION_NO_OPENROUTER;

/**
 Quota body Synthetic answers with: no `weeklyTokenLimit`, so the budgets count the meter unreadable and the provider
 spendable.
 */
const UNREADABLE_QUOTA_BODY = '{}';

/**
 DeepSeek's balance with credit left.
 */
const DEEPSEEK_WET_BODY = '{"is_available":true,"balance_infos":[{"currency":"USD","total_balance":"9.96",'
  + '"granted_balance":"0.00","topped_up_balance":"9.96"}]}';

/**
 Hyper's balance with credit left.
 */
const HYPER_WET_BODY = '{"balance":243}';

/**
 NeuralWatt's quota reply of an account with a plan and energy left.
 */
const NEURALWATT_QUOTA_BODY = JSON.stringify({
  snapshot_at: '2026-10-20T20:00:00Z',
  subscription: { plan: 'max', current_period_start: '2026-10-11T05:05:25Z', kwh_used: 13.9, kwh_remaining: 6.1, in_overage: false, },
},);

/**
 NeuralWatt's models listing: the two covered models and no alias, which is all a meter reading needs.
 */
const NEURALWATT_MODELS_BODY = JSON.stringify({
  object: 'list',
  data: [
    { id: 'deepseek-v4.1-flash', metadata: { huggingface_id: 'deepseek-ai/DeepSeek-V4.1-Flash-1017', }, },
    { id: 'glm-5.3-flash', metadata: { huggingface_id: 'zai-org/GLM-5.3-Flash', }, },
  ],
},);

/**
 A provider that can read wet beside the refusing Synthetic.
 */
type Alternative = 'deepseek' | 'neuralwatt' | 'hyper' | 'xiaomi';

/**
 The names of the providers a case lets read wet, by the key variable that switches each on.
 */
const KEY_VARIABLES: Readonly<Record<Alternative, string>> = {
  deepseek: 'DEEPSEEK_API_KEY',
  neuralwatt: 'TRANSLATION_REPAIR_NEURALWATT_API_KEY',
  hyper: 'TRANSLATION_REPAIR_CHARM_HYPER_API_KEY',
  // Xiaomi has no balance endpoint: its meter is the run's own priced spend, so a key alone reads it wet.
  xiaomi: 'TRANSLATION_REPAIR_XIAOMI_MIMO_API_KEY',
};

/**
 The provider that refuses every chat call right after each cooldown the entry waited out.
 */
type Refuser = 'synthetic' | 'hyper';

/**
 The key variable that switches on each refuser.
 */
const REFUSER_KEY_VARIABLES: Readonly<Record<Refuser, string>> = {
  synthetic: 'TRANSLATION_REPAIR_SYNTHETIC_API_KEY',
  hyper: 'TRANSLATION_REPAIR_CHARM_HYPER_API_KEY',
};

/**
 The state of XingZ6020 (2026-10-23): NeuralWatt, DeepSeek and Xiaomi read wet in all 518 readings of the meters, and
 Hyper, OpenRouter and Bedrock never did.
 */
const XINGZ6020_WET: readonly Alternative[] = [
  'neuralwatt',
  'deepseek',
  'xiaomi',
];

/**
 What one case needs of the client, the clock and the reader.
 */
type Rig = {
  /**
   The run client under test.
   */
  readonly client: RunClient;

  /**
   Lines the seat reader wrote.
   */
  readonly lines: string[];

  /**
   The names of the errors the readings ended with, none when every reading seated its benches.
   */
  readonly endings: string[];

  /**
   How many slices sent the refusing provider a chat request. A request the transport's retry ladder repeats counts once.
   */
  readonly slicesThatAskedRefuser: () => number;

  /**
   How long the refusing provider stays held out right after each slice that asked it, as the client reports it, in the
   order of the slices.
   */
  readonly holdsAfterEachAsk: () => readonly number[];

  /**
   Asks for a seat only the refusing provider serves among the providers read wet, as a slice does, then reads the seats
   of the translate lane while the clock passes the cooldown.
   */
  readonly sliceAfterTheLastCooldown: () => Promise<void>;
};

/**
 Whether an exchange is addressed to NeuralWatt.

 @param exchange - the request

 @returns Whether it is
 */
function isNeuralwatt(exchange: TransportExchange,): boolean {
  return exchange.url.includes('api.neuralwatt.com',);
}

/**
 Answers a NeuralWatt route.

 @param exchange - the request, addressed to NeuralWatt

 @returns The reply
 */
function neuralwattReply(exchange: TransportExchange,): TransportReply {
  if (exchange.url.endsWith('/models',))
    return { status: 200, bodyText: NEURALWATT_MODELS_BODY, };
  if (exchange.url.includes('/usage/by-model',)) {
    /**
     Query the request carried.
     */
    const { searchParams, } = new URL(exchange.url,);
    return {
      status: 200,
      bodyText: JSON.stringify({
        period: {
          start: (searchParams.get('start_date',) ?? '').replace('Z', '+00:00',),
          end: (searchParams.get('end_date',) ?? '').replace('Z', '000+00:00',),
        },
        models: [],
        products: [],
      },),
    };
  }
  return { status: 200, bodyText: NEURALWATT_QUOTA_BODY, };
}

/**
 Builds the rig: a fake clock for the budgets and the reader, the retry ladder's sleeps spent at once, the run client
 over a scripted transport, and the real seat reader over the client.

 @param sinon - case's own sandbox, which fakes the timers
 @param wet - which providers read wet beside the refusing one
 @param refuser - the provider that refuses every chat call with a meter the budgets cannot read: Synthetic as in
 XingZ6020 (default), or Hyper where a case needs a refuser whose return would lift the benches
 @param routerHoldsTheAsk - whether the router holds each ask through the cooldown itself, because no provider that
 writes reads wet (default: no, the ask ends on its own and the clock does not move until it has)
 @param ledgerPath - where NeuralWatt's energy ledger lives, in a throwaway directory

 @returns The rig, and the disposable that puts the real timers back

 @example
 ```ts
 using installed = rigOn({ sinon: ctx.sinon, wet: ['deepseek',], ledgerPath, });
 ```
 */
function rigOn(
  {
    sinon,
    wet,
    refuser = 'synthetic',
    routerHoldsTheAsk = false,
    ledgerPath,
  }: {
    readonly sinon: DisposableSandbox;
    readonly wet: readonly Alternative[];
    readonly refuser?: Refuser;
    readonly routerHoldsTheAsk?: boolean;
    readonly ledgerPath: string;
  },
): Rig & Disposable {
  /**
   Fake clock moving the budgets' holds and the reader's sleeps together.
   */
  const clock = sinon.useFakeTimers({
    toFake: [
      'setTimeout',
      'clearTimeout',
      'performance',
    ],
    shouldClearNativeTimers: true,
  },);
  // THE RETRY LADDER SLEEPS ON `node:timers/promises`, which the fake clock does not reach: its sleeps are spent at once,
  // since what a case reads is the refusal the ladder ends with, not the seconds it took.
  const spentAtOnce = sinon.stub(
    timersPromises,
    'setTimeout',
  )
    .callsFake(function sleepAtOnce(): Promise<void> {
      return Promise.resolve();
    },);
  // The module's property is replaced; its named exports follow only when they are told to.
  syncBuiltinESMExports();
  /**
   Lines the seat reader wrote.
   */
  const lines: string[] = [];
  /**
   The names of the errors the readings ended with.
   */
  const endings: string[] = [];
  /**
   What the transport and the slices have counted: chat requests the refusing provider was sent, the slices that sent
   one, and the hold each such slice's refusal set.
   */
  const counted: {
    requests: number;
    slices: number;
    holds: number[];
  } = {
    requests: 0,
    slices: 0,
    holds: [],
  };
  const client = runClientFrom({
    maxHoldMs: TEST_MAX_HOLD_MS,
    env: {
      [REFUSER_KEY_VARIABLES[refuser]]: 'tabby-not-a-real-key',
      ...Object.fromEntries(wet.map(function keyOf(provider,): readonly [string, string,] {
        return [KEY_VARIABLES[provider], 'calico-not-a-real-key',];
      },),),
      [NEURALWATT_LEDGER_PATH_VAR]: ledgerPath,
    },
    transport: async function answer(exchange: TransportExchange,): Promise<TransportReply> {
      if (exchange.url.includes('api.synthetic.new',)) {
        if (exchange.method === 'GET')
          return { status: 200, bodyText: UNREADABLE_QUOTA_BODY, };
        counted.requests += 1;
        return { status: 429, bodyText: '{"error":"rate limited"}', };
      }
      if (exchange.url.includes('api.deepseek.com',))
        return { status: 200, bodyText: DEEPSEEK_WET_BODY, };
      if (exchange.url.includes('hyper.charm.land',)) {
        // A Hyper that refuses answers its balance with a body the budgets cannot read, and every chat call with 429.
        if (refuser === 'hyper') {
          if (exchange.method === 'GET')
            return { status: 200, bodyText: UNREADABLE_QUOTA_BODY, };
          counted.requests += 1;
          return { status: 429, bodyText: '{"error":"rate limited"}', };
        }
        return (exchange.method === 'GET')
          ? { status: 200, bodyText: HYPER_WET_BODY, }
          : { status: 400, bodyText: '{"error":"no stream scripted"}', };
      }
      return isNeuralwatt(exchange,) ? neuralwattReply(exchange,) : { status: 404, bodyText: '{}', };
    },
  },);

  /**
   Asks for the Synthetic seat and reads the translate lane's seats as the next slice would, letting the clock pass the
   cooldown meanwhile.
   */
  async function sliceAfterTheLastCooldown(): Promise<void> {
    /**
     Requests Synthetic had been sent before this slice.
     */
    const requestsBefore = counted.requests;
    /**
     Asks for the seat and lets the ask end however it ends.
     */
    async function askForTheSeat(): Promise<void> {
      try {
        await client.chatText({
          modelId: SYNTHETIC_SEAT,
          messages: [{ role: 'user', content: 'Where does the cat sleep?', },],
          signal: SIGNAL,
        },);
      } catch (error) {
        // The ask ends in the refusal, or in the router's answer that no provider read wet serves the seat; the cases
        // read what the budgets and the reader did next, not the ask's own ending. Anything that is no error is a defect.
        if (!Error.isError(error,))
          throw error;
      }
    }
    /**
     The ask, started and then awaited to its end.
     */
    const asking = askForTheSeat();
    // THE CLOCK DOES NOT MOVE WHILE AN ASK IS OUT, except for the ask the router holds itself. A budget reading of
    // NeuralWatt reads its ledger file, which the thread pool finishes whenever it can; a count of event-loop turns that
    // took a slow read for a router waiting on the hold moved the clock past the 300 s the refusal had just set, and the
    // hold then read zero. The reader's own wait comes after the ask.
    // The ask the router holds reads no file (the case lets Xiaomi alone read wet, and NeuralWatt has no key, so no ledger
    // is read), so it has gone as far as its wait on the fake clock by the time the clock first moves.
    if (routerHoldsTheAsk)
      await clock.tickAsync(COOLDOWN_MS + POLL_MS,);
    await asking;
    if (counted.requests > requestsBefore) {
      counted.slices += 1;
      counted.holds.push(client.providerHolds()[refuser],);
    }
    /**
     Reads the seats once and notes the name of the error that ended the entry, if one did. Its answer is not read, only
     what it said and whether it ended the entry, so a refusal is noted where it happens and not left for the end.
     */
    async function readOnce(): Promise<void> {
      try {
        await readPhaseSeats({
          client,
          phase: 'translate lane',
          signal: SIGNAL,
          l: levelCapturingLogger({ lines, },),
          pollMs: POLL_MS,
        },);
      } catch (error) {
        endings.push(Error.isError(error,) ? error.name : String(error,),);
      }
    }
    /**
     The reading the slice takes, started before the clock passes the cooldown.
     */
    const reading = readOnce();
    await clock.tickAsync(COOLDOWN_MS + POLL_MS,);
    await reading;
  }

  return {
    client,
    lines,
    endings,
    slicesThatAskedRefuser: function slicesThatAskedRefuser(): number {
      return counted.slices;
    },
    holdsAfterEachAsk: function holdsAfterEachAsk(): readonly number[] {
      return counted.holds;
    },
    sliceAfterTheLastCooldown,
    [Symbol.dispose]: function removeClock(): void {
      spentAtOnce.restore();
      syncBuiltinESMExports();
      clock.restore();
    },
  };
}

/**
 Runs the slices one after another, each right after the cooldown the one before waited out.

 @param rig - the rig to run them on

 @example
 ```ts
 await runSlices({ rig, },);
 ```
 */
async function runSlices({ rig, }: { readonly rig: Rig; },): Promise<void> {
  for (const _slice of Array.from({ length: SLICES, },)
    .keys()) {
    // oxlint-disable-next-line no-await-in-loop -- each slice follows the reading the cooldown ended
    await rig.sliceAfterTheLastCooldown();
  }
}

await describe({
  name: 'an entry waits for a refusing provider only while its return would make the entry finishable',
  // One case at a time: each installs a fake clock and stubs the console and a built-in, which are one for the process.
  concurrency: 1,
  children: [
    it({
      name: 'WAITS FOR SYNTHETIC ONCE AND THEN STOPS in the state of XingZ6020 (NeuralWatt, DeepSeek and Xiaomi wet; '
        + 'Synthetic refusing right after each cooldown waited out): its return would lift two benches and leave five short',
      fn: async ctx => {
        await using scratch = await scratchDir({ prefix: 'run-client-create-', },);
        using rig = rigOn({
          sinon: ctx.sinon,
          wet: XINGZ6020_WET,
          ledgerPath: join(scratch.path, 'neuralwatt-energy.jsonl',),
        },);
        const lines = await consoleLinesDuring({
          run: function slices(): Promise<void> {
            return runSlices({ rig, },);
          },
          sinon: ctx.sinon,
        },);

        // POSITIVE CONTROL FIRST: the refusal set the cooldown the case's clock counts.
        expect(rig.holdsAfterEachAsk().slice(0, 1,),).toEqual([COOLDOWN_MS,],);
        // The first wait is taken; the refusal right after it gives Synthetic up, and no later wait is.
        expect(waitsIn({ lines: rig.lines, },).length,).toBe(1,);
        expect(rig.slicesThatAskedRefuser(),).toBe(2,);
        // The line that gives it up names the floors that stay unmet with Synthetic back.
        /**
         The lines that keep the give-up in force.
         */
        const kept = lines.filter(function isKept(line,): boolean {
          return line.includes('keeps its give-up in force',);
        },);
        expect(kept.length,).toBe(1,);
        expect(kept[0],).toContain('with synthetic back',);
        expect(kept[0],).toContain('wide 2 of 7 reachable, quorum 4',);
      },
    },),

    it({
      name: 'WAITS FOR SYNTHETIC ONCE AND THEN STOPS when the only provider beside it is DeepSeek: with Synthetic back '
        + 'the editors and the refiners stay below their floor, so its return would not make the entry finishable',
      fn: async ctx => {
        await using scratch = await scratchDir({ prefix: 'run-client-create-', },);
        using rig = rigOn({
          sinon: ctx.sinon,
          wet: ['deepseek',],
          ledgerPath: join(scratch.path, 'neuralwatt-energy.jsonl',),
        },);
        await consoleLinesDuring({
          run: function slices(): Promise<void> {
            return runSlices({ rig, },);
          },
          sinon: ctx.sinon,
        },);

        expect(rig.holdsAfterEachAsk().slice(0, 1,),).toEqual([COOLDOWN_MS,],);
        expect(waitsIn({ lines: rig.lines, },).length,).toBe(1,);
        expect(rig.slicesThatAskedRefuser(),).toBe(2,);
        // The entry cannot finish with DeepSeek alone, with Synthetic back or without it (the editors and the refiners
        // stay below their floor either way), so the two slices after Synthetic was given up stop the lane on the
        // translators' writing floor. That stop parks the entry; it is not the end of the run, which only a reading of
        // every provider dry is.
        expect(rig.endings,).toEqual([
          'WritingBenchUnreachableError',
          'WritingBenchUnreachableError',
        ],);
      },
    },),

    it({
      name: 'WAITS FOR SYNTHETIC ONCE AND THEN STOPS when the only provider beside it is NeuralWatt, which meets the '
        + 'writing floors and not the judging quorums, and which Synthetic\'s return would not lift to them',
      fn: async ctx => {
        await using scratch = await scratchDir({ prefix: 'run-client-create-', },);
        using rig = rigOn({
          sinon: ctx.sinon,
          wet: ['neuralwatt',],
          ledgerPath: join(scratch.path, 'neuralwatt-energy.jsonl',),
        },);
        await consoleLinesDuring({
          run: function slices(): Promise<void> {
            return runSlices({ rig, },);
          },
          sinon: ctx.sinon,
        },);

        expect(rig.endings,).toEqual([],);
        expect(waitsIn({ lines: rig.lines, },).length,).toBe(1,);
        expect(rig.slicesThatAskedRefuser(),).toBe(2,);
      },
    },),

    it({
      name: 'KEEPS THE GIVE-UP when Hyper reads wet beside Synthetic and meets every floor on its own: Synthetic is asked '
        + 'twice, and reads dry after the cooldown the entry gave it up on',
      fn: async ctx => {
        await using scratch = await scratchDir({ prefix: 'run-client-create-', },);
        using rig = rigOn({
          sinon: ctx.sinon,
          wet: ['hyper',],
          ledgerPath: join(scratch.path, 'neuralwatt-energy.jsonl',),
        },);
        await consoleLinesDuring({
          run: function slices(): Promise<void> {
            return runSlices({ rig, },);
          },
          sinon: ctx.sinon,
        },);

        expect(rig.slicesThatAskedRefuser(),).toBe(2,);
        expect(rig.endings,).toEqual([],);
        expect((await rig.client.providerDryness({ signal: SIGNAL, },)).synthetic,).toBe(true,);
      },
    },),

    it({
      name: '(b) WAITS FOR HYPER EVERY TIME when NeuralWatt is the only provider beside it: NeuralWatt alone leaves '
        + 'benches short, and Hyper back meets every floor, so its return makes the entry finishable',
      fn: async ctx => {
        await using scratch = await scratchDir({ prefix: 'run-client-create-', },);
        using rig = rigOn({
          sinon: ctx.sinon,
          wet: ['neuralwatt',],
          refuser: 'hyper',
          ledgerPath: join(scratch.path, 'neuralwatt-energy.jsonl',),
        },);
        await consoleLinesDuring({
          run: function slices(): Promise<void> {
            return runSlices({ rig, },);
          },
          sinon: ctx.sinon,
        },);

        // POSITIVE CONTROL FIRST: Hyper's refusal set the cooldown the case's clock counts.
        expect(rig.holdsAfterEachAsk().slice(0, 1,),).toEqual([COOLDOWN_MS,],);
        expect(rig.endings,).toEqual([],);
        expect(waitsIn({ lines: rig.lines, },).length,).toBe(SLICES,);
        expect(rig.slicesThatAskedRefuser(),).toBe(SLICES,);
      },
    },),

    it({
      name: '(a) WAITS FOR SYNTHETIC AND DOES NOT END THE RUN when the only provider beside it is Xiaomi, which writes '
        + 'nothing: Synthetic is the only writing provider not dry, so a reading finds a writing provider and waits',
      fn: async ctx => {
        await using scratch = await scratchDir({ prefix: 'run-client-create-', },);
        using rig = rigOn({
          sinon: ctx.sinon,
          wet: ['xiaomi',],
          routerHoldsTheAsk: true,
          ledgerPath: join(scratch.path, 'neuralwatt-energy.jsonl',),
        },);
        await consoleLinesDuring({
          run: function slices(): Promise<void> {
            return runSlices({ rig, },);
          },
          sinon: ctx.sinon,
        },);

        // POSITIVE CONTROL FIRST: the refusal set a cooldown, which the router that holds the ask has already spent part of.
        expect(rig.holdsAfterEachAsk()[0] ?? 0,).toBeGreaterThan(0,);
        expect(
          rig.endings.filter(function isEndOfRun(name,): boolean {
            return name === 'EveryProviderDryError';
          },),
        ).toEqual([],);
        // Synthetic is not read dry for good: the router held the first ask through the cooldown and asked again, and the
        // reader waited the second cooldown out, so no reading found every writing provider dry. Through the real client
        // the waits on one seat are bounded by `SEAT_WITHHOLD_ASKS`: the tally withholds the seat at the door after four
        // asks that threw, so later slices ask nobody. The pin of a wait in every slice for a provider that is the only
        // writing provider not dry is the seat reader's rig (`run-seats-cooldown-repeat.unit.test.ts`, Synthetic alone).
        expect(waitsIn({ lines: rig.lines, },).length,).toBeGreaterThan(0,);
      },
    },),
  ],
},);
