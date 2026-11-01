/**
 Unit tests for Advisor conversation text extraction and serialization.
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  ADVISOR_TOOL_RESULT_MAX_CHARS,
  advisorContentText,
  serializeAdvisorConversationText,
  truncateAdvisorSummaryText,
} from '../dist/final/node/index.mjs';

/**
 Type of the serialized conversation input array.

 Mirrors the pi message projection the serializer accepts.
 */
type SerializedMessages = Parameters<typeof serializeAdvisorConversationText>[0];

await describe({
  name: advisorContentText.name,
  children: [
    it({
      name: 'passes string content through unchanged',
      fn: async () => {
        expect(
          advisorContentText({
            content: 'plain',
          },),
        ).toBe('plain',);
      },
    },),
    it({
      name: 'joins text blocks with the requested separator',
      fn: async () => {
        expect(
          advisorContentText({
            content: [
              {
                type: 'text',
                text: 'a',
              },
              {
                type: 'thinking',
                thinking: 'hidden',
              },
              {
                type: 'text',
                text: 'b',
              },
            ],
            separator: '|',
          },),
        ).toBe('a|b',);
      },
    },),
  ],
},);

await describe({
  name: truncateAdvisorSummaryText.name,
  children: [
    it({
      name: 'returns the original text under budget',
      fn: async () => {
        expect(
          truncateAdvisorSummaryText({
            text: 'short',
            maxChars: 10,
          },),
        ).toBe('short',);
      },
    },),
    it({
      name: 'slices over-budget text and records the dropped count',
      fn: async () => {
        const bounded = truncateAdvisorSummaryText({
          text: 'abcdefghij',
          maxChars: 4,
        },);
        expect(bounded.startsWith('abcd',),).toBe(true,);
        expect(bounded.includes('6 more characters truncated',),).toBe(true,);
      },
    },),
  ],
},);

await describe({
  name: serializeAdvisorConversationText.name,
  children: [
    it({
      name: 'renders user, assistant, tool-call, and tool-result paragraphs',
      fn: async () => {
        const text = serializeAdvisorConversationText([
          {
            role: 'system',
            content: 'prompt',
          },
          {
            role: 'user',
            content: [
              {
                type: 'text',
                text: 'hello',
              },
            ],
          },
          {
            role: 'assistant',
            content: [
              {
                type: 'text',
                text: 'answer',
              },
              {
                type: 'toolCall',
                id: 'call-1',
                name: 'advisor',
                arguments: {
                  question: 'q',
                },
              },
            ],
          },
          {
            role: 'toolResult',
            toolCallId: 'call-1',
            toolName: 'advisor',
            content: [
              {
                type: 'text',
                text: 'result',
              },
            ],
          },
        ] as unknown as SerializedMessages,);
        expect(text,).toBe(
          '[User]: hello\n\n[Assistant]: answer\n\n[Assistant tool calls]: advisor(question="q")\n\n[Tool result]: result',
        );
      },
    },),
    it({
      name: 'skips system messages and empty user paragraphs',
      fn: async () => {
        const text = serializeAdvisorConversationText([
          {
            role: 'system',
            content: 'prompt',
          },
          {
            role: 'user',
            content: [],
          },
        ] as unknown as SerializedMessages,);
        expect(text,).toBe('',);
      },
    },),
    it({
      name: 'bounds tool results at the advisor tool-result budget',
      fn: async () => {
        const text = serializeAdvisorConversationText([
          {
            role: 'toolResult',
            toolCallId: 'call-1',
            toolName: 'advisor',
            content: [
              {
                type: 'text',
                text: 'x'.repeat(ADVISOR_TOOL_RESULT_MAX_CHARS + 25,),
              },
            ],
          },
        ] as unknown as SerializedMessages,);
        expect(
          text.includes('more characters truncated',),
        ).toBe(true,);
        expect(text.length < (ADVISOR_TOOL_RESULT_MAX_CHARS + 100),).toBe(true,);
      },
    },),
  ],
},);
