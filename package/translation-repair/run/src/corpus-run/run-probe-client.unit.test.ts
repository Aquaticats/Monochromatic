/**
 Tests that a probe's run client cuts a hold a refusal states for itself.

 A probe command has no hard cap, so the budgets its client builds once held a provider out for as long as a refusal
 stated: a body saying "try again in 99999999h" kept Synthetic out of routing for about eleven thousand years while the
 wait for the hold polled once a second. Every case here builds a client on an environment it hands over and a transport
 that refuses every call with that body, so nothing reaches a provider.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
  type DisposableSandbox,
} from '@monochromatic-dev/module-test/ts';

import { HOLD_POLL_MS, } from '@monochromatic-dev/translation-repair-provider';
import {
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import {
  withFreshRunSeats,
} from '@monochromatic-dev/translation-repair-stage/ts/corpus-run/fresh-run-seats.test-fixture.ts';
import {
  PROBE_MAX_HOLD_MS,
  probeRunClientFrom,
  type RunClient,
  runClientFrom,
} from '../../dist/final/node/index.mjs';

/**
 A bound no probe waits out, far past {@link PROBE_MAX_HOLD_MS} and far under the wait the refusals here state.
 */
const FAR_BOUND_MS = 86_400_000;

/**
 Milliseconds of fake time the call is given to meet its refusal before the hold is read.
 */
const FAKE_TICK_MS = 1;

/**
 Environment holding the first provider's key and nothing else. Invented.
 */
const ONE_KEY = { TRANSLATION_REPAIR_SYNTHETIC_API_KEY: 'whiskers-not-a-real-key', };

/**
 Transport that refuses every call with a body stating a wait of 99,999,999 hours.

 @returns Reply with HTTP 429 and the stated wait

 @example
 ```ts
 const client = probeRunClientFrom({ env: ONE_KEY, transport: refusingWithAHugeWait, },);
 ```
 */
async function refusingWithAHugeWait(): Promise<{ readonly status: number; readonly bodyText: string; }> {
  return { status: 429, bodyText: 'Rate limit reached. Please try again in 99999999h', };
}

/**
 Asks the client's one served seat on a fake clock, reads the hold the refusal left on Synthetic, and abandons the call.

 THE WAIT FOR A HOLD IS ON THE TIMER, so a fake clock stands in for it and the call is abandoned by its signal once the
 hold is read; a client that holds the provider out for the whole stated wait would otherwise never let the call end.

 @param client - client under test
 @param sinon - case's own sandbox, which restores the clock when the case ends

 @returns Milliseconds Synthetic is held out for after the first refusal

 @example
 ```ts
 const held = await heldAfterRefusal({ client, sinon: ctx.sinon, },);
 ```
 */
async function heldAfterRefusal(
  {
    client,
    sinon,
  }: {
    readonly client: RunClient;
    readonly sinon: DisposableSandbox;
  },
): Promise<number> {
  /**
   The fake clock the wait for the hold sleeps on.
   */
  const clock = sinon.useFakeTimers({ toFake: ['setTimeout', 'clearTimeout', 'performance',], },);
  /**
   Abort the call is abandoned by.
   */
  const abandon = new AbortController();

  /**
   Asks the seat and notes how the call ended, in a list of none or one.

   @returns What the call threw
   */
  async function askSeat(): Promise<readonly unknown[]> {
    try {
      await client.chatText({
        modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
        messages: [{ role: 'user', content: 'Where does the cat nap?', },],
        signal: abandon.signal,
      },);
      return [];
    }
    catch (error) {
      return [error,];
    }
  }

  /**
   The call, started on the fake clock.
   */
  const asking = askSeat();
  await clock.tickAsync(FAKE_TICK_MS,);
  /**
   Hold the refusal left on Synthetic.
   */
  const held = client.providerHolds().synthetic;
  abandon.abort();
  await clock.tickAsync(HOLD_POLL_MS,);
  await asking;
  return held;
}

await describe({
  name: probeRunClientFrom.name,
  children: [
    it({
      name: 'CUTS A HOLD A REFUSAL STATES TO THE PROBE BOUND, so a probe never waits out a stated '
        + 'wait of 99,999,999 hours',
      fn: async (ctx) => {
        using _fresh = withFreshRunSeats();
        const held = await heldAfterRefusal({
          client: probeRunClientFrom({ env: ONE_KEY, transport: refusingWithAHugeWait, },),
          sinon: ctx.sinon,
        },);

        // THE HOLD COUNTS DOWN ON THE CLOCK THE BUDGETS READ, which `heldAfterRefusal` fakes with the timers, and it
        // ticks one millisecond between the refusal and the read: the hold is the bound less that millisecond at the
        // most, on any load, where a window on the real clock was flipped by a host that stalled the process.
        expect(held,).toBeLessThanOrEqual(PROBE_MAX_HOLD_MS,);
        expect(held,).toBeGreaterThanOrEqual(PROBE_MAX_HOLD_MS - FAKE_TICK_MS,);
      },
    },),

    it({
      name: 'LETS THE SAME REFUSAL HOLD FAR PAST THE PROBE BOUND UNDER A FAR BOUND (the control arm): the rig shows an '
        + 'uncut hold, so it is the probe bound that cut the hold in the other case',
      fn: async (ctx) => {
        using _fresh = withFreshRunSeats();
        const held = await heldAfterRefusal({
          client: runClientFrom({
            env: ONE_KEY,
            transport: refusingWithAHugeWait,
            maxHoldMs: Number.MAX_SAFE_INTEGER,
          },),
          sinon: ctx.sinon,
        },);

        expect(held,).toBeGreaterThan(FAR_BOUND_MS,);
      },
    },),
  ],
},);
