/**
 Built payload composition preserves caller replacement and mutation semantics. @module
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
import { forcePriorityPayload, } from '../dist/final/node/index.mjs';
import { createStreamFixtureModel, } from './stream-fixture.ts';

//region Callback composition: caller mutations and replacements precede priority enforcement.

await describe({
  name: '',
  children: [
    describe({
      name: forcePriorityPayload.name,
      children: [
        it({
          name: 'copies frozen native record without changing its fields or nested identities',
          fn: async function frozenNativeRecord() {
            /**
             Nested caller data is retained without deep cloning.
             */
            const nested = { fixture: true, };
            /**
             Frozen native request cannot be mutated by priority enforcement.
             */
            const payload = Object.freeze({
              model: 'base-model',
              service_tier: 'auto',
              nested,
            },);
            /**
             Enforced result retains all native fields but overrides the requested tier.
             */
            const result = await forcePriorityPayload({ payload, model: createStreamFixtureModel(), },);
            expect(result,).toEqual({
              model: 'base-model',
              service_tier: 'priority',
              nested,
            },);
            expect(result,).not.toBe(payload,);
            /**
             Destructuring reads an index-signature field without conflicting dot-notation rules.
             */
            const { nested: resultNested, } = result;
            expect(resultNested,).toBe(nested,);
            expect(payload.service_tier,).toBe('auto',);
            /**
             Serialization must preserve the enforced tier at the consumer wire boundary.
             */
            const serialized = JSON.stringify(result,);
            expect(JSON.parse(serialized,),).toHaveProperty('service_tier', 'priority',);
          },
        },),
        it({
          name: 'awaits async undefined callback and preserves its original-record mutations',
          fn: async function nativeMutation(ctx) {
            /**
             Exact live model must reach the caller callback unchanged.
             */
            const model = createStreamFixtureModel();
            /**
             Test-owned structural shape models the fields this caller intentionally edits.
             */
            const payload: {
              model: string;
              service_tier: string;
              caller_edit?: string;
            } = {
              model: model.id,
              service_tier: 'auto',
            };
            /**
             Spy verifies original identity, asynchronous completion, and single invocation.
             */
            const onPayload = ctx.sinon.spy(async function editNativePayload(
              received: unknown,
              receivedModel: ForeignBorrowed<Model<Api>>,
            ) {
              expect(received,).toBe(payload,);
              expect(receivedModel,).toBe(model,);
              await Promise.resolve();
              payload.caller_edit = 'retained';
              payload.service_tier = 'default';
              return undefined;
            },);
            /**
             Priority is applied after asynchronous customization settles.
             */
            const result = await forcePriorityPayload({ payload, model, onPayload, },);
            expect(onPayload,).toHaveBeenCalledExactlyOnceWith(payload, model,);
            expect(result,).toEqual({
              model: model.id,
              caller_edit: 'retained',
              service_tier: 'priority',
            },);
            expect(payload.service_tier,).toBe('default',);
          },
        },),
        it({
          name: 'awaits async replacement without merging or mutating the discarded original',
          fn: async function replacementSemantics() {
            /**
             Original-only fields must not survive native replacement semantics.
             */
            const payload = Object.freeze({ original_only: true, },);
            /**
             Caller replacement may be frozen and request a different service tier.
             */
            const replacement = Object.freeze({
              replacement_only: true,
              service_tier: 'flex',
            },);
            /**
             Selected replacement receives priority after the callback resolves.
             */
            const result = await forcePriorityPayload({
              payload,
              model: createStreamFixtureModel(),
              onPayload: async function replacePayload() {
                return await Promise.resolve(replacement,);
              },
            },);
            expect(result,).toEqual({
              replacement_only: true,
              service_tier: 'priority',
            },);
            expect(result,).not.toBe(replacement,);
            expect(result,).not.toHaveProperty('original_only',);
            expect(replacement.service_tier,).toBe('flex',);
            expect(payload,).toEqual({ original_only: true, },);
          },
        },),
        it({
          name: 'allows caller replacement to repair an invalid original before record validation',
          fn: async function repairOriginal() {
            /**
             Callback composition happens before validation of the selected record.
             */
            const result = await forcePriorityPayload({
              payload: null,
              model: createStreamFixtureModel(),
              onPayload: function repairedPayload() {
                return { repaired: true, };
              },
            },);
            expect(result,).toEqual({
              repaired: true,
              service_tier: 'priority',
            },);
          },
        },),
        ...(['auto', 'default', 'flex', 'priority', undefined,] as const).map(function tierCase(tier) {
          return it({
            name: `forces priority after replacement requesting ${String(tier,)}`,
            fn: async function forceReplacementTier() {
              /**
               Caller tier is overwritten regardless of its requested value.
               */
              const result = await forcePriorityPayload({
                payload: { native: true, },
                model: createStreamFixtureModel(),
                onPayload: function replaceTier() {
                  return {
                    service_tier: tier,
                    caller: true,
                  };
                },
              },);
              expect(result,).toEqual({
                service_tier: 'priority',
                caller: true,
              },);
            },
          },);
        },),
      ],
    },),
  ],
},);

//endregion Callback composition
