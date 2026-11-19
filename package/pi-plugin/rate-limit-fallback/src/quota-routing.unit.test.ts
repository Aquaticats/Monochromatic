/**
 Exercises provider-supplied rate data, including the NeuralWatt overage exception.
 @module
 */

import { createAssistantMessageEventStream, lazyStream, } from '@earendil-works/pi-ai';
import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import extension from '../dist/final/node/index.mjs';
import {
  assistant,
  collect,
  finish,
  fixture,
  immediate,
  MEMBER_KEY,
  PHYSICAL_MODELS,
  poolModel,
  TRANSCRIPT,
} from './stand-in-pi-fixture.ts';

/** A quota window in the fixture clock, not a plugin cooldown. */
const WINDOW_MS = 60_000;

/** Header reset duration in seconds, matching the fixture clock window. */
const WINDOW_SECONDS = WINDOW_MS / 1_000;

/** Known advisory and observed caps recorded on 2026-11-11. */
const PUBLISHED_CAP = 20;
const OVERAGE_CAP = 6;

await describe({
  name: 'provider quota evidence',
  children: [
    it({
      name: 'uses a 429 Retry-After and resumes the preferred provider at the reset',
      fn: async function retryWindow(ctx,) {
        const clock = { now: Date.now(), };
        ctx.sinon.stub(Date, 'now',).callsFake(() => clock.now,);
        const { provider, calls, options, } = await fixture({
          extension,
          play({ model, options: member, }) {
            if (calls.length !== 1)
              return immediate({ model, },);
            return lazyStream(model, async function refused() {
              await member?.onResponse?.({
                status: 429,
                headers: { 'Retry-After': String(WINDOW_SECONDS,), },
              }, model,);
              return immediate({ model, errorMessage: 'HTTP 429: rate limit', },);
            },);
          },
        },);
        const model = poolModel({ provider, id: 'writer', },);
        const first = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        const beforeReset = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        clock.now += WINDOW_MS;
        const afterReset = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect([first.result.provider, beforeReset.result.provider, afterReset.result.provider,],).toEqual([
          'qwen-token-plan-individual', 'qwen-token-plan-individual', 'openai',
        ],);
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
          models: PHYSICAL_MODELS.filter(model => model.provider === 'synthetic' || model.provider === 'opencode-go',),
          play({ model, }) { return immediate({ model, },); },
          async fetchQuota(input, init,) {
            // Assert credentials inside the test without printing or returning them.
            if (new Headers(init?.headers,).get('authorization',) !== `Bearer ${MEMBER_KEY}`)
              throw new Error('Incorrect stand-in bearer',);
            if (init?.redirect !== 'error')
              throw new Error('Quota fetch permits redirects',);
            endpoints.push(String(input,),);
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
        let touched = false;
        const { provider, options, } = await fixture({
          extension,
          models: PHYSICAL_MODELS.filter(model => model.provider === 'synthetic',),
          resolvedBaseUrl: 'https://stand-in-proxy.example/v2',
          play({ model, }) { return immediate({ model, },); },
          async fetchQuota() {
            touched = true;
            return Response.json({});
          },
        },);
        const model = poolModel({ provider, id: 'reviewer', },);
        const { result, } = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(result.provider,).toBe('synthetic',);
        expect(touched,).toBe(false,);
      },
    },),
    it({
      name: 'a NeuralWatt overage 429 overrides 20 with 6 until the subscription renews',
      fn: async function overageCap(ctx,) {
        const clock = { now: Date.now(), };
        ctx.sinon.stub(Date, 'now',).callsFake(() => clock.now,);
        const end = clock.now + WINDOW_MS;
        const endpoints: string[] = [];
        const ready = Promise.withResolvers<void>();
        const held: (() => void)[] = [];
        const { provider, calls, options, } = await fixture({
          extension,
          models: PHYSICAL_MODELS.filter(model => model.provider === 'neuralwatt',),
          play({ model, }) {
            if (calls.length === 1) {
              return immediate({
                model,
                errorMessage: `429 Concurrent limit reached for deepseek-ai/DeepSeek-V4.1-Flash: ${OVERAGE_CAP}/${OVERAGE_CAP} slots in use. Your subscription is in overage with auto-renew off, so pay-as-you-go limits apply until it renews.`,
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
            const url = new URL(String(input,),);
            endpoints.push(url.pathname,);
            if (url.pathname === '/v1/rate_limits') {
              if (url.searchParams.get('model',) !== 'deepseek-v4.1-flash-flex')
                throw new Error('The quota query did not name the flex model',);
              return Response.json({ models: [{ effective_concurrency: PUBLISHED_CAP, flex_floor: PUBLISHED_CAP, },], },);
            }
            return Response.json({
              subscription: {
                in_overage: clock.now < end,
                auto_renew: clock.now >= end,
                current_period_end: new Date(end,).toISOString(),
              },
            },);
          },
        },);
        const model = poolModel({ provider, id: 'reviewer', },);
        const first = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(first.result.errorMessage,).toContain('overage with auto-renew off',);
        const inFlight = Array.from({ length: OVERAGE_CAP, }, () => provider.streamSimple(model, TRANSCRIPT, options,),);
        await ready.promise;
        try {
          const limited = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
          expect(limited.result.errorMessage,).toContain('local provider concurrency ceiling',);
          expect(calls.length,).toBe(1 + OVERAGE_CAP,);
          clock.now = end;
          const renewed = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
          expect(renewed.result.provider,).toBe('neuralwatt',);
          expect(renewed.result.stopReason,).toBe('stop',);
          expect(endpoints,).toEqual([
            '/v1/rate_limits', '/v1/quota', '/v1/rate_limits', '/v1/quota', '/v1/rate_limits', '/v1/quota',
          ],);
        } finally {
          for (const release of held)
            release();
          for (const stream of inFlight)
            await collect(stream,);
        }
      },
    },),
  ],
},);
