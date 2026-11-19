/**
 Quota requests resolve credentials and cancellation independently from chat.
 @module
 */

import { setImmediate, } from 'node:timers/promises';
import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import extension from '../dist/final/node/index.mjs';
import { collect, fixture, immediate, PHYSICAL_MODELS, poolModel, TRANSCRIPT, } from './stand-in-pi-fixture.ts';

/** The only fixture route requiring a quota request. */
const SYNTHETIC_ONLY = PHYSICAL_MODELS.filter(function isSynthetic(model,) {
  return model.provider === 'synthetic';
},);

/** Bearer supplied only through the registry's resolved headers. */
const HEADER_ONLY_BEARER = 'Bearer stand-in-header-only';

await describe({
  name: 'quota authentication and cancellation',
  children: [
    it({
      name: 'uses registry-resolved bearer headers without forwarding unrelated headers',
      fn: async function resolvedAuthorization() {
        const { provider, options, } = await fixture({
          extension,
          models: SYNTHETIC_ONLY,
          resolvedHeaders: { Authorization: HEADER_ONLY_BEARER, Cookie: 'stand-in-session-cookie', },
          play({ model, }) { return immediate({ model, },); },
          async fetchQuota(_url, init,) {
            const headers = new Headers(init?.headers,);
            if (headers.get('authorization',) !== HEADER_ONLY_BEARER || headers.has('cookie',))
              throw new Error('Unexpected quota authorization',);
            const nextRegenAt = new Date(Date.now() + 60_000,).toISOString();
            return Response.json({ planWeeklyLimit: { percentRemaining: 0, nextRegenAt, }, },);
          },
        },);
        const model = poolModel({ provider, id: 'reviewer', },);
        const { result, } = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(result.errorMessage,).toContain('no eligible reviewer member until',);
      },
    },),
    it({
      name: 'a canceled waiter does not wait for or abort another request’s quota probe',
      fn: async function abortWaiter() {
        const reading = Promise.withResolvers<Response>();
        const ready = Promise.withResolvers<void>();
        const cancel = new AbortController();
        const { provider, calls, options, } = await fixture({
          extension,
          models: SYNTHETIC_ONLY,
          play({ model, }) { return immediate({ model, },); },
          async fetchQuota() {
            ready.resolve();
            return await reading.promise;
          },
        },);
        const model = poolModel({ provider, id: 'reviewer', },);
        const first = collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        await ready.promise;
        const second = collect(provider.streamSimple(model, TRANSCRIPT, { ...options, signal: cancel.signal, },),);
        await setImmediate();
        cancel.abort();
        const early = await Promise.race([second, setImmediate(),],);
        reading.resolve(Response.json({}),);
        const [answer, canceled,] = await Promise.all([first, second,],);
        expect(early,).not.toBeUndefined();
        expect(canceled.result.stopReason,).toBe('aborted',);
        expect(answer.result.provider,).toBe('synthetic',);
        expect(calls.length,).toBe(1,);
      },
    },),
  ],
},);
