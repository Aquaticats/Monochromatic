/**
 Unit tests for the shared-schema validation gate over the built artifact.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  SearchFetchCliUsageError,
  validateFetchParams,
  validateSearchParams,
} from '../dist/final/node/index.mjs';

await describe({
  name: '',
  children: [
    describe({
      name: validateSearchParams.name,
      children: [
        it({
          name: 'accepts params that fit the shared web_search schema',
          fn: async () => {
            validateSearchParams({
              rawParams: {
                query: 'cachyos kernel',
                includeDomains: ['cachyos.org',],
                fromDate: '2026-01-01',
                toDate: '2026-02-02T03:04:05Z',
              },
            },);
          },
        },),
        it({
          name: 'refuses a non-string query as a usage error naming the schema',
          fn: async () => {
            /**
             Captured schema refusal.
             */
            let caught: unknown;
            try {
              validateSearchParams({ rawParams: { query: 42, }, },);
            }
            catch (error: unknown) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(SearchFetchCliUsageError,);
            expect((caught as Error).message.includes('web_search schema',),).toBe(true,);
          },
        },),
        it({
          name: 'refuses a non-array includeDomains as a usage error',
          fn: async () => {
            /**
             Captured schema refusal.
             */
            let caught: unknown;
            try {
              validateSearchParams({
                rawParams: {
                  query: 'docs',
                  includeDomains: 'a.com',
                },
              },);
            }
            catch (error: unknown) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(SearchFetchCliUsageError,);
          },
        },),
      ],
    },),
    describe({
      name: validateFetchParams.name,
      children: [
        it({
          name: 'accepts params that fit the shared web_fetch schema',
          fn: async () => {
            validateFetchParams({ rawParams: { url: 'https://example.com/page', }, },);
          },
        },),
        it({
          name: 'refuses a missing url as a usage error naming the schema',
          fn: async () => {
            /**
             Captured schema refusal.
             */
            let caught: unknown;
            try {
              validateFetchParams({ rawParams: {}, },);
            }
            catch (error: unknown) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(SearchFetchCliUsageError,);
            expect((caught as Error).message.includes('web_fetch schema',),).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
