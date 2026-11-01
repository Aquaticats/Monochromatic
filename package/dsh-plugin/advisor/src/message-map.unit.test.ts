/**
 Unit tests for the DSH message, content, and tool-argument projections.
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  MessageId,
  ToolCallId,
  type ContentBlock,
  type Message as DshMessage,
} from '@deepseek-ai/dsh-llm';

import {
  collectAdvisorToolNames,
  emptyAdvisorUsage,
  isPiAgentMessage,
  mapDshAssistantContent,
  mapDshDisplayContent,
  mapDshMessageToAdvisorMessage,
  parseAdvisorToolArguments,
} from '../dist/final/node/index.mjs';

/**
 Build one text display block.

 @param text - block text

 @returns text content block

 @example
 ```ts
 const block = textBlock('hello');
 ```
 */
function textBlock(text: string): ContentBlock {
  return {
    type: 'text',
    text,
  };
}

/**
 Build one image display block with the given attachment facts.

 @param name - optional display name of the attachment

 @returns image content block carrying a synthetic attachment id

 @example
 ```ts
 const block = imageBlock('shot.png');
 ```
 */
function imageBlock(name?: string): ContentBlock {
  return {
    type: 'image',
    attachment: {
      attachmentId: 'attachment-1',
      mediaType: 'image/png',
      bytes: 10,
      width: 4,
      height: 2,
      ...(name === undefined ? {} : { name, }),
    },
  } as unknown as ContentBlock;
}

/**
 Build one file display block.

 @returns file content block carrying a synthetic attachment id

 @example
 ```ts
 const block = fileBlock();
 ```
 */
function fileBlock(): ContentBlock {
  return {
    type: 'file',
    attachment: {
      attachmentId: 'attachment-2',
      name: 'log.txt',
      bytes: 12,
    },
  } as unknown as ContentBlock;
}

await describe({
  name: parseAdvisorToolArguments.name,
  children: [
    it({
      name: 'returns the parsed object arguments',
      fn: async () => {
        const parsed = parseAdvisorToolArguments('{"model":"p/m","question":"q"}',);
        expect(parsed.arguments,).toEqual({
          model: 'p/m',
          question: 'q',
        },);
        expect('argumentsError' in parsed,).toBe(false,);
      },
    },),
    it({
      name: 'records a diagnostic on malformed JSON',
      fn: async () => {
        const parsed = parseAdvisorToolArguments('{oops',);
        expect(parsed.arguments,).toEqual({},);
        expect(
          parsed.argumentsError?.includes('arguments were not valid JSON',),
        ).toBe(true,);
      },
    },),
    it({
      name: 'records a diagnostic on non-object JSON',
      fn: async () => {
        const arrayParse = parseAdvisorToolArguments('[1,2]',);
        expect(arrayParse.arguments,).toEqual({},);
        expect(
          arrayParse.argumentsError?.includes('arguments were not a JSON object',),
        ).toBe(true,);
        expect(
          parseAdvisorToolArguments('42',).argumentsError
            ?.includes('arguments were not a JSON object',),
        ).toBe(true,);
      },
    },),
  ],
},);

await describe({
  name: mapDshDisplayContent.name,
  children: [
    it({
      name: 'renders text blocks and attachment placeholders',
      fn: async () => {
        const blocks = mapDshDisplayContent([
          textBlock('hello',),
          imageBlock('shot.png',),
          fileBlock(),
        ],);
        expect(blocks.map((block,) => block.text,),).toEqual([
          'hello',
          '[image: shot.png (4x2, image/png)]',
          '[file: log.txt (12 bytes)]',
        ],);
      },
    },),
    it({
      name: 'falls back to a generic image name when the attachment has none',
      fn: async () => {
        const blocks = mapDshDisplayContent([imageBlock(),],);
        expect(blocks[0]?.text.includes('attached image',),).toBe(true,);
      },
    },),
    it({
      name: 'drops reasoning and tool-registration blocks from display text',
      fn: async () => {
        const blocks = mapDshDisplayContent([
          {
            type: 'reasoning',
            text: 'hidden thoughts',
          },
          {
            type: 'tool-addition',
            toolName: 'tool',
          },
          {
            type: 'tool-removal',
            toolName: 'tool',
          },
          textBlock('kept',),
        ],);
        expect(blocks.map((block,) => block.text,),).toEqual(['kept',],);
      },
    },),
  ],
},);

