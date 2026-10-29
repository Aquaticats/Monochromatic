/**
 Built payload rejections preserve exact caller failures and forbid best-effort fallback. @module
 */
import {
  describe,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  forcePriorityPayload,
  PriorityRequestError,
} from '../dist/final/node/index.mjs';
import { PAYLOAD_FIXTURE_INVALID_REPLACEMENTS, } from './payload-fixture-invalid.ts';
import { createStreamFixtureModel, } from './stream-fixture.ts';

//region Rejections: invalid values never become ordinary-tier requests.

await describe({
  name: '',
  children: [
    describe({
      name: forcePriorityPayload.name,
      children: [
        ...PAYLOAD_FIXTURE_INVALID_REPLACEMENTS.map(function invalidCase(invalid) {
          return it({
            name: `rejects async ${invalid.name} replacement with custom diagnostic`,
            fn: async function invalidReplacement(ctx) {
              ctx.expect.assertions(2,);
              try {
                await forcePriorityPayload({
                  payload: { valid_native: true, },
                  model: createStreamFixtureModel(),
                  onPayload: async function invalidPayload() {
                    return await Promise.resolve(invalid.value,);
                  },
                },);
              }
              catch (error) {
                ctx.expect(error,).toBeInstanceOf(PriorityRequestError,);
                if (!(error instanceof PriorityRequestError))
                  throw error;
                ctx.expect(error.message,).toContain('Return an object from onPayload',);
              }
            },
          },);
        },),
        ...([null, 'invalid native', [] as readonly unknown[],] as const).map(function invalidNativeCase(
          payload,
          index,
        ) {
          return it({
            name: `rejects invalid native record ${String(index,)}`,
            fn: async function invalidNativeRecord(ctx) {
              ctx.expect.assertions(1,);
              try {
                await forcePriorityPayload({ payload, model: createStreamFixtureModel(), },);
              }
              catch (error) {
                ctx.expect(error,).toBeInstanceOf(PriorityRequestError,);
              }
            },
          },);
        },),
        ...(['sync', 'async',] as const).map(function rejectionCase(mode) {
          return it({
            name: `preserves exact ${mode} caller rejection without wrapping or fallback`,
            fn: async function callerRejection(ctx) {
              ctx.expect.assertions(1,);
              /**
               Original caller error identity must survive callback composition.
               */
              const failure = new Error('stream fixture caller rejection',);
              try {
                await forcePriorityPayload({
                  payload: { native: true, },
                  model: createStreamFixtureModel(),
                  onPayload: mode === 'sync'
                    ? function rejectSynchronousPayload() {
                      throw failure;
                    }
                    : async function rejectAsynchronousPayload() {
                      return await Promise.reject(failure,);
                    },
                },);
              }
              catch (error) {
                ctx.expect(error,).toBe(failure,);
              }
            },
          },);
        },),
        it({
          name: 'preserves non-Error async caller rejection',
          fn: async function nonErrorRejection(ctx) {
            ctx.expect.assertions(1,);
            /**
             Non-Error callback rejection remains owned by the caller, not normalized by the wrapper.
             */
            const failure = { fixture_rejection: true, };
            try {
              await forcePriorityPayload({
                payload: {},
                model: createStreamFixtureModel(),
                onPayload: async function rejectCallerValue() {
                  /**
                   Caller-owned deferred rejection can carry arbitrary upstream failure values.
                   */
                  const rejection = Promise.withResolvers<never>();
                  rejection.reject(failure,);
                  return await rejection.promise;
                },
              },);
            }
            catch (error) {
              ctx.expect(error,).toBe(failure,);
            }
          },
        },),
      ],
    },),
  ],
},);

//endregion Rejections
