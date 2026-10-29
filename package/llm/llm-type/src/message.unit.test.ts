/**
 Tests for the text chat message envelope.
 
 @module
 */

import {
  describe,
  expect,
  expectTypeOf,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type {
  ChatMessage,
  ChatRole,
} from '../dist/final/neutral/index.mjs';

await describe({
  name: 'message',
  children: [
    it({
      name: 'accepts a system/user/assistant text message',
      fn: async () => {
        const messages: readonly ChatMessage[] = [
          { role: 'system', content: 'You are concise.', },
          { role: 'user', content: 'Hi.', },
          { role: 'assistant', content: 'Hello.', },
        ];
        expect(messages.length,).toBe(3,);
        expect(messages[0]?.content,).toBe('You are concise.',);
      },
    },),

    it({
      name: 'has exactly role: ChatRole and content: string',
      fn: async () => {
        expectTypeOf<ChatMessage>().toEqualTypeOf<{
          readonly role: ChatRole;
          readonly content: string;
        }>();
      },
    },),

    it({
      name: 'immutable messages stay assignable to a mutable turn list',
      fn: async () => {
        const turn: ChatMessage = { role: 'user', content: 'Build me a list.', };
        const list: ChatMessage[] = [turn,];
        expect(list,).toHaveLength(1,);
      },
    },),
  ],
},);
