/**
 Exercises provider-supplied rate data, including the NeuralWatt overage exception.
 @module
 */

import { createAssistantMessageEventStream, isRetryableAssistantError, lazyStream, } from '@earendil-works/pi-ai';
import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import extension, { headerReset, } from '../dist/final/node/index.mjs';
import {
  assistant,
  collect,
  finish,
  fixture,
  immediate,
  MEMBER_KEY,
  PHYSICAL_MODELS,
  poolModel,
  rateLimitRefusal,
  RATE_LIMIT_STATUS,
  TRANSCRIPT,
} from './stand-in-pi-fixture.ts';
import { REVIEWER_TARGETS, WRITER_TARGETS, } from './member-order-fixture.ts';

/** Empty flex lane; a positive standard ceiling must not let it admit any requests. */
const NO_FLEX_REQUESTS = 0;

/** A quota window in the fixture clock, not a plugin cooldown. */
const WINDOW_MS = 60_000;

/** Header reset duration in seconds, matching the fixture clock window. */
const WINDOW_SECONDS = WINDOW_MS / 1_000;

/** Known advisory and observed caps recorded on 2026-11-11. */
const PUBLISHED_CAP = 20;
const OVERAGE_CAP = 6;

/**
 Reviewer's NeuralWatt member, the only NeuralWatt model either pool asks,
 with the full catalog record the stand-in registry returns.
 */
const NEURALWATT_FLEX = REVIEWER_TARGETS.at(-1,);
if (NEURALWATT_FLEX === undefined)
  throw new Error('Expected the reviewer pool to end with its NeuralWatt member',);

/**
 Read the URL from each fetch input form without stringifying an arbitrary object.

 @param input - quota fetch destination in the fixture

 @returns URL to check against the quota contract
 */
function quotaUrl(input: Parameters<typeof globalThis.fetch>[0],): string {
  if ((typeof input) === 'string')
    return input;
  if (input instanceof URL)
    return input.href;
  return input.url;
}

