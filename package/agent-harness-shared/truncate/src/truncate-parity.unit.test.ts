/**
 Parity tests proving this truncation port matches Pi's helpers byte for byte.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  formatSize as piFormatSize,
  truncateHead as piTruncateHead,
} from '@earendil-works/pi-coding-agent';

import {
  formatSize,
  truncateHead,
  type TruncationOptions,
} from '../dist/final/node/index.mjs';

//region Fixtures

/**
 Content corpus spanning empty, single-line, multiline, trailing-newline, and multi-byte shapes.
 */
const CONTENT_CORPUS = [
  '',
  'a',
  'a\n',
  'alpha\nbeta\ngamma',
  'alpha\nbeta\ngamma\n',
  'x'.repeat(64,),
  'é\né\né',
  'line with spaces and punctuation: <, > & "quotes"\nsecond',
  'leading\n\nblank line above\n',
] as const;

/**
 Ceiling corpus spanning no-op, line-limited, byte-limited, and first-line-exceeds outcomes.
 */
const OPTION_CORPUS: readonly TruncationOptions[] = [
  {},
  { maxLines: 1, },
  { maxLines: 2, },
  { maxLines: 100, },
  { maxBytes: 1, },
  { maxBytes: 5, },
  { maxBytes: 8, },
  { maxBytes: 32, },
  { maxLines: 1, maxBytes: 5, },
  { maxLines: 3, maxBytes: 5, },
];

/**
 Byte magnitudes spanning every size suffix.
 */
const BYTE_MAGNITUDES = [
  0,
  1,
  512,
  1_023,
  1_024,
  1_536,
  1_048_576,
  3_145_728,
  10_737_418_240,
] as const;

//endregion Fixtures

await describe({
  name: '',
  children: [
    describe({
      name: 'pi truncateHead parity',
      children: [
        it({
          name: 'returns the same truncation record as Pi for every corpus pair',
          fn: async () => {
            for (const content of CONTENT_CORPUS) {
              for (const options of OPTION_CORPUS) {
                expect(truncateHead({
                  content,
                  options,
                },),)
                  .toEqual(piTruncateHead(
                    content,
                    options,
                  ),);
              }
            }
          },
        },),
        it({
          name: 'returns the same size text as Pi across byte magnitudes',
          fn: async () => {
            for (const bytes of BYTE_MAGNITUDES) {
              expect(formatSize(bytes,),).toBe(piFormatSize(bytes,),);
            }
          },
        },),
      ],
    },),
  ],
},);
