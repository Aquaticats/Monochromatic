/**
 Built payload validation preserves priority through prototype and JSON serialization boundaries. @module
 */
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  forcePriorityPayload,
  PriorityRequestError,
} from '../dist/final/node/index.mjs';
import { createStreamFixtureModel, } from './stream-fixture.ts';

await describe({
  name: '',
  children: [
    describe({
      name: forcePriorityPayload.name,
      children: [
        //region JSON hooks: callable root serialization cannot erase enforced priority.

        ...(['native', 'replacement',] as const).map(function serializationCase(mode) {
          return it({
            name: `rejects ${mode} root toJSON that would erase priority during serialization`,
            fn: async function rejectSerializer(ctx) {
              ctx.expect.assertions(2,);
              /**
               Root serializer would override priority if allowed to reach native JSON encoding.
               */
              const record = {
                toJSON: function lowerSerializedTier() {
                  return { service_tier: 'default', };
                },
              };
              try {
                await forcePriorityPayload({
                  payload: mode === 'native' ? record : {},
                  model: createStreamFixtureModel(),
                  ...(mode === 'native'
                    ? {}
                    : {
                      onPayload: function replaceWithSerializer() {
                        return record;
                      },
                    }),
                },);
              }
              catch (error) {
                ctx.expect(error,).toBeInstanceOf(PriorityRequestError,);
                if (!(error instanceof PriorityRequestError))
                  throw error;
                ctx.expect(error.message,).toContain('callable toJSON',);
              }
            },
          },);
        },),
        it({
          name: 'permits inert toJSON data without interfering with priority serialization',
          fn: async function inertSerializerData() {
            /**
             Non-callable JSON field cannot override object serialization.
             */
            const result = await forcePriorityPayload({
              payload: { toJSON: 'ordinary data', },
              model: createStreamFixtureModel(),
            },);
            /**
             Actual serialization is the security boundary, not a structured clone.
             */
            const serialized = JSON.stringify(result,);
            expect(JSON.parse(serialized,),).toEqual({
              toJSON: 'ordinary data',
              service_tier: 'priority',
            },);
          },
        },),
        it({
          name: 'reads caller getters only once while rejecting callable root serialization',
          fn: async function serializerGetter(ctx) {
            ctx.expect.assertions(3,);
            /**
             Owned counter records getter evaluation without function-root mutable bindings.
             */
            const state = { reads: 0, };
            try {
              await forcePriorityPayload({
                payload: {
                  get toJSON() {
                    state.reads += 1;
                    return function lowerSerializedTier() {
                      return { service_tier: 'default', };
                    };
                  },
                },
                model: createStreamFixtureModel(),
              },);
            }
            catch (error) {
              ctx.expect(error,).toBeInstanceOf(PriorityRequestError,);
              if (!(error instanceof PriorityRequestError))
                throw error;
              ctx.expect(error.message,).toContain('callable toJSON',);
            }
            ctx.expect(state.reads,).toBe(1,);
          },
        },),
        //endregion JSON hooks

        //region Prototype safety: data keys cannot alter object prototypes.

        it({
          name: 'accepts null-prototype replacements without losing caller data',
          fn: async function nullPrototypeRecord() {
            /**
             Project-owned structural shape avoids index-signature dot-notation conflicts.
             */
            const replacement = Object.create(null,) as {
              fixture: string;
              service_tier: string;
            };
            replacement.fixture = 'retained';
            replacement.service_tier = 'default';
            Object.freeze(replacement,);
            /**
             Native replacement retains enumerable fields while enforcing priority.
             */
            const result = await forcePriorityPayload({
              payload: {},
              model: createStreamFixtureModel(),
              onPayload: function replaceNullPrototype() {
                return replacement;
              },
            },);
            expect(result,).toEqual({
              fixture: 'retained',
              service_tier: 'priority',
            },);
            expect(replacement.service_tier,).toBe('default',);
            expect(Object.getPrototypeOf(replacement,),).toBeNull();
          },
        },),
        it({
          name: 'copies adversarial prototype-named JSON fields as inert own data',
          fn: async function prototypeDataKeys() {
            /**
             JSON parsing creates an own prototype-named key without mutating shared prototypes.
             */
            const payload: unknown = JSON.parse(
              '{"__proto__":{"fixture_pollution":"inert"},"constructor":{"prototype":{"fixture_pollution":"inert"}},"service_tier":"default"}',
            );
            /**
             Copying native record must preserve the original safe prototype and priority.
             */
            const result = await forcePriorityPayload({ payload, model: createStreamFixtureModel(), },);
            expect(Object.getPrototypeOf(result,),).toBe(Object.prototype,);
            expect(Object.hasOwn(result, '__proto__',),).toBe(true,);
            expect(result,).not.toHaveProperty('fixture_pollution',);
            /**
             Wire encoding must retain inert caller data without changing the enforced tier.
             */
            const serialized = JSON.stringify(result,);
            expect(JSON.parse(serialized,),).toEqual({
              ...JSON.parse(
                '{"__proto__":{"fixture_pollution":"inert"},"constructor":{"prototype":{"fixture_pollution":"inert"}}}',
              ),
              service_tier: 'priority',
            },);
          },
        },),
        //endregion Prototype safety
      ],
    },),
  ],
},);
