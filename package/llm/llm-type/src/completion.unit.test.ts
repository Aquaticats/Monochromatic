/**
 Tests for non-streaming chat-completion response shapes.
 
 @module
 */

import {
  describe,
  expect,
  expectTypeOf,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type {
  ChatCompletionResponse,
  CompletionUsage,
} from '../dist/final/neutral/index.mjs';

await describe({
  name: 'completion',
  children: [
    it({
      name: 'accepts a minimal content-only response',
      fn: async () => {
        const data: ChatCompletionResponse = {
          choices: [{ message: { content: 'Done.', }, },],
        };
        expect(data.choices[0]?.message.content,).toBe('Done.',);
      },
    },),

    it({
      name: 'accepts the optional role companion on a choice message',
      fn: async () => {
        const data: ChatCompletionResponse = {
          choices: [{ message: { role: 'assistant', content: 'Hi.', }, },],
        };
        expect(data.choices[0]?.message.role,).toBe('assistant',);
      },
    },),

    it({
      name: 'content is a non-null string a consumer can read without a guard',
      fn: async () => {
        const data: ChatCompletionResponse = {
          choices: [{ message: { content: 'x', }, },],
        };
        const [first,] = data.choices;
        if (first !== undefined)
          expectTypeOf(first.message.content,).toEqualTypeOf<string>();
      },
    },),

    it({
      name: 'composes with CompletionUsage via intersection',
      fn: async () => {
        type WithUsage = ChatCompletionResponse & { readonly usage: CompletionUsage; };
        const data: WithUsage = {
          choices: [{ message: { content: 'ok', }, },],
          usage: { prompt_tokens: 10, completion_tokens: 5, },
        };
        expect(data.usage.prompt_tokens,).toBe(10,);
        expectTypeOf(data.usage.prompt_tokens,).toEqualTypeOf<number>();
      },
    },),
  ],
},);
