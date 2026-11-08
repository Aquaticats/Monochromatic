import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  mapNeuralWattProviders,
  NEURALWATT_API,
  NEURALWATT_API_KEY_ENV,
  NEURALWATT_BASE_URL,
  NEURALWATT_ROUTE,
} from '../dist/final/node/index.mjs';

await describe({
  name: '',
  children: [
    //region Listing to provider dict

    describe({
      name: mapNeuralWattProviders.name,
      children: [
        it({
          name: 'serves one route keyed by the route constant with only serviceable models',
          fn: async () => {
            const providers = mapNeuralWattProviders({
              listing: [
                { id: 'kept' },
                { id: 'dropped', deprecated: true },
                { metadata: { display_name: 'no id' } },
              ],
            });
            expect(Object.keys(providers)).toEqual([NEURALWATT_ROUTE]);
            expect(providers[NEURALWATT_ROUTE]?.models).toHaveLength(1);
            expect(providers[NEURALWATT_ROUTE]?.models?.[0]?.id).toBe('kept');
          },
        }),
        it({
          name: 'pins the credential, protocol, endpoint, effort dispatch, and role mapping',
          fn: async () => {
            const route = mapNeuralWattProviders({ listing: [] })[NEURALWATT_ROUTE];
            if (route === undefined) {
              throw new Error('expected a route profile');
            }
            expect(route.apiKeyEnv).toBe(NEURALWATT_API_KEY_ENV);
            expect(route.api).toBe(NEURALWATT_API);
            expect(route.baseURL).toBe(NEURALWATT_BASE_URL);
            expect(route.compat?.supportsReasoningEffort).toBe(true);
            expect(route.compat?.supportsDeveloperRole).toBe(false);
            expect(route.models).toEqual([]);
          },
        }),
        it({
          name: 'preserves listing order across serviceable models',
          fn: async () => {
            const providers = mapNeuralWattProviders({
              listing: [{ id: 'first' }, { id: 'second' }, { id: 'third' }],
            });
            expect(providers[NEURALWATT_ROUTE]?.models?.map((model) => model.id))
              .toEqual(['first', 'second', 'third']);
          },
        }),
      ],
    }),

    //endregion Listing to provider dict
  ],
});
