import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  expandQwenTokenPlanThinkingLevelMap,
  QWEN_TOKEN_PLAN_THINKING_LEVELS,
} from '../dist/final/node/index.mjs';

await describe({
  name: expandQwenTokenPlanThinkingLevelMap.name,
  children: [
    it({
      name: 'pins every undeclared level to null',
      fn: async () => {
        expect(expandQwenTokenPlanThinkingLevelMap({
          efforts: { high: 'high' },
        })).toEqual({
          off: null,
          minimal: null,
          low: null,
          medium: null,
          high: 'high',
          xhigh: null,
          max: null,
        });
      },
    }),
    it({
      name: 'carries the wire spelling each declared level names',
      fn: async () => {
        expect(expandQwenTokenPlanThinkingLevelMap({
          efforts: { off: 'none', low: 'low', xhigh: 'xhigh' },
        })).toEqual({
          off: 'none',
          minimal: null,
          low: 'low',
          medium: null,
          high: null,
          xhigh: 'xhigh',
          max: null,
        });
      },
    }),
    it({
      name: 'keeps a declared null absent so dispatch sends nothing',
      fn: async () => {
        const map = expandQwenTokenPlanThinkingLevelMap({
          efforts: { off: null, high: 'high' },
        });
        expect(Object.hasOwn(map, 'off')).toBe(false);
        expect(map.high).toBe('high');
        expect(map.minimal).toBe(null);
      },
    }),
    it({
      name: 'decides every level of the vocabulary for a silent dict',
      fn: async () => {
        const map = expandQwenTokenPlanThinkingLevelMap({ efforts: {} });
        expect(Object.keys(map)).toHaveLength(
          QWEN_TOKEN_PLAN_THINKING_LEVELS.length,
        );
        for (const level of QWEN_TOKEN_PLAN_THINKING_LEVELS) {
          expect(map[level]).toBe(null);
        }
      },
    }),
    it({
      name: 'pins undeclared levels even when the dict carries only off',
      fn: async () => {
        expect(expandQwenTokenPlanThinkingLevelMap({
          efforts: { off: 'none' },
        })).toEqual({
          off: 'none',
          minimal: null,
          low: null,
          medium: null,
          high: null,
          xhigh: null,
          max: null,
        });
      },
    }),
  ],
});
