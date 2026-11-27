/**
 The request size the router compares with a member's own window.
 @module
 */

import {
  normalizeContext,
  type AssistantMessage,
  type TranscriptContext,
} from '@earendil-works/pi-ai';
import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import { estimateRequestTokens, } from '../dist/final/node/index.mjs';

/**
 Characters per token Pi's estimator counts prose at (`@earendil-works/pi-ai` dist/utils/estimate.js,
 `CHARS_PER_TOKEN`). The router believes whatever a member reported and falls back to this ratio, so a
 change to it here is a change to what the pool's window comparison means.
 */
const PI_CHARS_PER_TOKEN = 3.5;

/**
 Text of a known length, made up: it stands for no program's output and exists to give a request a
 size.
 */
const PADDING = 'x'.repeat(Math.ceil(PI_CHARS_PER_TOKEN * 1_000,),);

/**
 A completed answer reporting a token count, as a member reports one.

 Stand-in: the count is this fixture's, and it is here because Pi's estimator takes a member's own
 report verbatim. Boundary: a member that answered how large the request was.

 @param totalTokens - count the answer reports

 @param stopReason - how that answer ended

 @returns assistant message carrying that count
 */
function answerReporting({
  totalTokens,
  stopReason = 'stop',
}: {
  readonly totalTokens: number;
  readonly stopReason?: AssistantMessage['stopReason'];
},): AssistantMessage {
  return {
    role: 'assistant',
    api: 'openai-completions',
    provider: 'stand-in',
    model: 'stand-in',
    content: [],
    usage: {
      input: totalTokens,
      output: 0,
      cacheRead: 0,
      cacheWrite: 0,
      totalTokens,
      cost: {
        input: 0,
        output: 0,
        cacheRead: 0,
        cacheWrite: 0,
        total: 0,
      },
    },
    stopReason,
    timestamp: 1,
  };
}

/**
 One transcript: a system message, a user message the size under test, and an optional answer.

 @param parts - messages after the system message

 @returns normalized transcript, as Pi hands it to a provider
 */
function transcriptOf(parts: Parameters<typeof normalizeContext>[0]['messages'],): TranscriptContext {
  return normalizeContext({
    messages: [
      {
        role: 'system',
        content: '',
        timestamp: 0,
      },
      ...parts,
    ],
  },);
}

await describe({
  name: estimateRequestTokens.name,
  children: [
    it({
      name: 'counts every message at Pi’s own ratio when no answer reported a count',
      fn: async function countedFromText() {
        const transcript = transcriptOf([
          {
            role: 'user',
            content: PADDING,
            timestamp: 0,
          },
        ],);
        expect(estimateRequestTokens(transcript,),).toBe(
          Math.ceil(PADDING.length / PI_CHARS_PER_TOKEN,),
        );
      },
    },),
    it({
      name: 'takes the last answer’s own count and counts only what came after it',
      fn: async function anchoredOnTheMember() {
        /**
         Count the stand-in answer reports, the request's size up to that answer.
         */
        const reported = 1_500;
        const transcript = transcriptOf([
          {
            role: 'user',
            content: 'Fixtures request',
            timestamp: 0,
          },
          answerReporting({ totalTokens: reported, },),
          {
            role: 'user',
            content: PADDING,
            timestamp: 2,
          },
        ],);
        // The member's own count is used as it reported it, so a conversation that already had an
        // answer is sized from that answer rather than from an estimate of the whole transcript.
        expect(estimateRequestTokens(transcript,),).toBe(
          reported + Math.ceil(PADDING.length / PI_CHARS_PER_TOKEN,),
        );
      },
    },),
    it({
      name: 'ignores an answer that failed, so a refused request is counted from its text',
      fn: async function failedAnswerNotAnAnchor() {
        /**
         Count the failed answer claims, which must not stand for the request's size.
         */
        const claimed = 1_000_000;
        const transcript = transcriptOf([
          {
            role: 'user',
            content: 'Fixtures request',
            timestamp: 0,
          },
          answerReporting({ totalTokens: claimed, stopReason: 'error', },),
          {
            role: 'user',
            content: PADDING,
            timestamp: 2,
          },
        ],);
        const estimated = estimateRequestTokens(transcript,);
        expect(estimated,).toBeLessThan(claimed,);
        expect(estimated,).toBe(Math.ceil(('Fixtures request'.length + PADDING.length) / PI_CHARS_PER_TOKEN,),);
      },
    },),
  ],
},);
