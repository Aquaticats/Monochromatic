/**
 Tests for visible Morph Compact context injection during session start.
 
 @module
 */

import type {
  ContextEvent,
  ExtensionAPI,
} from '@earendil-works/pi-coding-agent';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { writeCompactFile, } from './ipc-file.ts';
import { handleSessionStartInject, } from './ipc-launch.ts';
import {
  buildVisibleContextMessage,
  filterVisibleContextMessages,
} from './visible-context.ts';

//region Test helpers

/**
 API surface exercised by {@link handleSessionStartInject} in these tests.
 */
type InjectionApi = Pick<
  ExtensionAPI,
  'getFlag' | 'sendMessage' | 'sendUserMessage'
  >;

/**
 Build an extension API stub with only injection-related methods.
 
 @param filePath - compact payload path returned for the file-tier flag
 
 @param sendMessage - spy capturing visible custom messages
 
 @param sendUserMessage - spy capturing agent-facing user messages
 
 @returns minimal extension API understood by session-start injection
 
 @example
 ```typescript
 const api = createInjectionApi({ filePath, sendMessage, sendUserMessage });
 await handleSessionStartInject(api);
 ```
 */
function createInjectionApi(
  {
    filePath,
    sendMessage,
    sendUserMessage,
  }: {
    readonly filePath: string;
    readonly sendMessage: InjectionApi['sendMessage'];
    readonly sendUserMessage: InjectionApi['sendUserMessage'];
  },
): InjectionApi {
  return {
    getFlag: function getFlag(name,): boolean | string {
      if (name
        === 'morph-compact-file')
        return filePath;
      return false;
    },
    sendMessage,
    sendUserMessage,
  };
}

//endregion Test helpers

await describe({
  name: '',
  children: [
    describe({
      name: filterVisibleContextMessages.name,
      children: [
        it({
          name: 'removes visible transcript markers from agent context',
          fn: async () => {
            /**
             Full compacted context that should reach the agent only once,
             through the actual user message, not through the visible marker.
             */
            const text = 'Morph Compact hidden-from-agent sentinel';
            /**
             Visible custom-message payload created by production code.
             */
            const visibleMessage = buildVisibleContextMessage({ text, },);
            /**
             User message that represents the real agent-facing context.
             */
            const userMessage = {
              role: 'user',
              content: [
                {
                  type: 'text',
                  text,
                },
              ],
              timestamp: Date.now(),
            };
            /**
             Context event messages before filtering.
             */
            const messages = [
              {
                role: 'custom',
                customType: visibleMessage.customType,
                content: visibleMessage.content,
                display: visibleMessage.display,
                details: visibleMessage.details,
                timestamp: Date.now(),
              },
              userMessage,
            ] as ContextEvent['messages'];

            const filtered = filterVisibleContextMessages({ messages, },);

            expect(filtered,).toEqual([userMessage,],);
          },
        },),
      ],
    },),
    describe({
      name: handleSessionStartInject.name,
      children: [
        it({
          name: 'shows restored compact context in the UI before delivering it to the agent',
          fn: async ({ sinon, },) => {
            /**
             Compact payload used to prove both visible and agent-facing paths.
             */
            const text = [
              'Morph Compact probe sentinel.',
              '<morph-compacted-history>',
              'Earlier context line.',
              '</morph-compacted-history>',
            ].join('\n',);
            /**
             Temp file created through the same helper used by production code.
             */
            const compactFile = await writeCompactFile(text,);
            await using file = {
              filePath: compactFile.filePath,
              [Symbol.asyncDispose]: compactFile.cleanup,
            };
            /**
             Target function whose calls are captured by the visible-message spy.
             
             @param _message - visible custom message payload
             
             @param _options - custom-message delivery options
             */
            function sendMessageTarget(
              _message: Parameters<InjectionApi['sendMessage']>[0],
              _options?: Parameters<InjectionApi['sendMessage']>[1],
            ): void {}
            /**
             Target function whose calls are captured by the agent-message spy.
             
             @param _content - agent-facing user message content
             
             @param _options - user-message delivery options
             */
            function sendUserMessageTarget(
              _content: Parameters<InjectionApi['sendUserMessage']>[0],
              _options?: Parameters<InjectionApi['sendUserMessage']>[1],
            ): void {}
            /**
             Visible custom-message spy.
             */
            const sendMessage = sinon.spy(sendMessageTarget,);
            /**
             Agent-facing user-message spy.
             */
            const sendUserMessage = sinon.spy(sendUserMessageTarget,);
            /**
             Minimal API carrying file-tier flag and message methods.
             */
            const api = createInjectionApi({
              filePath: file.filePath,
              sendMessage,
              sendUserMessage,
            },);

            await handleSessionStartInject(api,);

            sinon.assert.calledOnce(sendMessage,);
            sinon.assert.calledOnceWithExactly(
              sendUserMessage,
              text,
            );
            sinon.assert.callOrder(
              sendMessage,
              sendUserMessage,
            );

            /**
             Custom message passed to pi's visible transcript API.
             */
            const customMessage = sendMessage.firstCall
              .args[0] as {
                customType?: unknown;
                content?: unknown;
                display?: unknown;
                details?: unknown;
              };
            expect(customMessage.customType,).toBe('morph-compact-context',);
            expect(customMessage.display,).toBe(true,);
            expect(customMessage.content,).toContain('Morph Compact restored',);

            /**
             Renderer details carry full compacted text without duplicating it
             into custom-message content.
             */
            const details = customMessage.details as {
              text?: unknown;
            };
            expect(details.text,).toBe(text,);
          },
        },),
      ],
    },),
  ],
},);
