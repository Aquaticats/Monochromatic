/**
 Tests for the decoded forms of a credential header's secret: what a `Basic`
 pair and a percent-escaped token spell once a decoder has read them.
 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { decodedFormsOf, } from '../dist/final/node/index.mjs';

await describe({
  name: decodedFormsOf.name,
  children: [
    it({
      name: 'READS A BASIC BODY, padded or not, as the pair and each side of its first colon',
      fn: async () => {
        expect(decodedFormsOf({ secret: 'a2l0OndoaXNrZXIta2V5LTc0MjE=', },),).toEqual([
          'kit:whisker-key-7421',
          'kit',
          'whisker-key-7421',
        ],);
        expect(decodedFormsOf({ secret: 'a2l0OndoaXNrZXIta2V5LTc0MjE', },),).toEqual([
          'kit:whisker-key-7421',
          'kit',
          'whisker-key-7421',
        ],);
      },
    },),
    it({
      name: 'READS A BASE64 BODY WITHOUT A COLON as the one text, and the url-safe alphabet as well',
      fn: async () => {
        expect(decodedFormsOf({ secret: 'd2hpc2tlci1rZXktNzQyMQ==', },),).toEqual(['whisker-key-7421',],);
        expect(decodedFormsOf({ secret: 'Pz8-Pz8_', },),).toEqual(['??>???',],);
      },
    },),
    it({
      name: 'READS NOTHING OUT OF A SECRET that is no base64 of printable text, or has a length no base64 text has',
      fn: async () => {
        expect(decodedFormsOf({ secret: 'whisker-key-7421', },),).toEqual([],);
        expect(decodedFormsOf({ secret: 'a2l0Ondoa=', },),).toEqual([],);
        expect(decodedFormsOf({ secret: 'a2l0OndoaX=@', },),).toEqual([],);
        expect(decodedFormsOf({ secret: 'AAAAAAAAAAAAAAAA', },),).toEqual([],);
      },
    },),
    it({
      name: 'RESOLVES THE PERCENT ESCAPES of a token the client wrote escaped',
      fn: async () => {
        expect(decodedFormsOf({ secret: 'cat%2Fkey%2Bodd%3D7421', },),).toEqual(['cat/key+odd=7421',],);
      },
    },),
  ],
},);
