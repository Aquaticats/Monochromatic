/**
 Tests for applying model-aware thinking defaults.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { applyThinkingDefault, } from './apply-thinking-default.ts';
import type { ThinkingDefaultLevel, } from './model-policy.ts';

//region Test helpers

/** Mutable fake thinking API state used by apply tests. */
type ThinkingHarness = {
  /** Reads current fake thinking level. */
  getThinkingLevel: () => string;
  /** Records requested fake thinking level changes. */
  setThinkingLevel: (level: ThinkingDefaultLevel,) => void;
  /** Ordered list of levels requested through `setThinkingLevel`. */
  setCalls: ThinkingDefaultLevel[];
};

/**
 Creates fake pi thinking accessors backed by a current-level value.
 
 @param currentLevel - current level returned by `getThinkingLevel`
 
 @returns fake accessors and recorded set calls
 
 @example
 ```typescript
 const harness = createThinkingHarness({ currentLevel: 'high' });
 ```
 */
function createThinkingHarness(
  {
    currentLevel,
  }: {
    currentLevel: string;
  },
): ThinkingHarness {
  /** Levels requested through the fake setter. */
  const setCalls: ThinkingDefaultLevel[] = [];
  return {
    getThinkingLevel: function getThinkingLevel(): string {
      return currentLevel;
    },
    setThinkingLevel: function setThinkingLevel(level: ThinkingDefaultLevel,): void {
      setCalls.push(level,);
    },
    setCalls,
  };
}

//endregion Test helpers

await describe({
  name: applyThinkingDefault.name,
  children: [
    it({
      name: 'does not change thinking when model is unavailable',
      fn: async function testNoModel() {
        const harness = createThinkingHarness({ currentLevel: 'high', },);

        const result = applyThinkingDefault({
          getThinkingLevel: harness.getThinkingLevel,
          setThinkingLevel: harness.setThinkingLevel,
        },);

        expect(result,).toEqual({ changed: false, },);
        expect(harness.setCalls,).toHaveLength(0,);
      },
    },),
    it({
      name: 'sets xhigh for GPT when current level is high',
      fn: async function testGptChangesFromHigh() {
        const harness = createThinkingHarness({ currentLevel: 'high', },);

        const result = applyThinkingDefault({
          model: { id: 'gpt-5.5', },
          getThinkingLevel: harness.getThinkingLevel,
          setThinkingLevel: harness.setThinkingLevel,
        },);

        expect(result,).toEqual({ changed: true, target: 'xhigh', },);
        expect(harness.setCalls,).toEqual(['xhigh',],);
      },
    },),
    it({
      name: 'does not set xhigh for GPT when already xhigh',
      fn: async function testGptAlreadyXhigh() {
        const harness = createThinkingHarness({ currentLevel: 'xhigh', },);

        const result = applyThinkingDefault({
          model: { id: 'openai/gpt-5.4', },
          getThinkingLevel: harness.getThinkingLevel,
          setThinkingLevel: harness.setThinkingLevel,
        },);

        expect(result,).toEqual({ changed: false, target: 'xhigh', },);
        expect(harness.setCalls,).toHaveLength(0,);
      },
    },),
    it({
      name: 'sets high for non-GPT when current level is xhigh',
      fn: async function testNonGptChangesFromXhigh() {
        const harness = createThinkingHarness({ currentLevel: 'xhigh', },);

        const result = applyThinkingDefault({
          model: { id: 'synthetic/hf:moonshotai/Kimi-K2.6', reasoning: true, },
          getThinkingLevel: harness.getThinkingLevel,
          setThinkingLevel: harness.setThinkingLevel,
        },);

        expect(result,).toEqual({ changed: true, target: 'high', },);
        expect(harness.setCalls,).toEqual(['high',],);
      },
    },),
    it({
      name: 'does not set high for non-GPT when already high',
      fn: async function testNonGptAlreadyHigh() {
        const harness = createThinkingHarness({ currentLevel: 'high', },);

        const result = applyThinkingDefault({
          model: { id: 'claude-sonnet-4-5', reasoning: true, },
          getThinkingLevel: harness.getThinkingLevel,
          setThinkingLevel: harness.setThinkingLevel,
        },);

        expect(result,).toEqual({ changed: false, target: 'high', },);
        expect(harness.setCalls,).toHaveLength(0,);
      },
    },),
    it({
      name: 'sets xhigh for non-GPT model that supports xhigh when current level is high',
      fn: async function testNonGptXhighAvailableChangesFromHigh() {
        const harness = createThinkingHarness({ currentLevel: 'high', },);

        const result = applyThinkingDefault({
          model: {
            id: 'synthetic/hf:zai-org/GLM-5.2',
            reasoning: true,
            thinkingLevelMap: { xhigh: 'max', },
          },
          getThinkingLevel: harness.getThinkingLevel,
          setThinkingLevel: harness.setThinkingLevel,
        },);

        expect(result,).toEqual({ changed: true, target: 'xhigh', },);
        expect(harness.setCalls,).toEqual(['xhigh',],);
      },
    },),
    it({
      name: 'does not set xhigh for non-GPT model that supports xhigh when already xhigh',
      fn: async function testNonGptXhighAvailableAlreadyXhigh() {
        const harness = createThinkingHarness({ currentLevel: 'xhigh', },);

        const result = applyThinkingDefault({
          model: {
            id: 'synthetic/hf:zai-org/GLM-5.2',
            reasoning: true,
            thinkingLevelMap: { xhigh: 'max', },
          },
          getThinkingLevel: harness.getThinkingLevel,
          setThinkingLevel: harness.setThinkingLevel,
        },);

        expect(result,).toEqual({ changed: false, target: 'xhigh', },);
        expect(harness.setCalls,).toHaveLength(0,);
      },
    },),
  ],
},);
