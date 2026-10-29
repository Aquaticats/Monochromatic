import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  collectTags,
  stripInlineCodeSpans,
} from './tag-names.ts';

/** Count of repeated units used to exercise the scans on long input. */
const LONG_RUN = 1_000;

await describe({
  name: 'tag-names text scanners',
  children: [
    describe({
      name: stripInlineCodeSpans.name,
      children: [
        it({
          name: 'returns empty string unchanged',
          fn: async () => {
            expect(stripInlineCodeSpans('',),).toBe('',);
          },
        },),
        it({
          name: 'returns text without backticks unchanged',
          fn: async () => {
            expect(stripInlineCodeSpans('no code here',),).toBe('no code here',);
          },
        },),
        it({
          name: 'removes a backtick span including its delimiters',
          fn: async () => {
            expect(stripInlineCodeSpans('a `code` b',),).toBe('a  b',);
          },
        },),
        it({
          name: 'removes an entire string that is one span',
          fn: async () => {
            expect(stripInlineCodeSpans('`code`',),).toBe('',);
          },
        },),
        it({
          name: 'preserves an unmatched opening backtick at the start',
          fn: async () => {
            expect(stripInlineCodeSpans('`abc',),).toBe('`abc',);
          },
        },),
        it({
          name: 'preserves an unmatched opening backtick mid-string',
          fn: async () => {
            expect(stripInlineCodeSpans('abc`def',),).toBe('abc`def',);
          },
        },),
        it({
          name: 'removes multiple spans, keeping the text between them',
          fn: async () => {
            expect(stripInlineCodeSpans('a`b`c`d`e',),).toBe('ace',);
          },
        },),
        it({
          name: 'removes an empty span',
          fn: async () => {
            expect(stripInlineCodeSpans('x``y',),).toBe('xy',);
          },
        },),
        it({
          name: 'handles a long run of spans without overflow',
          fn: async () => {
            /** Repeated inline code spans used to prove linear scanning. */
            const inlineCodeSpans = 'a`x`'.repeat(LONG_RUN,);
            /** Expected text after each inline span is stripped. */
            const expectedText = 'a'.repeat(LONG_RUN,);
            expect(stripInlineCodeSpans(inlineCodeSpans,),).toBe(expectedText,);
          },
        },),
      ],
    },),
    describe({
      name: collectTags.name,
      children: [
        it({
          name: 'returns empty array for empty string',
          fn: async () => {
            expect(collectTags('',),).toEqual([],);
          },
        },),
        it({
          name: 'returns empty array when no tags are present',
          fn: async () => {
            expect(collectTags('no tags',),).toEqual([],);
          },
        },),
        it({
          name: 'collects tag words without the leading at-sign',
          fn: async () => {
            expect(collectTags('see @param and @returns',),).toEqual([
              'param',
              'returns',
            ],);
          },
        },),
        it({
          name: 'skips a bare at-sign and collects the following tag',
          fn: async () => {
            expect(collectTags('@@a',),).toEqual(['a',],);
          },
        },),
        it({
          name: 'returns empty array for a lone at-sign',
          fn: async () => {
            expect(collectTags('@',),).toEqual([],);
          },
        },),
        it({
          name: 'returns empty array for a trailing at-sign',
          fn: async () => {
            expect(collectTags('text @',),).toEqual([],);
          },
        },),
        it({
          name: 'keeps duplicate tags in order',
          fn: async () => {
            expect(collectTags('@param @param',),).toEqual([
              'param',
              'param',
            ],);
          },
        },),
        it({
          name: 'splits adjacent at-words',
          fn: async () => {
            expect(collectTags('@a@b',),).toEqual([
              'a',
              'b',
            ],);
          },
        },),
        it({
          name: 'collects a long run of tags without overflow',
          fn: async () => {
            const result = collectTags('@a '.repeat(LONG_RUN,),);
            expect(result.length,).toBe(LONG_RUN,);
            expect(result.every(function isA(word,): boolean {
              return word === 'a';
            },),).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
