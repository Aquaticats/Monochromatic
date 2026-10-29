/**
 Built full stream delegation preserves native options and caller capabilities. @module
 */
import type { OpenAICodexResponsesOptions, } from '@earendil-works/pi-ai';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { streamPriority, } from '../dist/final/node/index.mjs';
import { createStreamFixtureDispatch, } from './stream-fixture-dispatch.ts';
import { createStreamFixtureFullOptions, } from './stream-fixture-options.ts';
import {
  createStreamFixtureContext,
  createStreamFixtureModel,
} from './stream-fixture.ts';

//region Full options: optional registry dispatch sees all native caller capabilities.

await describe({
  name: '',
  children: [
    describe({
      name: streamPriority.name,
      children: [
        it({
          name: 'delegates original model, context, and every option except composed priority fields',
          fn: async function fullDelegation(ctx) {
            /**
             Model-specific headers belong to the original model, not a routing ID.
             */
            const model = createStreamFixtureModel({ headers: { 'x-stream-fixture-model': 'original', }, },);
            /**
             Original transcript identity must pass through unchanged.
             */
            const context = createStreamFixtureContext();
            /**
             Frozen replacement proves enforcement never mutates caller data.
             */
            const replacement = Object.freeze({
              fixture_replacement: true,
              service_tier: 'default',
            },);
            /**
             Spy records original native payload and model callback identities.
             */
            const onPayload = ctx.sinon.spy(async function replacePayload() {
              return await Promise.resolve(replacement,);
            },);
            /**
             Full options exercise transport, auth, telemetry, callbacks, and native intent.
             */
            const options = Object.freeze(createStreamFixtureFullOptions({ onPayload, },),);
            /**
             Local dispatcher records native options without consuming transport.
             */
            const dispatch = createStreamFixtureDispatch();
            /**
             Built wrapper must return the injected stream without wrapping it.
             */
            const returned = streamPriority({
              model,
              context,
              options,
              stream: dispatch.stream,
            },);
            expect(returned,).toBe(dispatch.events,);
            expect(dispatch.calls,).toHaveLength(1,);
            /**
             Captured call exposes complete options with original input identity.
             */
            const [call,] = dispatch.calls;
            if (call?.options?.onPayload === undefined)
              throw new Error('Expected composed native payload callback',);
            expect(call.model,).toBe(model,);
            expect(call.context,).toBe(context,);
            expect(call.options.serviceTier,).toBe('priority',);
            /**
             Enumerated original fields catch any silently dropped native capability.
             */
            const keys = Object.keys(options,) as readonly (keyof OpenAICodexResponsesOptions)[];
            /**
             Native option key selects identity comparison, excluding intentional compositions.
             */
            for (const key of keys) {
              if ((key !== 'serviceTier') && (key !== 'onPayload'))
                expect(call.options[key],).toBe(options[key],);
            }
            /**
             Caller receives the exact native object before choosing its replacement.
             */
            const original = { fixture_native: true, };
            /**
             Final body receives priority only after asynchronous replacement settles.
             */
            const payload = await call.options.onPayload(original, model,);
            expect(onPayload,).toHaveBeenCalledExactlyOnceWith(original, model,);
            expect(payload,).toEqual({
              fixture_replacement: true,
              service_tier: 'priority',
            },);
            expect(replacement.service_tier,).toBe('default',);
            expect(options.serviceTier,).toBe('flex',);
            expect(options.onPayload,).toBe(onPayload,);
          },
        },),
        it({
          name: 'snapshots caller payload callback before deferred registry dispatch',
          fn: async function snapshotCallback(ctx) {
            /**
             Initial customization belongs to this request at composition time.
             */
            const originalCallback = ctx.sinon.spy(async function originalPayload() {
              return await Promise.resolve({ original_callback: true, },);
            },);
            /**
             Reused mutable options must not replace the deferred request callback.
             */
            const laterCallback = ctx.sinon.spy(async function laterPayload() {
              return await Promise.resolve({ later_callback: true, },);
            },);
            /**
             Mutable host options simulate reuse after initial wrapper composition.
             */
            const options: OpenAICodexResponsesOptions = { onPayload: originalCallback, };
            /**
             Live model is retained across deferred callback invocation.
             */
            const model = createStreamFixtureModel();
            /**
             Dispatcher retains the composed callback without invoking it immediately.
             */
            const dispatch = createStreamFixtureDispatch();
            streamPriority({
              model,
              context: createStreamFixtureContext(),
              options,
              stream: dispatch.stream,
            },);
            options.onPayload = laterCallback;
            /**
             Deferred native callback must retain the initially composed caller function.
             */
            const [call,] = dispatch.calls;
            if (call?.options?.onPayload === undefined)
              throw new Error('Expected deferred composed payload callback',);
            expect(await call.options.onPayload({ native: true, }, model,),).toEqual({
              original_callback: true,
              service_tier: 'priority',
            },);
            expect(originalCallback,).toHaveBeenCalledTimes(1,);
            expect(laterCallback,).not.toHaveBeenCalled();
          },
        },),
        it({
          name: 'composes priority when native caller options are absent',
          fn: async function absentOptions() {
            /**
             Unconfigured full stream may delegate to an injected native implementation.
             */
            const model = createStreamFixtureModel();
            /**
             Original transcript remains branded by the installed normalizer.
             */
            const context = createStreamFixtureContext();
            /**
             Standalone dispatcher records options without touching transport or credentials.
             */
            const dispatch = createStreamFixtureDispatch();
            expect(streamPriority({ model, context, stream: dispatch.stream, },),).toBe(dispatch.events,);
            expect(dispatch.calls,).toHaveLength(1,);
            /**
             Even absent customization receives a native-compatible priority callback.
             */
            const [call,] = dispatch.calls;
            if (call?.options?.onPayload === undefined)
              throw new Error('Expected composed payload callback without options',);
            expect(call.model,).toBe(model,);
            expect(call.context,).toBe(context,);
            expect(call.options.serviceTier,).toBe('priority',);
            expect(await call.options.onPayload({ native: true, }, model,),).toEqual({
              native: true,
              service_tier: 'priority',
            },);
          },
        },),
      ],
    },),
  ],
},);

//endregion Full options