await describe({
  name: mapDshAssistantContent.name,
  children: [
    it({
      name: 'maps text, reasoning, and tool calls with parsed arguments',
      fn: async () => {
        const content = mapDshAssistantContent([
          textBlock('answer',),
          {
            type: 'reasoning',
            text: 'thought',
          },
          {
            type: 'tool-call',
            id: ToolCallId('call-1',),
            name: 'advisor',
            arguments: '{"question":"q"}',
          },
        ],);
        expect(content.map((block,) => block.type,),).toEqual([
          'text',
          'thinking',
          'toolCall',
        ],);
        const toolCall = content.at(2,);
        expect(toolCall?.type === 'toolCall' ? toolCall.arguments : undefined,)
          .toEqual(
          {
            question: 'q',
          },
          );
      },
    },),
    it({
      name: 'attaches a parse diagnostic to malformed tool arguments',
      fn: async () => {
        const content = mapDshAssistantContent([
          {
            type: 'tool-call',
            id: ToolCallId('call-2',),
            name: 'advisor',
            arguments: '{bad',
          },
        ],);
        const toolCall = content.at(0,);
        expect(
          toolCall?.type === 'toolCall'
            ? toolCall.argumentsError?.includes('arguments were not valid JSON',)
            : undefined,
        ).toBe(true,);
      },
    },),
    it({
      name: 'drops display-only and tool-registration blocks from assistant content',
      fn: async () => {
        const content = mapDshAssistantContent([
          imageBlock('a.png',),
          {
            type: 'tool-addition',
            toolName: 'tool',
          },
          textBlock('kept',),
        ],);
        expect(content.map((block,) => block.type,),).toEqual(['text',],);
      },
    },),
  ],
},);

await describe({
  name: emptyAdvisorUsage.name,
  children: [
    it({
      name: 'returns a fully zeroed usage record',
      fn: async () => {
        const usage = emptyAdvisorUsage();
        expect(usage.totalTokens,).toBe(0,);
        expect(usage.cost.total,).toBe(0,);
        expect(usage.cacheRead,).toBe(0,);
      },
    },),
  ],
},);

await describe({
  name: mapDshMessageToAdvisorMessage.name,
  children: [
    it({
      name: 'maps system and developer messages to the system role',
      fn: async () => {
        const system = mapDshMessageToAdvisorMessage({
          message: {
            id: MessageId('m1',),
            role: 'system',
            content: [textBlock('prompt',),],
            source: {
              kind: 'system-prompt',
            },
          },
          toolNamesByCallId: new Map(),
          timestampMs: 42,
        },);
        expect(system.role,).toBe('system',);
        const developer = mapDshMessageToAdvisorMessage({
          message: {
            id: MessageId('m2',),
            role: 'developer',
            content: [textBlock('note',),],
            source: {
              kind: 'system-prompt',
            },
          },
          toolNamesByCallId: new Map(),
          timestampMs: 42,
        },);
        expect(developer.role,).toBe('system',);
      },
    },),
    it({
      name: 'maps user messages through the display projection',
      fn: async () => {
        const user = mapDshMessageToAdvisorMessage({
          message: {
            id: MessageId('m3',),
            role: 'user',
            content: [textBlock('hi',),],
            source: {
              kind: 'user',
            },
          },
          toolNamesByCallId: new Map(),
          timestampMs: 7,
        },);
        expect(user.role,).toBe('user',);
      },
    },),
    it({
      name: 'fabricates zero usage and picks toolUse when the assistant called tools',
      fn: async () => {
        const assistant = mapDshMessageToAdvisorMessage({
          message: {
            id: MessageId('m4',),
            role: 'assistant',
            content: [
              {
                type: 'tool-call',
                id: ToolCallId('call-9',),
                name: 'advisor',
                arguments: '{}',
              },
            ],
            source: {
              kind: 'model',
              provider: 'p',
              model: 'm',
            },
          },
          toolNamesByCallId: new Map([['call-9', 'advisor',],],),
          timestampMs: 9,
        },);
        expect(assistant.role,).toBe('assistant',);
        expect(
          assistant.role === 'assistant' ? assistant.stopReason : undefined,
        ).toBe('toolUse',);
        expect(
          assistant.role === 'assistant' ? assistant.usage.totalTokens : -1,
        ).toBe(0,);
        expect(
          assistant.role === 'assistant' ? assistant.model : undefined,
        ).toBe('m',);
      },
    },),
    it({
      name: 'picks stop when the assistant sent no tool calls',
      fn: async () => {
        const assistant = mapDshMessageToAdvisorMessage({
          message: {
            id: MessageId('m5',),
            role: 'assistant',
            content: [textBlock('done',),],
            source: {
              kind: 'model',
              provider: 'p',
              model: 'm',
            },
          },
          toolNamesByCallId: new Map(),
          timestampMs: 9,
        },);
        expect(
          assistant.role === 'assistant' ? assistant.stopReason : undefined,
        ).toBe('stop',);
      },
    },),
    it({
      name: 'resolves tool-result names and falls back to unknown-tool',
      fn: async () => {
        const known = mapDshMessageToAdvisorMessage({
          message: {
            id: MessageId('m6',),
            role: 'tool',
            toolCallId: ToolCallId('call-1',),
            content: [textBlock('ok',),],
            source: {
              kind: 'tool',
              callId: ToolCallId('call-1',),
            },
          },
          toolNamesByCallId: new Map([['call-1', 'advisor',],],),
          timestampMs: 3,
        },);
        expect(known.role,).toBe('toolResult',);
        expect(
          known.role === 'toolResult' ? known.toolName : undefined,
        ).toBe('advisor',);
        const unknown = mapDshMessageToAdvisorMessage({
          message: {
            id: MessageId('m7',),
            role: 'tool',
            toolCallId: ToolCallId('call-x',),
            isError: true,
            content: [textBlock('boom',),],
            source: {
              kind: 'tool',
              callId: ToolCallId('call-x',),
            },
          },
          toolNamesByCallId: new Map(),
          timestampMs: 3,
        },);
        expect(
          unknown.role === 'toolResult' ? unknown.toolName : undefined,
        ).toBe('unknown-tool',);
        expect(
          unknown.role === 'toolResult' ? unknown.isError : undefined,
        ).toBe(true,);
      },
    },),
    it({
      name: 'throws on a message role the pinned vocabulary lacks',
      fn: async () => {
        let thrown: unknown;
        try {
          mapDshMessageToAdvisorMessage({
            message: {
              id: MessageId('m8',),
              role: 'future-role',
              content: [],
              source: {
                kind: 'user',
              },
            } as unknown as DshMessage,
            toolNamesByCallId: new Map(),
            timestampMs: 1,
          },);
        }
        catch (error) {
          thrown = error;
        }
        expect(Error.isError(thrown,),).toBe(true,);
        expect((thrown as Error).message.includes('unmapped role',),).toBe(true,);
      },
    },),
  ],
},);

