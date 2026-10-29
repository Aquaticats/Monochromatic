/**
 Unit tests for the Advisor tool execution core.
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { SessionId, } from '@deepseek-ai/dsh-session';

import {
  executeDshAdvisorToolCall,
  type ExecuteDshAdvisorToolCallOptions,
} from '../dist/final/node/index.mjs';

/**
 Build one minimal tool-call execution fixture.

 @param options - fixture switches for the disabled and missing-agent branches

 @returns execution options with inert host surfaces

 @example
 ```ts
 const options = toolCallFixture({ disabled: true, agentAbsent: false });
 ```
 */
function toolCallFixture(
  options: {
    readonly disabled: boolean;
    readonly agentAbsent: boolean;
  },
): ExecuteDshAdvisorToolCallOptions {
  return {
    getConfig: () => Promise.resolve({} as Awaited<
      ReturnType<ExecuteDshAdvisorToolCallOptions['getConfig']>
      >),
    getSessionEnabled: () => !options.disabled,
    recordTranscriptDegradation: () => undefined,
    llm: {},
    systemPrompt: {
      assemble: () => ({
        sections: [],
      }),
    },
    workingDirectory: {
      get: () => '/work',
    },
    args: {
      question: 'q',
    },
    callId: 'call-1',
    signal: new AbortController().signal,
    ...(options.agentAbsent
      ? {}
      : {
        agent: {
          sessionId: SessionId('session-tool-call-fixture',),
          session: {
            deriveMessages: () => [],
          },
          cwd: '/work',
          options: {},
          whenIdle: () => Promise.resolve(),
          appendOperationRecord: () => undefined,
          assembleContext: () => ({}),
        },
      }),
  } as unknown as ExecuteDshAdvisorToolCallOptions;
}

await describe({
  name: executeDshAdvisorToolCall.name,
  children: [
    it({
      name: 'fails loud while Advisor is disabled for the session',
      fn: async () => {
        let thrown: unknown;
        try {
          await executeDshAdvisorToolCall(
            toolCallFixture({
              disabled: true,
              agentAbsent: false,
            },),
          );
        }
        catch (error) {
          thrown = error;
        }
        expect(
          (thrown as Error).message.includes('disabled for this session',),
        ).toBe(true,);
      },
    },),
    it({
      name: 'fails loud when the call has no agent-backed session',
      fn: async () => {
        let thrown: unknown;
        try {
          await executeDshAdvisorToolCall(
            toolCallFixture({
              disabled: false,
              agentAbsent: true,
            },),
          );
        }
        catch (error) {
          thrown = error;
        }
        expect(
          (thrown as Error).message.includes('no agent-backed session',),
        ).toBe(true,);
      },
    },),
  ],
},);
