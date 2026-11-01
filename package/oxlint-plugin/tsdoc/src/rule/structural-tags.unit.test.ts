import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  extractLeadingTag,
  NO_LEADING_TAG,
} from './structural-tags.ts';

/** Length of the long repeated run used to exercise the scan boundary on big input. */
const LONG_RUN = 1_000;

await describe({
  name: extractLeadingTag.name,
  children: [
    it({
      name: 'returns NO_LEADING_TAG for empty string',
      fn: async () => {
        expect(extractLeadingTag('',),).toBe(NO_LEADING_TAG,);
      },
    },),
    it({
      name: 'returns NO_LEADING_TAG for a bare at-sign with no tag name',
      fn: async () => {
        expect(extractLeadingTag('@',),).toBe(NO_LEADING_TAG,);
      },
    },),
    it({
      name: 'returns NO_LEADING_TAG when the at-sign is not followed by a word char',
      fn: async () => {
        expect(extractLeadingTag('@ foo',),).toBe(NO_LEADING_TAG,);
      },
    },),
    it({
      name: 'returns NO_LEADING_TAG for a doubled at-sign',
      fn: async () => {
        expect(extractLeadingTag('@@x',),).toBe(NO_LEADING_TAG,);
      },
    },),
    it({
      name: 'returns NO_LEADING_TAG for text without a leading at-sign',
      fn: async () => {
        expect(extractLeadingTag('no tag',),).toBe(NO_LEADING_TAG,);
      },
    },),
    it({
      name: 'captures a bare tag with no trailing content',
      fn: async () => {
        expect(extractLeadingTag('@param',),).toBe('@param',);
      },
    },),
    it({
      name: 'captures only the tag, stopping at the first whitespace',
      fn: async () => {
        expect(extractLeadingTag('@param foo',),).toBe('@param',);
      },
    },),
    it({
      name: 'stops at the first non-word char such as a hyphen',
      fn: async () => {
        expect(extractLeadingTag('@param-name',),).toBe('@param',);
      },
    },),
    it({
      name: 'treats digits as word chars',
      fn: async () => {
        expect(extractLeadingTag('@123abc rest',),).toBe('@123abc',);
      },
    },),
    it({
      name: 'captures a long tag-name run',
      fn: async () => {
        const tagBody = 'a'.repeat(LONG_RUN,);
        expect(extractLeadingTag(`@${tagBody} x`,),).toBe(`@${tagBody}`,);
      },
    },),
  ],
},);
