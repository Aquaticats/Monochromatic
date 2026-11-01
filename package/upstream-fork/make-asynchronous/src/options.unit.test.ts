/**
 Tests for the wrapper options type: `baseUrl` acceptance in both forms.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { makeAsynchronous, } from '../dist/final/neutral/index.mjs';

await describe({
  name: 'wrapper options',
  children: [
    it({
      name: 'accepts a string baseUrl',
      fn: async () => {
        const fn = makeAsynchronous({
          fn: function identity(value: number,): number {
            return value;
          },
          options: { baseUrl: 'file:///fixture.js', },
        },);
        expect(await fn({ args: [3], }),).toBe(3,);
      },
    },),

    it({
      name: 'accepts a URL baseUrl',
      fn: async () => {
        const fn = makeAsynchronous({
          fn: function identity(value: number,): number {
            return value;
          },
          options: { baseUrl: new URL('file:///fixture.js',), },
        },);
        expect(await fn({ args: [4], }),).toBe(4,);
      },
    },),
  ],
},);
