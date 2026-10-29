import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  stripBetweenDelims,
  stripLinesStartingWith,
} from './index.ts';

await describe({
  name: 'text-scan delimiter stripping',
  children: [
    describe({
      name: stripBetweenDelims.name,
      children: [
        it({
          name: 'strips a single delimited region',
          fn: async () => {
            expect(stripBetweenDelims({
              text: 'a `code` b',
              openDelim: '`',
              closeDelim: '`',
            },),).toBe('a  b',);
          },
        },),
        it({
          name: 'strips multiple delimited regions',
          fn: async () => {
            expect(stripBetweenDelims({
              text: 'x `a` y `b` z',
              openDelim: '`',
              closeDelim: '`',
            },),).toBe('x  y  z',);
          },
        },),
        it({
          name: 'strips multi-char delimiters for fenced code blocks',
          fn: async () => {
            expect(stripBetweenDelims({
              text: 'pre ```js\nmaybe()\n``` post',
              openDelim: '```',
              closeDelim: '```',
            },),).toBe('pre  post',);
          },
        },),
        it({
          name: 'leaves unmatched openers in place',
          fn: async () => {
            expect(stripBetweenDelims({
              text: 'a `unclosed code',
              openDelim: '`',
              closeDelim: '`',
            },),).toBe('a `unclosed code',);
          },
        },),
        it({
          name: 'respects disallowedInside to skip cross-newline spans',
          fn: async () => {
            expect(stripBetweenDelims({
              text: 'start "good" mid "bad\nnewline" end',
              openDelim: '"',
              closeDelim: '"',
              disallowedInside: '\n',
            },),).toBe('start  mid "bad\nnewline" end',);
          },
        },),
      ],
    },),
    describe({
      name: stripLinesStartingWith.name,
      children: [
        it({
          name: 'removes lines starting with the prefix',
          fn: async () => {
            expect(stripLinesStartingWith({
              text: 'a\n> quote\nb',
              prefix: '>',
            },),).toBe('a\nb',);
          },
        },),
        it({
          name: 'ignores leading whitespace when checking the prefix',
          fn: async () => {
            expect(stripLinesStartingWith({
              text: 'a\n   > quote\nb',
              prefix: '>',
            },),).toBe('a\nb',);
          },
        },),
        it({
          name: 'leaves non-matching lines untouched',
          fn: async () => {
            expect(stripLinesStartingWith({
              text: 'one\ntwo\nthree',
              prefix: '#',
            },),).toBe('one\ntwo\nthree',);
          },
        },),
      ],
    },),
  ],
},);
