/**
 Tests for the pi projection of shared NeuralWatt listing facts.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  NEURALWATT_API,
  NEURALWATT_BASE_URL,
  NEURALWATT_ROUTE,
  NEURALWATT_THINKING_LEVELS,
  type NeuralWattModelFacts,
} from '@monochromatic-dev/agent-harness-shared-neuralwatt-listing/ts';
import {
  hasEffortsDict,
  mapNeuralWattModelCost,
  NEURALWATT_COST_RATE_ZERO,
  NEURALWATT_OPENAI_COMPLETIONS_COMPAT,
  PI_SCHEMA_DEFAULT_CONTEXT_WINDOW,
  PI_SCHEMA_DEFAULT_MAX_TOKENS,
  projectNeuralWattModel,
  projectNeuralWattModels,
} from '../dist/final/node/index.mjs';

//region Fixtures

/**
 Minimal non-reasoning facts entry the variant builders start from.
 */
const TEXT_ONLY_FACTS: NeuralWattModelFacts = {
  id: 'fixture-model',
  name: 'Fixture Model',
  input: ['text'],
};

//endregion Fixtures

//region Cost mapping

await describe({
  name: mapNeuralWattModelCost.name,
  children: [
    it({
      name: 'serves listing pricing as the four pi cost rates',
      fn: async () => {
        expect(mapNeuralWattModelCost({
          pricing: {
            inputPerMillion: 1,
            outputPerMillion: 2,
            cachedInputPerMillion: 3,
            cachedOutputPerMillion: 4,
          },
        },)).toEqual({
          input: 1,
          output: 2,
          cacheRead: 3,
          cacheWrite: 4,
        });
      },
    }),
    it({
      name: 'serves the zero rate for every field the listing leaves out',
      fn: async () => {
        expect(mapNeuralWattModelCost({
          pricing: { inputPerMillion: 1, },
        },)).toEqual({
          input: 1,
          output: NEURALWATT_COST_RATE_ZERO,
          cacheRead: NEURALWATT_COST_RATE_ZERO,
          cacheWrite: NEURALWATT_COST_RATE_ZERO,
        });
      },
    }),
    it({
      name: 'serves zero rates when the facts carry no pricing',
      fn: async () => {
        expect(mapNeuralWattModelCost({},)).toEqual({
          input: NEURALWATT_COST_RATE_ZERO,
          output: NEURALWATT_COST_RATE_ZERO,
          cacheRead: NEURALWATT_COST_RATE_ZERO,
          cacheWrite: NEURALWATT_COST_RATE_ZERO,
        });
      },
    }),
  ],
});

//endregion Cost mapping

//region Efforts narrowing

await describe({
  name: hasEffortsDict.name,
  children: [
    it({
      name: 'recognizes the offered dict and rejects the other declarations',
      fn: async () => {
        expect(hasEffortsDict({
          ...TEXT_ONLY_FACTS,
          reasoningEfforts: { high: 'high', },
        },)).toBe(true);
        expect(hasEffortsDict({
          ...TEXT_ONLY_FACTS,
          reasoningEfforts: false,
        },)).toBe(false);
        expect(hasEffortsDict(TEXT_ONLY_FACTS,)).toBe(false);
      },
    }),
  ],
});

//endregion Efforts narrowing

//region Model projection

