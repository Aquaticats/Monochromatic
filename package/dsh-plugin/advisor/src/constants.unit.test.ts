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
      name: 'pins the never-sent max effort label',
      fn: async () => {
        expect(DSH_MAX_REASONING_EFFORT_LABEL,).toBe('max',);
      },
    },),
    it({
      name: 'pins the unknown context window substitute',
      fn: async () => {
        expect(DSH_UNKNOWN_CONTEXT_WINDOW_TOKENS,).toBe(128_000,);
      },
    },),
    it({
      name: 'pins the pi-ai model api tag and empty base url',
      fn: async () => {
        expect(DSH_MODEL_API,).toBe('dsh',);
        expect(DSH_MODEL_BASE_URL,).toBe('',);
      },
    },),
    it({
      name: 'pins the unknown tool-name fallback',
      fn: async () => {
        expect(DSH_UNKNOWN_TOOL_NAME,).toBe('unknown-tool',);
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
