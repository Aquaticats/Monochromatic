/**
 Unit tests for the shared Advisor review pipeline wiring.
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import type {
  StreamChunk,
} from '@deepseek-ai/dsh-llm';

import { SessionId, } from '@deepseek-ai/dsh-session';

import {
  DEFAULT_CONFIG,
  type AdvisorOperationSnapshot,
} from '@monochromatic-dev/agent-harness-shared-advisor/ts';

import {
  runDshAdvisorReview,
  type DshAdvisorAgentSurface,
  type DshAdvisorLlmSurface,
} from '../dist/final/node/index.mjs';

/**
 Advisor config fixture for the review pipeline.
 */
const advisorConfig = {
  ...DEFAULT_CONFIG,
  timeoutMs: 5_000,
  maxAdvisorOutputTokens: 100,
} as unknown as Parameters<typeof runDshAdvisorReview>[0]['config'];

/**
 Scripted chunks completing one review with canned text.
 */
const SUCCESSFUL_CHUNKS: readonly StreamChunk[] = [
  {
    type: 'block-start',
    index: 0,
    blockType: 'text',
  },
  {
    type: 'text-delta',
    index: 0,
    text: 'review answer',
  },
  {
    type: 'block-end',
    index: 0,
    block: {
      type: 'text',
      text: 'review answer',
    },
  },
  {
    type: 'finish',
    reason: { kind: 'stop', },
  },
];

/**
 Scripted chunks failing the only attempt at the provider.
 */
const FAILING_CHUNKS: readonly StreamChunk[] = [
  {
    type: 'finish',
    reason: {
      kind: 'error',
      failure: {
        code: 'transport',
        message: 'scripted transport failure',
      },
    },
  },
];

/**
 Build one scripted LLM surface exposing a single reviewer route.

 @param chunks - stream chunks returned for every request

 @returns LLM surface whose single route replays the canned chunks

 @example
 ```ts
 const llm = scriptedLlmSurface(SUCCESSFUL_CHUNKS);
 ```
 */
function scriptedLlmSurface(
  chunks: readonly StreamChunk[],
): DshAdvisorLlmSurface {
  return {
    listProviders: function listProviders() {
      return [{
        id: 'fake',
        name: 'Fake',
      },];
    },
    listModels: async function listModels() {
      return [{
        provider: 'fake',
        id: 'reviewer',
        name: 'Reviewer',
      },];
    },
    resolveModelInfo: async function resolveModelInfo() {
      return {
        provider: 'fake',
        id: 'reviewer',
        name: 'Reviewer',
        context: { contextWindow: 100_000, },
        defaultMaxTokens: 1_000,
      };
    },
    stream: function stream() {
      return (async function* replayChunks(): AsyncGenerator<StreamChunk> {
        for (const chunk of chunks)
          yield chunk;
      })();
    },
  } as unknown as DshAdvisorLlmSurface;
}

/**
 Build one review-pipeline agent surface recording persisted operation ledgers.

 @param persisted - mutable sink collecting appended operation records

 @returns agent surface with inert host behaviors and a recording ledger sink

 @example
 ```ts
 const agent = recordingAgentSurface(persisted);
 ```
 */
function recordingAgentSurface(
  persisted: AdvisorOperationSnapshot[],
): DshAdvisorAgentSurface {
  return {
    sessionId: SessionId('session-run-review-test',),
    session: {
      deriveMessages: () => [],
    },
    cwd: '/repo',
    options: {
      provider: 'fake',
      model: 'reviewer',
    },
    whenIdle: () => Promise.resolve(),
    appendOperationRecord: function appendOperationRecord(
      operation: AdvisorOperationSnapshot,
    ): void {
      persisted.push(operation,);
    },
    assembleContext: () => ({}),
  } as unknown as DshAdvisorAgentSurface;
}

await describe({
  name: runDshAdvisorReview.name,
  children: [
    it({
      name: 'persists the operation ledger when the review completes',
      fn: async () => {
        /** Operation records appended through the agent surface. */
        const persisted: AdvisorOperationSnapshot[] = [];
        const result = await runDshAdvisorReview({
          config: advisorConfig,
          llm: scriptedLlmSurface(SUCCESSFUL_CHUNKS,),
          systemPrompt: {
            assemble: () => ({ sections: [], }),
          } as unknown as Parameters<typeof runDshAdvisorReview>[0]['systemPrompt'],
          agent: recordingAgentSurface(persisted,),
          recordTranscriptDegradation: () => undefined,
        },);

        expect(result.result.text,).toContain('review answer',);
        expect(persisted,).toHaveLength(1,);
        expect(persisted[0]?.attempts
          .length,).toBe(1,);
        expect(persisted[0]?.attempts[0]?.state,).toBe('succeeded',);
        expect(persisted[0]?.reviews,).toHaveLength(1,);
      },
    },),
    it({
      name: 'persists no ledger when every attempt fails',
      fn: async () => {
        /** Operation records appended through the agent surface. */
        const persisted: AdvisorOperationSnapshot[] = [];
        let thrown: unknown;
        try {
          await runDshAdvisorReview({
            config: advisorConfig,
            llm: scriptedLlmSurface(FAILING_CHUNKS,),
            systemPrompt: {
              assemble: () => ({ sections: [], }),
            } as unknown as Parameters<typeof runDshAdvisorReview>[0]['systemPrompt'],
            agent: recordingAgentSurface(persisted,),
            recordTranscriptDegradation: () => undefined,
          },);
        }
        catch (error) {
          thrown = error;
        }

        expect(thrown,).toBeInstanceOf(Error,);
        expect(persisted,).toHaveLength(0,);
      },
    },),
  ],
},);
