import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { wait, } from '@monochromatic-dev/async-time-core/ts';

import { mapInChunks, } from '../dist/final/node/map-in-chunks.mjs';

await describe({
  name: mapInChunks.name,
  children: [
    it({
      name: 'never runs more tasks at once than the chunk size, and returns results in input order',
      fn: async () => {
        /**
         Tasks running now.
         */
        let running = 0;
        /**
         Most tasks seen running together.
         */
        let peak = 0;
        /**
         Results of forty items in chunks of eight.
         */
        const results = await mapInChunks({
          items: Array.from({ length: 40, }, (_unused, index,) => index,),
          size: 8,
          task: async (item: number,) => {
            running += 1;
            peak = Math.max(peak, running,);
            await wait(5,);
            running -= 1;
            return item * 2;
          },
        },);
        expect(peak,).toBe(8,);
        expect(results,).toEqual(Array.from({ length: 40, }, (_unused, index,) => index * 2,),);
      },
    },),
    it({
      name: 'handles an empty list, a size below one and a size above the item count',
      fn: async () => {
        expect(await mapInChunks({ items: [], size: 4, task: (item: number,) => Promise.resolve(item,), },),).toEqual([],);
        expect(await mapInChunks({ items: [1, 2, 3,], size: 0, task: (item: number,) => Promise.resolve(item + 1,), },),).toEqual([2, 3, 4,],);
        expect(await mapInChunks({ items: [1, 2,], size: 99, task: (item: number,) => Promise.resolve(item,), },),).toEqual([1, 2,],);
      },
    },),
    it({
      name: 'rejects when a task rejects, so a failure is never hidden',
      fn: async () => {
        await expect(mapInChunks({
          items: [1, 2, 3,],
          size: 2,
          task: (item: number,) => item === 3 ? Promise.reject(new Error('task failed',),) : Promise.resolve(item,),
        },),).rejects.toThrow('task failed',);
      },
    },),
  ],
},);
