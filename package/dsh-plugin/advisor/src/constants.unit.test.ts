/**
 Unit tests for the DSH Advisor constants module.
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  DSH_CREDIT_EXHAUSTION_CODES,
  DSH_CREDIT_EXHAUSTION_MARKER,
  DSH_MAX_REASONING_EFFORT_LABEL,
  DSH_MODEL_API,
  DSH_MODEL_BASE_URL,
  DSH_UNKNOWN_CONTEXT_WINDOW_TOKENS,
  DSH_UNKNOWN_TOOL_NAME,
} from '../dist/final/node/index.mjs';

await describe({
  name: 'DSH Advisor constants',
  children: [
    it({
      name: 'keeps a non-empty never-sent max effort label',
      fn: async () => {
        expect(DSH_MAX_REASONING_EFFORT_LABEL.length,).toBeGreaterThan(0,);
      },
    },),
    it({
      name: 'keeps a positive unknown context window substitute',
      fn: async () => {
        expect(DSH_UNKNOWN_CONTEXT_WINDOW_TOKENS,).toBeGreaterThan(0,);
      },
    },),
    it({
      name: 'carries a model api tag and no base url',
      fn: async () => {
        expect(DSH_MODEL_API.length,).toBeGreaterThan(0,);
        expect(DSH_MODEL_BASE_URL,).toBe('',);
      },
    },),
    it({
      name: 'keeps a non-empty unknown tool-name fallback',
      fn: async () => {
        expect(DSH_UNKNOWN_TOOL_NAME.length,).toBeGreaterThan(0,);
      },
    },),
    it({
      name: 'pins the credit-exhaustion codes and marker',
      fn: async () => {
        expect([...DSH_CREDIT_EXHAUSTION_CODES,],).toEqual([
          'QUOTA',
          'ACCOUNT_QUOTA',
        ],);
        expect(DSH_CREDIT_EXHAUSTION_MARKER,).toBe('insufficient_quota',);
      },
    },),
  ],
},);
