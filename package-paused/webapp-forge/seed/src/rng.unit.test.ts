/**
 * Unit tests for the seed RNG primitives.
 *
 * Determinism is the contract: same seed -> same output. These tests
 * pin a small number of expected values so a future change to the
 * algorithm shows up immediately.
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core';

import {
  rng,
  rngInt,
  rngPick,
} from './rng.ts';

await describe({
  name: '',
  children: [
    describe({
      name: rng.name,
      children: [
        it({
          name: 'returns the same value for the same seed',
          async fn() {
            const a = rng(42,);
            const b = rng(42,);
            expect(a,).toBe(b,);
          },
        },),
        it({
          name: 'stays in [0, 1) over a small range of seeds',
          async fn() {
            for (let i = 0; i < 100; i += 1) {
              const r = rng(i,);
              expect((r >= 0) && (r < 1),).toBe(true,);
            }
          },
        },),
      ],
    },),
    describe({
      name: rngInt.name,
      children: [
        it({
          name: 'returns lo when hi <= lo',
          async fn() {
            expect(rngInt({
              seed: 1,
              lo: 5,
              hi: 5,
            },),)
              .toBe(5,);
            expect(rngInt({
              seed: 1,
              lo: 5,
              hi: 3,
            },),)
              .toBe(5,);
          },
        },),
        it({
          name: 'returns a value in [lo, hi)',
          async fn() {
            for (let i = 0; i < 100; i += 1) {
              const v = rngInt({
                seed: i,
                lo: 1,
                hi: 10,
              },);
              expect((v >= 1) && (v < 10),).toBe(true,);
            }
          },
        },),
      ],
    },),
    describe({
      name: rngPick.name,
      children: [
        it({
          name: 'returns undefined for empty input',
          async fn() {
            expect(rngPick({
              seed: 1,
              items: [],
            },),)
              .toBeUndefined();
          },
        },),
        it({
          name: 'returns one of the input items',
          async fn() {
            const items = [
              'a',
              'b',
              'c',
            ];
            const seen = new Set<string>();
            for (let i = 0; i < 30; i += 1) {
              const v = rngPick({
                seed: i,
                items,
              },);
              if (v !== undefined)
                seen.add(v,);
            }
            for (const v of seen)
              expect(items.includes(v,),).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
