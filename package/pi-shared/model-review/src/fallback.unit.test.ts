/**
 Built-artifact tests for exhausted reviewer diagnostics.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { ReviewUnavailableError, } from '../dist/final/node/index.mjs';

await describe({
  name: ReviewUnavailableError.name,
  children: [
    it({
      name: 'records complete copied attempt and diagnostic audit',
      fn: async () => {
        /** Mutable fixture identities changed after error construction. */
        const attempted = ['test/first', 'test/fallback',];
        /** Mutable fixture diagnostics changed after error construction. */
        const diagnostics = ['test/first: timeout', 'test/fallback: unavailable',];
        /** Exhausted-review error under test. */
        const error = new ReviewUnavailableError({
          attemptedCandidateIdentities: attempted,
          diagnostics,
          cause: new Error('all failed',),
        },);
        attempted[0] = 'changed';
        diagnostics[0] = 'changed';
        expect(error.attemptedCandidateIdentities,).toEqual([
          'test/first',
          'test/fallback',
        ],);
        expect(error.diagnostics,).toEqual([
          'test/first: timeout',
          'test/fallback: unavailable',
        ],);
        expect(error.message,).toContain('test/first, test/fallback',);
        expect(error.message,).toContain('test/first: timeout; test/fallback: unavailable',);
      },
    },),
  ],
},);
