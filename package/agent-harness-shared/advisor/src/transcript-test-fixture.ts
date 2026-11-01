/**
 Real transcript serialization seam for Advisor core tests.

 Mirrors the pi adapter's wiring so moved tests keep asserting against the
 same pi-coding-agent conversion and serialization the pre-extraction code
 used directly. @module
 */

import {
  convertToLlm,
  serializeConversation,
} from '@earendil-works/pi-coding-agent';
import { estimateAdvisorInputTokens, } from '@monochromatic-dev/agent-harness-shared-model-selection/ts';
import type { AdvisorTranscriptSerialization, } from '../dist/final/node/index.mjs';

/**
 Serialization seam using pi-coding-agent conversion and shared token estimation.
 */
export const testAdvisorTranscriptSerialization: AdvisorTranscriptSerialization = {
  convertToLlm,
  serializeConversation,
  estimateInputTokens: estimateAdvisorInputTokens,
};
