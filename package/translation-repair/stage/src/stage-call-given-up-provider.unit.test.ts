/**
 Tests what a stage makes of a call for a seat that only a given-up provider serves.

 An entry gives a provider up after it refuses right after a cooldown the entry waited out, while another writing
 provider reads wet. A seat that only the given-up provider serves then has nowhere to go: the real router, over the
 real budgets, ends the call as `NoProviderForModelError` before any request is sent. The stage must read that as that
 one seat unreachable, which the quorum and bench rules then decide on, and never as a crash, an error of the entry, or
 the end of the run (which `EveryProviderDryError` alone is).

 The budgets and the router are the real ones over stub meters and stub callers on a clock the case moves, so no case
 waits and no request is sent. Fixtures are cat-themed invention.

 @module
 */

import { tagged, } from '@monochromatic-dev/module-logger/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  createProviderBudgets,
  createRoutingClient,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import { TEST_MAX_HOLD_MS, } from '@monochromatic-dev/translation-repair-provider/ts/budget-hold.test-fixture.ts';
import { consoleLinesDuring, } from '@monochromatic-dev/translation-repair-provider/ts/neuralwatt-ledger.test-fixture.ts';
import {
  isPurrReply,
  PURR_FORMAT,
} from '@monochromatic-dev/translation-repair-provider/ts/stage-trivial-reply.test-fixture.ts';
import { unreadableQuotas, } from '@monochromatic-dev/translation-repair-provider/ts/unreadable-quotas.test-fixture.ts';
import { levelCapturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SEAT_SYNTHETIC_VISION_WITHHELD, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { attemptStageCall, } from '../dist/final/node/index.mjs';

/**
 Abort signal every call here carries.
 */
const SIGNAL = new AbortController().signal;

/**
 The refusal cooldown the budgets set by default, in milliseconds.
 */
const COOLDOWN_MS = 300_000;

/**
 Logger of the calling stage.
 */
const l = tagged({ tag: 'stage-call-given-up-provider-test', },);

await describe({
  name: `${attemptStageCall.name} over a provider the entry gave up`,
  // One case at a time: each stubs the console, which is one for the whole process.
  concurrency: 1,
  children: [
    it({
      name: 'READS A SEAT ONLY THE GIVEN-UP PROVIDER SERVES AS THAT SEAT UNREACHABLE: the router ends the call with '
        + 'no request on the wire, and the voice is neither heard nor a crash nor the end of the run',
      fn: async ctx => {
        const clock = { now: 1_000, };
        /**
         Requests each stub caller was asked to send.
         */
        const sent: string[] = [];
        /**
         A caller that records the request and fails, since none may be sent.
         */
        function callerFor(provider: string,): { readonly chatText: () => Promise<never>; } {
          return {
            chatText: async function chatText(): Promise<never> {
              sent.push(provider,);
              throw new Error(`${provider} was asked`,);
            },
          };
        }
        const budgets = createProviderBudgets({
          maxHoldMs: TEST_MAX_HOLD_MS,
          synthetic: { quotas: unreadableQuotas, },
          neuralwatt: {
            energy: function energy() {
              return Promise.resolve({
                dayStart: '2026-10-25T00:00:00.000Z',
                dailyWh: 550,
                spentWh: 0,
                reckonedWh: 0,
                reservedWh: 0,
                remainingWh: 550,
                calls: 0,
              },);
            },
          },
          now: function now(): number {
            return clock.now;
          },
        },);
        /**
         The real router over the real budgets.
         */
        const router = createRoutingClient({
          callers: {
            synthetic: callerFor('synthetic',),
            neuralwatt: callerFor('neuralwatt',),
          },
          budgets,
          l: levelCapturingLogger({ lines: [], },),
        },);
        /**
         The router as the stage takes it, with the meter surface a stage never calls.
         */
        const client: ModelClient = {
          ...router,
          quotas: async function quotas(): Promise<never> {
            throw new Error('quotas unused',);
          },
        };

        await consoleLinesDuring({
          run: async function giveUpThenCall(): Promise<void> {
            budgets.beginEntry({ entryId: 'CatEntry1', },);
            await budgets.markRefused({
              provider: 'synthetic',
              signal: SIGNAL,
            },);
            clock.now += COOLDOWN_MS + 12_000;
            await budgets.markRefused({
              provider: 'synthetic',
              signal: SIGNAL,
            },);
            expect((await budgets.read({ signal: SIGNAL, },)).synthetic,).toBe(true,);
            expect((await budgets.read({ signal: SIGNAL, },)).neuralwatt,).toBe(false,);

            expect(await attemptStageCall({
              client,
              modelId: SEAT_SYNTHETIC_VISION_WITHHELD,
              messages: [{ role: 'user', content: 'Where does the cat sleep?', },],
              signal: SIGNAL,
              exchangeTimeoutMs: HANG_STOP_MS,
              responseFormat: PURR_FORMAT,
              validate: isPurrReply,
              stage: 'critic',
              l,
            },),).toEqual({
              heard: false,
              answered: false,
              unreachable: true,
            },);
          },
          sinon: ctx.sinon,
        },);

        expect(sent,).toEqual([],);
      },
    },),
  ],
},);
