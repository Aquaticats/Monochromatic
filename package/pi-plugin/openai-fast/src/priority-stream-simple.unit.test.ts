/**
 Built simple stream conversion preserves native clamping and deferred authentication. @module
 */
import type { SimpleStreamOptions, } from '@earendil-works/pi-ai';
import { buildBaseOptions, } from '@earendil-works/pi-ai/api/simple-options';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  PriorityRequestError,
  streamSimplePriority,
} from '../dist/final/node/index.mjs';
import { createStreamFixtureDispatch, } from './stream-fixture-dispatch.ts';
import { stopStreamFixtureTransport, } from './stream-fixture-http.ts';
import { createStreamFixtureOptions, } from './stream-fixture-options.ts';
import { STREAM_FIXTURE_REASONING_CASES, } from './stream-fixture-reasoning.ts';
import {
  createStreamFixtureContext,
  createStreamFixtureModel,
} from './stream-fixture.ts';

await describe({
  name: '',
  children: [
    describe({
      name: streamSimplePriority.name,
      children: [
        //region Native reasoning: no fast-model capability allowlist.

        ...STREAM_FIXTURE_REASONING_CASES.map(function reasoningCase(testCase) {
          return it({
            name: testCase.name,
            fn: async function clampReasoning() {
              /**
               Live model capability variations determine native clamping.
               */
              const model = createStreamFixtureModel(testCase.modelOverrides,);
              /**
               Installed normalizer supplies native option preparation input.
               */
              const context = createStreamFixtureContext();
              /**
               Full native dispatch captures the final simple conversion.
               */
              const dispatch = createStreamFixtureDispatch();
              expect(streamSimplePriority({
                model,
                context,
                stream: dispatch.stream,
                options: {
                  apiKey: 'injected-stream-fixture-key',
                  ...(testCase.reasoning === undefined
                    ? {}
                    : { reasoning: testCase.reasoning as NonNullable<SimpleStreamOptions['reasoning']>, }),
                  toolChoice: 'none',
                },
              },),).toBe(dispatch.events,);
              expect(dispatch.calls,).toHaveLength(1,);
              /**
               Captured options prove full dispatch receives native effort and tool choice.
               */
              const [call,] = dispatch.calls;
              if (call?.options?.onPayload === undefined)
                throw new Error('Expected injected full stream call with composed callback',);
              expect(call.model,).toBe(model,);
              expect(call.context,).toBe(context,);
              expect(call.options?.reasoningEffort,).toBe(testCase.effort,);
              expect(call.options?.serviceTier,).toBe('priority',);
              expect(call.options?.toolChoice,).toBe('none',);
              expect(await call.options.onPayload({ native: true, }, model,),).toEqual({
                native: true,
                service_tier: 'priority',
              },);
            },
          },);
        },),
        //endregion Native reasoning

        //region Neutral capabilities: mirror the installed base helper exactly.

        it({
          name: 'preserves every native base option, instrumentation reference, and token clamp',
          fn: async function baseOptions(ctx) {
            /**
             Context window forces the native safety margin to cap output at one token.
             */
            const model = createStreamFixtureModel({ contextWindow: 512, },);
            /**
             Transcript leaves less than the native safety margin available.
             */
            const context = createStreamFixtureContext();
            /**
             Customization identity and asynchronous semantics remain caller-owned.
             */
            const onPayload = ctx.sinon.spy(async function keepNativePayload() {
              return await Promise.resolve(undefined,);
            },);
            /**
             Complete simple options exercise each neutral forwarding capability.
             */
            const options = createStreamFixtureOptions({ onPayload, },);
            /**
             Injected full dispatcher captures native conversion output.
             */
            const dispatch = createStreamFixtureDispatch();
            streamSimplePriority({ model, context, options, stream: dispatch.stream, },);
            /**
             Installed helper defines preservation semantics rather than a duplicate implementation.
             */
            const expected = buildBaseOptions(model, context, options, options.apiKey,);
            /**
             Actual native options must retain caller callback composition.
             */
            const [call,] = dispatch.calls;
            if (call?.options?.onPayload === undefined)
              throw new Error('Expected captured native simple options',);
            /**
             Every base field except intentionally composed payload callback is identity-preserved.
             */
            const keys = Object.keys(expected,) as readonly (keyof typeof expected)[];
            /**
             Native option key selects the corresponding preserved field.
             */
            for (const key of keys) {
              if (key !== 'onPayload')
                expect(call.options[key],).toBe(expected[key],);
            }
            expect(call.options.maxTokens,).toBe(1,);
            expect(call.options.reasoningEffort,).toBe('high',);
            expect(call.options.toolChoice,).toBe('none',);
            expect(call.options.serviceTier,).toBe('priority',);
            expect(await call.options.onPayload({ native: true, }, model,),).toEqual({
              native: true,
              service_tier: 'priority',
            },);
            expect(onPayload,).toHaveBeenCalledTimes(1,);
            expect(options.maxTokens,).toBe(8_192,);
          },
        },),
        //endregion Neutral capabilities

        //region Authentication: injected registry owns auth, direct native dispatch requires it.

        ...([undefined, '',] as const).map(function injectedAuthCase(apiKey) {
          return it({
            name: `delegates ${String(apiKey,)} authentication to injected full stream`,
            fn: async function deferredAuthentication() {
              /**
               Original model reaches the adapter that owns original-provider authentication.
               */
              const model = createStreamFixtureModel();
              /**
               Keyless request still carries original normalized transcript.
               */
              const context = createStreamFixtureContext();
              /**
               Independent adapter stands in for original registry auth resolution.
               */
              const dispatch = createStreamFixtureDispatch();
              expect(streamSimplePriority({
                model,
                context,
                stream: dispatch.stream,
                ...(apiKey === undefined ? {} : { options: { apiKey, }, }),
              },),).toBe(dispatch.events,);
              expect(dispatch.calls,).toHaveLength(1,);
              /**
               Native base options preserve absent auth for later registry resolution.
               */
              const [call,] = dispatch.calls;
              if (call?.options?.onPayload === undefined)
                throw new Error('Expected keyless request to reach injected dispatcher with composed callback',);
              expect(call.model,).toBe(model,);
              expect(call.context,).toBe(context,);
              expect(call.options?.apiKey,).toBe(apiKey,);
              expect(call.options?.serviceTier,).toBe('priority',);
              expect(await call.options.onPayload({ native: true, }, model,),).toEqual({
                native: true,
                service_tier: 'priority',
              },);
            },
          },);
        },),
        ...([undefined, '',] as const).map(function nativeAuthCase(apiKey) {
          return it({
            name: `rejects ${String(apiKey,)} authentication before default native dispatch`,
            fn: async function directAuthentication(ctx) {
              ctx.expect.assertions(3,);
              /**
               Fail-closed local transport prevents networking even if the native auth guard regresses.
               */
              const fetch = ctx.sinon.spy(stopStreamFixtureTransport,);
              await Promise.resolve();
              try {
                streamSimplePriority({
                  model: createStreamFixtureModel(),
                  context: createStreamFixtureContext(),
                  options: {
                    transport: 'sse',
                    fetch,
                    ...(apiKey === undefined ? {} : { apiKey, }),
                  },
                },);
              }
              catch (error) {
                ctx.expect(error,).toBeInstanceOf(PriorityRequestError,);
                if (!(error instanceof PriorityRequestError))
                  throw error;
                ctx.expect(error.message,).toContain('No API key for provider: openai-codex',);
              }
              ctx.expect(fetch,).not.toHaveBeenCalled();
            },
          },);
        },),
        //endregion Authentication
      ],
    },),
  ],
},);
