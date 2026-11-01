import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
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
 One recorded settings merge write.
 */
type RecordedWrite = {
  ns: string;
  patch: Record<string, unknown>;
};

/**
 Recording fake of the settings seam: serves a fixed form value through
 `describe`, records every accepted merge write, and can refuse writes at the
 given zero-based call indices to reach the failure branches.

 @param options - stored form value and refusing call indices

 @returns fake target plus the writes it accepted

 @example
 ```ts
 const fake = fakeSettingsOf({ stored: { providers: {} } });
 ```
 */
function fakeSettingsOf({
  stored,
  failWrites = [],
}: {
  readonly stored?: unknown;
  readonly failWrites?: readonly number[];
} = {}): {
  readonly target: NeuralWattSettingsTarget;
  readonly writes: RecordedWrite[];
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
        if (failWrites.includes(attempt)) throw new Error('settings entry unavailable');
        writes.push({ ns, patch });
      },
    },
    writes,
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
      ],
    },),

    //endregion Mount refresh
  ],
},);
