import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  isRecord,
  mapNeuralWattListing,
  reduceListingEntry,
} from '../dist/final/node/index.mjs';

/**
 Narrowed reasoning block one reduced entry carries, or an empty record when
 the reduction kept none.
 */
function reasoningOf(reduced: ReturnType<typeof reduceListingEntry>) {
  const metadata = isRecord(reduced.metadata) ? reduced.metadata : {};
  return isRecord(metadata.reasoning) ? metadata.reasoning : {};
}

await describe({
  name: reduceListingEntry.name,
  children: [
    it({
      name: 'keeps reasoning.default_enabled so a thinking-off-by-default gemma-4-31b can offer off after a refresh',
      fn: async () => {
        const reduced = reduceListingEntry({
          entry: {
            id: 'gemma-4-31b',
            max_model_len: 262_128,
            metadata: {
              display_name: 'Gemma 4 31B',
              capabilities: { reasoning: true, vision: false },
              reasoning: {
                default_enabled: false,
                supported_efforts: ['max', 'none'],
              },
              limits: {
                max_context_length: 262_128,
                max_output_tokens: 131_072,
              },
            },
          },
        });
        expect(reasoningOf(reduced).default_enabled).toBe(false);
        expect(mapNeuralWattListing({ listing: [reduced] })[0]
          ?.reasoningEfforts).toEqual({
            max: 'max',
            off: 'none',
          });
      },
    }),
    it({
      name: 'keeps a disclosed default_enabled true so the off level stays withheld',
      fn: async () => {
        const reduced = reduceListingEntry({
          entry: {
            id: 'm',
            metadata: {
              reasoning: {
                default_enabled: true,
                supported_efforts: ['max', 'none'],
              },
            },
          },
        });
        expect(reasoningOf(reduced).default_enabled).toBe(true);
        expect(mapNeuralWattListing({ listing: [reduced] })[0]
          ?.reasoningEfforts).toEqual({ max: 'max' });
      },
    }),
    it({
      name: 'carries deprecation only when the entry asserts it',
      fn: async () => {
        const unasserted = reduceListingEntry({
          entry: {
            id: 'm',
            deprecated: false,
            metadata: { display_name: 'M' },
          },
        });
        const asserted = reduceListingEntry({
          entry: {
            id: 'gone',
            deprecated: true,
            metadata: { display_name: 'Gone' },
          },
        });
        expect(Object.hasOwn(unasserted, 'deprecated')).toBe(false);
        expect(asserted.deprecated).toBe(true);
      },
    }),
    it({
      name: 'keeps the pricing block and drops fields the mapping never reads',
      fn: async () => {
        const reduced = reduceListingEntry({
          entry: {
            id: 'm',
            metadata: {
              display_name: 'M',
              description: 'a model the mapping does not read',
              capabilities: {
                reasoning: true,
                vision: true,
                tools: true,
              },
              reasoning: {
                mandatory: true,
                default_enabled: false,
                default_effort: 'max',
                supported_efforts: ['max', 'none'],
              },
              limits: {
                max_context_length: 262_128,
                max_images: 20,
              },
              pricing: {
                input_per_million: 0.45,
                cached_input_is_default: false,
                currency: 'USD',
              },
            },
          },
        });
        const metadata = isRecord(reduced.metadata) ? reduced.metadata : {};
        expect(metadata.display_name).toBe('M');
        expect(Object.hasOwn(metadata, 'description')).toBe(false);
        expect(metadata.capabilities).toEqual({
          reasoning: true,
          vision: true,
        });
        expect(metadata.reasoning).toEqual({
          default_enabled: false,
          supported_efforts: ['max', 'none'],
        });
        expect(metadata.limits).toEqual({ max_context_length: 262_128 });
        expect(metadata.pricing).toEqual({
          input_per_million: 0.45,
          currency: 'USD',
        });
      },
    }),
  ],
});
