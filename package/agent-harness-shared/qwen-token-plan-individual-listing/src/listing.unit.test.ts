import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  isRecord,
  mapQwenTokenPlanIndividualModels,
  QWEN_TOKEN_PLAN_COMPAT_PINS_WITH_EFFORTS,
  QWEN_TOKEN_PLAN_COMPAT_PINS_WITHOUT_EFFORTS,
  QWEN_TOKEN_PLAN_DEFAULT_CONTEXT_WINDOW,
  QWEN_TOKEN_PLAN_DEFAULT_MAX_TOKENS,
  QWEN_TOKEN_PLAN_MEDIA_NAME_MARKERS,
  qwenTokenPlanIndividualFacts,
  qwenTokenPlanIndividualListingSnapshot,
  type QwenTokenPlanModelFacts,
} from '../dist/final/node/index.mjs';

/**
 Single resolved model one id maps to against the curated table, or a thrown
 error when the mapping dropped it.
 */
function mappedModelOf(id: string) {
  const model = mapQwenTokenPlanIndividualModels({
    listing: qwenTokenPlanIndividualListingSnapshot.entries,
  }).find(function matchesId(candidate) {
    return candidate.id === id;
  });
  if (model === undefined) {
    throw new Error(`mapping dropped expected model ${id}`);
  }
  return model;
}

/**
 Media ids the curated table carries, derived from the rows the skip rule
 names rather than listed again here.
 */
function mediaIdsOfTable() {
  return qwenTokenPlanIndividualFacts.filter(function isMediaRow(row) {
    return row.kind !== 'chat';
  }).map(function toId(row) {
    return row.id;
  });
}

