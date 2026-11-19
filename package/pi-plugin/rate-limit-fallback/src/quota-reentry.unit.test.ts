/**
 Quota evidence and failures control when a member is checked again.
 @module
 */

import { syntheticSubscription, syntheticWeeklyWindow, } from './synthetic-quota-fixture.ts';
import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import extension from '../dist/final/node/index.mjs';
import {
  collect,
  fixture,
  immediate,
  PHYSICAL_MODELS,
  poolModel,
  rateLimitRefusal,
  TRANSCRIPT,
} from './stand-in-pi-fixture.ts';

/** Catalog subset that demonstrates a hold without exhausting the pool. */
const SYNTHETIC_AND_BACKUP = PHYSICAL_MODELS.filter(model => (model.provider === 'synthetic') || (model.provider === 'opencode-go'),);

/**
 Alibaba Cloud's published wait for its request-rate refusal, quoted from the Token Plan FAQ
 (https://www.alibabacloud.com/help/en/model-studio/token-plan-faq, "Rate limiting and concurrency":
 "Wait about 1 minute before retrying"). Documented wording, not a capture: no recorded refusal on
 this machine carries `Requests rate limit exceeded`.
 */
const QWEN_WAIT_MS = 60_000;

/** Duration of the stand-in Synthetic quota window, chosen independently of the Qwen recommendation. */
const SYNTHETIC_WINDOW_MS = 3_600_000;

/**
 Minimum wait the endpoint documents between two quota reads of one member: at most one a second
 (https://docs.neuralwatt.com/guides/rate-limits.md, "Checking your current limit").
 */
const MINIMUM_READ_INTERVAL_MS = 1_000;

/**
 Requests the subscription stand-in allows in its window, one, so a single request depletes it.
 */
const SUBSCRIPTION_REQUEST_LIMIT = 1;

await describe({
  name: 'quota reentry',
  // These cases stub the shared clock, so one at a time keeps them from reading each other's time.
  concurrency: 1,
  children: [
    it({
      name: 'holds Qwen for the documented minute when a request-rate 429 carries no reset',
      fn: async function qwenRecommendedReset(ctx,) {
        const clock = { now: Date.now(), };
        ctx.sinon.stub(Date, 'now',).callsFake(() => clock.now,);
        const { provider, calls, options, } = await fixture({
          extension,
          play({ model, }) {
            return calls.length === 1
              ? immediate({ model, errorMessage: rateLimitRefusal({ model, message: 'Requests rate limit exceeded', },), },)
              : immediate({ model, },);
          },
        },);
        const model = poolModel({ provider, id: 'reviewer', },);
        const first = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        clock.now += QWEN_WAIT_MS - 1;
        const beforeReset = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        clock.now += 1;
        const atReset = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(first.result.provider,).toBe('synthetic',);
        expect(beforeReset.result.provider,).toBe('synthetic',);
        expect(atReset.result.provider,).toBe('qwen-token-plan-individual',);
      },
    },),
    it({
      name: 'retries an unavailable first quota read and honors the next depleted window',
      fn: async function firstReadUnavailable(ctx,) {
        const clock = { now: Date.now(), };
        ctx.sinon.stub(Date, 'now',).callsFake(() => clock.now,);
        const observations = { reads: 0, };
        const { provider, options, } = await fixture({
          extension,
          models: SYNTHETIC_AND_BACKUP,
          play({ model, }) { return immediate({ model, },); },
          async fetchQuota() {
            observations.reads += 1;
            if (observations.reads === 1)
              return new Response(null, { status: 503, },);
            return syntheticWeeklyWindow({
              percentRemaining: 0,
              nextRegenAt: new Date(Date.now() + SYNTHETIC_WINDOW_MS,).toISOString(),
            },);
          },
        },);
        const model = poolModel({ provider, id: 'reviewer', },);
        const first = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        // The endpoint may be read again only after its documented interval.
        clock.now += MINIMUM_READ_INTERVAL_MS;
        const second = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(first.result.provider,).toBe('synthetic',);
        expect(second.result.provider,).toBe('opencode-go',);
        expect(observations.reads,).toBe(2,);
      },
    },),
    it({
      name: 'reads one member quota at most once per second',
      fn: async function readInterval(ctx,) {
        const clock = { now: Date.now(), };
        ctx.sinon.stub(Date, 'now',).callsFake(() => clock.now,);
        const reads = { count: 0, };
        const { provider, options, } = await fixture({
          extension,
          models: SYNTHETIC_AND_BACKUP,
          play({ model, }) { return immediate({ model, },); },
          async fetchQuota() {
            reads.count += 1;
            return new Response(null, { status: 503, },);
          },
        },);
        const model = poolModel({ provider, id: 'reviewer', },);
        await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(reads.count,).toBe(1,);
        await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(reads.count,).toBe(1,);
        clock.now += MINIMUM_READ_INTERVAL_MS;
        await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(reads.count,).toBe(2,);
      },
    },),
    it({
      name: 'holds a member on the previous Synthetic weekly field name',
      fn: async function legacyWeeklyField() {
        const { provider, options, } = await fixture({
          extension,
          models: SYNTHETIC_AND_BACKUP,
          play({ model, }) { return immediate({ model, },); },
          async fetchQuota() {
            return Response.json({
              weeklyTokenLimit: {
                percentRemaining: 0,
                nextRegenAt: new Date(Date.now() + SYNTHETIC_WINDOW_MS,).toISOString(),
              },
            },);
          },
        },);
        const model = poolModel({ provider, id: 'reviewer', },);
        const { result, } = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(result.provider,).toBe('opencode-go',);
        expect(result.stopReason,).toBe('stop',);
      },
    },),
    it({
      name: 'holds Synthetic on its subscription request limit until renewal',
      fn: async function syntheticSubscriptionReset(ctx,) {
        const clock = { now: Date.now(), };
        ctx.sinon.stub(Date, 'now',).callsFake(() => clock.now,);
        const end = clock.now + SYNTHETIC_WINDOW_MS;
        const limit = SUBSCRIPTION_REQUEST_LIMIT;
        const { provider, options, } = await fixture({
          extension,
          models: SYNTHETIC_AND_BACKUP,
          play({ model, }) { return immediate({ model, },); },
          async fetchQuota() {
            return syntheticSubscription({
              requests: clock.now < end ? limit : 0,
              limit,
              renewsAt: new Date(end,).toISOString(),
            },);
          },
        },);
        const model = poolModel({ provider, id: 'reviewer', },);
        const held = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        clock.now = end;
        const renewed = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(held.result.provider,).toBe('opencode-go',);
        expect(renewed.result.provider,).toBe('synthetic',);
      },
    },),
  ],
},);