await describe({
  name: 'provider quota evidence',
  children: [
    ...[false, true,].map(function retryAfterFormat(asDate,) {
      return it({
        name: `uses ${asDate ? 'HTTP-date' : 'numeric'} 429 Retry-After and resumes at the reported reset`,
        fn: async function retryWindow(ctx,) {
          const clock = { now: Date.now(), };
          ctx.sinon.stub(Date, 'now',).callsFake(() => clock.now,);
          const deadline = new Date(clock.now + WINDOW_MS,);
          const retryAfter = asDate ? deadline.toUTCString() : String(WINDOW_SECONDS,);
          const { provider, calls, options, } = await fixture({
            extension,
            play({ model, options: member, }) {
              if (calls.length !== 1)
                return immediate({ model, },);
              return lazyStream(model, async function refused() {
                await member?.onResponse?.({
                  status: RATE_LIMIT_STATUS,
                  headers: { 'Retry-After': retryAfter, },
                }, model,);
                return immediate({ model, errorMessage: rateLimitRefusal({ model, },), },);
              },);
            },
          },);
          const model = poolModel({ provider, id: 'writer', },);
          const first = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
          const beforeReset = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
          clock.now = asDate ? Date.parse(retryAfter,) : deadline.getTime();
          const afterReset = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
          /**
           Physical member each dispatch answered from, as `provider/id`.
           */
          const answered = [first, beforeReset, afterReset,].map(function member(result,) {
            return `${result.result.provider}/${result.result.model}`;
          },);
          expect(answered,).toEqual([
            `${WRITER_TARGETS[1].provider}/${WRITER_TARGETS[1].id}`,
            `${WRITER_TARGETS[1].provider}/${WRITER_TARGETS[1].id}`,
            `${WRITER_TARGETS[0].provider}/${WRITER_TARGETS[0].id}`,
          ],);
        },
      },);
    },),
    it({
      name: 'uses the later compound OpenAI reset when both zero-remaining headers are seen on success',
      // Neither pool has an OpenAI member since 2026-11-19, so this provider branch is checked
      // against the parser directly rather than through a dispatch.
      fn: async function openAiRemaining() {
        /**
         Observation time for a successful response carrying depleted windows.
         */
        const now = Date.parse('2026-11-19T00:00:00Z',);
        /**
         Later of the two depleted windows, the request-count one.
         */
        const reset = headerReset({
          provider: 'openai',
          headers: new Headers({
            'x-ratelimit-remaining-tokens': '0',
            'x-ratelimit-reset-tokens': `${WINDOW_SECONDS / 2}s`,
            'x-ratelimit-remaining-requests': '0',
            'x-ratelimit-reset-requests': '1m0s',
          },),
          now,
          status: 200,
        },);
        expect(reset - now,).toBe(WINDOW_MS,);
      },
    },),
    it({
      name: 'reads Synthetic on first use and at reset, not on every request',
      fn: async function syntheticWindow(ctx,) {
        const clock = { now: Date.now(), };
        ctx.sinon.stub(Date, 'now',).callsFake(() => clock.now,);
        const end = clock.now + WINDOW_MS;
        const endpoints: string[] = [];
        const { provider, calls, options, } = await fixture({
          extension,
          models: PHYSICAL_MODELS.filter(model => (model.provider === 'synthetic') || (model.provider === 'opencode-go'),),
          play({ model, }) { return immediate({ model, },); },
          async fetchQuota(input, init,) {
            // Assert credentials inside the test without printing or returning them.
            if (new Headers(init?.headers,).get('authorization',) !== `Bearer ${MEMBER_KEY}`)
              throw new Error('Incorrect stand-in bearer',);
            if (init?.redirect !== 'error')
              throw new Error('Quota fetch permits redirects',);
            endpoints.push(quotaUrl(input,),);
            return Response.json({
              planWeeklyLimit: { percentRemaining: clock.now < end ? 0 : 100, nextRegenAt: new Date(end,).toISOString(), },
            },);
          },
        },);
        const model = poolModel({ provider, id: 'reviewer', },);
        await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        clock.now = end;
        await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(endpoints,).toEqual(Array.from({ length: 2, }, () => 'https://api.synthetic.new/v2/quotas',),);
        expect(calls.map(call => call.model.provider,),).toEqual(['opencode-go', 'opencode-go', 'synthetic', 'synthetic',],);
      },
    },),
    it({
      name: 'does not send a credential to a quota endpoint after a registry origin override',
      fn: async function credentialsStayOnOrigin() {
        const probe = { invoked: false, };
        const { provider, options, } = await fixture({
          extension,
          models: PHYSICAL_MODELS.filter(model => model.provider === 'synthetic',),
          resolvedBaseUrl: 'https://stand-in-proxy.example/v2',
          play({ model, }) { return immediate({ model, },); },
          async fetchQuota() {
            probe.invoked = true;
            return Response.json({});
          },
        },);
        const model = poolModel({ provider, id: 'reviewer', },);
        const { result, } = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(result.provider,).toBe('synthetic',);
        expect(probe.invoked,).toBe(false,);
      },
    },),
    it({
      name: 'reports the named NeuralWatt member and a published flex floor of zero without retrying',
      fn: async function zeroFlexFloor() {
        const redirects: RequestRedirect[] = [];
        const { provider, calls, options, } = await fixture({
          extension,
          models: [NEURALWATT_FLEX,],
          play({ model, }) { return immediate({ model, },); },
          async fetchQuota(input, init,) {
            if (init?.redirect !== undefined)
              redirects.push(init.redirect,);
            return quotaUrl(input,).includes('/v1/rate_limits',)
              ? Response.json({ models: [{ effective_concurrency: PUBLISHED_CAP, flex_floor: NO_FLEX_REQUESTS, },], },)
              : Response.json({});
          },
        },);
        const model = poolModel({ provider, id: 'reviewer', },);
        const { result, } = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(calls,).toEqual([],);
        expect(redirects,).toEqual(['error', 'error',],);
        expect(result.errorMessage,).toContain(`${NEURALWATT_FLEX.provider}/${NEURALWATT_FLEX.id} has a published flex floor of zero`,);
        expect(isRetryableAssistantError(result,),).toBe(false,);
      },
    },),
    it({
      name: 'admits only the reported flex ceiling, not the standard concurrency ceiling',
      fn: async function flexCeiling() {
        const physical = NEURALWATT_FLEX;
        const reading = { flex: 1, };
        const ready = Promise.withResolvers<void>();
        const held = createAssistantMessageEventStream();
        const { provider, calls, options, } = await fixture({
          extension,
          models: [physical,],
          play({ model, }) {
            if (calls.length !== 1)
              return immediate({ model, },);
            ready.resolve();
            return held;
          },
          async fetchQuota(input,) {
            return quotaUrl(input,).includes('/v1/rate_limits',)
              ? Response.json({ models: [{ effective_concurrency: PUBLISHED_CAP, flex_floor: reading.flex, },], },)
              : Response.json({});
          },
        },);
        const model = poolModel({ provider, id: 'reviewer', },);
        const pending = provider.streamSimple(model, TRANSCRIPT, options,);
        {
          await using cleanup: AsyncDisposable = {
            async [Symbol.asyncDispose](): Promise<void> {
              finish({ stream: held, message: assistant({ model: physical, },), },);
              await collect(pending,);
            },
          };
          await ready.promise;
          const denied = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
          expect(denied.result.errorMessage,).toContain(`${physical.provider}/${physical.id} is at its local concurrency ceiling (1 of 1 in use)`,);
          expect(isRetryableAssistantError(denied.result,),).toBe(false,);
          expect(calls.length,).toBe(1,);
          reading.flex += 1;
          const afterIncrease = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
          expect(afterIncrease.result.stopReason,).toBe('stop',);
        }
        const again = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(again.result.provider,).toBe('neuralwatt',);
      },
    },),
    it({
      name: `a NeuralWatt overage 429 overrides ${PUBLISHED_CAP} with ${OVERAGE_CAP} until a renewal learned on reentry`,
      fn: async function overageCap(ctx,) {
        const clock = { now: Date.now(), };
        ctx.sinon.stub(Date, 'now',).callsFake(() => clock.now,);
        const end = clock.now + WINDOW_MS;
        const endpoints: string[] = [];
        const ready = Promise.withResolvers<void>();
        const held: (() => void)[] = [];
        const { provider, calls, options, } = await fixture({
          extension,
          models: [NEURALWATT_FLEX,],
          play({ model, }) {
            if (calls.length === 1) {
              return immediate({
                model,
                errorMessage: rateLimitRefusal({
                  model,
                  message: `Concurrent limit reached for deepseek-ai/DeepSeek-V4.1-Flash: ${OVERAGE_CAP}/${OVERAGE_CAP} slots in use. Your subscription is in overage with auto-renew off, so pay-as-you-go limits apply until it renews.`,
                },),
              },);
            }
            if (clock.now >= end)
              return immediate({ model, },);
            const stream = createAssistantMessageEventStream();
            held.push(() => finish({ stream, message: assistant({ model, },), },),);
            if (held.length === OVERAGE_CAP)
              ready.resolve();
            // Deliberately complete any unexpected extra stream so a mutant fails without hanging.
            if (held.length > OVERAGE_CAP)
              finish({ stream, message: assistant({ model, },), },);
            return stream;
          },
          async fetchQuota(input,) {
            const url = new URL(quotaUrl(input,),);
            endpoints.push(url.pathname,);
            if (url.pathname === '/v1/rate_limits') {
              if (url.searchParams.get('model',) !== 'deepseek-v4.1-flash-flex')
                throw new Error('The quota query did not name the flex model',);
              return Response.json({ models: [{ effective_concurrency: PUBLISHED_CAP, flex_floor: PUBLISHED_CAP, },], },);
            }
            return Response.json({
              subscription: {
                in_overage: true,
                auto_renew: false,
                ...(calls.length === 0 ? {} : { current_period_end: new Date(end,).toISOString(), }),
              },
            },);
          },
        },);
        const model = poolModel({ provider, id: 'reviewer', },);
        const first = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(first.result.errorMessage,).toContain('overage with auto-renew off',);
        const inFlight = Array.from({ length: OVERAGE_CAP, }, () => provider.streamSimple(model, TRANSCRIPT, options,),);
        await using cleanup: AsyncDisposable = {
          async [Symbol.asyncDispose](): Promise<void> {
            for (const release of held)
              release();
            await Promise.all(inFlight.map(function consume(stream,) {
              return collect(stream,);
            },),);
          },
        };
        await ready.promise;
        {
          const limited = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
          expect(limited.result.errorMessage,).toContain(`local concurrency ceiling (${OVERAGE_CAP} of ${OVERAGE_CAP} in use)`,);
          expect(isRetryableAssistantError(limited.result,),).toBe(false,);
          expect(calls.length,).toBe(1 + OVERAGE_CAP,);
          clock.now = end;
          const renewed = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
          expect(renewed.result.provider,).toBe('neuralwatt',);
          expect(renewed.result.stopReason,).toBe('stop',);
          expect(endpoints,).toEqual([
            '/v1/rate_limits', '/v1/quota', '/v1/rate_limits', '/v1/quota', '/v1/rate_limits', '/v1/quota',
          ],);
        }
      },
    },),
  ],
},);
