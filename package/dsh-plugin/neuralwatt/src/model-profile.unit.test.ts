import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  mapNeuralWattProviders,
  NEURALWATT_ROUTE,
  type NeuralWattListingEntry,
} from '../dist/final/node/index.mjs';

/**
 Served model profile one entry mapped to, or nothing when the mapping
 dropped the entry as unserviceable.
 */
function servedModelOf(entry: NeuralWattListingEntry) {
  return mapNeuralWattProviders({ listing: [entry] })[NEURALWATT_ROUTE]?.models?.[0];
}

await describe({
  name: 'facts to model profile projection',
  children: [
    it({
      name: 'projects every disclosed fact onto the served model profile',
      fn: async () => {
        expect(servedModelOf({
          id: 'qwen-3.8-27b',
          max_model_len: 262_128,
          metadata: {
            display_name: 'Qwen 3.8 27B',
            capabilities: { reasoning: true, vision: true },
            reasoning: {
              default_enabled: false,
              supported_efforts: ['xhigh', 'medium', 'low', 'none'],
            },
            limits: { max_context_length: 262_128, max_output_tokens: 131_072 },
          },
        })).toEqual({
          id: 'qwen-3.8-27b',
          name: 'Qwen 3.8 27B',
          contextWindow: 262_128,
          maxTokens: 131_072,
          input: ['text', 'image'],
          reasoningEfforts: {
            off: 'none',
            xhigh: 'xhigh',
            medium: 'medium',
            low: 'low',
          },
        });
      },
    }),
    it({
      name: 'omits the fields the listing does not disclose',
      fn: async () => {
        expect(servedModelOf({ id: 'nw-small' })).toEqual({
          id: 'nw-small',
          name: 'nw-small',
          input: ['text'],
        });
      },
    }),
    it({
      name: 'stays inert toward pricing the facts carry',
      fn: async () => {
        expect(servedModelOf({
          id: 'm',
          metadata: {
            pricing: {
              input_per_million: 0.45,
              output_per_million: 3.2,
            },
          },
        })).toEqual({
          id: 'm',
          name: 'm',
          input: ['text'],
        });
      },
    }),
    it({
      name: 'projects a non-reasoning model as reasoningEfforts false',
      fn: async () => {
        const model = servedModelOf({
          id: 'kimi-k3-fast',
          metadata: { capabilities: { reasoning: false } },
        });
        if (model === undefined) {
          throw new Error('expected a served model profile');
        }
        expect(model.reasoningEfforts).toBe(false);
      },
    }),
  ],
});
