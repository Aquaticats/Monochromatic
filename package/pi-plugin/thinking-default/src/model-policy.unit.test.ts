/**
 Tests for model-id thinking policy helpers.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  getModelIdLeaf,
  getThinkingDefaultForModel,
  isGptModelId,
  isXhighAvailable,
  type ThinkingLevelMapFragment,
} from './model-policy.ts';

//region Test cases

/** Model-id leaf extraction test case. */
type LeafCase = {
  /** Input model id. */
  modelId: string;
  /** Expected final slash-delimited segment. */
  expected: string;
};

/** GPT detection test case. */
type GptCase = {
  /** Input model id. */
  modelId: string;
  /** Expected GPT-shaped status. */
  expected: boolean;
};

/** Thinking default mapping test case. */
type DefaultCase = {
  /** Input model id. */
  modelId: string;
  /** Whether the model emits reasoning. */
  reasoning?: boolean;
  /** Provider thinking-level map fragment, when the model declares one. */
  thinkingLevelMap?: ThinkingLevelMapFragment;
  /** Expected thinking default. */
  expected: 'high' | 'xhigh';
};

/** `xhigh` availability test case. */
type XhighCase = {
  /** Whether the model emits reasoning. */
  reasoning?: boolean;
  /** Provider thinking-level map fragment, when the model declares one. */
  thinkingLevelMap?: ThinkingLevelMapFragment;
  /** Expected `xhigh` availability. */
  expected: boolean;
};

/** Cases covering bare, slash-prefixed, mixed-case, and non-GPT ids. */
const LEAF_CASES: readonly LeafCase[] = [
  { modelId: 'gpt-5.5', expected: 'gpt-5.5', },
  { modelId: 'openai/gpt-5.4', expected: 'gpt-5.4', },
  { modelId: 'synthetic/hf:moonshotai/Kimi-K2.6', expected: 'Kimi-K2.6', },
  { modelId: 'claude-sonnet-4-5', expected: 'claude-sonnet-4-5', },
];

/** Cases covering GPT-shaped and non-GPT-shaped ids. */
const GPT_CASES: readonly GptCase[] = [
  { modelId: 'gpt-5.5', expected: true, },
  { modelId: 'openai/gpt-5.4', expected: true, },
  { modelId: 'GPT-5.5', expected: true, },
  { modelId: 'synthetic/hf:moonshotai/Kimi-K2.6', expected: false, },
  { modelId: 'synthetic/hf:zai-org/GLM-5.1', expected: false, },
  { modelId: 'claude-sonnet-4-5', expected: false, },
];

/** Cases covering target level mapping from GPT detection and xhigh support. */
const DEFAULT_CASES: readonly DefaultCase[] = [
  { modelId: 'gpt-5.5', expected: 'xhigh', },
  { modelId: 'openai/gpt-5.4', expected: 'xhigh', },
  { modelId: 'synthetic/hf:moonshotai/Kimi-K2.6', reasoning: true, expected: 'high', },
  { modelId: 'claude-sonnet-4-5', reasoning: true, expected: 'high', },
  { modelId: 'synthetic/hf:zai-org/GLM-5.2', reasoning: true, thinkingLevelMap: { xhigh: 'max', }, expected: 'xhigh', },
  { modelId: 'some/xhigh-hidden', reasoning: true, thinkingLevelMap: { xhigh: null, }, expected: 'high', },
  { modelId: 'some/non-reasoning', reasoning: false, thinkingLevelMap: { xhigh: 'max', }, expected: 'high', },
];

/** Cases covering `xhigh` availability across reasoning and map variations. */
const XHIGH_CASES: readonly XhighCase[] = [
  { reasoning: true, thinkingLevelMap: { xhigh: 'max', }, expected: true, },
  { reasoning: true, thinkingLevelMap: { xhigh: 'high', }, expected: true, },
  { reasoning: true, expected: false, },
  { reasoning: true, thinkingLevelMap: { xhigh: null, }, expected: false, },
  { reasoning: false, thinkingLevelMap: { xhigh: 'max', }, expected: false, },
  { expected: false, },
];

//endregion Test cases

//region Test helpers

/** Fixed id for capability-only cases where the id is irrelevant. */
const CAPABILITY_TEST_MODEL_ID = 'any/model';

//endregion Test helpers

await describe({
  name: '',
  children: [
    describe({
      name: getModelIdLeaf.name,
      children: LEAF_CASES.map(function createLeafCase(testCase,) {
        return it({
          name: `returns ${testCase.expected} for ${testCase.modelId}`,
          fn: async function runLeafCase() {
            expect(getModelIdLeaf({ modelId: testCase.modelId, },),).toBe(
              testCase.expected,
            );
          },
        },);
      },),
    },),
    describe({
      name: isGptModelId.name,
      children: GPT_CASES.map(function createGptCase(testCase,) {
        return it({
          name: `returns ${String(testCase.expected,)} for ${testCase.modelId}`,
          fn: async function runGptCase() {
            expect(isGptModelId({ modelId: testCase.modelId, },),).toBe(
              testCase.expected,
            );
          },
        },);
      },),
    },),
    describe({
      name: getThinkingDefaultForModel.name,
      children: DEFAULT_CASES.map(function createDefaultCase(testCase,) {
        return it({
          name: `returns ${testCase.expected} for ${testCase.modelId}`,
          fn: async function runDefaultCase() {
            expect(
              getThinkingDefaultForModel({
                model: {
                  id: testCase.modelId,
                  ...(testCase.reasoning !== undefined ? { reasoning: testCase.reasoning, } : {}),
                  ...(testCase.thinkingLevelMap !== undefined ? { thinkingLevelMap: testCase.thinkingLevelMap, } : {}),
                },
              },),
            )
              .toBe(testCase.expected,);
          },
        },);
      },),
    },),
    describe({
      name: isXhighAvailable.name,
      children: XHIGH_CASES.map(function createXhighCase(testCase,) {
        return it({
          name: `returns ${String(testCase.expected,)} for reasoning=${String(testCase.reasoning,)} xhigh=${String(testCase.thinkingLevelMap?.xhigh,)}`,
          fn: async function runXhighCase() {
            expect(
              isXhighAvailable({
                model: {
                  id: CAPABILITY_TEST_MODEL_ID,
                  ...(testCase.reasoning !== undefined ? { reasoning: testCase.reasoning, } : {}),
                  ...(testCase.thinkingLevelMap !== undefined ? { thinkingLevelMap: testCase.thinkingLevelMap, } : {}),
                },
              },),
            )
              .toBe(testCase.expected,);
          },
        },);
      },),
    },),
  ],
},);
