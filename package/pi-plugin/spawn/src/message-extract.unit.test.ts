import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  assistantContentText,
  extractLastAssistantText,
  isAssistantMessage,
  isTextBlock,
} from './message-extract.ts';

await describe({
  name: '',
  children: [
    describe({
      name: isTextBlock.name,
      children: [
        it({
          name: 'accepts text blocks and rejects other content',
          fn: async function testTextBlockGuard() {
            expect(isTextBlock({ type: 'text', text: 'hello', },),).toBe(true,);
            expect(isTextBlock({ type: 'thinking', thinking: 'hmm', },),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: isAssistantMessage.name,
      children: [
        it({
          name: 'accepts assistant messages with string or array content',
          fn: async function testAssistantGuard() {
            expect(isAssistantMessage({ role: 'assistant', content: 'hello', },),).toBe(true,);
            expect(isAssistantMessage({ role: 'assistant', content: [], },),).toBe(true,);
            expect(isAssistantMessage({ role: 'user', content: 'hello', },),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: assistantContentText.name,
      children: [
        it({
          name: 'returns string content unchanged',
          fn: async function testStringContent() {
            expect(assistantContentText('hello',),).toBe('hello',);
          },
        },),
        it({
          name: 'joins only text blocks from array content',
          fn: async function testArrayContent() {
            expect(assistantContentText([
              { type: 'thinking', thinking: 'hidden', },
              { type: 'text', text: 'first', },
              { type: 'text', text: 'second', },
            ],),).toBe('first\nsecond',);
          },
        },),
      ],
    },),
    describe({
      name: extractLastAssistantText.name,
      children: [
        it({
          name: 'returns newest assistant text',
          fn: async function testNewestAssistantText() {
            expect(extractLastAssistantText([
              { role: 'assistant', content: [{ type: 'text', text: 'old', },], },
              { role: 'user', content: 'ignored', },
              { role: 'assistant', content: [{ type: 'text', text: 'new', },], },
            ],),).toBe('new',);
          },
        },),
        it({
          name: 'returns empty string when no assistant text exists',
          fn: async function testNoAssistantText() {
            expect(extractLastAssistantText([
              { role: 'user', content: 'ignored', },
              { role: 'assistant', content: [{ type: 'thinking', thinking: 'hidden', },], },
            ],),).toBe('',);
          },
        },),
      ],
    },),
  ],
},);
