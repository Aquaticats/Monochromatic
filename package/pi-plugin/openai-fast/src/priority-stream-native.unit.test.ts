/**
 Built wrappers exercise real native request preparation but stop before networking. @module
 */
import type {
  Api,
  Model,
} from '@earendil-works/pi-ai';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import {
  streamPriority,
  streamSimplePriority,
} from '../dist/final/node/index.mjs';
import { stopStreamFixtureTransport, } from './stream-fixture-http.ts';
import {
  createStreamFixtureContext,
  createStreamFixtureModel,
  STREAM_FIXTURE_TOKEN,
  StreamFixtureStopError,
} from './stream-fixture.ts';

//region Native boundary: real payload construction precedes caller customization and transport.

await describe({
  name: '',
  children: [
    describe({
      name: 'native Codex request boundary',
      children: [
        ...([streamPriority, streamSimplePriority,] as const).map(function stopCase(entryPoint) {
          return it({
            name: `${entryPoint.name} builds priority native payload before callback stops transport`,
            fn: async function stopBeforeNetwork(ctx) {
              /**
               Unknown original ID verifies no compatibility allowlist restricts dispatch.
               */
              const model = createStreamFixtureModel({ id: 'future-fixture-codex-model', },);
              /**
               Native builder converts normalized system instructions and user input.
               */
              const context = createStreamFixtureContext();
              /**
               Per-request spy detects forbidden networking after customization rejects.
               */
              const fetch = ctx.sinon.spy(stopStreamFixtureTransport,);
              /**
               Explicit stop preserves recognizable caller failure identity and message.
               */
              const stop = new StreamFixtureStopError();
              /**
               Native callback receives original model and already requested priority tier.
               */
              const onPayload = ctx.sinon.spy(function stopPayload(
                payload: unknown,
                receivedModel: ForeignBorrowed<Model<Api>>,
              ) {
                expect(receivedModel,).toBe(model,);
                expect(payload,).toMatchObject({
                  model: model.id,
                  service_tier: 'priority',
                  stream: true,
                  store: false,
                  instructions: 'Stream fixture instructions.',
                  tool_choice: 'none',
                },);
                throw stop;
              },);
              /**
               Default dependency is the installed full native Codex stream, not a mock wrapper.
               */
              const result = await entryPoint({
                model,
                context,
                options: {
                  apiKey: STREAM_FIXTURE_TOKEN,
                  transport: 'sse',
                  fetch,
                  onPayload,
                  toolChoice: 'none',
                },
              },).result();
              expect(onPayload,).toHaveBeenCalledTimes(1,);
              expect(fetch,).not.toHaveBeenCalled();
              expect(result.stopReason,).toBe('error',);
              expect(result.errorMessage,).toContain(stop.message,);
              expect(result.model,).toBe(model.id,);
            },
          },);
        },),
        ...([streamPriority, streamSimplePriority,] as const).map(function invalidCase(entryPoint) {
          return it({
            name: `${entryPoint.name} rejects invalid caller replacement before native networking`,
            fn: async function rejectBeforeNetwork(ctx) {
              /**
               Synthetic valid authentication reaches payload validation, never external HTTP.
               */
              const model = createStreamFixtureModel();
              /**
               Transport spy detects any attempted fallback following invalid customization.
               */
              const fetch = ctx.sinon.spy(stopStreamFixtureTransport,);
              /**
               Explicit null is a replacement, not native undefined keep semantics.
               */
              const onPayload = ctx.sinon.spy(async function nullReplacement() {
                return await Promise.resolve(null,);
              },);
              /**
               Native streaming reports preparation error using its standard error result.
               */
              const result = await entryPoint({
                model,
                context: createStreamFixtureContext(),
                options: {
                  apiKey: STREAM_FIXTURE_TOKEN,
                  transport: 'sse',
                  fetch,
                  onPayload,
                },
              },).result();
              expect(result.stopReason,).toBe('error',);
              expect(result.errorMessage,).toContain('Return an object from onPayload',);
              expect(onPayload,).toHaveBeenCalledTimes(1,);
              expect(fetch,).not.toHaveBeenCalled();
            },
          },);
        },),
      ],
    },),
  ],
},);

//endregion Native boundary
