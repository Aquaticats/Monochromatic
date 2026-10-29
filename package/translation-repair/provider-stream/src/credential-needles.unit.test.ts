/**
 Tests for the texts a credential is searched for as: the ones a reply may
 carry it in plainly, as a JSON string, as a form value, and the base64 stretches
 that its own bytes fix at each alignment.
 Fixtures are cat-themed invention; the key is the stand-in the quoting-failure
 fixture holds.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { needlesOf, } from '../dist/final/node/index.mjs';
import { WHISKER_KEY, } from '@monochromatic-dev/translation-repair-test-fixture/ts/quoting-failure.test-fixture.ts';

await describe({
  name: needlesOf.name,
  children: [
    it({
      name: 'LISTS THE KEY AS IT STANDS, then its base64 at each alignment, with how far each find reaches',
      fn: async () => {
        expect(needlesOf({ credential: WHISKER_KEY, },),).toEqual([
          { text: WHISKER_KEY, before: 0, after: 0, },
          { text: 'd2hpc2tlci1rZXktNzQyM', before: 0, after: 1, },
          { text: 'doaXNrZXIta2V5LTc0Mj', before: 1, after: 1, },
          { text: '3aGlza2VyLWtleS03NDIx', before: 1, after: 0, },
        ],);
      },
    },),
    it({
      name: 'LISTS A KEY WITH A QUOTE, A SLASH AND A SPACE as a JSON writer escapes it, as a form value, and in both base64 alphabets',
      fn: async () => {
        expect(needlesOf({ credential: String.raw`Bearer cat"key/ 7421`, },).map(function textOf(needle,): string {
          return needle.text;
        },),).toEqual([
          String.raw`Bearer cat"key/ 7421`,
          String.raw`Bearer cat\"key/ 7421`,
          String.raw`Bearer cat\"key\/ 7421`,
          String.raw`Bearer+cat"key/+7421`,
          String.raw`Bearer+cat\"key/+7421`,
          String.raw`Bearer+cat\"key\/+7421`,
          'QmVhcmVyIGNhdCJrZXkvIDc0Mj',
          'JlYXJlciBjYXQia2V5LyA3NDIx',
          'CZWFyZXIgY2F0ImtleS8gNzQyM',
        ],);
      },
    },),
    it({
      name: 'LISTS THE URL-SAFE ALPHABET apart where the standard one writes a plus or a slash',
      fn: async () => {
        /**
         Key whose bytes write sextets of sixty-two and sixty-three.
         */
        const odd = 'meow->>>-purr-???-7421';
        /**
         Units a base64 text in either alphabet is written with.
         */
        const base64Units = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/-_';
        /**
         Texts of the base64 needles: every plain spelling of this key holds a
         `>` or a `?`, or the `%` a form value writes one with, and no base64
         text is written with any of them.
         */
        const texts = needlesOf({ credential: odd, },)
          .filter(function isBase64({ text, },): boolean {
            return text.split('',).every(function inAlphabet(unit,): boolean {
              return base64Units.includes(unit,);
            },);
          },)
          .map(function textOf(needle,): string {
            return needle.text;
          },);
        expect(texts,).toEqual([
          'bWVvdy0+Pj4tcHVyci0/Pz8tNzQyM',
          'bWVvdy0-Pj4tcHVyci0_Pz8tNzQyM',
          '1lb3ctPj4+LXB1cnItPz8/LTc0Mj',
          '1lb3ctPj4-LXB1cnItPz8_LTc0Mj',
          'tZW93LT4+Pi1wdXJyLT8/Py03NDIx',
          'tZW93LT4-Pi1wdXJyLT8_Py03NDIx',
        ],);
      },
    },),
    it({
      name: 'LISTS A KEY BEYOND ASCII as its units and as its UTF-8 bytes one unit each, and base64 of the bytes',
      fn: async () => {
        expect(needlesOf({ credential: 'café-key-7421-x', },).map(function textOf(needle,): string {
          return needle.text;
        },),).toEqual([
          'café-key-7421-x',
          'cafÃ©-key-7421-x',
          'Y2Fmw6kta2V5LTc0MjEte',
          'NhZsOpLWtleS03NDIxLX',
          'jYWbDqS1rZXktNzQyMS14',
        ],);
      },
    },),
  ],
},);
