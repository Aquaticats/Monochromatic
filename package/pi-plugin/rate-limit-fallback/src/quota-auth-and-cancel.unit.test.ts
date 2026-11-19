/**
 Quota requests resolve credentials and cancellation independently from chat.
 @module
 */

import { isRetryableAssistantError, } from '@earendil-works/pi-ai';
import { setImmediate, } from 'node:timers/promises';
import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import extension from '../dist/final/node/index.mjs';
import { collect, fixture, immediate, PHYSICAL_MODELS, poolModel, TRANSCRIPT, } from './stand-in-pi-fixture.ts';
import { syntheticWeeklyWindow, } from './synthetic-quota-fixture.ts';

/** The only fixture route requiring a quota request. */
const SYNTHETIC_ONLY = PHYSICAL_MODELS.filter(function isSynthetic(model,) {
  return model.provider === 'synthetic';
},);

/** Bearer supplied only through the registry's resolved headers. */
const HEADER_ONLY_BEARER = 'Bearer stand-in-header-only';

/** Model request budget for the supplied-deadline case; independent of the probe deadline. */
const MODEL_TIMEOUT_MS = 1_000;

/**
 A caller timeout the plugin cannot use, so its own quota deadline has to apply.
 */
const NO_TIMEOUT_MS = 0;

/** Reset used when a fixture reports an exhausted Synthetic window. */
const RESET_MS = 60_000;

