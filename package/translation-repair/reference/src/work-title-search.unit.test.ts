/**
 Tests for reading one search result off the endpoint's answer. The request
 itself is exercised through the lookup, which scripts the transport
 (`work-title-lookup.unit.test.ts`); these cases read what one element of the
 answer's results yields. Fixtures are cat-themed invention; no corpus content
 appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { hitsOf, } from '../dist/final/node/index.mjs';

await describe({
  name: hitsOf.name,
  children: [
    it({
      name: 'READS NO HIT FROM AN ELEMENT THAT IS NO RECORD, whether the endpoint sent null, text or a number '
        + 'where a result belongs',
      fn: async () => {
        expect([
          hitsOf({ value: null, },),
          hitsOf({ value: 'The cat naps.', },),
          hitsOf({ value: 7, },),
        ],).toEqual([[], [], [],],);
      },
    },),
    it({
      name: 'READS NO HIT FROM A RESULT WITHOUT A TEXT URL, since a result naming no address cannot be cited, '
        + 'whatever else it carries',
      fn: async () => {
        expect([
          hitsOf({ value: { title: 'Whiskers Abroad', highlights: ['The cat naps.',], }, },),
          hitsOf({ value: { title: 'Whiskers Abroad', url: 7, }, },),
        ],).toEqual([[], [],],);
      },
    },),
    it({
      name: 'READS ONE HIT FROM A RESULT CARRYING A URL, with an empty title and highlight for what is not text',
      fn: async () => {
        expect([
          hitsOf({
            value: {
              title: 'Whiskers Abroad',
              url: 'https://example.invalid/whiskers',
              highlights: ['The cat naps.', 'It naps again.',],
            },
          },),
          hitsOf({
            value: {
              title: 3,
              url: 'https://example.invalid/mittens',
              highlights: [4,],
            },
          },),
        ],).toEqual([
          [{
            title: 'Whiskers Abroad',
            url: 'https://example.invalid/whiskers',
            highlight: 'The cat naps.',
          },],
          [{
            title: '',
            url: 'https://example.invalid/mittens',
            highlight: '',
          },],
        ],);
      },
    },),
  ],
},);
