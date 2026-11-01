/**
 Tests for reading whether a quote can be moved without its markup: emphasis,
 code and brackets in pairs, and no footnote mark.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { carriesOrCutsMarkup, } from '../dist/final/node/index.mjs';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: carriesOrCutsMarkup.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'LEAVES PLAIN WORDS AND MARKUP IN PAIRS ALONE: strong, emphasis, strikethrough, code, brackets',
          fn: async () => {
            expect([
              'the cat naps',
              'the **cat** naps',
              'the *cat* naps',
              'the ***cat*** naps',
              'the ~~dog~~ cat naps',
              'the `cat` naps',
              'the [cat](https://cats.example/) naps',
              'the cat (the tabby) naps',
              '',
            ].map(function carries(text,): boolean {
              return carriesOrCutsMarkup({ text, },);
            },),).toEqual([false, false, false, false, false, false, false, false, false,],);
          },
        },),
        it({
          name: 'REFUSES A QUOTE THAT OPENS OR CLOSES A DELIMITER WITHOUT ITS PAIR, at the start, at the end or inside',
          fn: async () => {
            expect([
              'on the sill** lulled the kitten',
              '**sunlight on the sill',
              'the *cat naps',
              'the cat~~ naps',
              'the cat` naps',
              'sill](https://cats.example/sill) lulled',
              'the cat (the tabby naps',
              'the tabby) naps',
            ].map(function carries(text,): boolean {
              return carriesOrCutsMarkup({ text, },);
            },),).toEqual([true, true, true, true, true, true, true, true,],);
          },
        },),
        it({
          name: 'REFUSES A QUOTE THAT HOLDS A FOOTNOTE MARK, which would give a place a reference whose definition stays '
            + 'in another slice',
          fn: async () => {
            expect(carriesOrCutsMarkup({ text: 'the cat naps[^1]', },),).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
