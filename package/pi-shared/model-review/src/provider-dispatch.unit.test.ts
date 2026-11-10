/**
 Provider dispatch coverage for the keyed lazy-provider table.

 The dispatch shape is data rather than branches, so these tests read the
 authored source for the single call site and drive the public API for the
 derived supported-API list.

 @module
 */

import { readFile, } from 'node:fs/promises';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { runStructuredToolRequest, } from '../dist/final/node/index.mjs';

/**
 Production provider dispatch source.
 */
const PROVIDER_STREAMS_SOURCE = new URL(
  'provider-streams.ts',
  import.meta.url,
);

/**
 Fixture reviewer context window.
 */
const CONTEXT_WINDOW = 128_000;

/**
 Fixture reviewer output capacity.
 */
const MAX_TOKENS = 16_384;

/**
 Fixture structured verdict tool.
 */
const TOOL = {
  name: 'submit_fixture_review',
  description: 'Submit fixture review.',
  parameters: {
    type: 'object',
    properties: {
      approved: { type: 'boolean', },
    },
    required: ['approved',],
  },
};

/**
 Fixture APIs the package must dispatch directly.
 */
const EXPECTED_DISPATCHED_APIS: readonly string[] = [
  'anthropic-messages',
  'azure-openai-responses',
  'bedrock-converse-stream',
  'google-generative-ai',
  'google-vertex',
  'mistral-conversations',
  'openai-codex-responses',
  'openai-completions',
  'openai-responses',
];

/**
 Capture an async error without promise matcher indirection.
 
 @param action - action expected to fail
 
 @returns thrown value
 
 @example
 ```ts
 await captureError(async () => { throw new Error('x'); });
 ```
 */
async function captureError(action: () => Promise<unknown>,): Promise<unknown> {
  try {
    await action();
  }
  catch (error) {
    return error;
  }
  throw new Error('expected action to fail',);
}

/**
 Count non-overlapping occurrences of one literal text.
 
 @param source - text to search
 
 @param needle - literal text to count
 
 @returns number of occurrences
 
 @example
 ```ts
 countOccurrences({ source: 'a b a', needle: 'a' });
 ```
 */
function countOccurrences(
  {
    source,
    needle,
  }: {
    readonly source: string;
    readonly needle: string;
  },
): number {
  return source.split(needle,).length - 1;
}

await describe({
  name: 'provider dispatch',
  children: [
    it({
      name: 'calls the lazy provider module from one shared dispatch site',
      fn: async () => {
        /**
         Authored provider dispatch source text.
         */
        const source = await readFile(
          PROVIDER_STREAMS_SOURCE,
          'utf8',
        );
        expect(countOccurrences({
          source,
          needle: '.streamSimple(',
        },),).toBe(1,);
        expect(countOccurrences({
          source,
          needle: 'if (model.api === ',
        },),).toBe(0,);
      },
    },),
    it({
      name: 'names every dispatched API when the selected model has no implementation',
      fn: async () => {
        /**
         Dispatch failure for an API the package does not implement.
         */
        const error = await captureError(function dispatchUnimplementedApi() {
          return runStructuredToolRequest({
            model: {
              id: 'reviewer',
              name: 'Reviewer',
              api: 'pi-messages',
              provider: 'test',
              baseUrl: 'https://example.invalid',
              reasoning: false,
              input: ['text',],
              cost: {
                input: 1,
                output: 1,
                cacheRead: 0,
                cacheWrite: 0,
              },
              contextWindow: CONTEXT_WINDOW,
              maxTokens: MAX_TOKENS,
            },
            auth: { apiKey: 'fixture-key', },
            prompt: {
              systemPrompt: 'Judge.',
              userContent: 'Evidence.',
            },
            tool: TOOL,
            toolName: TOOL.name,
            signal: AbortSignal.timeout(1_000,),
          },);
        },);
        expect(error,).toBeInstanceOf(Error,);
        /**
         Dispatch failure message listing supported APIs.
         */
        const { message, } = error as Error;
        expect(message,).toContain('No Pi AI API implementation available',);
        /**
         Supported-API list appended to the dispatch failure message.
         */
        const listedApis = message
          .slice(message.indexOf('Supported APIs: ',) + 'Supported APIs: '.length,)
          .split(', ',)
          .toSorted();
        expect(listedApis,).toEqual(EXPECTED_DISPATCHED_APIS.toSorted(),);
      },
    },),
  ],
},);
