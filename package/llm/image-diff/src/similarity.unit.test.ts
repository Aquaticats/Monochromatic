import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  cosineSimilarity,
  dotProduct,
} from '@monochromatic-dev/llm-image-diff';

await describe({
  name: '',
  children: [
    describe({
      name: dotProduct.name,
      children: [
        it({
          name: 'identical unit vectors yield 1',
          fn: async () => {
            const v = [1, 0, 0,];
            expect(dotProduct({ a: v, b: v, },),).toBe(1,);
          },
        },),
        it({
          name: 'orthogonal vectors yield 0',
          fn: async () => {
            expect(dotProduct({ a: [1, 0,], b: [0, 1,], },),).toBe(0,);
          },
        },),
        it({
          name: 'opposite vectors yield -1',
          fn: async () => {
            expect(dotProduct({ a: [1, 0,], b: [-1, 0,], },),).toBe(-1,);
          },
        },),
        it({
          name: 'throws on length mismatch',
          fn: async () => {
            expect(function callWithMismatch() {
              dotProduct({ a: [1, 2,], b: [1, 2, 3,], },);
            },)
              .toThrow('Vector length mismatch',);
          },
        },),
      ],
    },),
    describe({
      name: cosineSimilarity.name,
      children: [
        it({
          name: 'parallel vectors yield 1 regardless of magnitude',
          fn: async () => {
            expect(cosineSimilarity({ a: [3, 0,], b: [7, 0,], },),).toBeCloseTo(1, 10,);
          },
        },),
        it({
          name: 'orthogonal vectors yield 0',
          fn: async () => {
            expect(cosineSimilarity({ a: [1, 0,], b: [0, 1,], },),).toBeCloseTo(0, 10,);
          },
        },),
        it({
          name: 'anti-parallel vectors yield -1',
          fn: async () => {
            expect(cosineSimilarity({ a: [2, 0,], b: [-5, 0,], },),).toBeCloseTo(-1, 10,);
          },
        },),
        it({
          name: 'throws on zero vector',
          fn: async () => {
            expect(function callWithZero() {
              cosineSimilarity({ a: [0, 0,], b: [1, 1,], },);
            },)
              .toThrow('zero magnitude',);
          },
        },),
        it({
          name: 'throws on length mismatch',
          fn: async () => {
            expect(function callWithMismatch() {
              cosineSimilarity({ a: [1,], b: [1, 2,], },);
            },)
              .toThrow('Vector length mismatch',);
          },
        },),
      ],
    },),
  ],
},);
