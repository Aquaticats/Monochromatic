/**
 Tests for multimodal content parts.
 
 @module
 */

import {
  describe,
  expect,
  expectTypeOf,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type {
  ChatRole,
  ContentPart,
} from '../dist/final/neutral/index.mjs';

await describe({
  name: 'content-part',
  children: [
    it({
      name: 'accepts interleaved text and image_url parts',
      fn: async () => {
        const parts: readonly ContentPart[] = [
          { type: 'text', text: 'Describe this image.', },
          { type: 'image_url', image_url: { url: 'data:image/png;base64,AAAA', }, },
          { type: 'image_url', image_url: { url: 'https://example.test/cat.png', }, },
        ];
        expect(parts,).toHaveLength(3,);
      },
    },),

    it({
      name: 'discriminates on type',
      fn: async () => {
        const parts: readonly ContentPart[] = [
          { type: 'text', text: 'hi', },
          { type: 'image_url', image_url: { url: 'x', }, },
        ];
        for (const part of parts)
          if (part.type === 'text')
            expectTypeOf(part.text,).toEqualTypeOf<string>();
          else
            expectTypeOf(part.image_url.url,).toEqualTypeOf<string>();
        expect(parts,).toHaveLength(2,);
      },
    },),

    it({
      name: 'composes into a vision message envelope',
      fn: async () => {
        type VisionMessage = {
          readonly role: Extract<ChatRole, 'user'>;
          readonly content: readonly ContentPart[];
        };
        const message: VisionMessage = {
          role: 'user',
          content: [{ type: 'text', text: 'caption', },],
        };
        expect(message.role,).toBe('user',);
      },
    },),
  ],
},);
