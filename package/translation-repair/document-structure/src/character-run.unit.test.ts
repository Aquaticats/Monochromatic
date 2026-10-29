/**
 Tests the one longest-run count both fence builders share (audit area six).

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { longestRunOf, } from '../dist/final/node/index.mjs';

await describe({
  name: longestRunOf.name,
  children: [
    it({
      name: 'COUNTS THE LONGEST RUN of the character asked for, and no other character',
      fn: async () => {
        expect(longestRunOf({ text: 'a ` b ``` c == d', character: '`', },),).toBe(3,);
        expect(longestRunOf({ text: 'a ` b ``` c == d', character: '=', },),).toBe(2,);
        expect(longestRunOf({ text: 'The cat naps.', character: '=', },),).toBe(0,);
        expect(longestRunOf({ text: '', character: '`', },),).toBe(0,);
      },
    },),
    it({
      name: 'COUNTS A CHARACTER OUTSIDE THE BASIC PLANE ONCE per occurrence, reading by code point',
      fn: async () => {
        expect(longestRunOf({ text: 'a \u{1F408}\u{1F408}\u{1F408} b \u{1F408}', character: '\u{1F408}', },),).toBe(3,);
      },
    },),
  ],
},);
