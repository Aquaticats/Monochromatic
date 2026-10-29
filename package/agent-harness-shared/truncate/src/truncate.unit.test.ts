/**
 Unit tests for truncation and size formatting helpers.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  DEFAULT_MAX_BYTES,
  DEFAULT_MAX_LINES,
  formatSize,
  truncateHead,
} from '../dist/final/node/index.mjs';

//region Fixtures

/**
 Multiline fixture whose lines stay under every byte ceiling used here.
 */
const THREE_LINES = 'alpha\nbeta\ngamma';

/**
 Multiline fixture with a trailing newline.
 */
const TRAILING_NEWLINE = 'alpha\nbeta\ngamma\n';

/**
 Fixture whose single line exceeds a tiny byte ceiling on its own.
 */
const ONE_LONG_LINE = 'x'.repeat(64,);

//endregion Fixtures

await describe({
  name: '',
  children: [
    describe({
      name: formatSize.name,
      children: [
        it({
          name: 'renders whole bytes below one kibibyte',
          fn: async () => {
            expect(formatSize(0,),).toBe('0B',);
            expect(formatSize(1_023,),).toBe('1023B',);
          },
        },),
        it({
          name: 'renders one decimal kibibytes below one mebibyte',
          fn: async () => {
            expect(formatSize(1_024,),).toBe('1.0KB',);
            expect(formatSize(2_048,),).toBe('2.0KB',);
          },
        },),
        it({
          name: 'renders one decimal mebibytes at one mebibyte and above',
          fn: async () => {
            expect(formatSize(1_024 * 1_024,),).toBe('1.0MB',);
            expect(formatSize(1_024 * 1_024 * 3,),).toBe('3.0MB',);
          },
        },),
      ],
    },),
    describe({
      name: truncateHead.name,
      children: [
        it({
          name: 'returns content untouched under both ceilings',
          fn: async () => {
            const result = truncateHead({ content: THREE_LINES, },);
            expect(result.truncated,).toBe(false,);
            expect(result.truncatedBy,).toBe(null,);
            expect(result.content,).toBe(THREE_LINES,);
            expect(result.outputLines,).toBe(result.totalLines,);
          },
        },),
        it({
          name: 'drops nothing for empty content',
          fn: async () => {
            const result = truncateHead({ content: '', },);
            expect(result.truncated,).toBe(false,);
            expect(result.totalLines,).toBe(0,);
            expect(result.totalBytes,).toBe(0,);
            expect(result.content,).toBe('',);
          },
        },),
        it({
          name: 'does not count a trailing newline as a line',
          fn: async () => {
            const result = truncateHead({ content: TRAILING_NEWLINE, },);
            expect(result.totalLines,).toBe(3,);
            expect(result.truncated,).toBe(false,);
          },
        },),
        it({
          name: 'keeps whole lines and reports the line ceiling',
          fn: async () => {
            const result = truncateHead({
              content: THREE_LINES,
              options: { maxLines: 2, },
            },);
            expect(result.truncated,).toBe(true,);
            expect(result.truncatedBy,).toBe('lines',);
            expect(result.content,).toBe('alpha\nbeta',);
            expect(result.outputLines,).toBe(2,);
          },
        },),
        it({
          name: 'reports the byte ceiling when bytes stop the walk first',
          fn: async () => {
            const result = truncateHead({
              content: THREE_LINES,
              options: { maxBytes: 8, },
            },);
            expect(result.truncated,).toBe(true,);
            expect(result.truncatedBy,).toBe('bytes',);
            expect(result.outputLines,).toBe(1,);
          },
        },),
        it({
          name: 'keeps nothing when the first line alone exceeds the byte ceiling',
          fn: async () => {
            const result = truncateHead({
              content: ONE_LONG_LINE,
              options: { maxBytes: 8, },
            },);
            expect(result.truncated,).toBe(true,);
            expect(result.truncatedBy,).toBe('bytes',);
            expect(result.firstLineExceedsLimit,).toBe(true,);
            expect(result.content,).toBe('',);
            expect(result.outputLines,).toBe(0,);
          },
        },),
        it({
          name: 'counts multi-byte characters as multiple bytes',
          fn: async () => {
            const result = truncateHead({
              content: 'é\né\né',
              options: { maxBytes: 5, },
            },);
            expect(result.truncated,).toBe(true,);
            expect(result.truncatedBy,).toBe('bytes',);
            expect(result.outputLines,).toBe(2,);
          },
        },),
        it({
          name: 'applies the documented defaults when options are absent',
          fn: async () => {
            const result = truncateHead({ content: THREE_LINES, },);
            expect(result.maxLines,).toBe(DEFAULT_MAX_LINES,);
            expect(result.maxBytes,).toBe(DEFAULT_MAX_BYTES,);
          },
        },),
      ],
    },),
  ],
},);
