/**
 Unit tests for optional pi-coding-agent wrappers.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { estimateAdvisorInputTokens, } from '@monochromatic-dev/agent-harness-shared-model-selection';

await describe({
  name: estimateAdvisorInputTokens.name,
  children: [
    it({
      name: 'returns a positive token estimate for Advisor-shaped input',
      fn: async function testEstimateAdvisorInputTokens() {
        expect(estimateAdvisorInputTokens({
          systemPrompt: 'review carefully',
          contextText: 'hello world',
        },),)
          .toBeGreaterThan(0,);
      },
    },),
  ],
},);
