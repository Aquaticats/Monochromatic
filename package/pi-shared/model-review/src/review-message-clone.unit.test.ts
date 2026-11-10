/**
 Built-artifact-adjacent tests for the one review-message clone implementation.

 The provider request and the request snapshot read the same caller messages,
 so both must share one validation and clone rule. These tests read the
 authored sources because neither isolator is reachable from the package API
 with an unsupported message.

 @module
 */

import { readFile, } from 'node:fs/promises';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

/**
 Production isolation source.
 */
const PROVIDER_DATA_SOURCE = new URL(
  'provider-data.ts',
  import.meta.url,
);

/**
 Production dispatch and snapshot source.
 */
const PROVIDER_STREAMS_SOURCE = new URL(
  'provider-streams.ts',
  import.meta.url,
);

/**
 Count non-overlapping occurrences of one literal text.
 
 @param source - text to search
 
 @param needle - literal text to count
 
 @returns number of occurrences
 
 @example
 ```ts
 countOccurrences('a b a', 'a');
 ```
 */
function countOccurrences(
  {
    source,
    needle,
  }: {
    readonly source: string;
    readonly needle: string;
  },
): number {
  return source.split(needle,).length - 1;
}

await describe({
  name: 'review message clone',
  children: [
    it({
      name: 'keeps one validation and clone rule shared by both isolators',
      fn: async () => {
        /**
         Authored isolation source text.
         */
        const providerDataSource = await readFile(
          PROVIDER_DATA_SOURCE,
          'utf8',
        );
        /**
         Authored snapshot source text.
         */
        const providerStreamsSource = await readFile(
          PROVIDER_STREAMS_SOURCE,
          'utf8',
        );
        expect(countOccurrences({
          source: providerDataSource,
          needle: 'message.role} message content',
        },),).toBe(1,);
        expect(countOccurrences({
          source: providerDataSource,
          needle: 'content: message.content,',
        },),).toBe(1,);
        expect(countOccurrences({
          source: providerStreamsSource,
          needle: 'message.role} message content',
        },),).toBe(0,);
        expect(countOccurrences({
          source: providerStreamsSource,
          needle: 'content: message.content,',
        },),).toBe(0,);
        expect(providerStreamsSource.includes('cloneReviewMessages(',),).toBe(true,);
      },
    },),
  ],
},);