await describe({
  name: 'quota authentication and cancellation',
  concurrency: 1,
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
            if ((headers.get('authorization',) !== HEADER_ONLY_BEARER) || headers.has('cookie',))
              throw new Error('Unexpected quota authorization',);
            const nextRegenAt = new Date(Date.now() + RESET_MS,).toISOString();
            return syntheticWeeklyWindow({ percentRemaining: 0, nextRegenAt, },);
          },
        },);
        const model = poolModel({ provider, id: 'reviewer', },);
        const { result, } = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(result.errorMessage,).toContain('no eligible reviewer member',);
        expect(result.errorMessage,).toContain('synthetic/hf:deepseek-ai/DeepSeek-V4.1-Flash-1017 is held until',);
        expect(isRetryableAssistantError(result,),).toBe(false,);
      },
    },),
    ...[undefined, NO_TIMEOUT_MS, MODEL_TIMEOUT_MS,].map(function callerDeadline(timeoutMs,) {
      return it({
        name: `bounds a stalled quota read ${((timeoutMs === undefined) || (timeoutMs <= 0)) ? 'without a usable' : 'before the'} model timeout`,
        fn: async function probeDeadline(ctx,) {
          const deadline = new AbortController();
          const timeout = ctx.sinon.stub(AbortSignal, 'timeout',).returns(deadline.signal,);
          const entered = Promise.withResolvers<{ readonly signal?: AbortSignal; }>();
          const reading = Promise.withResolvers<Response>();
          const { provider, calls, options, } = await fixture({
            extension,
            models: SYNTHETIC_ONLY,
            play({ model, }) { return immediate({ model, },); },
            async fetchQuota(_url, init,) {
              const signal = init?.signal;
              entered.resolve((signal === null) || (signal === undefined) ? {} : { signal, },);
              return await reading.promise;
            },
          },);
          const model = poolModel({ provider, id: 'reviewer', },);
          const result = collect(provider.streamSimple(model, TRANSCRIPT, {
            ...options,
            ...(timeoutMs === undefined ? {} : { timeoutMs, }),
          },),);
          const { signal, } = await entered.promise;
          const budget = timeout.firstCall?.args[0];
          deadline.abort();
          const beforeTransportSettles = await Promise.race([result, setImmediate(),],);
          reading.resolve(Response.json({},),);
          await result;
          expect(signal,).toBe(deadline.signal,);
          expect(budget,).toBeGreaterThan(0,);
          if ((timeoutMs !== undefined) && (timeoutMs > 0))
            expect(budget,).toBeLessThan(timeoutMs,);
          expect(beforeTransportSettles,).not.toBeUndefined();
          expect(calls[0]?.options?.timeoutMs,).toBe(timeoutMs,);
        },
      },);
    },),
    it({
      name: 'a shorter wait deadline does not cancel a shared quota read or discard its evidence',
      fn: async function sharedQuotaDeadline(ctx,) {
        const reader = new AbortController();
        const waiter = new AbortController();
        const timeout = ctx.sinon.stub(AbortSignal, 'timeout',).returns(waiter.signal,);
        timeout.onFirstCall().returns(reader.signal,);
        const started = Promise.withResolvers<void>();
        const reading = Promise.withResolvers<Response>();
        const { provider, calls, options, } = await fixture({
          extension,
          models: SYNTHETIC_ONLY,
          play({ model, }) { return immediate({ model, },); },
          async fetchQuota() {
            started.resolve();
            return await reading.promise;
          },
        },);
        const model = poolModel({ provider, id: 'reviewer', },);
        const first = collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        await started.promise;
        const second = collect(provider.streamSimple(model, TRANSCRIPT, { ...options, timeoutMs: MODEL_TIMEOUT_MS, },),);
        await setImmediate();
        waiter.abort();
        const early = await Promise.race([second, setImmediate(),],);
        reading.resolve(syntheticWeeklyWindow({
          percentRemaining: 0,
          nextRegenAt: new Date(Date.now() + RESET_MS,).toISOString(),
        },),);
        const [held, answered,] = await Promise.all([first, second,],);
        expect(early,).not.toBeUndefined();
        expect(answered.result.stopReason,).toBe('stop',);
        expect(held.result.errorMessage,).toContain('is held until',);
        expect(reader.signal.aborted,).toBe(false,);
        expect(calls.length,).toBe(1,);
      },
    },),
    it({
      name: 'admits the member when the quota transport throws a non-Error value',
      fn: async function nonErrorTransport() {
        const { provider, options, } = await fixture({
          extension,
          models: SYNTHETIC_ONLY,
          play({ model, }) { return immediate({ model, },); },
          async fetchQuota() {
            // A transport may reject with something that is not an Error; the catch must survive it.
            // oxlint-disable-next-line no-throw-literal, typescript/only-throw-error -- testing non-Error throw
            throw 'stand-in transport failure';
          },
        },);
        const model = poolModel({ provider, id: 'reviewer', },);
        const { result, } = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(result.stopReason,).toBe('stop',);
        expect(result.provider,).toBe('synthetic',);
      },
    },),
    it({
      name: 'admits the member when the registry resolver throws a non-Error value',
      fn: async function nonErrorResolver() {
        const { provider, options, } = await fixture({
          extension,
          models: SYNTHETIC_ONLY,
          async resolveAuth() {
            // A resolver may fail with something that is not an Error; the catch must survive it.
            // oxlint-disable-next-line no-throw-literal, typescript/only-throw-error -- testing non-Error throw
            throw { name: 'stand-in resolver failure', };
          },
          play({ model, }) { return immediate({ model, },); },
        },);
        const model = poolModel({ provider, id: 'reviewer', },);
        const { result, } = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(result.stopReason,).toBe('stop',);
        expect(result.provider,).toBe('synthetic',);
      },
    },),
    ...[
      {
        reason: 'non-Bearer header',
        resolveAuth() {
          return Promise.resolve({ ok: true as const, headers: { Authorization: 'Basic stand-in', }, },);
        },
      },
      {
        reason: 'resolver exception',
        resolveAuth() {
          return Promise.reject(new Error('Stand-in registry resolution failure',),);
        },
      },
      {
        reason: 'registry refusal',
        resolveAuth() {
          return Promise.resolve({ ok: false as const, error: 'Stand-in registry refusal', },);
        },
      },
    ].map(function credentialFailure(slot,) {
      return it({
        name: `admits the member without sending a quota request after a ${slot.reason}`,
        fn: async function unavailableAuth() {
          const observations = { resolves: 0, fetches: 0, };
          const { provider, options, } = await fixture({
            extension,
            models: SYNTHETIC_ONLY,
            resolveAuth() {
              observations.resolves += 1;
              return slot.resolveAuth();
            },
            play({ model, }) { return immediate({ model, },); },
            async fetchQuota() {
              observations.fetches += 1;
              return Response.json({});
            },
          },);
          const model = poolModel({ provider, id: 'reviewer', },);
          const { result, } = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
          expect(result.stopReason,).toBe('stop',);
          expect(observations,).toEqual({ resolves: 1, fetches: 0, },);
        },
      },);
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