await describe({
  name: collectAdvisorToolNames.name,
  children: [
    it({
      name: 'indexes tool-call names by call id',
      fn: async () => {
        const names = collectAdvisorToolNames([
          {
            id: MessageId('m9',),
            role: 'assistant',
            content: [
              {
                type: 'tool-call',
                id: ToolCallId('call-3',),
                name: 'advisor',
                arguments: '{}',
              },
            ],
            source: {
              kind: 'model',
              provider: 'p',
              model: 'm',
            },
          },
        ] as unknown as DshMessage[],);
        expect(names.get('call-3',),).toBe('advisor',);
      },
    },),
    it({
      name: 'ignores messages carrying no tool calls',
      fn: async () => {
        const names = collectAdvisorToolNames([
          {
            id: MessageId('m10',),
            role: 'user',
            content: [textBlock('hi',),],
            source: {
              kind: 'user',
            },
          },
        ] as unknown as DshMessage[],);
        expect(names.size,).toBe(0,);
      },
    },),
  ],
},);

await describe({
  name: isPiAgentMessage.name,
  children: [
    it({
      name: 'accepts only the pinned pi message roles',
      fn: async () => {
        expect(
          isPiAgentMessage({
            role: 'system',
          } as unknown as Parameters<typeof isPiAgentMessage>[0],),
        ).toBe(true,);
        expect(
          isPiAgentMessage({
            role: 'user',
          } as unknown as Parameters<typeof isPiAgentMessage>[0],),
        ).toBe(true,);
        expect(
          isPiAgentMessage({
            role: 'toolResult',
          } as unknown as Parameters<typeof isPiAgentMessage>[0],),
        ).toBe(true,);
        expect(
          isPiAgentMessage({
            role: 'developer',
          } as unknown as Parameters<typeof isPiAgentMessage>[0],),
        ).toBe(false,);
        expect(
          isPiAgentMessage({
            role: 'nonsense',
          } as unknown as Parameters<typeof isPiAgentMessage>[0],),
        ).toBe(false,);
      },
    },),
  ],
},);