await describe({
  name: '',
  children: [
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

    describe({
      name: 'listing entry mapping',
      children: [
        it({
          name: 'maps deepseek-v4.1-flash-1017 with its sibling catalog facts',
          fn: async () => {
            expect(mappedModelOf('deepseek-v4.1-flash-1017')).toEqual({
              id: 'deepseek-v4.1-flash-1017',
              name: 'DeepSeek V4.1 Flash 1017',
              kind: 'chat',
              contextWindow: 1_000_000,
              maxTokens: 384_000,
              input: [
                'text',
                'image',
              ],
              reasoning: true,
              efforts: {
                low: 'low',
                high: 'high',
                max: 'max',
              },
              compat: QWEN_TOKEN_PLAN_COMPAT_PINS_WITH_EFFORTS,
            });
          },
        }),
        it({
          name: 'maps qwen4-max-preview with its documented facts',
          fn: async () => {
            expect(mappedModelOf('qwen4-max-preview')).toEqual({
              id: 'qwen4-max-preview',
              name: 'Qwen4 Max Preview',
              kind: 'chat',
              contextWindow: 1_000_000,
              maxTokens: 393_216,
              input: [
                'text',
                'image',
              ],
              reasoning: true,
              efforts: {
                low: 'low',
                medium: 'medium',
                xhigh: 'xhigh',
              },
              compat: QWEN_TOKEN_PLAN_COMPAT_PINS_WITH_EFFORTS,
            });
          },
        }),
        it({
          name: 'maps qwen3.7-max as reasoning without selectable efforts',
          fn: async () => {
            const model = mappedModelOf('qwen3.7-max');
            expect(model.reasoning).toBe(true);
            expect(model.efforts).toBeUndefined();
            expect(model.compat).toEqual(QWEN_TOKEN_PLAN_COMPAT_PINS_WITHOUT_EFFORTS);
          },
        }),
        it({
          name: 'maps auto with pi schema defaults and its id as the name',
          fn: async () => {
            const model = mappedModelOf('auto');
            expect(model).toEqual({
              id: 'auto',
              name: 'auto',
              kind: 'chat',
              contextWindow: QWEN_TOKEN_PLAN_DEFAULT_CONTEXT_WINDOW,
              maxTokens: QWEN_TOKEN_PLAN_DEFAULT_MAX_TOKENS,
              input: [
                'text',
              ],
              reasoning: true,
              compat: QWEN_TOKEN_PLAN_COMPAT_PINS_WITHOUT_EFFORTS,
            });
          },
        }),
        it({
          name: 'never serves the media kind rows the table carries',
          fn: async () => {
            const servedIds = mapQwenTokenPlanIndividualModels({
              listing: qwenTokenPlanIndividualListingSnapshot.entries,
            }).map(function toId(model) {
              return model.id;
            });
            for (const mediaId of mediaIdsOfTable()) {
              expect(servedIds).not.toContain(mediaId);
            }
          },
        }),
        it({
          name: 'skips media-named ids the table carries no row for',
          fn: async () => {
            const listing = QWEN_TOKEN_PLAN_MEDIA_NAME_MARKERS.map(
              function toEntry(marker) {
                return {
                  id: `new-${marker}-model`,
                  created: 1,
                };
              },
            );
            const models = mapQwenTokenPlanIndividualModels({ listing, });
            expect(models).toHaveLength(0);
          },
        }),
        it({
          name: 'maps a rowless id with no family to the schema defaults',
          fn: async () => {
            expect(mapQwenTokenPlanIndividualModels({
              listing: [
                {
                  id: 'mystery-model',
                  created: 1,
                },
              ],
            })).toEqual([
              {
                id: 'mystery-model',
                name: 'mystery-model',
                kind: 'chat',
                contextWindow: QWEN_TOKEN_PLAN_DEFAULT_CONTEXT_WINDOW,
                maxTokens: QWEN_TOKEN_PLAN_DEFAULT_MAX_TOKENS,
                input: [
                  'text',
                ],
                reasoning: true,
                compat: QWEN_TOKEN_PLAN_COMPAT_PINS_WITHOUT_EFFORTS,
              },
            ]);
          },
        }),
        it({
          name: 'keeps listing order and drops entries without a usable id',
          fn: async () => {
            const models = mapQwenTokenPlanIndividualModels({
              listing: [
                {
                  id: 'auto',
                  created: 1,
                },
                {
                  id: '',
                  created: 2,
                },
                {
                  id: 'qwen3.8-max',
                  created: 3,
                },
              ],
            });
            expect(models.map(function toId(model) {
              return model.id;
            })).toEqual([
              'auto',
              'qwen3.8-max',
            ]);
          },
        }),
        it({
          name: 'never serves a facts row whose kind is not chat',
          fn: async () => {
            const routerRow: QwenTokenPlanModelFacts = {
              id: 'router-alias',
              name: 'Router Alias',
              kind: 'router',
              input: [
                'text',
              ],
              reasoning: true,
              compat: QWEN_TOKEN_PLAN_COMPAT_PINS_WITHOUT_EFFORTS,
            };
            expect(mapQwenTokenPlanIndividualModels({
              listing: [
                {
                  id: 'router-alias',
                  created: 1,
                },
              ],
              facts: [
                routerRow,
              ],
            })).toHaveLength(0);
          },
        }),
        it({
          name: 'never serves efforts on a row that says the model does not reason',
          fn: async () => {
            const contradictedRow: QwenTokenPlanModelFacts = {
              id: 'silent-model',
              name: 'Silent Model',
              kind: 'chat',
              contextWindow: 100,
              maxTokens: 50,
              input: [
                'text',
              ],
              reasoning: false,
              efforts: {
                high: 'high',
              },
              compat: QWEN_TOKEN_PLAN_COMPAT_PINS_WITH_EFFORTS,
            };
            const [model] = mapQwenTokenPlanIndividualModels({
              listing: [
                {
                  id: 'silent-model',
                  created: 1,
                },
              ],
              facts: [
                contradictedRow,
              ],
            });
            expect(model?.reasoning).toBe(false);
            expect(model?.efforts).toBeUndefined();
            expect(model?.compat.supportsReasoningEffort).toBe(false);
          },
        }),
        it({
          name: 'falls to schema defaults for numbers a row leaves open',
          fn: async () => {
            const openRow: QwenTokenPlanModelFacts = {
              id: 'open-model',
              name: 'Open Model',
              kind: 'chat',
              input: [
                'text',
              ],
              reasoning: true,
              compat: QWEN_TOKEN_PLAN_COMPAT_PINS_WITHOUT_EFFORTS,
            };
            const [model] = mapQwenTokenPlanIndividualModels({
              listing: [
                {
                  id: 'open-model',
                  created: 1,
                },
              ],
              facts: [
                openRow,
              ],
            });
            expect(model?.contextWindow).toBe(QWEN_TOKEN_PLAN_DEFAULT_CONTEXT_WINDOW);
            expect(model?.maxTokens).toBe(QWEN_TOKEN_PLAN_DEFAULT_MAX_TOKENS);
          },
        }),
      ],
    }),
  ],
});
