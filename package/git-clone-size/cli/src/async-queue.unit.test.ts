/**
 Tests that mergeAsArrived yields in completion order, not submission order.
 
 @module
 */

import { wait, } from '@monochromatic-dev/module-async-time/ts';
import {
  describe,
  it,
} from '@monochromatic-dev/module-test/ts';

import { mergeAsArrived, } from './async-queue.ts';

/**
 Resolves to a label after a delay, modelling a probe that settles later.
 
 @param label - identifier yielded on completion
 
 @param ms - delay before resolving
 
 @returns the label after the delay
 */
async function delayed(label: string, ms: number): Promise<string> {
  await wait(ms);
  return label;
}

await describe({
  name: mergeAsArrived.name,
  children: [
    it({
      name: 'yields fastest-first regardless of task order',
      fn: async ({ expect, }) => {
        const order: string[] = [];
        for await (const value of mergeAsArrived({
          tasks: [
            delayed('slow', 40),
            delayed('fast', 5),
            delayed('medium', 20),
          ],
        })) {
          order.push(value);
        }
        expect(order).toEqual(['fast', 'medium', 'slow',]);
      },
    }),

    it({
      name: 'yields every task exactly once',
      fn: async ({ expect, }) => {
        const seen: string[] = [];
        for await (const value of mergeAsArrived({
          tasks: [delayed('a', 5), delayed('b', 5), delayed('c', 5),],
        })) {
          seen.push(value);
        }
        expect(seen.toSorted()).toEqual(['a', 'b', 'c',]);
      },
    }),
  ],
});