await describe({
  name: projectNeuralWattModel.name,
  children: [
    it({
      name: 'serves the route identity and declared limits verbatim',
      fn: async () => {
        const model = projectNeuralWattModel({
          facts: {
            ...TEXT_ONLY_FACTS,
            contextWindow: 262_128,
            maxTokens: 131_072,
          },
        },);
        expect(model.id).toBe(TEXT_ONLY_FACTS.id);
        expect(model.name).toBe(TEXT_ONLY_FACTS.name);
        expect(model.api).toBe(NEURALWATT_API);
        expect(model.provider).toBe(NEURALWATT_ROUTE);
        expect(model.baseUrl).toBe(NEURALWATT_BASE_URL);
        expect(model.contextWindow).toBe(262_128);
        expect(model.maxTokens).toBe(131_072);
        expect(model.input).toEqual(['text']);
      },
    }),
    it({
      name: 'fills pi schema defaults when the facts omit limits',
      fn: async () => {
        const model = projectNeuralWattModel({ facts: TEXT_ONLY_FACTS, },);
        expect(model.contextWindow).toBe(PI_SCHEMA_DEFAULT_CONTEXT_WINDOW);
        expect(model.maxTokens).toBe(PI_SCHEMA_DEFAULT_MAX_TOKENS);
      },
    }),
    it({
      name: 'projects vision input modality verbatim',
      fn: async () => {
        const model = projectNeuralWattModel({
          facts: {
            ...TEXT_ONLY_FACTS,
            input: ['text', 'image'],
          },
        },);
        expect(model.input).toEqual(['text', 'image']);
      },
    }),
    it({
      name: 'marks reasoning false for the non-reasoning declaration and for silence',
      fn: async () => {
        const declaredFalse = projectNeuralWattModel({
          facts: {
            ...TEXT_ONLY_FACTS,
            reasoningEfforts: false,
          },
        },);
        const silent = projectNeuralWattModel({ facts: TEXT_ONLY_FACTS, },);
        expect(declaredFalse.reasoning).toBe(false);
        expect(silent.reasoning).toBe(false);
        expect(declaredFalse.thinkingLevelMap).toBe(undefined);
        expect(silent.thinkingLevelMap).toBe(undefined);
      },
    }),
    it({
      name: 'marks reasoning true for a dict and expands every level explicitly',
      fn: async () => {
        const model = projectNeuralWattModel({
          facts: {
            ...TEXT_ONLY_FACTS,
            reasoningEfforts: { high: 'high', },
          },
        },);
        expect(model.reasoning).toBe(true);
        const map = model.thinkingLevelMap;
        expect(map).toBeDefined();
        expect(Object.keys(map ?? {},)).toHaveLength(NEURALWATT_THINKING_LEVELS.length);
        for (const level of NEURALWATT_THINKING_LEVELS) {
          expect(Object.hasOwn(map ?? {}, level)).toBe(true);
          expect(map?.[level]).toBe(level === 'high' ? 'high' : null);
        }
      },
    }),
    it({
      name: 'keeps a declared null off level absent, so dispatch sends nothing',
      fn: async () => {
        const model = projectNeuralWattModel({
          facts: {
            ...TEXT_ONLY_FACTS,
            reasoningEfforts: {
              off: null,
              high: 'high',
            },
          },
        },);
        expect(Object.hasOwn(model.thinkingLevelMap ?? {}, 'off')).toBe(false);
      },
    }),
    it({
      name: 'carries the off wire value the listing spells',
      fn: async () => {
        const model = projectNeuralWattModel({
          facts: {
            ...TEXT_ONLY_FACTS,
            reasoningEfforts: {
              off: 'none',
              high: 'high',
            },
          },
        },);
        expect(model.thinkingLevelMap?.off).toBe('none');
      },
    }),
    it({
      name: 'pins the compat switches on every projected model',
      fn: async () => {
        const model = projectNeuralWattModel({ facts: TEXT_ONLY_FACTS, },);
        expect(model.compat?.supportsDeveloperRole)
          .toBe(NEURALWATT_OPENAI_COMPLETIONS_COMPAT.supportsDeveloperRole);
        expect(model.compat?.supportsReasoningEffort)
          .toBe(NEURALWATT_OPENAI_COMPLETIONS_COMPAT.supportsReasoningEffort);
        expect(model.compat?.thinkingFormat)
          .toBe(NEURALWATT_OPENAI_COMPLETIONS_COMPAT.thinkingFormat);
      },
    }),
  ],
});

await describe({
  name: projectNeuralWattModels.name,
  children: [
    it({
      name: 'projects a facts list in listing order',
      fn: async () => {
        const models = projectNeuralWattModels({
          facts: [
            TEXT_ONLY_FACTS,
            {
              ...TEXT_ONLY_FACTS,
              id: 'fixture-model-b',
              name: 'Fixture Model B',
            },
          ],
        },);
        expect(models.map(function readId(model,) {
          return model.id;
        },)).toEqual([
          TEXT_ONLY_FACTS.id,
          'fixture-model-b',
        ]);
      },
    }),
  ],
});

//endregion Model projection
