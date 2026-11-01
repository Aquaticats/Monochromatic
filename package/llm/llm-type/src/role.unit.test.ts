/**
 Tests for chat roles.
 
 @module
 */

import {
  describe,
  expect,
  expectTypeOf,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  CHAT_ROLES,
  type ChatRole,
} from '../dist/final/neutral/index.mjs';

await describe({
  name: 'role',
  children: [
    it({
      name: 'CHAT_ROLES lists system, user, assistant in order',
      fn: async () => {
        expect(CHAT_ROLES,).toEqual([
          'system',
          'user',
          'assistant',
        ],);
      },
    },),

    it({
      name: 'every CHAT_ROLES entry is a valid ChatRole',
      fn: async () => {
        for (const role of CHAT_ROLES)
          expectTypeOf(role,).toEqualTypeOf<ChatRole>();
      },
    },),

    it({
      name: 'ChatRole is exactly the union of the three roles',
      fn: async () => {
        expectTypeOf<ChatRole>().toEqualTypeOf<'system' | 'user' | 'assistant'>();
      },
    },),
  ],
},);
