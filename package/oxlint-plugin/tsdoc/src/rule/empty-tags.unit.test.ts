import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  parseTaggedLine,
  UNTAGGED_LINE,
} from './empty-tags.ts';

/** Length of the long repeated runs used to exercise scan boundaries on big inputs. */
const LONG_RUN = 1_000;

await describe({
  name: parseTaggedLine.name,
  children: [
    it({
      name: 'returns UNTAGGED_LINE for empty string',
      fn: async () => {
        expect(parseTaggedLine('',),).toBe(UNTAGGED_LINE,);
      },
    },),
    it({
      name: 'returns UNTAGGED_LINE for a bare at-sign with no tag name',
      fn: async () => {
        expect(parseTaggedLine('@',),).toBe(UNTAGGED_LINE,);
      },
    },),
    it({
      name: 'returns UNTAGGED_LINE when the at-sign is not followed by a word char',
      fn: async () => {
        expect(parseTaggedLine('@ foo',),).toBe(UNTAGGED_LINE,);
      },
    },),
    it({
      name: 'returns UNTAGGED_LINE for text without a leading at-sign',
      fn: async () => {
        expect(parseTaggedLine('plain text',),).toBe(UNTAGGED_LINE,);
      },
    },),
    it({
      name: 'parses a simple tag and rest',
      fn: async () => {
        expect(parseTaggedLine('@param foo',),).toEqual({
          tag: '@param',
          rest: 'foo',
        },);
      },
    },),
    it({
      name: 'collapses the whitespace gap but preserves internal spacing in rest',
      fn: async () => {
        expect(parseTaggedLine('@param   foo bar',),).toEqual({
          tag: '@param',
          rest: 'foo bar',
        },);
      },
    },),
    it({
      name: 'treats a tab as the whitespace gap',
      fn: async () => {
        expect(parseTaggedLine('@param\tfoo',),).toEqual({
          tag: '@param',
          rest: 'foo',
        },);
      },
    },),
    it({
      name: 'returns UNTAGGED_LINE when there is no whitespace gap after the tag',
      fn: async () => {
        expect(parseTaggedLine('@param',),).toBe(UNTAGGED_LINE,);
      },
    },),
    it({
      name: 'returns UNTAGGED_LINE when only trailing whitespace follows the tag (empty rest)',
      fn: async () => {
        expect(parseTaggedLine('@param ',),).toBe(UNTAGGED_LINE,);
      },
    },),
    it({
      name: 'accepts a long tag-name run',
      fn: async () => {
        const tagBody = 'a'.repeat(LONG_RUN,);
        expect(parseTaggedLine(`@${tagBody} x`,),).toEqual({
          tag: `@${tagBody}`,
          rest: 'x',
        },);
      },
    },),
    it({
      name: 'accepts a long whitespace gap',
      fn: async () => {
        const gap = ' '.repeat(LONG_RUN,);
        expect(parseTaggedLine(`@a${gap}x`,),).toEqual({
          tag: '@a',
          rest: 'x',
        },);
      },
    },),
  ],
},);
