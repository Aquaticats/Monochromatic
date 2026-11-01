/**
 Tests for how the streaming map hands out and takes back its mapper slots:
 a failure gives back the slot it held, and only that one, so a worker that
 still waits for a slot is neither starved nor let in early.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { pMapIterable, } from '../dist/final/neutral/index.mjs';

import {
  collect,
  createGate,
  rejectionOf,
  yieldTurn,
} from './test-support.ts';

await describe({
  name: `${pMapIterable.name} mapper slots`,
  children: [
    it({
      name: 'keeps a waiting element waiting when a read fails, as the failing read held no slot',
      fn: async () => {
        const gates = [
          createGate(),
          createGate(),
        ];
        /**
         Failure of the element read the test settles last.
         */
        const failure = new Error('read failed',);
        /**
         Element that rejects once the test says so.
         */
        const failingElement = Promise.withResolvers<number>();
        /**
         Values the mapper was called with.
         */
        const started: number[] = [];
        const collecting = collect(pMapIterable({
          iterable: [
            0,
            1,
            2,
            failingElement.promise,
          ],
          mapper: async function gatedMapper(value: number,): Promise<number> {
            started.push(value,);
            await gates[value]?.open;
            return value;
          },
          options: {
            concurrency: 2,
            backpressure: Number.POSITIVE_INFINITY,
          },
        },),);
        await yieldTurn();
        // Elements 0 and 1 hold both slots, element 2 waits for one, and
        // element 3 is still being read.
        expect(started,).toEqual([
          0,
          1,
        ],);
        failingElement.reject(failure,);
        await yieldTurn();
        expect(started,).toEqual([
          0,
          1,
        ],);
        for (const gate of gates)
          gate.release();

        expect(await rejectionOf(collecting,),).toBe(failure,);
      },
    },),

    it({
      name: 'gives back the slot of a failing mapper so an element read late can use it',
      fn: async () => {
        const gates = [
          createGate(),
          createGate(),
        ];
        /**
         Failure the last element's mapper throws.
         */
        const failure = new Error('mapper failed',);
        /**
         Values the mapper was called with.
         */
        const started: number[] = [];
        const firstElementGate = createGate();
        /**
         First element, which stays pending until the test releases its gate.
         */
        async function heldFirstElement(): Promise<number> {
          await firstElementGate.open;
          return 0;
        }
        const collecting = collect(pMapIterable({
          iterable: [
            heldFirstElement(),
            1,
            2,
          ],
          mapper: async function failingMapper(value: number,): Promise<number> {
            started.push(value,);
            if (value === 1)
              await gates[1]?.open;
            if (value === 2)
              throw failure;
            return value;
          },
          options: {
            concurrency: 2,
            backpressure: Number.POSITIVE_INFINITY,
          },
        },),);
        await yieldTurn();
        // Elements 1 and 2 hold both slots, and the last one failed.
        expect(started,).toEqual([
          1,
          2,
        ],);
        firstElementGate.release();
        await yieldTurn();
        // The slot of the failed mapper is free again, so the first element,
        // which comes before the failure, is mapped without waiting for the
        // mapper of element 1.
        expect(started,).toEqual([
          1,
          2,
          0,
        ],);
        gates[1]?.release();

        expect(await rejectionOf(collecting,),).toBe(failure,);
      },
    },),
  ],
},);
