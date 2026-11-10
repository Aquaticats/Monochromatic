/**
 Text-estimate cases captured from the former Pi 1.5.0 wrapper.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { estimateAdvisorInputTokens, } from '../dist/final/node/index.mjs';

//region Captured estimates

/**
 First nonempty token bucket, including the empty request's separator.
 */
const SINGLE_TOKEN = 1;

/**
 First token bucket after crossing the text estimator's boundary.
 */
const NEXT_TOKEN = SINGLE_TOKEN + SINGLE_TOKEN;

/**
 Recorded inputs distinguish separator overhead, combined rounding, UTF-16
 code units, non-Latin text, and caller-supplied line breaks.
 */
const CAPTURED_ESTIMATES = [
  { name: 'empty request retains separator overhead', systemPrompt: '', contextText: '', expected: SINGLE_TOKEN, },
  { name: 'rounds the combined prompt and context once', systemPrompt: 'a', contextText: 'b', expected: SINGLE_TOKEN, },
  { name: 'rounds a partial next token upward', systemPrompt: 'a', contextText: 'bc', expected: NEXT_TOKEN, },
  { name: 'counts a surrogate pair as UTF-16 code units', systemPrompt: '🙂', contextText: '', expected: SINGLE_TOKEN, },
  { name: 'surrogate pair and added character cross the boundary', systemPrompt: '🙂', contextText: 'a', expected: NEXT_TOKEN, },
  { name: 'retains the non-Latin text heuristic', systemPrompt: '漢字', contextText: '仮名', expected: NEXT_TOKEN, },
  { name: 'counts input line breaks in addition to the separator', systemPrompt: 'a\n', contextText: '\nb', expected: NEXT_TOKEN, },
] as const;

//endregion Captured estimates

await describe({
  name: estimateAdvisorInputTokens.name,
  children: CAPTURED_ESTIMATES.map(function capturedEstimate(entry) {
    return it({
      name: entry.name,
      fn: async function testEstimateAdvisorInputTokens(): Promise<void> {
        expect(estimateAdvisorInputTokens({
          systemPrompt: entry.systemPrompt,
          contextText: entry.contextText,
        },),).toBe(entry.expected,);
      },
    },);
  },),
},);
