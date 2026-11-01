import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  isBlankAnswer,
  normalizeEditorAnswer,
  visibleTerminalText,
} from '../dist/final/node/index.mjs';

await describe({
  name: '',
  children: [
    describe({
      name: normalizeEditorAnswer.name,
      children: [
        {
          name: 'removes one final LF',
          input: 'first\nsecond\n',
          expected: 'first\nsecond',
        },
        {
          name: 'removes one final CRLF',
          input: 'first\r\nsecond\r\n',
          expected: 'first\r\nsecond',
        },
        {
          name: 'preserves preceding final line ending',
          input: 'answer\n\n',
          expected: 'answer\n',
        },
        {
          name: 'preserves text without final line ending',
          input: 'answer',
          expected: 'answer',
        },
        {
          name: 'preserves lone final carriage return',
          input: 'answer\r',
          expected: 'answer\r',
        },
      ].map(function toNormalizationTest(testCase,) {
        return it({
          name: testCase.name,
          fn: async () => {
            expect(normalizeEditorAnswer({ text: testCase.input, }),)
              .toBe(testCase.expected,);
          },
        },);
      },),
    },),
    describe({
      name: isBlankAnswer.name,
      children: [
        it({
          name: 'accepts visible multiline text',
          fn: async () => {
            expect(isBlankAnswer({ text: '  answer\n', }),)
              .toBe(false,);
          },
        },),
        ...[
          '',
          ' ',
          '\n\t',
        ].map(function toBlankTest(text,) {
          return it({
            name: `cancels blank ${JSON.stringify(text,)}`,
            fn: async () => {
              expect(isBlankAnswer({ text, }),)
                .toBe(true,);
            },
          },);
        },),
      ],
    },),
    describe({
      name: visibleTerminalText.name,
      children: [
        it({
          name: 'preserves printable Unicode newlines and tabs',
          fn: async () => {
            expect(visibleTerminalText({ text: 'Question 日本語\n\tanswer', }),)
              .toBe('Question 日本語\n\tanswer',);
          },
        },),
        it({
          name: 'makes escape and delete controls visible',
          fn: async () => {
            expect(visibleTerminalText({ text: 'before\u001Bafter\u007F', }),)
              .toBe('before<U+001B>after<U+007F>',);
          },
        },),
      ],
    },),
  ],
},);
