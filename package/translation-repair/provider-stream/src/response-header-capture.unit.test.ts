/**
 Tests the pick of response headers a caller asked for (`response-header-capture.ts`): exact names and prefixes
 ending in an asterisk, read case-blind, and nothing the caller did not name. The transport that hands the pick back
 on a reply is exercised in `transport-capture-headers.unit.test.ts`. Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import { capturedHeadersOf, } from '../dist/final/node/index.mjs';

await describe({
  name: capturedHeadersOf.name,
  children: [
    it({
      name: 'PICKS BY EXACT NAME AND BY PREFIX, case-blind, and drops every header it was not asked for',
      fn: async () => {
        expect(capturedHeadersOf({
          headers: new Headers({
            'Retry-After': '2',
            'Anthropic-Ratelimit-Requests-Limit': '1000',
            'x-cat': 'purr',
          },),
          wanted: ['retry-after', 'anthropic-ratelimit-*',],
        },),).toEqual({
          'retry-after': '2',
          'anthropic-ratelimit-requests-limit': '1000',
        },);
      },
    },),

    it({
      name: 'PICKS NOTHING from a name that only shares a start with an exact request, and from an empty wanted list',
      fn: async () => {
        expect(capturedHeadersOf({
          headers: new Headers({ 'retry-after-ms': '2', },),
          wanted: ['retry-after',],
        },),).toEqual({},);
        expect(capturedHeadersOf({
          headers: new Headers({ 'retry-after': '2', },),
          wanted: [],
        },),).toEqual({},);
      },
    },),
  ],
},);
