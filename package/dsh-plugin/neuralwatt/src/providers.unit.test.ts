import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  isNeuralWattSettingsWriteDeferral,
  mapNeuralWattProviders,
  NEURALWATT_ROUTE,
  NEURALWATT_SETTINGS_NAMESPACE,
  neuralWattListingSnapshot,
  refreshNeuralWattRoute,
  type NeuralWattSettingsTarget,
  writeNeuralWattRoute,
} from '../dist/final/node/index.mjs';

//region Test fixtures

/**
 Exact refusal `@deepseek-ai/dsh-hmr` `runExclusive` throws when a settings
 write is issued while the caller's chain already runs inside a reload
 transaction (`dsh-hmr` `lib/index.js`). Quoted here rather than derived from
 the plugin's markers so marker drift fails this test.
 */
const UPSTREAM_NESTED_TRANSACTION_MESSAGE = 'HMR transactions cannot be nested';

/**
 One recorded settings merge write.
 */
type RecordedWrite = {
  ns: string;
  patch: Record<string, unknown>;
};

/**
 Recording fake of the settings seam: serves a fixed form value through
 `describe`, records every accepted merge write, and can refuse writes at the
 given zero-based call indices to reach the failure branches. Refusals throw
 `refuseWith`, so a test decides between a permanent refusal and the exact
 contention text an upstream tool throws.

 @param options - stored form value, refusing call indices, and the failure
 those refusals throw

 @returns fake target plus the writes it accepted and every call it saw

 @example
 ```ts
 const fake = fakeSettingsOf({ stored: { providers: {} } });
 ```
 */
function fakeSettingsOf({
  stored,
  failWrites = [],
  refuseWith = new Error('settings entry unavailable'),
}: {
  readonly stored?: unknown;
  readonly failWrites?: readonly number[];
  readonly refuseWith?: Error;
} = {}): {
  readonly target: NeuralWattSettingsTarget;
  readonly writes: RecordedWrite[];
  readonly attempts: RecordedWrite[];
} {
  /**
   Writes the fake accepted.
   */
  const writes: RecordedWrite[] = [];
  /**
   Every update call the fake received, including the refused ones: refusal
   indexing counts attempts, so a refused call still advances the counter.
   */
  const attempts: RecordedWrite[] = [];
  return {
    target: {
      describe: () => (stored === undefined
        ? []
        : [{ ns: NEURALWATT_SETTINGS_NAMESPACE, value: stored }]),
      update: async (ns, patch) => {
        /**
         Position of this call among every attempt, refused or accepted.
         */
        const attempt = attempts.length;
        attempts.push({ ns, patch });
        if (failWrites.includes(attempt)) throw refuseWith;
        writes.push({ ns, patch });
      },
    },
    writes,
    attempts,
  };
}

//endregion Test fixtures

