/**
 Tests the run client the packages export (`run-client-create.ts`): which provider an entry's give-up of a refusing
 provider waits for, over the run client the passes build, with the seat reader's bench floors it hands the budgets.

 An entry gives a provider up when it refuses right after a cooldown the entry waited out. The give-up holds only while
 the providers left (not dry, not held, not given up) can still meet every bench floor the entry's stages need, the
 writing floors and the judging quorums, counted the way the seat reader counts them. Otherwise the cooldown is waited
 out as before, because reading the provider dry would cost the entry what waiting would have saved.

 Every case builds the run client itself (`runClientFrom`) over a scripted transport on a fake clock: Synthetic
 answers its quota with a body the budgets cannot read (so the meter counts spendable) and every chat call with HTTP
 429, as in XingZ6020, and the providers beside it read wet. The cases ask a seat only Synthetic and Hyper serve, then
 read the seats of the translate lane as the next slice would, so the wiring from the client to the budgets and the
 seat reader is part of what they run. No case waits, and no request leaves the process.

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
} from '@monochromatic-dev/module-test/ts';

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
 Which provider reads wet beside the refusing Synthetic.
 */
type Alternative = 'deepseek' | 'neuralwatt' | 'hyper';

/**
 The names of the providers a case lets read wet, by the key variable that switches each on.
 */
const KEY_VARIABLES: Readonly<Record<Alternative, string>> = {
  deepseek: 'DEEPSEEK_API_KEY',
  neuralwatt: 'TRANSLATION_REPAIR_NEURALWATT_API_KEY',
  hyper: 'TRANSLATION_REPAIR_CHARM_HYPER_API_KEY',
};

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
   How many slices sent Synthetic a chat request. A request the transport's retry ladder repeats counts once.
   */
  readonly slicesThatAskedSynthetic: () => number;

  /**
   How long Synthetic stays held out right after each slice that asked it, as the client reports it, in the order of
   the slices.
   */
  readonly holdsAfterEachAsk: () => readonly number[];

  /**
   Asks for a seat only Synthetic serves among the providers read wet, as a slice does, then reads the seats of the
   translate lane while the clock passes the cooldown.
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
 @param alternative - which provider reads wet beside Synthetic
 @param ledgerPath - where NeuralWatt's energy ledger lives, in a throwaway directory

 @returns The rig, and the disposable that puts the real timers back

 @example
 ```ts
 using installed = rigOn({ sinon: ctx.sinon, alternative: 'deepseek', ledgerPath, });
 ```
 */
function rigOn(
  {
    sinon,
    alternative,
    ledgerPath,
  }: {
    readonly sinon: DisposableSandbox;
    readonly alternative: Alternative;
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
   What the transport and the slices have counted: chat requests Synthetic was sent, the slices that sent one, and the
   hold the first slice's refusal set.
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
      TRANSLATION_REPAIR_SYNTHETIC_API_KEY: 'tabby-not-a-real-key',
      [KEY_VARIABLES[alternative]]: 'calico-not-a-real-key',
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
    if (counted.requests > requestsBefore) {
      counted.slices += 1;
      counted.holds.push(client.providerHolds().synthetic,);
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
    slicesThatAskedSynthetic: function slicesThatAskedSynthetic(): number {
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
  name: 'an entry gives a refusing provider up only while the providers left meet every bench floor',
  // One case at a time: each installs a fake clock and stubs the console and a built-in, which are one for the process.
  concurrency: 1,
  children: [
    it({
      name: 'WAITS FOR SYNTHETIC EVERY TIME, AND THE ENTRY IS NOT ENDED, when the only provider beside it is DeepSeek, '
        + 'which serves one of the eight translators where the writing floor is two',
      fn: async ctx => {
        await using scratch = await scratchDir({ prefix: 'run-client-create-', },);
        using rig = rigOn({
          sinon: ctx.sinon,
          alternative: 'deepseek',
          ledgerPath: join(scratch.path, 'neuralwatt-energy.jsonl',),
        },);
        await consoleLinesDuring({
          run: function slices(): Promise<void> {
            return runSlices({ rig, },);
          },
          sinon: ctx.sinon,
        },);

        // POSITIVE CONTROLS FIRST: the refusal set the cooldown the case's clock counts, and the later slices were asked
        // after it ended, so the fake clock moves the budgets' clock and not only the reader's sleeps.
        expect(rig.holdsAfterEachAsk().slice(0, 1,),).toEqual([COOLDOWN_MS,],);
        // The entry is not ended for want of a writing bench: DeepSeek serves one translator and the floor is two.
        expect(
          rig.endings.filter(function isWritingBenchStop(name,): boolean {
            return name === 'WritingBenchUnreachableError';
          },),
        ).toEqual([],);
        expect(rig.endings,).toEqual([],);
        expect(waitsIn({ lines: rig.lines, },).length,).toBe(SLICES,);
        expect(rig.slicesThatAskedSynthetic(),).toBe(SLICES,);
      },
    },),

    it({
      name: 'WAITS FOR SYNTHETIC EVERY TIME when the only provider beside it is NeuralWatt, which meets the writing '
        + 'floors and not the judging quorums: a judging bench counts too',
      fn: async ctx => {
        await using scratch = await scratchDir({ prefix: 'run-client-create-', },);
        using rig = rigOn({
          sinon: ctx.sinon,
          alternative: 'neuralwatt',
          ledgerPath: join(scratch.path, 'neuralwatt-energy.jsonl',),
        },);
        await consoleLinesDuring({
          run: function slices(): Promise<void> {
            return runSlices({ rig, },);
          },
          sinon: ctx.sinon,
        },);

        expect(rig.endings,).toEqual([],);
        expect(waitsIn({ lines: rig.lines, },).length,).toBe(SLICES,);
        expect(rig.slicesThatAskedSynthetic(),).toBe(SLICES,);
      },
    },),

    it({
      name: 'KEEPS THE GIVE-UP when Hyper reads wet beside Synthetic and meets every floor on its own: Synthetic is asked '
        + 'twice, and reads dry after the cooldown the entry gave it up on',
      fn: async ctx => {
        await using scratch = await scratchDir({ prefix: 'run-client-create-', },);
        using rig = rigOn({
          sinon: ctx.sinon,
          alternative: 'hyper',
          ledgerPath: join(scratch.path, 'neuralwatt-energy.jsonl',),
        },);
        await consoleLinesDuring({
          run: function slices(): Promise<void> {
            return runSlices({ rig, },);
          },
          sinon: ctx.sinon,
        },);

        expect(rig.slicesThatAskedSynthetic(),).toBe(2,);
        expect(rig.endings,).toEqual([],);
        expect((await rig.client.providerDryness({ signal: SIGNAL, },)).synthetic,).toBe(true,);
      },
    },),
  ],
},);
