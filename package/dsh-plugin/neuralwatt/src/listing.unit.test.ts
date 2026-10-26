import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  isRecord,
  mapNeuralWattProviders,
  neuralWattListingSnapshot,
  NEURALWATT_API,
  NEURALWATT_API_KEY_ENV,
  NEURALWATT_BASE_URL,
  NEURALWATT_ROUTE,
  type NeuralWattListingEntry,
} from '../dist/final/node/index.mjs';

/**
 Single model profile one entry mapped to, or nothing when the mapping
 dropped the entry as unserviceable.
 */
function mappedModelOf(entry: NeuralWattListingEntry) {
  return mapNeuralWattProviders({ listing: [entry] })[NEURALWATT_ROUTE]?.models?.[0];
}

await describe({
  name: '',
  children: [
    //region External JSON narrowing

    describe({
      name: isRecord.name,
      children: [
        it({
          name: 'accepts a plain object',
          fn: async () => {
            expect(isRecord({ a: 1 })).toBe(true);
          },
        }),
        it({
          name: 'rejects arrays, null, and primitives',
          fn: async () => {
            expect(isRecord([1])).toBe(false);
            expect(isRecord(null)).toBe(false);
            expect(isRecord('x')).toBe(false);
            expect(isRecord(5)).toBe(false);
          },
        }),
      ],
    }),

    //endregion External JSON narrowing

    //region Listing entry mapping

    describe({
      name: 'listing entry mapping',
      children: [
        it({
          name: 'maps a vision reasoning model with its disclosed limits only',
          fn: async () => {
            expect(mappedModelOf({
              id: 'qwen-3.8-27b',
              max_model_len: 262_128,
              deprecated: false,
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
          name: 'drops an entry the listing marks deprecated',
          fn: async () => {
            expect(mappedModelOf({ id: 'gone', deprecated: true })).toBeUndefined();
          },
        }),
        it({
          name: 'drops an entry without an id',
          fn: async () => {
            expect(mappedModelOf({ metadata: { display_name: 'nameless' } })).toBeUndefined();
          },
        }),
        it({
          name: 'drops an entry whose id is empty',
          fn: async () => {
            expect(mappedModelOf({ id: '' })).toBeUndefined();
          },
        }),
        it({
          name: 'falls back to the entry id for the name',
          fn: async () => {
            const model = mappedModelOf({ id: 'nw-small' });
            if (model === undefined) {
              throw new Error('expected a model profile');
            }
            expect(model.name).toBe('nw-small');
          },
        }),
        it({
          name: 'reports text input for a model the listing marks without vision',
          fn: async () => {
            const model = mappedModelOf({
              id: 'm',
              metadata: { capabilities: { vision: false } },
            });
            if (model === undefined) {
              throw new Error('expected a model profile');
            }
            expect(model.input).toEqual(['text']);
          },
        }),
        it({
          name: 'falls back to max_model_len for the context window and leaves maxTokens unset',
          fn: async () => {
            const model = mappedModelOf({ id: 'm', max_model_len: 262_128 });
            if (model === undefined) {
              throw new Error('expected a model profile');
            }
            expect(model.contextWindow).toBe(262_128);
            expect(Object.hasOwn(model, 'maxTokens')).toBe(false);
          },
        }),
        it({
          name: 'leaves the context window unset when the listing sizes nothing',
          fn: async () => {
            const model = mappedModelOf({ id: 'm' });
            if (model === undefined) {
              throw new Error('expected a model profile');
            }
            expect(Object.hasOwn(model, 'contextWindow')).toBe(false);
          },
        }),
        it({
          name: 'leaves maxTokens unset when the listing discloses no output limit',
          fn: async () => {
            const model = mappedModelOf({
              id: 'm',
              metadata: {
                limits: { max_context_length: 1_048_560, max_output_tokens: null },
              },
            });
            if (model === undefined) {
              throw new Error('expected a model profile');
            }
            expect(model.contextWindow).toBe(1_048_560);
            expect(Object.hasOwn(model, 'maxTokens')).toBe(false);
          },
        }),
        it({
          name: 'ignores non-positive and fractional context lengths',
          fn: async () => {
            const zeroed = mappedModelOf({
              id: 'm',
              max_model_len: 262_128,
              metadata: { limits: { max_context_length: 0 } },
            });
            const fractional = mappedModelOf({
              id: 'm',
              max_model_len: 262_128,
              metadata: { limits: { max_context_length: 1_024.5 } },
            });
            if ((zeroed === undefined) || (fractional === undefined)) {
              throw new Error('expected model profiles');
            }
            expect(zeroed.contextWindow).toBe(262_128);
            expect(fractional.contextWindow).toBe(262_128);
          },
        }),
        it({
          name: 'declares non-reasoning for a model capabilities mark non-reasoning',
          fn: async () => {
            const model = mappedModelOf({
              id: 'kimi-k3-fast',
              metadata: { capabilities: { reasoning: false } },
            });
            if (model === undefined) {
              throw new Error('expected a model profile');
            }
            expect(model.reasoningEfforts).toBe(false);
          },
        }),
        it({
          name: 'omits reasoning efforts a reasoning-capable entry does not list',
          fn: async () => {
            const model = mappedModelOf({
              id: 'm',
              metadata: { capabilities: { reasoning: true } },
            });
            if (model === undefined) {
              throw new Error('expected a model profile');
            }
            expect(Object.hasOwn(model, 'reasoningEfforts')).toBe(false);
          },
        }),
        it({
          name: 'treats a non-record metadata as absent',
          fn: async () => {
            expect(mappedModelOf({ id: 'm', max_model_len: 99, metadata: 5 })).toEqual({
              id: 'm',
              name: 'm',
              contextWindow: 99,
              input: ['text'],
            });
          },
        }),
        it({
          name: 'offers supported efforts each sending its own spelling',
          fn: async () => {
            const model = mappedModelOf({
              id: 'm',
              metadata: {
                reasoning: {
                  default_enabled: false,
                  supported_efforts: ['xhigh', 'medium', 'low', 'none'],
                },
              },
            });
            if (model === undefined) {
              throw new Error('expected a model profile');
            }
            expect(model.reasoningEfforts).toEqual({
              off: 'none',
              xhigh: 'xhigh',
              medium: 'medium',
              low: 'low',
            });
          },
        }),
        it({
          name: 'drops the off level for a model the listing says thinks by default',
          fn: async () => {
            const model = mappedModelOf({
              id: 'm',
              metadata: {
                reasoning: {
                  default_enabled: true,
                  supported_efforts: ['high', 'none'],
                },
              },
            });
            if (model === undefined) {
              throw new Error('expected a model profile');
            }
            expect(model.reasoningEfforts).toEqual({ high: 'high' });
          },
        }),
        it({
          name: 'keeps the off level for a model the listing says thinks off by default',
          fn: async () => {
            const model = mappedModelOf({
              id: 'm',
              metadata: {
                reasoning: {
                  default_enabled: false,
                  supported_efforts: ['high', 'none'],
                },
              },
            });
            if (model === undefined) {
              throw new Error('expected a model profile');
            }
            expect(model.reasoningEfforts).toEqual({ high: 'high', off: 'none' });
          },
        }),
        it({
          name: 'drops the off level when the listing names no default',
          fn: async () => {
            const model = mappedModelOf({
              id: 'm',
              metadata: {
                reasoning: { supported_efforts: ['high', 'none'] },
              },
            });
            if (model === undefined) {
              throw new Error('expected a model profile');
            }
            expect(model.reasoningEfforts).toEqual({ high: 'high' });
          },
        }),
        it({
          name: 'omits the off level when the listing does not accept none',
          fn: async () => {
            const model = mappedModelOf({
              id: 'm',
              metadata: {
                reasoning: { supported_efforts: ['max', 'high', 'low'] },
              },
            });
            if (model === undefined) {
              throw new Error('expected a model profile');
            }
            const efforts = model.reasoningEfforts;
            if (((typeof efforts) !== 'object') || (efforts === null)) {
              throw new Error('expected an effort dict');
            }
            expect(Object.hasOwn(efforts, 'off')).toBe(false);
            expect(efforts).toEqual({ max: 'max', high: 'high', low: 'low' });
          },
        }),
        it({
          name: 'skips spellings it cannot express instead of guessing',
          fn: async () => {
            const model = mappedModelOf({
              id: 'm',
              metadata: {
                reasoning: {
                  default_enabled: false,
                  supported_efforts: ['ultra', 'high', 'none'],
                },
              },
            });
            if (model === undefined) {
              throw new Error('expected a model profile');
            }
            expect(model.reasoningEfforts).toEqual({ high: 'high', off: 'none' });
          },
        }),
        it({
          name: 'declares non-reasoning when no level beyond off is selectable',
          fn: async () => {
            const model = mappedModelOf({
              id: 'm',
              metadata: { reasoning: { supported_efforts: ['none'] } },
            });
            if (model === undefined) {
              throw new Error('expected a model profile');
            }
            expect(model.reasoningEfforts).toBe(false);
          },
        }),
        it({
          name: 'falls back to accepted efforts minus alias spellings',
          fn: async () => {
            const model = mappedModelOf({
              id: 'm',
              metadata: {
                reasoning: {
                  default_enabled: false,
                  accepted_efforts: ['max', 'xhigh', 'high', 'low', 'minimal', 'none'],
                  effort_aliases: { max: 'xhigh', minimal: 'low' },
                },
              },
            });
            if (model === undefined) {
              throw new Error('expected a model profile');
            }
            expect(model.reasoningEfforts).toEqual({
              off: 'none',
              xhigh: 'xhigh',
              high: 'high',
              low: 'low',
            });
          },
        }),
        it({
          name: 'declares non-reasoning when the alias fallback leaves only off',
          fn: async () => {
            const model = mappedModelOf({
              id: 'm',
              metadata: {
                reasoning: {
                  accepted_efforts: ['max', 'none'],
                  effort_aliases: { max: 'xhigh' },
                },
              },
            });
            if (model === undefined) {
              throw new Error('expected a model profile');
            }
            expect(model.reasoningEfforts).toBe(false);
          },
        }),
        it({
          name: 'falls back to accepted efforts when the supported list holds a non-string member',
          fn: async () => {
            const model = mappedModelOf({
              id: 'm',
              metadata: {
                reasoning: {
                  default_enabled: false,
                  supported_efforts: ['high', 3],
                  accepted_efforts: ['low', 'none'],
                },
              },
            });
            if (model === undefined) {
              throw new Error('expected a model profile');
            }
            expect(model.reasoningEfforts).toEqual({ low: 'low', off: 'none' });
          },
        }),
        it({
          name: 'omits the field when neither effort list is usable',
          fn: async () => {
            const model = mappedModelOf({
              id: 'm',
              metadata: { reasoning: { supported_efforts: 'high' } },
            });
            if (model === undefined) {
              throw new Error('expected a model profile');
            }
            expect(Object.hasOwn(model, 'reasoningEfforts')).toBe(false);
          },
        }),
      ],
    }),

    //endregion Listing entry mapping

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

    //region Checked-in snapshot

    describe({
      name: 'checked-in listing snapshot',
      children: [
        it({
          name: 'maps to the full snapshot catalog',
          fn: async () => {
            const providers = mapNeuralWattProviders({
              listing: neuralWattListingSnapshot,
            });
            expect(providers[NEURALWATT_ROUTE]?.models).toHaveLength(27);
          },
        }),
        it({
          name: 'sizes mimo-v2.6-pro at its disclosed context window and thinking levels',
          fn: async () => {
            const providers = mapNeuralWattProviders({
              listing: neuralWattListingSnapshot,
            });
            const mimo = providers[NEURALWATT_ROUTE]?.models
              ?.find((model) => model.id === 'mimo-v2.6-pro');
            if (mimo === undefined) {
              throw new Error('expected mimo-v2.6-pro in the snapshot');
            }
            expect(mimo.contextWindow).toBe(1_048_560);
            expect(mimo.maxTokens).toBe(393_216);
            expect(mimo.input).toEqual(['text', 'image']);
            expect(mimo.reasoningEfforts).toEqual({ high: 'high' });
          },
        }),
        it({
          name: 'keeps glm-5.3 mandatory reasoning without an off level',
          fn: async () => {
            const providers = mapNeuralWattProviders({
              listing: neuralWattListingSnapshot,
            });
            const glm = providers[NEURALWATT_ROUTE]?.models
              ?.find((model) => model.id === 'glm-5.3');
            if (glm === undefined) {
              throw new Error('expected glm-5.3 in the snapshot');
            }
            expect(glm.reasoningEfforts).toEqual({ max: 'max', high: 'high', low: 'low' });
            expect(glm.maxTokens).toBeUndefined();
          },
        }),
        it({
          name: 'declares kimi-k3-fast non-reasoning',
          fn: async () => {
            const providers = mapNeuralWattProviders({
              listing: neuralWattListingSnapshot,
            });
            const fast = providers[NEURALWATT_ROUTE]?.models
              ?.find((model) => model.id === 'kimi-k3-fast');
            if (fast === undefined) {
              throw new Error('expected kimi-k3-fast in the snapshot');
            }
            expect(fast.reasoningEfforts).toBe(false);
          },
        }),
      ],
    }),

    //endregion Checked-in snapshot
  ],
});