await describe({
  name: '',
  children: [
    //region Route write

    describe({
      name: writeNeuralWattRoute.name,
      children: [
        it({
          name: 'writes the mapped route into the llm-pi-ai settings entry',
          fn: async () => {
            const fake = fakeSettingsOf();
            await writeNeuralWattRoute({
              settings: fake.target,
              listing: [{ id: 'seed', max_model_len: 10, },],
            },);
            expect(fake.writes.length,)
              .toBe(1,);
            expect(fake.writes[0]
              ?.ns,)
              .toBe(NEURALWATT_SETTINGS_NAMESPACE,);
            expect(fake.writes[0]
              ?.patch,)
              .toEqual({
                providers: mapNeuralWattProviders({
                  listing: [{ id: 'seed', max_model_len: 10, },],
                },),
              },);
          },
        },),
        it({
          name: 'skips the write when the stored route already holds the same content regardless of key order',
          fn: async () => {
            /**
             Route profile whose object keys are reversed, standing in for the
             key order a YAML round-trip can produce.
             */
            const reorderedProfile = Object.fromEntries(Object.entries(mapNeuralWattProviders({
              listing: [{ id: 'seed', max_model_len: 10, },],
            },)[NEURALWATT_ROUTE] ?? {},)
              .toReversed(),);
            const fake = fakeSettingsOf({
              stored: {
                providers: { [NEURALWATT_ROUTE]: reorderedProfile, },
              },
            },);
            await writeNeuralWattRoute({
              settings: fake.target,
              listing: [{ id: 'seed', max_model_len: 10, },],
            },);
            expect(fake.writes.length,)
              .toBe(0,);
          },
        },),
        it({
          name: 'writes when the stored route holds different content',
          fn: async () => {
            const fake = fakeSettingsOf({
              stored: {
                providers: mapNeuralWattProviders({ listing: [{ id: 'older', },], },),
              },
            },);
            await writeNeuralWattRoute({
              settings: fake.target,
              listing: [{ id: 'seed', },],
            },);
            expect(fake.writes.length,)
              .toBe(1,);
          },
        },),
        it({
          name: 'writes when the entry carries no stored form value',
          fn: async () => {
            const fake = fakeSettingsOf();
            await writeNeuralWattRoute({
              settings: fake.target,
              listing: [{ id: 'seed', },],
            },);
            expect(fake.writes.length,)
              .toBe(1,);
          },
        },),
        it({
          name: 'classifies the nested reload-transaction refusal as a deferral and every other refusal as not one',
          fn: async () => {
            expect(isNeuralWattSettingsWriteDeferral({
              error: new Error(UPSTREAM_NESTED_TRANSACTION_MESSAGE),
            },))
              .toBe(true,);
            expect(isNeuralWattSettingsWriteDeferral({
              error: new Error('settings entry unavailable'),
            },))
              .toBe(false,);
            expect(isNeuralWattSettingsWriteDeferral({
              error: 'not even an error object',
            },))
              .toBe(false,);
          },
        },),
      ],
    },),

    //endregion Route write

    //region Mount refresh

    describe({
      name: refreshNeuralWattRoute.name,
      children: [
        it({
          name: 'writes the checked-in snapshot first and the live listing second',
          fn: async () => {
            const fake = fakeSettingsOf();
            await refreshNeuralWattRoute({
              settings: fake.target,
              fetchListing: async () => [{ id: 'live-a', }, { id: 'live-b', },],
            },);
            expect(fake.writes.length,)
              .toBe(2,);
            expect(fake.writes[0]
              ?.patch,)
              .toEqual({
                providers: mapNeuralWattProviders({
                  listing: neuralWattListingSnapshot,
                },),
              },);
            expect(fake.writes[1]
              ?.patch,)
              .toEqual({
                providers: mapNeuralWattProviders({
                  listing: [{ id: 'live-a', }, { id: 'live-b', },],
                },),
              },);
          },
        },),
        it({
          name: 'keeps the served snapshot and returns when the fetch fails',
          fn: async () => {
            const fake = fakeSettingsOf();
            await refreshNeuralWattRoute({
              settings: fake.target,
              fetchListing: async () => {
                throw new Error('offline',);
              },
            },);
            expect(fake.writes.length,)
              .toBe(1,);
            expect(fake.writes[0]
              ?.patch,)
              .toEqual({
                providers: mapNeuralWattProviders({
                  listing: neuralWattListingSnapshot,
                },),
              },);
          },
        },),
        it({
          name: 'keeps the served snapshot when the live listing maps to no serviceable model',
          fn: async () => {
            const fake = fakeSettingsOf();
            await refreshNeuralWattRoute({
              settings: fake.target,
              fetchListing: async () => [
                { id: 'gone', deprecated: true, },
                { metadata: { display_name: 'no id', }, },
              ],
            },);
            expect(fake.writes.length,)
              .toBe(1,);
          },
        },),
        it({
          name: 'keeps the served snapshot when the live listing is empty',
          fn: async () => {
            const fake = fakeSettingsOf();
            await refreshNeuralWattRoute({
              settings: fake.target,
              fetchListing: async () => [],
            },);
            expect(fake.writes.length,)
              .toBe(1,);
          },
        },),
        it({
          name: 'still lands the live listing when the snapshot write is refused',
          fn: async () => {
            const fake = fakeSettingsOf({ failWrites: [0,], },);
            await refreshNeuralWattRoute({
              settings: fake.target,
              fetchListing: async () => [{ id: 'live-a', },],
            },);
            expect(fake.writes.length,)
              .toBe(1,);
            expect(fake.writes[0]
              ?.patch,)
              .toEqual({
                providers: mapNeuralWattProviders({
                  listing: [{ id: 'live-a', },],
                },),
              },);
          },
        },),
        it({
          name: 'keeps the served snapshot when the live write is refused',
          fn: async () => {
            const fake = fakeSettingsOf({ failWrites: [1,], },);
            await refreshNeuralWattRoute({
              settings: fake.target,
              fetchListing: async () => [{ id: 'live-a', },],
            },);
            expect(fake.writes.length,)
              .toBe(1,);
            expect(fake.writes[0]
              ?.patch,)
              .toEqual({
                providers: mapNeuralWattProviders({
                  listing: neuralWattListingSnapshot,
                },),
              },);
          },
        },),
        it({
          name: 'stops before the listing fetch and hands one retry to the deferral sink when the snapshot write is refused inside the reload transaction',
          fn: async () => {
            const fake = fakeSettingsOf({
              failWrites: [0,],
              refuseWith: new Error(UPSTREAM_NESTED_TRANSACTION_MESSAGE),
            },);
            /**
             Retries the deferral sink was handed.
             */
            const retries: (() => Promise<void>)[] = [];
            /**
             Listing fetch calls seen; the deferred run must not fetch.
             */
            const fetches: number[] = [];
            await refreshNeuralWattRoute({
              settings: fake.target,
              fetchListing: async () => {
                fetches.push(1,);
                return [{ id: 'live-a', },];
              },
              onWriteDeferral: function onWriteDeferral(retry) {
                retries.push(retry,);
              },
            },);
            expect(retries.length,)
              .toBe(1,);
            expect(fake.writes.length,)
              .toBe(0,);
            expect(fetches)
              .toEqual([]);
          },
        },),
        it({
          name: 're-runs the whole mount refresh from the deferral retry, snapshot first and live listing second',
          fn: async () => {
            const fake = fakeSettingsOf({
              failWrites: [0,],
              refuseWith: new Error(UPSTREAM_NESTED_TRANSACTION_MESSAGE),
            },);
            /**
             Retries the deferral sink was handed.
             */
            const retries: (() => Promise<void>)[] = [];
            await refreshNeuralWattRoute({
              settings: fake.target,
              fetchListing: async () => [{ id: 'live-a', },],
              onWriteDeferral: function onWriteDeferral(retry) {
                retries.push(retry,);
              },
            },);
            /**
             Retry the deferral sink was handed, before the premise check.
             */
            const [retry] = retries;
            if (retry === undefined) {
              throw new Error('expected one deferred mount refresh retry');
            }
            await retry();
            expect(fake.writes.length,)
              .toBe(2,);
            expect(fake.writes[0]
              ?.patch,)
              .toEqual({
                providers: mapNeuralWattProviders({
                  listing: neuralWattListingSnapshot,
                },),
              },);
            expect(fake.writes[1]
              ?.patch,)
              .toEqual({
                providers: mapNeuralWattProviders({
                  listing: [{ id: 'live-a', },],
                },),
              },);
          },
        },),
        it({
          name: 'hands the deferral sink one retry when the live write is refused inside the reload transaction',
          fn: async () => {
            const fake = fakeSettingsOf({
              failWrites: [1,],
              refuseWith: new Error(UPSTREAM_NESTED_TRANSACTION_MESSAGE),
            },);
            /**
             Retries the deferral sink was handed.
             */
            const retries: (() => Promise<void>)[] = [];
            await refreshNeuralWattRoute({
              settings: fake.target,
              fetchListing: async () => [{ id: 'live-a', },],
              onWriteDeferral: function onWriteDeferral(retry) {
                retries.push(retry,);
              },
            },);
            expect(retries.length,)
              .toBe(1,);
            expect(fake.writes.length,)
              .toBe(1,);
            expect(fake.writes[0]
              ?.patch,)
              .toEqual({
                providers: mapNeuralWattProviders({
                  listing: neuralWattListingSnapshot,
                },),
              },);
          },
        },),
        it({
          name: 'runs the live listing write anyway when no deferral sink is given for a nested reload-transaction refusal',
          fn: async () => {
            const fake = fakeSettingsOf({
              failWrites: [0,],
              refuseWith: new Error(UPSTREAM_NESTED_TRANSACTION_MESSAGE),
            },);
            await refreshNeuralWattRoute({
              settings: fake.target,
              fetchListing: async () => [{ id: 'live-a', },],
            },);
            expect(fake.writes.length,)
              .toBe(1,);
            expect(fake.writes[0]
              ?.patch,)
              .toEqual({
                providers: mapNeuralWattProviders({
                  listing: [{ id: 'live-a', },],
                },),
              },);
          },
        },),
      ],
    },),

    //endregion Mount refresh
  ],
},);
