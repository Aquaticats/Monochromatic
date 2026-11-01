/**
 Tests for category-leaf remapping of untrusted model categories onto
 the listed taxonomy.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { remapCategoryLeaf, } from '../dist/final/node/index.mjs';

await describe({
  name: remapCategoryLeaf.name,
  children: [
    it({
      name: 'remaps a known leaf reported under an invented family',
      fn: async () => {
        expect(remapCategoryLeaf({ category: 'completeness/omission', },),)
          .toEqual({
            remapped: true,
            category: 'accuracy/omission',
          },);
      },
    },),
    it({
      name: 'confirms an already-correct category through the same path',
      fn: async () => {
        expect(remapCategoryLeaf({ category: 'accuracy/mistranslation', },),)
          .toEqual({
            remapped: true,
            category: 'accuracy/mistranslation',
          },);
      },
    },),
    it({
      name: 'declines categories without a family separator',
      fn: async () => {
        expect(remapCategoryLeaf({ category: 'omission', },),)
          .toEqual({ remapped: false, },);
      },
    },),
    it({
      name: 'declines leaves no listed category owns',
      fn: async () => {
        expect(remapCategoryLeaf({ category: 'accuracy/cat-nap', },),)
          .toEqual({ remapped: false, },);
      },
    },),
  ],
},);
